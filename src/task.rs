use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{self, Duration};
use tracing::{debug, error, info, instrument};

use crate::TaskError;

pub trait Runnable<Input, Output>: Send + Sync {
    fn name(&self) -> String;
    fn run(&self, data: &Input, start: u64, end: u64) -> Result<Output, TaskError<Output>>;
}

pub struct Worker<Input, Output> {
    task: Arc<dyn Runnable<Input, Output>>,
    ctx: TaskContext<Input, Output>,
}

pub struct TaskContext<Input, Output> {
    pub data: Arc<RwLock<Input>>,
    pub receiver: broadcast::Receiver<(u64, u64)>,
    pub sender: mpsc::Sender<TaskResult<Output>>,
}

pub struct TaskResult<Output> {
    pub name: String,
    pub result: Output,
}

impl<Input, Output> Worker<Input, Output> {
    pub fn new(task: Arc<dyn Runnable<Input, Output>>, ctx: TaskContext<Input, Output>) -> Self {
        Worker { task, ctx }
    }

    #[instrument(skip(self, shutdown_rx), fields(task_name = %self.task.name()))]
    pub async fn run(
        &mut self,
        mut shutdown_rx: broadcast::Receiver<()>,
        timeout_secs: u64,
    ) -> Result<(), TaskError<Output>> {
        let name = self.task.name().clone();
        let timeout_duration = Duration::from_secs(timeout_secs);
        debug!("Starting worker task");

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break Ok(());
                }
                result = self.ctx.receiver.recv() => {
                    match result {
                        Ok((start, end)) => {
                            debug!(start = %start, end = %end, "Processing time window");
                            
                            let data = match time::timeout(
                                timeout_duration,
                                self.ctx.data.read(),
                            ).await {
                                Ok(guard) => guard,
                                Err(_) => {
                                    error!("Data read timeout, abandoning current work");
                                    continue;
                                }
                            };

                            match self.task.run(&*data, start, end) {
                                Ok(result) => {
                                    debug!("Task completed successfully");
                                    if let Err(e) = self.ctx.sender.send(TaskResult {
                                        name: name.clone(),
                                        result,
                                    }).await {
                                        error!(error = %e, "Failed to send task result");
                                        return Err(e.into());
                                    }
                                }
                                Err(e) => {
                                    error!(error = %e, "Task execution failed");
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to receive time window");
                            return Err(TaskError::RecvError(e));
                        }
                    }
                }
            }
        }
    }
}

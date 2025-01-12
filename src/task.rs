use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, instrument};

use crate::TaskError;

pub trait Runnable<T, D>: Send + Sync {
    fn name(&self) -> String;
    fn run(&self, data: &T, start: u64, end: u64) -> Result<D, TaskError<D>>;
}

pub struct Worker<T, D> {
    pub task: Arc<dyn Runnable<T, D>>,
    pub ctx: TaskContext<T, D>,
}

pub struct TaskContext<T, D> {
    pub data: Arc<RwLock<T>>,
    pub receiver: broadcast::Receiver<(u64, u64)>,
    pub sender: mpsc::Sender<TaskResult<D>>,
}

pub struct TaskResult<D> {
    pub name: String,
    pub result: D,
}

impl<T, D> Worker<T, D> {
    pub fn new(task: Arc<dyn Runnable<T, D>>, ctx: TaskContext<T, D>) -> Self {
        Worker { task, ctx }
    }

    #[instrument(skip(self, shutdown_rx), fields(task_name = %self.task.name()))]
    pub async fn run(
        &mut self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), TaskError<D>> {
        let name = self.task.name().clone();
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
                            if start == 0 && end == 0 {
                                debug!("Received shutdown signal, abandoning current work");
                                continue;
                            }

                            let data = self.ctx.data.read().await;
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

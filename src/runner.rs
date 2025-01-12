use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tokio::time::interval;
use tracing::{debug, error, info, instrument, warn};

use crate::context::Context;
use crate::data_types::DataSet;
use crate::error::TaskError;
use crate::interval::TaskInterval;
use crate::task::{TaskContext, TaskResult, Worker};

pub struct Runner<T, D> {
    pub ctx: Context<T, D>,
    shutdown: broadcast::Sender<()>,
}

impl<T: Send + Sync + 'static, D: Send + Sync + 'static> Runner<T, D> {
    pub fn new(ctx: Context<T, D>) -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Runner { ctx, shutdown }
    }

    pub fn shutdown(&self) -> Result<(), TaskError<D>> {
        debug!("Initiating runner shutdown");
        self.shutdown
            .send(())
            .map_err(|e| TaskError::ShutdownError(e.to_string()))?;
        debug!("Runner shutdown signal sent");
        Ok(())
    }

    #[instrument(skip(self), name = "run_task_runner", fields(tasks_count = %self.ctx.tasks.len()))]
    pub async fn run(&self) -> Result<(), TaskError<D>> {
        info!("Starting task runner");
        let mut shutdown = self.shutdown.subscribe();
        let timeout = self.ctx.config.shutdown_timeout;
        let result_sender = self.ctx.sender.clone();
        let task_interval = self.ctx.interval.clone();

        debug!(interval_micros = %task_interval.as_micros(), "Configuring runner");

        let mut interval = interval(Duration::from_micros(task_interval.as_micros()));
        let task_count = self.ctx.tasks.len();

        let (time_broadcaster, _) =
            broadcast::channel::<(u64, u64)>(self.ctx.config.broadcast_channel_capacity);
        let (output_sender, mut output_receiver) = mpsc::channel(task_count);

        debug!("Spawning {} worker tasks", task_count);
        let mut worker_handles = Vec::new();
        for task in self.ctx.tasks.iter() {
            let task = Arc::clone(task);
            let task_ctx = TaskContext {
                data: Arc::clone(&self.ctx.data),
                receiver: time_broadcaster.subscribe(),
                sender: output_sender.clone(),
            };
            let mut worker = Worker::new(task, task_ctx);
            let shutdown_rx = self.shutdown.subscribe();
            let handle = tokio::spawn(async move { worker.run(shutdown_rx).await });
            worker_handles.push(handle);
        }

        let consolidator = async move {
            debug!("Starting result consolidator");
            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        info!("Received shutdown signal, stopping consolidator");
                        break;
                    }
                    _ = interval.tick() => {
                        let end = get_current_time(&task_interval);
                        let start = end - task_interval.as_u64();
                        let mut dataset = DataSet::new(end);

                        if let Err(e) = time_broadcaster.send((start, end)) {
                            warn!(error = %e, "Failed to broadcast time window");
                            return Err(TaskError::BroadcastError(e.to_string()));
                        }

                        collect_results(&mut output_receiver, &mut dataset, task_count, timeout).await?;

                        if let Err(e) = result_sender.send(dataset).await {
                            warn!(error = %e, "Failed to send dataset");
                            return Err(TaskError::DataSetSendError(e));
                        }
                    }
                }
            }
            Ok(())
        };

        let consolidator_handle = tokio::spawn(consolidator);

        debug!("Waiting for worker tasks to complete");
        for handle in worker_handles {
            handle.await??;
        }
        debug!("Worker tasks completed, waiting for consolidator");
        consolidator_handle.await??;
        info!("Runner shutdown complete");

        Ok(())
    }
}

#[instrument(
    skip(output_receiver, dataset),
    fields(task_count = %task_count),
    name = "collect_task_results"
)]
async fn collect_results<D>(
    output_receiver: &mut mpsc::Receiver<TaskResult<D>>,
    dataset: &mut DataSet<D>,
    task_count: usize,
    timeout: Duration,
) -> Result<(), TaskError<D>> {
    debug!("Starting result collection");

    for i in 0..task_count {
        match tokio::time::timeout(timeout, output_receiver.recv()).await {
            Ok(Some(TaskResult { name, result })) => {
                debug!(task_name = %name, remaining = %(task_count - i - 1), "Collected task result");
                dataset.insert(&name, result);
            }
            Ok(None) => {
                warn!("Result channel closed unexpectedly");
                break;
            }
            Err(_) => {
                error!(timeout_ms = %timeout.as_millis(), "Timeout while collecting results");
                return Err(TaskError::TimeoutError);
            }
        }
    }
    debug!("Result collection complete");
    Ok(())
}

fn get_current_time(task_interval: &TaskInterval) -> u64 {
    match task_interval {
        TaskInterval::Micros(_) => chrono::Utc::now().timestamp_micros() as u64,
        TaskInterval::Millis(_) => chrono::Utc::now().timestamp_millis() as u64,
        TaskInterval::Seconds(_) => chrono::Utc::now().timestamp() as u64,
        TaskInterval::Minutes(_) => (chrono::Utc::now().timestamp() * 60) as u64,
    }
}

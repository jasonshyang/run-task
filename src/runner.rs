use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tokio::time::interval;

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
        self.shutdown
            .send(())
            .map_err(|e| TaskError::ShutdownError(e.to_string()))?;
        Ok(())
    }

    pub async fn run(&self) -> Result<(), TaskError<D>> {
        let mut shutdown = self.shutdown.subscribe();
        let timeout = self.ctx.config.shutdown_timeout;
        let result_sender = self.ctx.sender.clone();
        let task_interval = self.ctx.interval.clone();
        let mut interval = interval(Duration::from_micros(task_interval.as_micros()));
        let task_count = self.ctx.tasks.len();

        let (time_broadcaster, _) =
            broadcast::channel::<(u64, u64)>(self.ctx.config.broadcast_channel_capacity);
        let (output_sender, mut output_receiver) = mpsc::channel(task_count);

        // Spawn workers for each task which will run in the background waiting for time_broadcast signal to start the task
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
            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        break;
                    }
                    _ = interval.tick() => {
                        let end = get_current_time(&task_interval);
                        let start = end - task_interval.as_u64();
                        let mut dataset = DataSet::new(end);

                        if let Err(e) = time_broadcaster.send((start, end)) {
                            return Err(TaskError::BroadcastError(e.to_string()));
                        }

                        collect_results(&mut output_receiver, &mut dataset, task_count, timeout).await?;

                        result_sender.send(dataset).await
                            .map_err(|e| TaskError::DataSetSendError(e))?;
                    }
                }
            }
            Ok(())
        };

        let consolidator_handle = tokio::spawn(consolidator);

        for handle in worker_handles {
            handle.await??;
        }
        consolidator_handle.await??;

        Ok(())
    }
}

async fn collect_results<D>(
    output_receiver: &mut mpsc::Receiver<TaskResult<D>>,
    dataset: &mut DataSet<D>,
    task_count: usize,
    timeout: Duration,
) -> Result<(), TaskError<D>> {
    for _ in 0..task_count {
        match tokio::time::timeout(timeout, output_receiver.recv()).await {
            Ok(Some(TaskResult { name, result })) => {
                dataset.insert(&name, result);
            }
            Ok(None) => break,
            Err(_) => {
                return Err(TaskError::TimeoutError);
            }
        }
    }
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

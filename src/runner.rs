use core::fmt;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;

use crate::data_types::DataSet;
use crate::task::{TaskContext, TaskResult, Worker};
use crate::{Runnable, TaskError, TaskInterval};

pub struct Context<T, D> {
    pub tasks: Vec<Arc<dyn Runnable<T, D>>>,
    pub data: Arc<RwLock<T>>,
    pub interval: TaskInterval,
    pub sender: mpsc::Sender<DataSet<D>>,
}

impl<T, D> Context<T, D> {
    pub fn new(
        tasks: Vec<Arc<dyn Runnable<T, D>>>,
        data: Arc<RwLock<T>>,
        interval: TaskInterval,
    ) -> (Self, mpsc::Receiver<DataSet<D>>) {
        let (sender, receiver) = mpsc::channel(1024);
        let ctx = Context {
            tasks,
            data,
            interval,
            sender,
        };
        (ctx, receiver)
    }
}

pub struct ContextBuilder<T: Default, D> {
    tasks: Vec<Arc<dyn Runnable<T, D>>>,
    data: Arc<RwLock<T>>,
    interval: TaskInterval,
}

impl<T: Default, D> ContextBuilder<T, D> {
    pub fn new() -> Self {
        ContextBuilder {
            tasks: Vec::new(),
            data: Arc::new(RwLock::new(Default::default())),
            interval: TaskInterval::Seconds(5),
        }
    }

    pub fn with_task(mut self, task: impl Runnable<T, D> + 'static) -> Self {
        self.tasks.push(Arc::new(task));
        self
    }

    pub fn with_tasks(mut self, tasks: Vec<impl Runnable<T, D> + 'static>) -> Self {
        for task in tasks {
            self.tasks.push(Arc::new(task));
        }
        self
    }

    pub fn with_data(mut self, data: Arc<RwLock<T>>) -> Self {
        self.data = data;
        self
    }

    pub fn with_interval(mut self, interval: TaskInterval) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> (Context<T, D>, mpsc::Receiver<DataSet<D>>) {
        Context::new(self.tasks, self.data, self.interval)
    }
}

impl<T: Default, D> Default for ContextBuilder<T, D> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Runner<T, D> {
    pub ctx: Context<T, D>,
}

impl<T: Send + Sync + 'static, D: Send + Sync + 'static> Runner<T, D> {
    pub fn new(ctx: Context<T, D>) -> Self {
        Runner { ctx }
    }

    pub async fn run(&self) -> Result<(), TaskError<D>> {
        let result_sender = self.ctx.sender.clone();
        let task_interval = self.ctx.interval.clone();
        let mut interval = interval(Duration::from_micros(task_interval.as_micros()));
        let task_count = self.ctx.tasks.len();

        let (time_broadcaster, _) = broadcast::channel::<(u64, u64)>(1);
        let (output_sender, mut output_receiver) = mpsc::channel(task_count);

        // Spawn workers for each task which will run in the background waiting for StartWorkTime to start the task
        let mut worker_handles = Vec::new();
        for task in self.ctx.tasks.iter() {
            let task = Arc::clone(task);
            let task_ctx = TaskContext {
                data: Arc::clone(&self.ctx.data),
                receiver: time_broadcaster.subscribe(),
                sender: output_sender.clone(),
            };
            let mut worker = Worker::new(task, task_ctx);
            let handle = tokio::spawn(async move { worker.run().await });
            worker_handles.push(handle);
        }

        // Start the main loop which will broadcast the StartWorkTime to all workers
        let consolidator_handle = tokio::spawn(async move {
            loop {
                interval.tick().await;
                let end = get_current_time(&task_interval);
                let start = end - task_interval.as_u64();
                let mut dataset = DataSet::new(end);
                time_broadcaster.send((start, end)).unwrap();

                // Collect the results from all workers, this blocks until all workers have sent their results
                for _ in 0..task_count {
                    let TaskResult { name, result } = output_receiver.recv().await.unwrap();
                    dataset.insert(&name, result);
                }
                // Send the dataset to the receiver
                result_sender.send(dataset).await.unwrap();
            }
        });

        // Wait for all workers to finish
        for handle in worker_handles {
            let _ = handle.await?;
        }

        // Wait for the consolidator to finish
        consolidator_handle.await?;

        Ok(())
    }
}

pub fn spawn_runner<T: Send + Sync + 'static, D: Send + Sync + 'static + fmt::Debug>(
    ctx: Context<T, D>,
) {
    tokio::spawn(async move {
        let runner = Runner::new(ctx);
        runner.run().await.unwrap();
    });
}

fn get_current_time(task_interval: &TaskInterval) -> u64 {
    match task_interval {
        TaskInterval::Micros(_) => chrono::Utc::now().timestamp_micros() as u64,
        TaskInterval::Millis(_) => chrono::Utc::now().timestamp_millis() as u64,
        TaskInterval::Seconds(_) => chrono::Utc::now().timestamp() as u64,
        TaskInterval::Minutes(_) => (chrono::Utc::now().timestamp() * 60) as u64,
    }

}
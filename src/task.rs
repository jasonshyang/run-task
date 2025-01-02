use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

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

    pub async fn run(&mut self) -> Result<(), TaskError<D>> {
        let name = self.task.name().clone();

        loop {
            let (start, end) = self.ctx.receiver.recv().await?;
            let data = self.ctx.data.read().await;
            let result = self.task.run(&*data, start, end)?;
            self.ctx
                .sender
                .send(TaskResult {
                    name: name.clone(),
                    result,
                })
                .await?;
        }
    }
}

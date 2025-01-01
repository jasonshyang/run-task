use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::data_types::Data;
use crate::TaskError;

pub trait Runnable<T>: Send + Sync {
    fn name(&self) -> String;
    fn run(&self, data: &T, at: u64) -> Result<Data, TaskError>;
}

pub struct Worker<T> {
    pub task: Arc<dyn Runnable<T>>,
    pub ctx: TaskContext<T>,
}

pub struct TaskContext<T> {
    pub data: Arc<RwLock<T>>,
    pub receiver: broadcast::Receiver<u64>,
    pub sender: mpsc::Sender<TaskResult>,
}

pub struct TaskResult {
    pub name: String,
    pub result: Data,
}

impl<T> Worker<T> {
    pub fn new(task: Arc<dyn Runnable<T>>, ctx: TaskContext<T>) -> Self {
        Worker { task, ctx }
    }

    pub async fn run(&mut self) -> Result<(), TaskError> {
        let name = self.task.name().clone();

        loop {
            let timestamp = self.ctx.receiver.recv().await?;
            let data = self.ctx.data.read().await;
            let result = self.task.run(&*data, timestamp)?;
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

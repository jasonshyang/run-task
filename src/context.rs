use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::config::RunnerConfig;
use crate::interval::TaskInterval;
use crate::task::Runnable;
use crate::data_types::DataSet;

pub struct Context<T, D> {
    pub config: RunnerConfig,
    pub tasks: Vec<Arc<dyn Runnable<T, D>>>,
    pub data: Arc<RwLock<T>>,
    pub interval: TaskInterval,
    pub sender: mpsc::Sender<DataSet<D>>,
}

impl<T, D> Context<T, D> {
    pub fn new(
        config: RunnerConfig,
        tasks: Vec<Arc<dyn Runnable<T, D>>>,
        data: Arc<RwLock<T>>,
        interval: TaskInterval,
    ) -> (Self, mpsc::Receiver<DataSet<D>>) {
        let (sender, receiver) = mpsc::channel(config.task_channel_capacity);
        let ctx = Context {
            config,
            tasks,
            data,
            interval,
            sender,
        };
        (ctx, receiver)
    }
}

pub struct ContextBuilder<T: Default, D> {
    config: RunnerConfig,
    tasks: Vec<Arc<dyn Runnable<T, D>>>,
    data: Arc<RwLock<T>>,
    interval: TaskInterval,
}

impl<T: Default, D> ContextBuilder<T, D> {
    pub fn new() -> Self {
        ContextBuilder {
            config: RunnerConfig::default(),
            tasks: Vec::new(),
            data: Arc::new(RwLock::new(Default::default())),
            interval: TaskInterval::Seconds(5),
        }
    }

    pub fn with_config(mut self, config: RunnerConfig) -> Self {
        self.config = config;
        self
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
        Context::new(self.config, self.tasks, self.data, self.interval)
    }
}

impl<T: Default, D> Default for ContextBuilder<T, D> {
    fn default() -> Self {
        Self::new()
    }
}
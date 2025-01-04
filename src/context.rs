use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::config::RunnerConfig;
use crate::data_types::DataSet;
use crate::interval::TaskInterval;
use crate::task::Runnable;

pub type DataReceiver<D> = mpsc::Receiver<DataSet<D>>;
pub type BuildResult<T, D> = (Context<T, D>, DataReceiver<D>, Arc<RwLock<T>>);

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
    ) -> (Self, DataReceiver<D>) {
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
    tasks: Vec<Arc<dyn Runnable<T, D>>>,
    data: Option<Arc<RwLock<T>>>,
    interval: TaskInterval,
    config: RunnerConfig,
}

impl<T: Default, D> ContextBuilder<T, D> {
    pub fn new() -> Self {
        ContextBuilder {
            tasks: Vec::new(),
            data: None,
            interval: TaskInterval::Seconds(5),
            config: RunnerConfig::default(),
        }
    }

    pub fn with_data(mut self, data: Arc<RwLock<T>>) -> Self {
        self.data = Some(data);
        self
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

    pub fn with_interval(mut self, interval: TaskInterval) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> BuildResult<T, D> {
        let data = self
            .data
            .unwrap_or_else(|| Arc::new(RwLock::new(T::default())));
        let (ctx, rx) = Context::new(self.config, self.tasks, Arc::clone(&data), self.interval);
        (ctx, rx, data)
    }

    pub fn get_data_or_default(&self) -> Arc<RwLock<T>> {
        self.data
            .clone()
            .unwrap_or_else(|| Arc::new(RwLock::new(T::default())))
    }
}

impl<T: Default, D> Default for ContextBuilder<T, D> {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::config::RunnerConfig;
use crate::data_types::DataSet;
use crate::interval::TaskInterval;
use crate::task::Runnable;

pub type DataReceiver<Output> = mpsc::Receiver<DataSet<Output>>;
pub type BuildResult<Input, Output> = (Context<Input, Output>, DataReceiver<Output>, Arc<RwLock<Input>>);

pub struct Context<Input, Output> {
    pub config: RunnerConfig,
    pub tasks: Vec<Arc<dyn Runnable<Input, Output>>>,
    pub data: Arc<RwLock<Input>>,
    pub interval: TaskInterval,
    pub sender: mpsc::Sender<DataSet<Output>>,
}

impl<Input, Output> Context<Input, Output> {
    pub fn new(
        config: RunnerConfig,
        tasks: Vec<Arc<dyn Runnable<Input, Output>>>,
        data: Arc<RwLock<Input>>,
        interval: TaskInterval,
    ) -> (Self, DataReceiver<Output>) {
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

pub struct ContextBuilder<Input: Default, Output> {
    tasks: Vec<Arc<dyn Runnable<Input, Output>>>,
    data: Option<Arc<RwLock<Input>>>,
    interval: TaskInterval,
    config: RunnerConfig,
}

impl<Input: Default, Output> ContextBuilder<Input, Output> {
    pub fn new() -> Self {
        ContextBuilder {
            tasks: Vec::new(),
            data: None,
            interval: TaskInterval::Seconds(5),
            config: RunnerConfig::default(),
        }
    }

    pub fn with_data(mut self, data: Arc<RwLock<Input>>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_config(mut self, config: RunnerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_task(mut self, task: impl Runnable<Input, Output> + 'static) -> Self {
        self.tasks.push(Arc::new(task));
        self
    }

    pub fn with_tasks(mut self, tasks: Vec<impl Runnable<Input, Output> + 'static>) -> Self {
        for task in tasks {
            self.tasks.push(Arc::new(task));
        }
        self
    }

    pub fn with_interval(mut self, interval: TaskInterval) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> BuildResult<Input, Output> {
        let data = self
            .data
            .unwrap_or_else(|| Arc::new(RwLock::new(Input::default())));
        let (ctx, rx) = Context::new(self.config, self.tasks, Arc::clone(&data), self.interval);
        (ctx, rx, data)
    }

    pub fn get_data_or_default(&self) -> Arc<RwLock<Input>> {
        self.data
            .clone()
            .unwrap_or_else(|| Arc::new(RwLock::new(Input::default())))
    }
}

impl<Input: Default, Output> Default for ContextBuilder<Input, Output> {
    fn default() -> Self {
        Self::new()
    }
}

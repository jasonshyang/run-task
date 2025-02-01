use tokio::time::Duration;

#[derive(Clone, Debug)]
pub struct RunnerConfig {
    pub task_channel_capacity: usize,
    pub broadcast_channel_capacity: usize,
    pub shutdown_timeout: Duration,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            task_channel_capacity: 1024,
            broadcast_channel_capacity: 16,
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

impl RunnerConfig {
    pub fn new(
        task_channel_capacity: usize,
        broadcast_channel_capacity: usize,
        shutdown_timeout: Duration,
    ) -> Self {
        Self {
            task_channel_capacity,
            broadcast_channel_capacity,
            shutdown_timeout,
        }
    }
}

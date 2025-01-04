mod data_types;
mod error;
mod interval;
mod runner;
mod task;
mod config;
mod context;
mod tests;

pub use data_types::DataSet;
pub use error::TaskError;
pub use interval::TaskInterval;
pub use runner::Runner;
pub use task::Runnable;
pub use config::RunnerConfig;
pub use context::{Context, ContextBuilder};

pub mod prelude {
    pub use crate::data_types::DataSet;
    pub use crate::error::TaskError;
    pub use crate::interval::TaskInterval;
    pub use crate::runner::Runner;
    pub use crate::task::Runnable;
    pub use crate::config::RunnerConfig;
    pub use crate::context::{Context, ContextBuilder};
}
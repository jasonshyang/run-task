mod data_types;
mod error;
mod interval;
mod runner;
mod task;

pub use data_types::DataSet;
pub use error::*;
pub use interval::TaskInterval;
pub use runner::{spawn_runner, Context, ContextBuilder};
pub use task::Runnable;

pub mod prelude {
    pub use crate::data_types::DataSet;
    pub use crate::error::TaskError;
    pub use crate::interval::TaskInterval;
    pub use crate::runner::{spawn_runner, Context, ContextBuilder};
    pub use crate::task::Runnable;
}

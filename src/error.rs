use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

use crate::task::TaskResult;
use crate::DataSet;

#[derive(Error, Debug)]
pub enum TaskError<D> {
    #[error("Task Error: {0}")]
    TaskError(String),
    #[error("Task Send Error: {0}")]
    TaskSendError(#[from] SendError<TaskResult<D>>),
    #[error("DataSet Send Error: {0}")]
    DataSetSendError(#[from] SendError<DataSet<D>>),
    #[error("Receive Error: {0}")]
    RecvError(#[from] RecvError),
    #[error("Join Error: {0}")]
    JoinError(#[from] JoinError),
    #[error("Broadcast Error: {0}")]
    BroadcastError(String),
    #[error("Shutdown Error: {0}")]
    ShutdownError(String),
    #[error("Timeout Error")]
    TimeoutError,
}

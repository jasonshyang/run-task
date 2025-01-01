use crate::task::TaskResult;
use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum TaskError<D> {
    #[error("TaskError: {0}")]
    TaskError(String),
    #[error("SendError: {0}")]
    SendError(SendError<TaskResult<D>>),
    #[error("RecvError: {0}")]
    RecvError(RecvError),
    #[error("JoinError: {0}")]
    JoinError(JoinError),
}

impl<D> From<SendError<TaskResult<D>>> for TaskError<D> {
    fn from(error: SendError<TaskResult<D>>) -> Self {
        TaskError::SendError(error)
    }
}

impl<D> From<RecvError> for TaskError<D> {
    fn from(error: RecvError) -> Self {
        TaskError::RecvError(error)
    }
}

impl<D> From<JoinError> for TaskError<D> {
    fn from(error: JoinError) -> Self {
        TaskError::JoinError(error)
    }
}

use crate::task::TaskResult;
use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("TaskError: {0}")]
    TaskError(String),
    #[error("SendError: {0}")]
    SendError(SendError<TaskResult>),
    #[error("RecvError: {0}")]
    RecvError(RecvError),
    #[error("JoinError: {0}")]
    JoinError(JoinError),
}

impl From<SendError<TaskResult>> for TaskError {
    fn from(error: SendError<TaskResult>) -> Self {
        TaskError::SendError(error)
    }
}

impl From<RecvError> for TaskError {
    fn from(error: RecvError) -> Self {
        TaskError::RecvError(error)
    }
}

impl From<JoinError> for TaskError {
    fn from(error: JoinError) -> Self {
        TaskError::JoinError(error)
    }
}

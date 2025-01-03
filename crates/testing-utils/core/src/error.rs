use gadget_runners::core::error::RunnerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestRunnerError {
    #[error("Runner setup failed: {0}")]
    SetupError(String),
    #[error("Runner execution failed: {0}")]
    ExecutionError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    RunnerError(#[from] RunnerError),
}
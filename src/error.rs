use std::io;
use thiserror::Error;

/// Custom error type for arc operations
#[derive(Error, Debug)]
pub enum ArcError {
    #[error("git not found in current directory tree")]
    GitNotFound,

    #[error("repository '{0}' not found")]
    RepositoryNotFound(String),

    #[error("arc.toml configuration file not found in current directory or any parent directory")]
    ConfigNotFound,

    #[error("failed to parse arc.toml: {0}")]
    ConfigParseError(String),

    #[error("invalid configuration: {0}")]
    ConfigValidation(String),

    #[error("failed to clone repository '{repo}': {message}")]
    CloneFailed { repo: String, message: String },

    #[error("failed to pull from remote: {0}")]
    PullFailed(String),

    #[error("failed to reset repository: {0}")]
    ResetFailed(String),

    #[error("failed to execute command: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("operation aborted by user")]
    Aborted,
}

pub type Result<T> = std::result::Result<T, ArcError>;

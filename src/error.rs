use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Custom error type for arc operations
#[derive(Error, Debug)]
pub enum ArcError {
    #[error("git not found in current directory tree")]
    GitNotFound,

    #[error("repository '{0}' not found")]
    RepositoryNotFound(String),

    #[allow(dead_code)]
    #[error("branch '{0}' not found")]
    BranchNotFound(String),

    #[error("arc.json configuration file not found in current directory or any parent directory")]
    ConfigNotFound,

    #[error("failed to parse arc.json: {0}")]
    ConfigParseError(String),

    #[error("failed to clone repository '{repo}': {message}")]
    CloneFailed { repo: String, message: String },

    #[allow(dead_code)]
    #[error("failed to checkout branch '{branch}': {message}")]
    CheckoutFailed { branch: String, message: String },

    #[error("failed to pull from remote: {0}")]
    PullFailed(String),

    #[error("failed to reset repository: {0}")]
    ResetFailed(String),

    #[error("failed to execute git command: {0}")]
    GitCommandFailed(String),

    #[allow(dead_code)]
    #[error("failed to change directory to '{0}'")]
    ChangeDirectoryFailed(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("operation aborted by user")]
    Aborted,

    #[allow(dead_code)]
    #[error("could not detect default branch for repository")]
    DefaultBranchNotFound,
}

pub type Result<T> = std::result::Result<T, ArcError>;


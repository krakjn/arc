use std::fmt;
use std::io;
use thiserror::Error;

/// Kind of failing external command (for `CommandFailed` messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOp {
    Clone,
    Pull,
    Reset,
    Fetch,
    Git,
    Shell,
    Status,
    Sync,
    Checkout,
}

impl CommandOp {
    fn verb(self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Pull => "pull",
            Self::Reset => "reset",
            Self::Fetch => "fetch",
            Self::Git => "run git",
            Self::Shell => "run command",
            Self::Status => "status",
            Self::Sync => "sync",
            Self::Checkout => "checkout",
        }
    }
}

/// Structured command failure shown to the user.
#[derive(Debug)]
pub struct CommandFailure {
    pub op: CommandOp,
    pub repo: Option<String>,
    pub message: String,
}

impl fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.op, self.repo.as_deref()) {
            (CommandOp::Clone, Some(repo)) => {
                write!(f, "failed to clone repository '{repo}': {}", self.message)
            }
            (CommandOp::Pull, repo) => match repo {
                Some(r) => write!(f, "failed to pull from remote in '{r}': {}", self.message),
                None => write!(f, "failed to pull from remote: {}", self.message),
            },
            (CommandOp::Reset, repo) => match repo {
                Some(r) => write!(f, "failed to reset repository '{r}': {}", self.message),
                None => write!(f, "failed to reset repository: {}", self.message),
            },
            (op, Some(repo)) => {
                write!(f, "failed to {} in '{repo}': {}", op.verb(), self.message)
            }
            (op, None) => write!(f, "failed to {}: {}", op.verb(), self.message),
        }
    }
}

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

    #[error("{0}")]
    CommandFailed(CommandFailure),

    #[error("{0} repo(s) failed")]
    ReposFailed(usize),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("operation aborted by user")]
    Aborted,
}

impl ArcError {
    pub fn command(op: CommandOp, repo: impl Into<Option<String>>, message: impl ToString) -> Self {
        Self::CommandFailed(CommandFailure {
            op,
            repo: repo.into(),
            message: message.to_string(),
        })
    }
}

pub type Result<T> = std::result::Result<T, ArcError>;

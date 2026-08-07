use crate::error::{ArcError, Result};
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Run `git` with the given args in `dir`.
pub fn run(dir: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| ArcError::CommandFailed(e.to_string()))
}

/// Run `git` inheriting stdout/stderr (for clone progress).
pub fn run_inherit(dir: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| ArcError::CommandFailed(e.to_string()))
}

/// Run an arbitrary command in `dir`.
pub fn run_cmd(dir: &Path, program: &str, args: &[String]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| ArcError::CommandFailed(e.to_string()))
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Detect the default branch for a repository at `dir`.
pub fn detect_default_branch(dir: &Path) -> String {
    if let Ok(output) = run(dir, &["symbolic-ref", "refs/remotes/origin/HEAD"])
        && output.status.success()
    {
        let stdout = stdout(&output);
        if let Some(branch) = stdout.trim().strip_prefix("refs/remotes/origin/") {
            return branch.to_string();
        }
    }

    if let Ok(output) = run(dir, &["branch", "-r"]) {
        let text = stdout(&output);
        if text.contains("origin/main") {
            return "main".to_string();
        }
        if text.contains("origin/master") {
            return "master".to_string();
        }
    }

    "master".to_string()
}

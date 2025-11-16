use crate::error::{ArcError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Lazy compiled regex patterns - compile once, use everywhere
pub static BRANCH_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^## (.*?)\..*$").unwrap());
pub static UNTRACKED_BRANCH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^## (.*?)$").unwrap());
pub static AHEAD: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^##.*\[ahead (\d+)\].*$").unwrap());
pub static BEHIND: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^##.*\[behind (\d+)\].*$").unwrap());
pub static DIVERGED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^##.*\[ahead (\d+), behind (\d+)\].*$").unwrap());
pub static CHANGED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^[^#]{2} .*$").unwrap());
pub static STAGED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)[A-Z]. .*$").unwrap());
pub static UNSTAGED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m).[A-Z] .*$").unwrap());
pub static UNTRACKED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\?\? (.*)$").unwrap());
pub static ADDED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^A  (.*)$").unwrap());
pub static MODIFIED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^.M (.*)$").unwrap());
pub static MODIFIED_STAGED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^M  (.*)$").unwrap());
pub static DELETE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^ D (.*)$").unwrap());
pub static DELETE_STAGED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^D  (.*)$").unwrap());
pub static ROOT_DIR: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z]:\\+$|/$").unwrap());
pub static SWITCHED: Lazy<Regex> = Lazy::new(|| Regex::new(r"Switch.*'(.*?)'").unwrap());
pub static ALREADY: Lazy<Regex> = Lazy::new(|| Regex::new(r".*[A|a]lready.*").unwrap());
pub static UP_TO_DATE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*up.to.date.*").unwrap());
pub static UPDATING: Lazy<Regex> = Lazy::new(|| Regex::new(r".*Updating.*").unwrap());
pub static CONFLICT: Lazy<Regex> = Lazy::new(|| Regex::new(r".*conflict.*").unwrap());
pub static RESET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"at [[:alnum:]]+ (.*)").unwrap());
pub static FATAL: Lazy<Regex> = Lazy::new(|| Regex::new(r".*[f|F]+atal.*").unwrap());
pub static BRANCH_QUOTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"'(.*?)'").unwrap());
pub static FETCHED: Lazy<Regex> = Lazy::new(|| Regex::new(r"remote branch '(.*?)'").unwrap());

/// Find .git directory by traversing up the directory tree
pub fn find_git() -> bool {
    let start_dir = env::current_dir().unwrap();
    let mut cwd = start_dir.clone();

    loop {
        if ROOT_DIR.is_match(&cwd.to_string_lossy()) {
            let _ = env::set_current_dir(&start_dir);
            return false;
        }

        if let Ok(entries) = fs::read_dir(&cwd) {
            let names: Vec<String> = entries
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str().map(|s| s.to_string()))
                    })
                })
                .collect();

            if names.contains(&".git".to_string()) {
                let _ = env::set_current_dir(&start_dir);
                return true;
            }
        }

        if !cwd.pop() {
            let _ = env::set_current_dir(&start_dir);
            return false;
        }
    }
}

/// Find arc.json by traversing up the directory tree and load its path
pub fn find_arc(tld: &mut PathBuf, arc_path: &mut PathBuf) -> Result<()> {
    let start_dir = env::current_dir()?;
    let mut cwd = start_dir.clone();

    loop {
        if ROOT_DIR.is_match(&cwd.to_string_lossy()) {
            env::set_current_dir(&start_dir)?;
            return Err(ArcError::ConfigNotFound);
        }

        if let Ok(entries) = fs::read_dir(&cwd) {
            let names: Vec<String> = entries
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str().map(|s| s.to_string()))
                    })
                })
                .collect();

            if names.contains(&"arc.json".to_string()) {
                *tld = cwd.clone();
                *arc_path = cwd.join("arc.json");
                env::set_current_dir(&start_dir)?;
                return Ok(());
            }
        }

        if !cwd.pop() {
            env::set_current_dir(&start_dir)?;
            return Err(ArcError::ConfigNotFound);
        }
    }
}

/// Parse a repository name into (relative_path, short_name)
/// e.g., "path/to/repo" -> ("path/to", "repo")
pub fn clone_path(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.split(|c| c == '/' || c == '\\').collect();
    let short_name = parts.last().unwrap_or(&"").to_string();

    if parts.len() == 1 {
        return ("shallow".to_string(), short_name);
    }

    let relative_path = parts[..parts.len() - 1].join("/");
    (relative_path, short_name)
}

/// Detect the default branch for the current repository
/// Tries to query git for the default branch, falls back to master
pub fn detect_default_branch() -> Result<String> {
    // Try to get the symbolic ref for origin/HEAD
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(branch) = stdout.trim().strip_prefix("refs/remotes/origin/") {
                return Ok(branch.to_string());
            }
        }
    }

    // Fallback: check for main or master in remote branches
    let output = Command::new("git")
        .args(["branch", "-r"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("origin/main") {
            return Ok("main".to_string());
        } else if stdout.contains("origin/master") {
            return Ok("master".to_string());
        }
    }

    // Last resort: default to master
    Ok("master".to_string())
}



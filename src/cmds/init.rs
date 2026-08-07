use crate::defs::{Repo, Workspace};
use crate::error::{ArcError, Result};
use crate::git;
use crate::utils;
use owo_colors::OwoColorize;
use std::fs;

/// Initialize all repositories defined in arc.toml
pub fn init(ws: &Workspace) -> Result<()> {
    for repo in &ws.repos {
        clone_repo(ws, repo)?;
    }

    println!(
        "{} {}",
        ">>>".cyan().bold(),
        "arc initialized".green().bold()
    );
    Ok(())
}

pub(crate) fn clone_repo(ws: &Workspace, repo: &Repo) -> Result<()> {
    let dest = ws.repo_dir(repo);
    let (parent, short_name) = utils::clone_destination(&ws.path.tld, &repo.path);
    fs::create_dir_all(&parent)?;

    let repo_url = Workspace::clone_url(repo);
    let fmt_str = format!("loading >>> {:_>50}", short_name);
    println!("{}", fmt_str.cyan().bold());

    // Clone into the exact destination path (supports `..` and absolute paths).
    let output = git::run_inherit(
        &parent,
        &["clone", "--progress", &repo_url, &dest.to_string_lossy()],
    )
    .map_err(|e| ArcError::CloneFailed {
        repo: repo.path.clone(),
        message: e.to_string(),
    })?;

    if !output.status.success() {
        return Err(ArcError::CloneFailed {
            repo: repo.path.clone(),
            message: git::stderr(&output),
        });
    }

    Ok(())
}

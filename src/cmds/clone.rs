use crate::defs::{Repo, Workspace};
use crate::error::{ArcError, CommandOp, Result};
use crate::git;
use crate::run::{MissingDir, RepoCtx, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;
use std::fs;

/// Clone all repositories defined in arc.toml (skips existing git worktrees).
pub fn clone(ws: &Workspace, opts: &RunOpts) -> Result<()> {
    for_each_repo(ws, opts, MissingDir::Call, clone_one)?;
    println!(
        "{} {}",
        ">>>".cyan().bold(),
        "arc clone finished".green().bold()
    );
    Ok(())
}

pub(crate) fn clone_one(ctx: &RepoCtx<'_>) -> Result<()> {
    clone_repo(ctx.ws, ctx.repo, ctx)
}

pub(crate) fn clone_repo(ws: &Workspace, repo: &Repo, ctx: &RepoCtx<'_>) -> Result<()> {
    let dest = ws.repo_dir(repo);
    if dest.is_dir() && utils::is_git_repo(&dest) {
        ctx.emit(|| {
            print!("{}", utils::format_path_column(&repo.path, ctx.path_col));
            println!("{}", "skipped (already cloned)".yellow());
        });
        return Ok(());
    }

    let (parent, short_name) = utils::clone_destination(&ws.path.tld, &repo.path);
    fs::create_dir_all(&parent)?;

    let repo_url = Workspace::clone_url(repo);
    ctx.emit(|| {
        print!("{}", utils::format_path_column(&repo.path, ctx.path_col));
        println!("{}", format!("cloning {short_name}").cyan().bold());
    });

    let mode = ctx.stream_mode();
    let output = git::run(
        &parent,
        &["clone", "--progress", &repo_url, &dest.to_string_lossy()],
        mode,
    )
    .map_err(|e| ArcError::command(CommandOp::Clone, Some(repo.path.clone()), e.to_string()))?;

    if !output.status.success() {
        return Err(ArcError::command(
            CommandOp::Clone,
            Some(repo.path.clone()),
            git::stderr(&output),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{ArcPath, Repo, Workspace};
    use crate::run::{MissingDir, RunOpts, for_each_repo};

    #[test]
    fn skip_if_already_cloned() {
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("arc");
        fs::create_dir_all(repo_dir.join(".git")).unwrap();

        let ws = Workspace {
            path: ArcPath {
                tld: dir.path().to_path_buf(),
                config: dir.path().join("arc.toml"),
            },
            repos: vec![Repo {
                path: "arc".into(),
                remote: "origin".into(),
                url: "http://example.invalid/arc.git".into(),
                deps: vec![],
            }],
        };
        clone(&ws, &RunOpts::new(vec![], Some(1), false)).unwrap();
    }

    #[test]
    fn missing_dir_fails_sweep() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace {
            path: ArcPath {
                tld: dir.path().to_path_buf(),
                config: dir.path().join("arc.toml"),
            },
            repos: vec![Repo {
                path: "missing".into(),
                remote: "origin".into(),
                url: "http://example.invalid/x.git".into(),
                deps: vec![],
            }],
        };
        let err = for_each_repo(
            &ws,
            &RunOpts::new(vec![], Some(1), false),
            MissingDir::Fail,
            |_| Ok(()),
        )
        .unwrap_err();
        assert!(err.to_string().contains("failed"), "{err}");
    }
}

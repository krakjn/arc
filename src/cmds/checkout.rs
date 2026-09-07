use crate::cmds::status;
use crate::config;
use crate::defs::{Repo, Workspace};
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::path::Path;

/// How to align a repo onto `branch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackAction {
    /// `git checkout --track {remote}/{branch}`
    Track,
    /// `git checkout {branch}`
    Local,
    /// Neither remote nor local ref exists
    Missing,
}

pub fn track_action(remote_exists: bool, local_exists: bool) -> TrackAction {
    if remote_exists && !local_exists {
        TrackAction::Track
    } else if local_exists {
        TrackAction::Local
    } else {
        TrackAction::Missing
    }
}

fn resolve_root<'a>(ws: &'a Workspace, repo: Option<&str>) -> Result<&'a Repo> {
    if let Some(path) = repo {
        return config::repo_by_path(ws, path).ok_or_else(|| {
            ArcError::ConfigValidation(format!(
                "unknown repo path '{path}' (must match path in arc.toml exactly)"
            ))
        });
    }

    let start_dir = env::current_dir()?;
    config::repo_for_cwd(ws, &start_dir).ok_or_else(|| {
        ArcError::ConfigValidation("current directory is not inside a configured repo".into())
    })
}

fn tree_repos<'a>(ws: &'a Workspace, root: &'a Repo) -> Vec<&'a Repo> {
    let mut out = Vec::new();
    for dep in &root.deps {
        if utils::path_has_parent_dir(dep) {
            eprintln!(
                "{} skipping dependency '{dep}' (path contains '..')",
                utils::paint("warn:", |s| s.yellow().bold().to_string())
            );
            continue;
        }
        if let Some(r) = config::repo_by_path(ws, dep) {
            out.push(r);
        } else {
            eprintln!(
                "{} unknown dependency '{dep}'",
                utils::paint("Error:", |s| s.red().bold().to_string())
            );
        }
    }
    out.push(root);
    out
}

/// Scoped status (no branch) or align the selected repo + deps onto `branch`.
pub fn checkout(
    ws: &Workspace,
    branch: Option<&str>,
    repo: Option<&str>,
    sort: bool,
) -> Result<()> {
    let root = resolve_root(ws, repo)?;
    match branch {
        Some(b) if !b.is_empty() => align_tree(ws, root, b, sort),
        _ => scoped_status(ws, root, sort),
    }
}

fn scoped_status(ws: &Workspace, root: &Repo, sort: bool) -> Result<()> {
    let mut members = tree_repos(ws, root);
    if sort {
        members.sort_by(|a, b| a.path.cmp(&b.path));
    }
    let (path_col, branch_col) = utils::line_layout(members.iter().map(|r| r.path.as_str()));
    let root_dir = ws.repo_dir(root);
    let root_branch = git::current_branch(&root_dir).ok();

    let mut failures = 0usize;
    for repo in members {
        let dir = ws.repo_dir(repo);
        if !dir.is_dir() {
            eprintln!(
                "{} {}",
                utils::paint("Cannot find directory:", |s| s
                    .red()
                    .underline()
                    .to_string()),
                repo.path
            );
            failures += 1;
            continue;
        }
        let peer = if repo.path == root.path {
            None
        } else {
            root_branch.as_deref()
        };
        if let Err(e) = status::print_one(&dir, repo, path_col, branch_col, false, peer) {
            eprintln!(
                "{} {e}",
                utils::paint("Error:", |s| s.red().bold().to_string())
            );
            failures += 1;
        }
    }

    if failures > 0 {
        Err(ArcError::ReposFailed(failures))
    } else {
        Ok(())
    }
}

fn align_tree(ws: &Workspace, root: &Repo, branch: &str, sort: bool) -> Result<()> {
    let mut members = tree_repos(ws, root);
    if sort {
        members.sort_by(|a, b| a.path.cmp(&b.path));
    }
    let col = utils::line_layout(members.iter().map(|r| r.path.as_str())).0;

    let mut failures = 0usize;
    for repo in members {
        let is_dep = repo.path != root.path;
        let dir = ws.repo_dir(repo);
        if !dir.is_dir() {
            eprintln!(
                "{} {}",
                utils::paint("Cannot find directory:", |s| s
                    .red()
                    .underline()
                    .to_string()),
                repo.path
            );
            failures += 1;
            continue;
        }
        if let Err(e) = checkout_one(&dir, &repo.remote, branch, &repo.path, is_dep, col) {
            eprintln!(
                "{} {e}",
                utils::paint("Error:", |s| s.red().bold().to_string())
            );
            failures += 1;
        }
    }

    if failures > 0 {
        Err(ArcError::ReposFailed(failures))
    } else {
        Ok(())
    }
}

fn checkout_one(
    dir: &Path,
    remote: &str,
    branch: &str,
    name: &str,
    is_dep: bool,
    col: usize,
) -> Result<()> {
    let remote_ref = format!("refs/remotes/{remote}/{branch}");
    let local_ref = format!("refs/heads/{branch}");
    let action = track_action(
        git::ref_exists(dir, &remote_ref),
        git::ref_exists(dir, &local_ref),
    );

    match action {
        TrackAction::Missing if is_dep => fallback_dep(dir, remote, name, col),
        TrackAction::Missing => {
            print_tag(name, col, is_dep);
            println!(
                "{}",
                utils::paint("missing", |s| s.red().bold().to_string())
            );
            Err(ArcError::command(
                CommandOp::Checkout,
                Some(name.to_string()),
                format!("no local '{branch}' and no '{remote}/{branch}'"),
            ))
        }
        TrackAction::Track => {
            let tracked = format!("{remote}/{branch}");
            let output = git::run(dir, &["checkout", "--track", &tracked], StdioMode::Capture)?;
            print_result(name, col, is_dep, &output, branch, "tracked-> ")
        }
        TrackAction::Local => {
            let output = git::run(dir, &["checkout", branch], StdioMode::Capture)?;
            print_result(name, col, is_dep, &output, branch, "on-> ")
        }
    }
}

fn fallback_dep(dir: &Path, remote: &str, name: &str, col: usize) -> Result<()> {
    if let Some(default) = git::remote_head_branch(dir, remote) {
        let output = git::run(dir, &["checkout", &default], StdioMode::Capture)?;
        return print_result(name, col, true, &output, &default, "default-> ");
    }

    let head = git::current_branch(dir).unwrap_or_else(|_| "HEAD".into());
    print_tag(name, col, true);
    print!(
        "{}",
        utils::paint("on-> ", |s| s.yellow().bold().to_string())
    );
    println!("{head}");
    Ok(())
}

fn print_tag(name: &str, col: usize, is_dep: bool) {
    print!(
        "{}{}",
        utils::paint(if is_dep { " dep " } else { " >>> " }, |s| {
            if is_dep {
                s.magenta().bold().to_string()
            } else {
                s.cyan().bold().to_string()
            }
        }),
        utils::format_path_column(name, col)
    );
}

fn print_result(
    name: &str,
    col: usize,
    is_dep: bool,
    output: &std::process::Output,
    branch: &str,
    ok_label: &str,
) -> Result<()> {
    let stdout = git::stdout(output);
    let stderr = git::stderr(output);
    print_tag(name, col, is_dep);

    if output.status.success() {
        print!(
            "{}",
            utils::paint(ok_label, |s| s.cyan().bold().to_string())
        );
        println!("{branch}");
        Ok(())
    } else {
        println!("{}", utils::paint("unable", |s| s.red().bold().to_string()));
        eprint!("{}", utils::paint(&stderr, |s| s.red().to_string()));
        Err(ArcError::command(
            CommandOp::Checkout,
            Some(name.to_string()),
            if stderr.is_empty() { stdout } else { stderr },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_when_only_remote() {
        assert_eq!(track_action(true, false), TrackAction::Track);
    }

    #[test]
    fn local_when_already_have_branch() {
        assert_eq!(track_action(true, true), TrackAction::Local);
        assert_eq!(track_action(false, true), TrackAction::Local);
    }

    #[test]
    fn missing_when_neither() {
        assert_eq!(track_action(false, false), TrackAction::Missing);
    }
}

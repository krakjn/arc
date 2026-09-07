use crate::defs::{Repo, Workspace};
use crate::error::{ArcError, Result};
use crate::git::StdioMode;
use crate::utils::{self, path_has_parent_dir};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

/// Filters and parallelism for a multi-repo command.
#[derive(Debug, Clone)]
pub struct RunOpts {
    pub only: Vec<String>,
    pub jobs: usize,
    pub sort: bool,
}

impl RunOpts {
    pub fn new(only: Vec<String>, jobs: Option<usize>, sort: bool) -> Self {
        Self {
            only,
            jobs: jobs.unwrap_or_else(utils::default_jobs).max(1),
            sort,
        }
    }
}

struct PrintOrder {
    next: Mutex<usize>,
    cv: Condvar,
}

/// Whether a missing worktree should fail the sweep or still invoke the callback.
#[derive(Debug, Clone, Copy)]
pub enum MissingDir {
    Fail,
    Call,
}

/// Per-repo context passed to `for_each_repo` callbacks.
pub struct RepoCtx<'a> {
    pub ws: &'a Workspace,
    pub repo: &'a Repo,
    pub dir: PathBuf,
    pub path_col: usize,
    pub branch_col: usize,
    pub jobs: usize,
    index: usize,
    lock: &'a Mutex<()>,
    order: Option<&'a PrintOrder>,
    order_taken: AtomicBool,
}

impl<'a> RepoCtx<'a> {
    /// Print a complete structured line in list order (config or `--sort`).
    pub fn emit<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.order_taken.store(true, Ordering::Relaxed);
        if let Some(order) = self.order {
            let mut next = order.next.lock().unwrap_or_else(|e| e.into_inner());
            while *next != self.index {
                next = order.cv.wait(next).unwrap_or_else(|e| e.into_inner());
            }
            let out = f();
            *next += 1;
            order.cv.notify_all();
            out
        } else {
            let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
            f()
        }
    }

    fn release_turn(&self) {
        if self.order.is_some() && !self.order_taken.load(Ordering::Relaxed) {
            self.emit(|| ());
        }
    }

    pub fn stream_mode(&self) -> StdioMode<'_> {
        if self.jobs <= 1 {
            StdioMode::Inherit
        } else {
            StdioMode::Prefix {
                label: &self.repo.path,
                width: self.path_col,
                lock: self.lock,
            }
        }
    }

    pub fn capture_mode() -> StdioMode<'static> {
        StdioMode::Capture
    }
}

/// Select repos (`--only` exact path), skip `..`, optionally require a directory,
/// then run `f` sequentially or with `--jobs` workers.
pub fn for_each_repo<F>(ws: &Workspace, opts: &RunOpts, missing: MissingDir, f: F) -> Result<()>
where
    F: Fn(&RepoCtx<'_>) -> Result<()> + Send + Sync,
{
    let selected = select_repos(ws, &opts.only, opts.sort)?;
    let (path_col, branch_col) = utils::line_layout(selected.iter().map(|r| r.path.as_str()));
    let prefix_width = selected
        .iter()
        .map(|r| r.path.chars().count())
        .max()
        .unwrap_or(0)
        .max(1);

    let lock = Mutex::new(());
    let order = PrintOrder {
        next: Mutex::new(0),
        cv: Condvar::new(),
    };
    let mut failures = 0usize;
    let mut work: Vec<(&Repo, PathBuf)> = Vec::new();

    for repo in selected {
        if path_has_parent_dir(&repo.path) {
            eprintln!(
                "{} skipping '{}' (path contains '..'; repos must live next to or below arc.toml)",
                "warn:".yellow().bold(),
                repo.path
            );
            continue;
        }

        let dir = ws.repo_dir(repo);
        if matches!(missing, MissingDir::Fail) && !dir.is_dir() {
            eprintln!(
                "{} {}",
                "Cannot find directory:".red().underline(),
                repo.path
            );
            failures += 1;
            continue;
        }
        work.push((repo, dir));
    }

    let path_col = path_col.max(prefix_width);

    if opts.jobs <= 1 || work.len() <= 1 {
        for (index, (repo, dir)) in work.into_iter().enumerate() {
            let ctx = RepoCtx {
                ws,
                repo,
                dir,
                path_col,
                branch_col,
                jobs: opts.jobs,
                index,
                lock: &lock,
                order: None,
                order_taken: AtomicBool::new(false),
            };
            if let Err(e) = f(&ctx) {
                eprintln!("{} {e}", "Error:".red().bold());
                failures += 1;
            }
            ctx.release_turn();
        }
    } else {
        let indexed: Vec<(usize, &Repo, PathBuf)> = work
            .into_iter()
            .enumerate()
            .map(|(i, (r, d))| (i, r, d))
            .collect();
        let queue = Mutex::new(indexed.into_iter());
        let fail_count = Mutex::new(0usize);
        let jobs = opts.jobs.clamp(1, 32);

        std::thread::scope(|s| {
            for _ in 0..jobs {
                s.spawn(|| {
                    loop {
                        let next = queue.lock().unwrap_or_else(|e| e.into_inner()).next();
                        let Some((index, repo, dir)) = next else {
                            break;
                        };
                        let ctx = RepoCtx {
                            ws,
                            repo,
                            dir,
                            path_col,
                            branch_col,
                            jobs: opts.jobs,
                            index,
                            lock: &lock,
                            order: Some(&order),
                            order_taken: AtomicBool::new(false),
                        };
                        if let Err(e) = f(&ctx) {
                            let _g = lock.lock().unwrap_or_else(|e| e.into_inner());
                            eprintln!("{} {e}", "Error:".red().bold());
                            *fail_count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                        }
                        ctx.release_turn();
                    }
                });
            }
        });
        failures += *fail_count.lock().unwrap_or_else(|e| e.into_inner());
    }

    if failures > 0 {
        Err(ArcError::ReposFailed(failures))
    } else {
        Ok(())
    }
}

fn select_repos<'a>(ws: &'a Workspace, only: &[String], sort: bool) -> Result<Vec<&'a Repo>> {
    let mut out = if only.is_empty() {
        ws.repos.iter().collect()
    } else {
        let mut selected = Vec::with_capacity(only.len());
        for name in only {
            match ws.repos.iter().find(|r| r.path == *name) {
                Some(r) => selected.push(r),
                None => {
                    return Err(ArcError::ConfigValidation(format!(
                        "unknown repo path '{name}' (must match path in arc.toml exactly)"
                    )));
                }
            }
        }
        selected
    };
    if sort {
        out.sort_by(|a, b| a.path.cmp(&b.path));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{ArcPath, Repo, Workspace};

    fn ws(paths: &[&str]) -> Workspace {
        Workspace {
            path: ArcPath::default(),
            repos: paths
                .iter()
                .map(|p| Repo {
                    path: (*p).into(),
                    remote: "origin".into(),
                    url: "http://x".into(),
                    deps: vec![],
                })
                .collect(),
        }
    }

    #[test]
    fn only_exact_path() {
        let ws = ws(&["path/arc", "jetsam"]);
        let got = select_repos(&ws, &["jetsam".into()], false).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, "jetsam");
    }

    #[test]
    fn only_unknown_errors() {
        let ws = ws(&["path/arc"]);
        assert!(select_repos(&ws, &["arc".into()], false).is_err());
    }

    #[test]
    fn sort_is_alphanumeric_by_path() {
        let ws = ws(&["zeta", "alpha", "mid"]);
        let listed = select_repos(&ws, &[], false).unwrap();
        assert_eq!(
            listed.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
            ["zeta", "alpha", "mid"]
        );
        let sorted = select_repos(&ws, &[], true).unwrap();
        assert_eq!(
            sorted.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
            ["alpha", "mid", "zeta"]
        );
    }

    #[test]
    fn parent_dir_paths_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace {
            path: crate::defs::ArcPath {
                tld: dir.path().to_path_buf(),
                config: dir.path().join("arc.toml"),
            },
            repos: vec![Repo {
                path: "../escape".into(),
                remote: "origin".into(),
                url: "http://x".into(),
                deps: vec![],
            }],
        };
        let called = std::sync::atomic::AtomicBool::new(false);
        for_each_repo(
            &ws,
            &RunOpts::new(vec![], Some(1), false),
            MissingDir::Call,
            |_| {
                called.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            },
        )
        .unwrap();
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
    }
}

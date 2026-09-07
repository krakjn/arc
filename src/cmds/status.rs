use crate::defs::{Repo, Workspace};
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::parse::{self, PorcelainFiles};
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;
use std::path::Path;

/// Display status of all repositories.
pub fn status(ws: &Workspace, verbose: bool, fetch: bool, opts: &RunOpts) -> Result<()> {
    for_each_repo(ws, opts, MissingDir::Fail, |ctx| {
        if fetch {
            let output = git::run(&ctx.dir, &["fetch", &ctx.repo.remote], StdioMode::Capture)
                .map_err(|e| {
                    ArcError::command(CommandOp::Fetch, Some(ctx.repo.path.clone()), e.to_string())
                })?;
            if !output.status.success() {
                return Err(ArcError::command(
                    CommandOp::Fetch,
                    Some(ctx.repo.path.clone()),
                    git::stderr(&output),
                ));
            }
        }

        ctx.emit(|| {
            print_one(
                &ctx.dir,
                ctx.repo,
                ctx.path_col,
                ctx.branch_col,
                verbose,
                None,
            )
        })
    })
}

/// One status row. When `root_branch` is set and this repo is on a different
/// branch, show HEAD (yellow) and a default-branch hint if known.
pub(crate) fn print_one(
    dir: &Path,
    repo: &Repo,
    path_col: usize,
    branch_col: usize,
    verbose: bool,
    root_branch: Option<&str>,
) -> Result<()> {
    let output = git::run(dir, &["status", "-b", "--porcelain"], StdioMode::Capture)?;
    let stdout = git::stdout(&output);

    let Some(header) = parse::parse_branch_header(&stdout) else {
        print!("{}", utils::format_path_column(&repo.path, path_col));
        print!(
            "{}",
            utils::format_branch_column("unhandled state", branch_col)
        );
        println!();
        return Ok(());
    };

    let default_branch = git::remote_head_branch(dir, &repo.remote);
    let files = parse::parse_porcelain_files(&stdout);
    let diverged_from_root = root_branch.is_some_and(|root| root != header.name.as_str());

    print!("{}", utils::format_path_column(&repo.path, path_col));
    if default_branch.as_deref() == Some(header.name.as_str()) && !diverged_from_root {
        print!("{}", utils::format_branch_column(&header.name, branch_col));
    } else {
        let label = utils::format_branch_column(&header.name, branch_col);
        let trimmed = label.trim_end();
        print!("{} ", utils::paint(trimmed, |s| s.yellow().to_string()));
        if diverged_from_root
            && let Some(def) = default_branch.as_deref()
            && def != header.name
        {
            print!(
                "{}",
                utils::paint(&format!("(default {def}) "), |s| s.magenta().to_string())
            );
        }
    }

    if header.ahead > 0 && header.behind > 0 {
        let diverged = format!("↑{},↓{} ", header.ahead, header.behind);
        print!("{}", diverged.bright_magenta().bold());
    } else {
        if header.ahead > 0 {
            print!("{}", format!("↑{} ", header.ahead).cyan().bold());
        }
        if header.behind > 0 {
            print!("{}", format!("↓{} ", header.behind).cyan().bold());
        }
    }

    if verbose {
        display_verbose(header.tracked, &files);
    } else {
        display_simple(header.tracked, &files);
    }
    Ok(())
}

fn display_verbose(tracked: bool, files: &PorcelainFiles) {
    if !tracked {
        print!("{}", "not pushed ".bright_magenta().underline());
    }

    if !files.has_changes() {
        println!("{}", "clean".green().bold());
        return;
    }

    println!("{}", "changed".red().bold());

    fn row(label: &str, paths: &[String], color_green: bool) {
        for path in paths {
            let tag = format!("{label:>15}");
            if color_green {
                print!("{}", tag.green().bold());
            } else {
                print!("{}", tag.red().bold());
            }
            println!("{path}");
        }
    }

    row("deleted: ", &files.staged_deleted, true);
    row("added: ", &files.staged_added, true);
    row("modified: ", &files.staged_modified, true);
    row("renamed: ", &files.renamed, true);
    row("copied: ", &files.copied, true);
    row("typechanged: ", &files.typechanged, true);
    row("deleted: ", &files.unstaged_deleted, false);
    row("modified: ", &files.unstaged_modified, false);
    for path in &files.untracked {
        print!("{}", format!("{:>15}", "untracked: ").yellow().bold());
        println!("{path}");
    }
}

fn display_simple(tracked: bool, files: &PorcelainFiles) {
    if !tracked {
        print!("{}", "!".bright_magenta().bold());
    }

    if !files.has_changes() {
        println!("{}", "✔ ".green().bold());
        return;
    }

    let mut partial_clean = true;
    if !files.untracked.is_empty() {
        print!("{}", "?".yellow().bold());
    }
    if files.has_staged() {
        print!("{}", "✚".green().bold());
        partial_clean = false;
    }
    if files.has_unstaged() {
        print!("{}", "✘".red().bold());
        partial_clean = false;
    }

    if partial_clean {
        println!("{}", "✔ ".green().bold());
    } else {
        println!();
    }
}

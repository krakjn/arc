use crate::defs::Workspace;
use crate::error::Result;
use crate::git;
use crate::parse::{self, PorcelainFiles};
use crate::utils;
use owo_colors::OwoColorize;

/// Display status of all repositories.
/// Exits successfully even when repos are dirty/behind; missing dirs are reported and skipped.
pub fn status(ws: &Workspace, verbose: bool) -> Result<()> {
    let (path_col, branch_col) = utils::line_layout(ws.repos.iter().map(|r| r.path.as_str()));

    for repo in &ws.repos {
        let dir = ws.repo_dir(repo);
        if !dir.is_dir() {
            eprintln!(
                "{} {}",
                "Cannot find directory:".red().underline(),
                &repo.path
            );
            continue;
        }

        let output = git::run(&dir, &["status", "-b", "--porcelain"])?;
        let stdout = git::stdout(&output);

        print!("{}", utils::format_path_column(&repo.path, path_col));

        let Some(header) = parse::parse_branch_header(&stdout) else {
            print!("{}", utils::format_branch_column("unhandled state", branch_col));
            println!();
            continue;
        };

        let default_branch = git::detect_default_branch(&dir);
        if header.name == default_branch {
            print!("{}", utils::format_branch_column(&header.name, branch_col));
        } else {
            // Same layout as default, but color the branch name.
            let label = utils::format_branch_column(&header.name, branch_col);
            let trimmed = label.trim_end();
            print!("{} ", trimmed.yellow());
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

        let files = parse::parse_porcelain_files(&stdout);
        if verbose {
            display_verbose(header.tracked, &files);
        } else {
            display_simple(header.tracked, &files);
        }
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

    for path in &files.staged_deleted {
        print!("{}", format!("{:>15}", "deleted: ").green().bold());
        println!("{path}");
    }
    for path in &files.staged_added {
        print!("{}", format!("{:>15}", "added: ").green().bold());
        println!("{path}");
    }
    for path in &files.staged_modified {
        print!("{}", format!("{:>15}", "modified: ").green().bold());
        println!("{path}");
    }
    for path in &files.unstaged_deleted {
        print!("{}", format!("{:>15}", "deleted: ").red().bold());
        println!("{path}");
    }
    for path in &files.unstaged_modified {
        print!("{}", format!("{:>15}", "modified: ").red().bold());
        println!("{path}");
    }
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

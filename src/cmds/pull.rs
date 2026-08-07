use crate::defs::Workspace;
use crate::error::{ArcError, Result};
use crate::git;
use crate::parse::{self, PullResult};
use crate::utils;
use owo_colors::OwoColorize;

/// Pull changes from remote for all repositories
pub fn pull(ws: &Workspace) -> Result<()> {
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

        let status = git::run(&dir, &["status", "-bs"])
            .map_err(|e| ArcError::CommandFailed(e.to_string()))?;
        let branchout = git::stdout(&status);

        print!("{}", utils::format_path_column(&repo.path, path_col));

        let Some(header) = parse::parse_branch_header(&branchout) else {
            println!("{}", utils::format_branch_column("not on a branch?", branch_col));
            continue;
        };

        print!("{}", utils::format_branch_column(&header.name, branch_col));
        if !header.tracked {
            println!("{}", "untracked".magenta().bold());
            continue;
        }

        let output = git::run(&dir, &["pull"]).map_err(|e| ArcError::PullFailed(e.to_string()))?;
        let stdout = git::stdout(&output);
        let stderr = git::stderr(&output);

        print_pull_result(parse::classify_pull(&stdout, &stderr), &stdout, &stderr);
    }

    Ok(())
}

pub(crate) fn print_pull_result(result: PullResult, stdout: &str, stderr: &str) {
    match result {
        PullResult::UpToDate => println!("{}", "up-to-date".green().bold()),
        PullResult::Updating => {
            println!("{}", "updating".green().bold());
            print!("{stdout}");
        }
        PullResult::Conflict => {
            println!("{}", "conflict".yellow().bold());
            print!("{stdout}");
        }
        PullResult::Unable => {
            println!("{}", "unable".red().bold());
            print!("{stderr}");
        }
        PullResult::Ok => println!("{}", "ok".green()),
    }
}

use crate::defs::Workspace;
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::parse::{self, PullResult};
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;

/// Pull changes from remote for all repositories
pub fn pull(ws: &Workspace, opts: &RunOpts) -> Result<()> {
    for_each_repo(ws, opts, MissingDir::Fail, |ctx| {
        let status = git::run(&ctx.dir, &["status", "-bs"], StdioMode::Capture).map_err(|e| {
            ArcError::command(CommandOp::Pull, Some(ctx.repo.path.clone()), e.to_string())
        })?;
        let branchout = git::stdout(&status);

        let Some(header) = parse::parse_branch_header(&branchout) else {
            ctx.emit(|| {
                print!(
                    "{}",
                    utils::format_path_column(&ctx.repo.path, ctx.path_col)
                );
                println!(
                    "{}",
                    utils::format_branch_column("not on a branch?", ctx.branch_col)
                );
            });
            return Ok(());
        };

        if !header.tracked {
            ctx.emit(|| {
                print!(
                    "{}",
                    utils::format_path_column(&ctx.repo.path, ctx.path_col)
                );
                print!(
                    "{}",
                    utils::format_branch_column(&header.name, ctx.branch_col)
                );
                println!("{}", "no-upstream".magenta().bold());
            });
            return Ok(());
        }

        let output = git::run(&ctx.dir, &["pull"], StdioMode::Capture).map_err(|e| {
            ArcError::command(CommandOp::Pull, Some(ctx.repo.path.clone()), e.to_string())
        })?;
        let stdout = git::stdout(&output);
        let stderr = git::stderr(&output);
        let result = parse::classify_pull(output.status.success(), &stdout, &stderr);

        ctx.emit(|| {
            print!(
                "{}",
                utils::format_path_column(&ctx.repo.path, ctx.path_col)
            );
            print!(
                "{}",
                utils::format_branch_column(&header.name, ctx.branch_col)
            );
            print_pull_result(result, &stdout, &stderr);
        });

        if matches!(result, PullResult::Unable | PullResult::Conflict) {
            return Err(ArcError::command(
                CommandOp::Pull,
                Some(ctx.repo.path.clone()),
                if stderr.is_empty() { stdout } else { stderr },
            ));
        }
        Ok(())
    })
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

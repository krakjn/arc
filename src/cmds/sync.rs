use crate::cmds::clone::clone_repo;
use crate::cmds::pull::print_pull_result;
use crate::defs::Workspace;
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::parse;
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;

/// Sync all repositories to default branch and pull
pub fn sync(ws: &Workspace, opts: &RunOpts) -> Result<()> {
    for_each_repo(ws, opts, MissingDir::Call, |ctx| {
        if !ctx.dir.is_dir() {
            let short_name = utils::short_name(&ctx.repo.path);
            ctx.emit(|| {
                print!(
                    "{}",
                    utils::format_path_column(&ctx.repo.path, ctx.path_col)
                );
                print!(
                    "{}",
                    utils::format_branch_column(short_name, ctx.branch_col)
                );
                println!("{}", "cloning".cyan().bold());
            });
            return clone_repo(ctx.ws, ctx.repo, ctx);
        }

        let default_branch = git::detect_default_branch(&ctx.dir, &ctx.repo.remote);
        let output = git::run(&ctx.dir, &["checkout", &default_branch], StdioMode::Capture)?;
        let checkout_stdout = git::stdout(&output);
        let checkout_stderr = git::stderr(&output);

        let label = if parse::extract_switched(&checkout_stderr).is_some()
            || parse::extract_switched(&checkout_stdout).is_some()
        {
            format!("switched-to-{default_branch}")
        } else if parse::already_on_branch(&checkout_stderr)
            || parse::already_on_branch(&checkout_stdout)
            || output.status.success()
        {
            format!("on-{default_branch}")
        } else {
            ctx.emit(|| {
                print!(
                    "{}",
                    utils::format_path_column(&ctx.repo.path, ctx.path_col)
                );
                print!("{}", utils::format_branch_column("unable", ctx.branch_col));
                println!("{}", "dirty".red().bold());
                if !checkout_stdout.is_empty() {
                    println!("stdout: {checkout_stdout}");
                }
                if !checkout_stderr.is_empty() {
                    println!("stderr: {checkout_stderr}");
                }
            });
            return Err(ArcError::command(
                CommandOp::Sync,
                Some(ctx.repo.path.clone()),
                "checkout failed (dirty?)",
            ));
        };

        let pull_output = git::run(&ctx.dir, &["pull"], StdioMode::Capture)?;
        let pull_stdout = git::stdout(&pull_output);
        let pull_stderr = git::stderr(&pull_output);
        let result = parse::classify_pull(pull_output.status.success(), &pull_stdout, &pull_stderr);

        ctx.emit(|| {
            print!(
                "{}",
                utils::format_path_column(&ctx.repo.path, ctx.path_col)
            );
            print!("{}", utils::format_branch_column(&label, ctx.branch_col));
            print_pull_result(result, &pull_stdout, &pull_stderr);
        });

        if matches!(
            result,
            parse::PullResult::Unable | parse::PullResult::Conflict
        ) {
            return Err(ArcError::command(
                CommandOp::Sync,
                Some(ctx.repo.path.clone()),
                if pull_stderr.is_empty() {
                    pull_stdout
                } else {
                    pull_stderr
                },
            ));
        }
        Ok(())
    })
}

use crate::defs::Workspace;
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;

/// Execute a command across all repositories.
pub fn all(ws: &Workspace, git_mode: bool, args: &[String], opts: &RunOpts) -> Result<()> {
    if args.is_empty() {
        return Err(ArcError::command(
            if git_mode {
                CommandOp::Git
            } else {
                CommandOp::Shell
            },
            None,
            "no command provided",
        ));
    }

    for_each_repo(ws, opts, MissingDir::Fail, |ctx| {
        let mode = if opts.jobs <= 1 {
            ctx.emit(|| {
                print!(
                    "{}",
                    utils::format_path_column(&ctx.repo.path, ctx.path_col)
                );
                let pad = 8usize;
                println!(" {}", "#".repeat(pad).yellow());
            });
            StdioMode::Inherit
        } else {
            ctx.stream_mode()
        };

        let output = if git_mode {
            let git_args: Vec<&str> = args.iter().map(String::as_str).collect();
            git::run(&ctx.dir, &git_args, mode)?
        } else {
            git::run_cmd(&ctx.dir, &args[0], &args[1..], mode)?
        };

        if opts.jobs <= 1 {
            print!("{}", git::stdout(&output));
            print!("{}", git::stderr(&output));
        }

        if !output.status.success() {
            return Err(ArcError::command(
                if git_mode {
                    CommandOp::Git
                } else {
                    CommandOp::Shell
                },
                Some(ctx.repo.path.clone()),
                format!("exit {}", output.status),
            ));
        }
        Ok(())
    })
}

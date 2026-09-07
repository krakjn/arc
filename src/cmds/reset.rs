use crate::defs::Workspace;
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::parse;
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// Reset all repositories to the configured remote's default branch
pub fn reset(ws: &Workspace, sanitize: bool, yes: bool, opts: &RunOpts) -> Result<()> {
    if !yes {
        println!("{}", "!!!!! WARNING !!!!!".yellow().bold().on_yellow());
        println!("This will reset all project folders to remote's HEAD on default branch");
        print!("ALL edits will be ");
        println!("{}", "DISCARDED".red().bold().on_red());
        print!("(yes/NO): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let do_it = matches!(input.to_ascii_lowercase().trim(), "y" | "yes");
        if !do_it {
            println!("Aborting");
            return Err(ArcError::Aborted);
        }
    }

    for_each_repo(ws, opts, MissingDir::Fail, |ctx| {
        let default_branch = git::detect_default_branch(&ctx.dir, &ctx.repo.remote);
        let remote_branch = format!("{}/{default_branch}", ctx.repo.remote);

        let mut sanitized = false;
        if sanitize {
            let rm = git::run(&ctx.dir, &["rm", "--cached", "-r", "."], StdioMode::Capture)?;
            if !rm.status.success() {
                return Err(ArcError::command(
                    CommandOp::Reset,
                    Some(ctx.repo.path.clone()),
                    format!("sanitize failed: {}", git::stderr(&rm)),
                ));
            }
            sanitized = true;
        }

        let output = git::run(
            &ctx.dir,
            &["reset", "--hard", &remote_branch],
            StdioMode::Capture,
        )
        .map_err(|e| {
            ArcError::command(CommandOp::Reset, Some(ctx.repo.path.clone()), e.to_string())
        })?;
        let stdout = git::stdout(&output);

        if !output.status.success() {
            return Err(ArcError::command(
                CommandOp::Reset,
                Some(ctx.repo.path.clone()),
                git::stderr(&output),
            ));
        }

        ctx.emit(|| {
            print!(
                "{}",
                utils::format_path_column(&ctx.repo.path, ctx.path_col)
            );
            if sanitized {
                print!("{} ", "sanitized".cyan().bold());
            }
            print!("{} ", "now at->".yellow());
            if let Some(target) = parse::extract_reset_target(&stdout) {
                println!("{target}");
            } else {
                println!("{}", "done".green());
            }
        });
        Ok(())
    })
}

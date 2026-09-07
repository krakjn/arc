use crate::defs::Workspace;
use crate::error::{ArcError, CommandOp, Result};
use crate::git::{self, StdioMode};
use crate::run::{MissingDir, RunOpts, for_each_repo};
use crate::utils;
use owo_colors::OwoColorize;

/// Fetch from each repo's configured remote.
pub fn fetch(ws: &Workspace, opts: &RunOpts) -> Result<()> {
    for_each_repo(ws, opts, MissingDir::Fail, |ctx| {
        let output =
            git::run(&ctx.dir, &["fetch", &ctx.repo.remote], StdioMode::Capture).map_err(|e| {
                ArcError::command(CommandOp::Fetch, Some(ctx.repo.path.clone()), e.to_string())
            })?;

        ctx.emit(|| {
            print!(
                "{}",
                utils::format_path_column(&ctx.repo.path, ctx.path_col)
            );
            if output.status.success() {
                println!("{}", "fetched".green().bold());
            } else {
                println!("{}", "unable".red().bold());
                print!("{}", git::stderr(&output));
            }
        });

        if !output.status.success() {
            return Err(ArcError::command(
                CommandOp::Fetch,
                Some(ctx.repo.path.clone()),
                git::stderr(&output),
            ));
        }
        Ok(())
    })
}

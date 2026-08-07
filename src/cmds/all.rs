use crate::defs::Workspace;
use crate::error::{ArcError, Result};
use crate::git;
use owo_colors::OwoColorize;

/// Execute a command across all repositories.
/// `git_mode`: prefix with `git`. `args` are the command tokens (no `-c`/`-g` flag).
pub fn all(ws: &Workspace, git_mode: bool, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(ArcError::CommandFailed("no command provided".into()));
    }

    for repo in &ws.repos {
        let dir = ws.repo_dir(repo);
        if !dir.is_dir() {
            eprintln!(
                "{}: {}",
                "Error".red(),
                ArcError::RepositoryNotFound(repo.path.clone())
            );
            continue;
        }

        print!("{}", repo.path.cyan());
        let pad = 75usize.saturating_sub(repo.path.len());
        println!(" {}", "#".repeat(pad).yellow());

        let output = if git_mode {
            let git_args: Vec<&str> = args.iter().map(String::as_str).collect();
            git::run(&dir, &git_args)?
        } else {
            git::run_cmd(&dir, &args[0], &args[1..])?
        };

        print!("{}", git::stdout(&output));
        print!("{}", git::stderr(&output));
    }

    Ok(())
}

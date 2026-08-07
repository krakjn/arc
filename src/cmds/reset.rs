use crate::defs::Workspace;
use crate::error::{ArcError, Result};
use crate::git;
use crate::parse;
use crate::utils;
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// Reset all repositories to origin default branch
pub fn reset(ws: &Workspace, sanitize: bool) -> Result<()> {
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

    let path_col = utils::path_column_width(ws.repos.iter().map(|r| r.path.as_str()));

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

        print!("{}", utils::format_path_column(&repo.path, path_col));

        let default_branch = git::detect_default_branch(&dir);
        let origin_branch = format!("origin/{default_branch}");

        if sanitize {
            let _ = git::run(&dir, &["rm", "--cached", "-r", "."]);
            print!("{} ", "sanitized".cyan().bold());
        }

        print!("{} ", "now at->".yellow());

        let output = git::run(&dir, &["reset", "--hard", &origin_branch])
            .map_err(|e| ArcError::ResetFailed(e.to_string()))?;
        let stdout = git::stdout(&output);

        if let Some(target) = parse::extract_reset_target(&stdout) {
            println!("{target}");
        } else {
            println!("{}", "done".green());
        }
    }

    Ok(())
}

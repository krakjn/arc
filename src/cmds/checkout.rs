use crate::defs::Workspace;
use crate::error::{ArcError, Result};
use crate::git;
use crate::parse;
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::path::Path;

/// Checkout a branch across all dependent repositories
pub fn checkout(ws: &Workspace, new_branch: bool, branch_name: &str) -> Result<()> {
    let start_dir = env::current_dir()?;
    if !utils::find_git(&start_dir) {
        return Err(ArcError::GitNotFound);
    }

    let start_lossy = start_dir.to_string_lossy();
    let cwd = utils::short_name(&start_lossy);

    for repo in &ws.repos {
        if utils::short_name(&repo.path) != cwd {
            continue;
        }

        for dep in &repo.deps {
            let dep_dir = utils::resolve_repo_path(&ws.path.tld, dep);
            if !dep_dir.is_dir() {
                eprintln!("{} {}", "Cannot find directory:".red().underline(), dep);
                continue;
            }
            checkout_one(&dep_dir, new_branch, branch_name, dep, true)?;
        }
        break;
    }

    checkout_one(&start_dir, new_branch, branch_name, cwd, false)?;
    Ok(())
}

fn checkout_one(
    dir: &Path,
    new_branch: bool,
    branch_name: &str,
    name: &str,
    is_dep: bool,
) -> Result<()> {
    let args: Vec<&str> = if new_branch {
        vec!["checkout", "-b", branch_name]
    } else {
        vec!["checkout", branch_name]
    };

    let output = git::run(dir, &args)?;
    let stdout = git::stdout(&output);
    let stderr = git::stderr(&output);

    if is_dep {
        print!("{}", " dep ".magenta().bold());
    } else {
        print!("{}", " >>> ".cyan().bold());
    }
    print!("{}", utils::dotted_prefix(name, 50));

    if parse::extract_quoted_branch(&stdout).is_some() {
        if let Some(fetched) = parse::extract_fetched_branch(&stdout) {
            print!("{}", "fetched-> ".green().bold());
            println!("{fetched}");
        } else if let Some(switched) = parse::extract_switched(&stdout) {
            print!("{}", "switched-> ".cyan().bold());
            println!("{switched}");
        } else if new_branch {
            print!("{}", "created-> ".cyan().underline());
            println!("{branch_name}");
        } else {
            print!("{}", "on-> ".yellow().bold());
            println!("{branch_name}");
        }
    } else if let Some(switched) = parse::extract_switched(&stderr) {
        print!("{}", "switched-> ".cyan().bold());
        println!("{switched}");
    } else if parse::already_on_branch(&stderr) {
        print!("{}", stderr.yellow());
    } else if !stderr.is_empty() {
        println!("{}", "unable".red().bold());
        eprint!("{}", stderr.red());
    } else {
        println!("{}", "ok".green());
    }

    Ok(())
}

use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// _________ .__                   __                 __
// \_   ___ \|  |__   ____   ____ |  | ______  __ ___/  |_
// /    \  \/|  |  \_/ __ \_/ ___\|  |/ /  _ \|  |  \   __\
// \     \___|   Y  \  ___/\  \___|    <  <_> )  |  /|  |
//  \______  /___|  /\___  >\___  >__|_ \____/|____/ |__|
//         \/     \/     \/     \/     \/
/// Checkout a branch across all dependent repositories
pub fn checkout(arc: &Arc, new_branch: bool, branch_name: &str) -> Result<()> {
    if !utils::find_git() {
        return Err(ArcError::GitNotFound);
    }

    let checkout_cmd: Vec<&str> = if new_branch {
        vec!["checkout", "-b", branch_name]
    } else {
        vec!["checkout", branch_name]
    };

    let start_dir = env::current_dir()?;
    let path_str = start_dir.to_string_lossy();
    let this_proj: Vec<&str> = path_str.split(|c| c == '/' || c == '\\').collect();
    let cwd: &str = this_proj.last().unwrap_or(&"");

    for repo in &arc.repos {
        let proj_name: Vec<&str> = repo.name.split(|c| c == '/' || c == '\\').collect();
        let short_name = proj_name.last().unwrap_or(&"");

        if short_name == &cwd {
            // Checkout dependencies first
            for dep in &repo.deps {
                let mut this_path = PathBuf::from(&arc.path.tld);
                this_path.push(dep);
                
                if env::set_current_dir(&this_path).is_err() {
                    eprintln!("{} {}", "Cannot find directory:".red().underline(), dep);
                    let _ = env::set_current_dir(&arc.path.tld);
                    continue;
                }
                checkout_parse(new_branch, &checkout_cmd, dep, branch_name, true)?;
            }

            break;
        }
    }

    env::set_current_dir(&start_dir)?;
    checkout_parse(new_branch, &checkout_cmd, cwd, branch_name, false)?;
    
    Ok(())
}

fn checkout_parse(
    new_branch: bool,
    checkout_cmd: &[&str],
    name: &str,
    branch_name_arg: &str,
    is_dep: bool,
) -> Result<()> {
    let output = Command::new("git")
        .args(checkout_cmd)
        .output()
        .map_err(|e| ArcError::GitCommandFailed(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print prefix
    if is_dep {
        print!("{}", " dep ".magenta().bold());
    } else {
        print!("{}", " >>> ".cyan().bold());
    }
    print!("{:.<50}", name);

    // Check stdout for success patterns
    if utils::BRANCH_QUOTE.is_match(&stdout) {
        if utils::FETCHED.is_match(&stdout) {
            if let Some(caps) = utils::FETCHED.captures(&stdout) {
                print!("{}", "fetched-> ".green().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }
        } else if utils::SWITCHED.is_match(&stdout) {
            if let Some(caps) = utils::SWITCHED.captures(&stdout) {
                print!("{}", "switched-> ".cyan().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }
        } else if new_branch {
            print!("{}", "created-> ".cyan().underline());
            println!("{}", branch_name_arg);
        } else {
            print!("{}", "on-> ".yellow().bold());
            println!("{}", branch_name_arg);
        }
    } else if utils::SWITCHED.is_match(&stderr) {
        // Check stderr (git sometimes outputs to stderr)
        if let Some(caps) = utils::SWITCHED.captures(&stderr) {
            print!("{}", "switched-> ".cyan().bold());
            println!("{}", caps.get(1).unwrap().as_str());
        }
    } else if utils::ALREADY.is_match(&stderr) {
        print!("{}", stderr.yellow());
    } else if !stderr.is_empty() {
        // True error
        println!("{}", "unable".red().bold());
        eprint!("{}", stderr.red());
    } else {
        println!("{}", "ok".green());
    }

    Ok(())
}

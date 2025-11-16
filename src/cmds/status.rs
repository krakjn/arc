use crate::defs::*;
use crate::error::Result;
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

//   _________ __          __
//  /   _____//  |______ _/  |_ __ __  ______
//  \_____  \\   __\__  \\   __\  |  \/  ___/
//  /        \|  |  / __ \|  | |  |  /\___ \
// /_______  /|__| (____  /__| |____//____  >
//         \/           \/                \/
/// Display status of all repositories
pub fn status(arc: &Arc, verbose: bool) -> Result<()> {
    let start_dir = env::current_dir()?;

    for repo in &arc.repos {
        env::set_current_dir(&arc.path.tld)?;
        let mut os_path = PathBuf::from(&arc.path.tld);
        os_path.push(&repo.name);

        if env::set_current_dir(&os_path).is_err() {
            eprintln!("{} {}", "Cannot find directory:".red().underline(), &repo.name);
            env::set_current_dir(&arc.path.tld)?;
            continue;
        }

        let output = Command::new("git")
            .args(["status", "-b", "--porcelain"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print repo name and branch
        if utils::BRANCH_NAME.is_match(&stdout) {
            let caps = utils::BRANCH_NAME.captures(&stdout).unwrap();
            print!("{:.<50}", &repo.name);
            let curr_branch_name = caps.get(1).unwrap().as_str();
            
            // Detect if on default branch
            let default_branch = utils::detect_default_branch()
                .unwrap_or_else(|_| "master".to_string());
            
            if curr_branch_name == default_branch {
                print!("{:.>25} ", curr_branch_name);
            } else {
                print!("{:.>width$}", ".", width = 25 - curr_branch_name.len());
                print!("{} ", curr_branch_name.yellow());
            }
        } else if utils::UNTRACKED_BRANCH.is_match(&stdout) {
            let caps = utils::UNTRACKED_BRANCH.captures(&stdout).unwrap();
            print!("{:.<50}", &repo.name);
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
        } else {
            print!("{:.<50}", &repo.name);
            print!("{:.>25} ", "unhandled state");
            continue;
        }

        // Check divergence status
        if utils::DIVERGED.is_match(&stdout) {
            let caps = utils::DIVERGED.captures(&stdout).unwrap();
            let diverged_str = format!(
                "↑{},↓{} ",
                caps.get(1).unwrap().as_str(),
                caps.get(2).unwrap().as_str()
            );
            print!("{}", diverged_str.bright_magenta().bold());
        } else {
            if utils::AHEAD.is_match(&stdout) {
                let caps = utils::AHEAD.captures(&stdout).unwrap();
                let ahead_str = format!("↑{} ", caps.get(1).unwrap().as_str());
                print!("{}", ahead_str.cyan().bold());
            }

            if utils::BEHIND.is_match(&stdout) {
                let caps = utils::BEHIND.captures(&stdout).unwrap();
                let behind_str = format!("↓{} ", caps.get(1).unwrap().as_str());
                print!("{}", behind_str.cyan().bold());
            }
        }

        // Verbose or simple output
        if verbose {
            display_verbose_status(&stdout)?;
        } else {
            display_simple_status(&stdout)?;
        }

        env::set_current_dir(&arc.path.tld)?;
    }

    env::set_current_dir(&start_dir)?;
    Ok(())
}

fn display_verbose_status(stdout: &str) -> Result<()> {
    if !utils::BRANCH_NAME.is_match(stdout) {
        print!("{}", "not pushed ".bright_magenta().underline());
    }

    if utils::CHANGED.is_match(stdout) {
        println!("{}", "changed".red().bold());

        // Staged changes
        if utils::STAGED.is_match(stdout) {
            for caps in utils::DELETE_STAGED.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "deleted: ").green().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }

            for caps in utils::ADDED.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "added: ").green().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }

            for caps in utils::MODIFIED_STAGED.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "modified: ").green().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }
        }

        // Unstaged changes
        if utils::UNSTAGED.is_match(stdout) {
            for caps in utils::DELETE.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "deleted: ").red().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }

            for caps in utils::MODIFIED.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "modified: ").red().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }
        }

        // Untracked files
        if utils::UNTRACKED.is_match(stdout) {
            for caps in utils::UNTRACKED.captures_iter(stdout) {
                print!("{}", format!("{:>15}", "untracked: ").yellow().bold());
                println!("{}", caps.get(1).unwrap().as_str());
            }
        }
    } else {
        println!("{}", "clean".green().bold());
    }

    Ok(())
}

fn display_simple_status(stdout: &str) -> Result<()> {
    if !utils::BRANCH_NAME.is_match(stdout) {
        print!("{}", "!".bright_magenta().bold());
    }

    if utils::CHANGED.is_match(stdout) {
        let mut partial_clean = true;
        
        if utils::UNTRACKED.is_match(stdout) {
            print!("{}", "?".yellow().bold());
        }
        
        if utils::STAGED.is_match(stdout) {
            print!("{}", "✚".green().bold());
            partial_clean = false;
        }
        
        if utils::UNSTAGED.is_match(stdout) {
            print!("{}", "✘".red().bold());
            partial_clean = false;
        }
        
        if partial_clean {
            println!("{}", "✔ ".green().bold());
        } else {
            println!();
        }
    } else {
        println!("{}", "✔ ".green().bold());
    }

    Ok(())
}

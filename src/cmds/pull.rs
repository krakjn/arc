use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// __________      .__  .__
// \______   \__ __|  | |  |
//  |     ___/  |  \  | |  |
//  |    |   |  |  /  |_|  |__
//  |____|   |____/|____/____/

/// Pull changes from remote for all repositories
pub fn pull(arc: &Arc) -> Result<()> {
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

        // Get current branch status
        let status = Command::new("git")
            .args(["status", "-bs"])
            .output()
            .map_err(|e| ArcError::GitCommandFailed(e.to_string()))?;
        let branchout = String::from_utf8_lossy(&status.stdout);

        print!("{:.<50}", &repo.name);
        
        if utils::BRANCH_NAME.is_match(&branchout) {
            let caps = utils::BRANCH_NAME.captures(&branchout).unwrap();
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
        } else if utils::UNTRACKED_BRANCH.is_match(&branchout) {
            let caps = utils::UNTRACKED_BRANCH.captures(&branchout).unwrap();
            print!("{:.>25} ", caps.get(1).unwrap().as_str());
            println!("{}", "untracked".magenta().bold());
            continue;
        } else {
            println!("{:.>25} ", "not on a branch?");
            continue;
        }

        // Execute pull
        let output = Command::new("git")
            .arg("pull")
            .output()
            .map_err(|e| ArcError::PullFailed(e.to_string()))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if utils::UP_TO_DATE.is_match(&stdout) {
            println!("{}", "up-to-date".green().bold());
        } else if utils::UPDATING.is_match(&stdout) {
            println!("{}", "updating".green().bold().on_green());
            print!("{}", stdout);
        } else if utils::CONFLICT.is_match(&stdout) {
            println!("{}", "conflict".yellow().bold());
            print!("{}", stdout);
        } else if !stderr.is_empty() {
            println!("{}", "unable".red().bold());
            print!("{}", stderr);
        } else {
            println!("{}", "ok".green());
        }
    }

    env::set_current_dir(&start_dir)?;
    Ok(())
}

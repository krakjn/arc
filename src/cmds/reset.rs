use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// __________                      __
// \______   \ ____   ______ _____/  |_
//  |       _// __ \ /  ___// __ \   __\
//  |    |   \  ___/ \___ \\  ___/|  |
//  |____|_  /\___  >____  >\___  >__|
//         \/     \/     \/     \/
/// Reset all repositories to origin default branch
pub fn reset(arc: &Arc, sanitize: bool) -> Result<()> {
    let start_dir = env::current_dir()?;

    println!("{}", "!!!!! WARNING !!!!!".yellow().bold().on_yellow());
    println!("This will reset all project folders to remote's HEAD on default branch");
    print!("ALL edits will be ");
    println!("{}", "DISCARDED".red().bold().on_red());
    print!("(yes/NO): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let do_it = matches!(
        input.to_ascii_lowercase().trim(),
        "y" | "yes"
    );

    if !do_it {
        println!("Aborting");
        return Ok(());
    }

    for repo in &arc.repos {
        env::set_current_dir(&arc.path.tld)?;
        let mut path = PathBuf::from(&arc.path.tld);
        path.push(&repo.name);

        if env::set_current_dir(&path).is_err() {
            eprintln!("{} {}", "Cannot find directory:".red().underline(), &repo.name);
            env::set_current_dir(&arc.path.tld)?;
            continue;
        }

        print!("{:.<50}", &repo.name);

        // Detect default branch for this repo
        let default_branch = utils::detect_default_branch()
            .unwrap_or_else(|_| "master".to_string());
        let origin_branch = format!("origin/{}", default_branch);

        if sanitize {
            // Remove cached files
            let _ = Command::new("git")
                .args(["rm", "--cached", "-r", "."])
                .output();

            print!("{} ", "sanitized".cyan().bold());
            print!("{} ", "now at->".yellow());

            let output = Command::new("git")
                .args(["reset", "--hard", &origin_branch])
                .output()
                .map_err(|e| ArcError::ResetFailed(e.to_string()))?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(caps) = utils::RESET_REGEX.captures(&stdout) {
                println!("{}", caps.get(1).unwrap().as_str());
            } else {
                println!("{}", "done".green());
            }
        } else {
            // Regular reset
            let output = Command::new("git")
                .args(["reset", "--hard", &origin_branch])
                .output()
                .map_err(|e| ArcError::ResetFailed(e.to_string()))?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            print!("{} ", "now at->".yellow());
            
            if let Some(caps) = utils::RESET_REGEX.captures(&stdout) {
                println!("{}", caps.get(1).unwrap().as_str());
            } else {
                println!("{}", "done".green());
            }
        }
    }
    
    env::set_current_dir(start_dir)?;
    Ok(())
}

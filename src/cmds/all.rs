use crate::defs::*;
use crate::error::{ArcError, Result};
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

//    _____  .__  .__
//   /  _  \ |  | |  |
//  /  /_\  \|  | |  |
// /    |    \  |_|  |__
// \____|__  /____/____/
//         \/
/// Execute a command across all repositories
pub fn all(arc: &Arc, gcmd: bool, cmd_str: &Vec<String>) -> Result<()> {
    let start_dir = env::current_dir()?;

    for repo in &arc.repos {
        let mut full_path = PathBuf::from(&arc.path.tld);
        full_path.push(&repo.name);
        
        if env::set_current_dir(&full_path).is_err() {
            eprintln!("{}: {}", "Error".red(), ArcError::RepositoryNotFound(repo.name.clone()));
            continue;
        }

        // Print header
        print!("{}", repo.name.cyan());
        let header = format!(" {:#>width$}", "#", width = 75 - repo.name.len());
        println!("{}", header.yellow());

        if gcmd {
            // Git command execution
            let output = Command::new("git")
                .args(&cmd_str[2..])
                .output()
                .map_err(|e| ArcError::GitCommandFailed(e.to_string()))?;
            
            print!("{}", String::from_utf8_lossy(&output.stdout));
            print!("{}", String::from_utf8_lossy(&output.stderr));
        } else {
            // Regular command execution
            if cmd_str.len() < 3 {
                return Err(ArcError::GitCommandFailed("no command provided".to_string()));
            }
            
            let output = Command::new(&cmd_str[2])
                .args(&cmd_str[3..])
                .output()
                .map_err(|e| ArcError::GitCommandFailed(e.to_string()))?;
            
            print!("{}", String::from_utf8_lossy(&output.stdout));
            print!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    env::set_current_dir(start_dir)?;
    Ok(())
}

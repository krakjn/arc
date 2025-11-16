use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

// .___       .__  __
// |   | ____ |__|/  |_
// |   |/    \|  \   __\
// |   |   |  \  ||  |
// |___|___|  /__||__|
//          \/
/// Initialize all repositories defined in arc.json
pub fn init(arc: &Arc) -> Result<()> {
    for repo in &arc.repos {
        let (relative_path, short_name) = utils::clone_path(&repo.name);
        
        // Always start from top level directory
        env::set_current_dir(&arc.path.tld)?;
        
        // Create nested directories if needed
        if relative_path != "shallow" {
            fs::create_dir_all(&relative_path)
                .map_err(|e| ArcError::Io(e))?;
            env::set_current_dir(&relative_path)?;
        }

        let repo_url_name = format!("{}/{}.git", &arc.remote.url, &repo.name);
        let fmt_str = format!("loading >>> {:_>50}", short_name);
        println!("{}", fmt_str.cyan().bold());

        let output = Command::new("git")
            .arg("clone")
            .arg("--progress")
            .arg(&repo_url_name)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| ArcError::CloneFailed {
                repo: repo.name.clone(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ArcError::CloneFailed {
                repo: repo.name.clone(),
                message: stderr.to_string(),
            });
        }

        env::set_current_dir(&arc.path.tld)?;
    }
    
    println!("{} {}", ">>>".cyan().bold(), "arc initialized".green().bold());
    Ok(())
}

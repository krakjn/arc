use crate::defs::*;
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::fs;

// new feature
// need to add .bashrc to `gitbash` for nice printouts
/**
 * ⤧ diverged ahead
 * ⤩ diverged behind
 * ↰ detached
 * ☣ dirty
 * ⚠ ⚡±
 * ◒ ♝ ☣ ✇ ☢ ☯ ☸ ✈
 * ⥢ ⥣ ⥤ ⥥ ⥦ ⥧ ⥨ ⥩ ⥪ ⥫ ⥬ ⥭ ⥮ ⥯ ⥰
 * ← → ↑ ↓ ↔ ↕ ↖ ↗ ↘ ↙ ↜ ↝ ↞ ↟ ↠ ↡ ↢ ↣
 */

/// Load arc.json configuration and populate Arc struct
pub fn load_arc(arc: &mut Arc) -> Result<()> {
    // Find and set arc.json path
    utils::find_arc(&mut arc.path.tld, &mut arc.path.arc)?;

    // Read arc.json file
    let contents = fs::read_to_string(&arc.path.arc)
        .map_err(|_| ArcError::ConfigParseError("couldn't read arc.json".to_string()))?;

    // Parse JSON
    let arcj: ArcJson = serde_json::from_str(&contents)
        .map_err(|e| ArcError::ConfigParseError(format!("invalid JSON: {}", e)))?;

    // Populate arc instance
    arc.remote = arcj.remote;
    arc.repos = arcj
        .projects
        .into_iter()
        .map(|(name, deps)| Repo { name, deps })
        .collect();

    Ok(())
}

pub fn print_info() {
    println!(
        "   _____ ___________________
  /  _  \\\\______   \\_   ____\\
 /  /_\\  \\|       _/    \\       A_ggregate
/    |    \\    |   \\     \\____  R_epository
\\____|____/____|___/\\________/  C_ontrol

-- The bridge across repositories --
The purpose of this tool is to help larger projects based on git have
a smarter way of checking out projects especially if those projects
have dependencies. Once those dependencies are known to arc, then arc
will handle the rest. Dependent projects will all be checked out to
the same branch, that way you don't have to worry about unrelated
histories ever again. 

Submit issues through GitLab to:
    http://domain/path/arc

The simplest solution, is the best solution - Tony Bright"
    )
}

/// Display arc.json configuration in a formatted way
pub fn show_json(arc: &Arc) {
    for repo in &arc.repos {
        print!("{}", arc.remote.url.cyan());
        print!(" {}", repo.name.yellow());
        print!("{:.>width$}", ".", width = 50 - repo.name.len());
        print!("{}", "deps:".magenta());
        print!(" [ ");
        for (i, dep) in repo.deps.iter().enumerate() {
            if i == repo.deps.len() - 1 {
                print!("{} ", dep.bright_magenta());
            } else {
                print!("{}, ", dep.bright_magenta());
            }
        }
        println!("]");
    }
}

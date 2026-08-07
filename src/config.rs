use crate::defs::{ArcConfig, ArcPath, ProjectConfig, Repo, Workspace};
use crate::error::{ArcError, Result};
use crate::utils;
use owo_colors::OwoColorize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Load workspace from `--config` or upward `arc.toml` search.
pub fn load(config_override: Option<&Path>) -> Result<Workspace> {
    let (tld, config_path) = utils::resolve_config(config_override)?;
    let contents = fs::read_to_string(&config_path)
        .map_err(|e| ArcError::ConfigParseError(format!("couldn't read arc.toml: {e}")))?;

    let parsed: ArcConfig = toml::from_str(&contents)
        .map_err(|e| ArcError::ConfigParseError(format!("invalid TOML: {e}")))?;

    validate(&parsed)?;

    Ok(Workspace {
        path: ArcPath {
            tld,
            config: config_path,
        },
        repos: parsed
            .projects
            .into_iter()
            .map(|p| Repo {
                path: p.path,
                remote: p.remote,
                url: p.url,
                deps: p.deps,
            })
            .collect(),
    })
}

fn validate(config: &ArcConfig) -> Result<()> {
    if config.projects.is_empty() {
        return Err(ArcError::ConfigValidation(
            "no [[project]] entries found (use singular [[project]], not [[projects]])".into(),
        ));
    }

    let mut seen = HashSet::new();

    for ProjectConfig { path, url, .. } in &config.projects {
        let path = path.trim();
        let url = url.trim();

        if path.is_empty() {
            return Err(ArcError::ConfigValidation(
                "project path must not be empty".into(),
            ));
        }
        if url.is_empty() {
            return Err(ArcError::ConfigValidation(format!(
                "project '{path}' has empty url"
            )));
        }
        if !seen.insert(path.to_string()) {
            return Err(ArcError::ConfigValidation(format!(
                "duplicate project path '{path}'"
            )));
        }
    }

    for project in &config.projects {
        for dep in &project.deps {
            if !seen.contains(dep.as_str()) {
                return Err(ArcError::ConfigValidation(format!(
                    "project '{}' depends on unknown path '{dep}'",
                    project.path
                )));
            }
        }
    }

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
    );
}

/// Display loaded configuration.
pub fn show(ws: &Workspace) {
    for repo in &ws.repos {
        print!("{} ", repo.url.cyan());
        print!("{}", repo.path.yellow());
        let pad = 50usize.saturating_sub(repo.path.len());
        print!("{}", ".".repeat(pad));
        print!("{}", "deps:".magenta());
        print!(" [ ");
        for (i, dep) in repo.deps.iter().enumerate() {
            if i + 1 == repo.deps.len() {
                print!("{} ", dep.bright_magenta());
            } else {
                print!("{}, ", dep.bright_magenta());
            }
        }
        println!("]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_round_trip_defaults_remote() {
        let raw = r#"
[[project]]
path = "path/arc"
url = "http://domain"
dependencies = []
"#;
        let cfg: ArcConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.projects.len(), 1);
        assert_eq!(cfg.projects[0].remote, "origin");
        assert_eq!(cfg.projects[0].path, "path/arc");
        validate(&cfg).unwrap();
    }

    #[test]
    fn rejects_unknown_dep() {
        let raw = r#"
[[project]]
path = "a"
url = "http://x"
dependencies = ["missing"]
"#;
        let cfg: ArcConfig = toml::from_str(raw).unwrap();
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn allows_relative_parent_and_absolute_paths() {
        let raw = r#"
[[project]]
path = "../arc"
url = "http://x/arc.git"
dependencies = []

[[project]]
path = "/opt/other"
url = "http://x/other.git"
dependencies = []
"#;
        let cfg: ArcConfig = toml::from_str(raw).unwrap();
        validate(&cfg).unwrap();
    }

    #[test]
    fn rejects_plural_projects_key() {
        let raw = r#"
[[projects]]
path = "a"
url = "http://x"
dependencies = []
"#;
        let err = toml::from_str::<ArcConfig>(raw).unwrap_err().to_string();
        assert!(
            err.contains("projects") || err.contains("unknown"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_empty_config() {
        let cfg: ArcConfig = toml::from_str("").unwrap();
        assert!(validate(&cfg).is_err());
    }
}

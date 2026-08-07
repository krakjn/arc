use crate::defs::{ArcConfig, ArcPath, ProjectConfig, Repo, Workspace};
use crate::error::{ArcError, Result};
use crate::utils::{self, CONFIG_NAME};
use owo_colors::OwoColorize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Embedded starter template (repo-root `arc.toml`).
const ARC_TOML_TEMPLATE: &str = include_str!("../arc.toml");

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
The purpose of this tool is to help maintainers of 
multiple repositories keep tabs on all of them at once,
especially if there is inter-dependence between them.
Dependent repositories will all be checked out to the
same branch, that way you don't have to worry about
unrelated histories ever again."
    );
}

/// Write a starter `arc.toml` to cwd or `path`.
///
/// - `None` → `./arc.toml`
/// - directory (or path without `.toml`) → `path/arc.toml`
/// - path ending in `.toml` → that file
pub fn init_config(path: Option<&Path>) -> Result<()> {
    let target = resolve_init_config_path(path)?;
    if target.exists() {
        return Err(ArcError::ConfigValidation(format!(
            "refusing to overwrite existing file '{}'",
            target.display()
        )));
    }
    if let Some(parent) = target.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, ARC_TOML_TEMPLATE)?;
    println!(
        "{} wrote {}",
        ">>>".cyan().bold(),
        target.display().to_string().green()
    );
    Ok(())
}

fn resolve_init_config_path(path: Option<&Path>) -> Result<PathBuf> {
    let base = match path {
        Some(p) if p.as_os_str().is_empty() => env::current_dir()?,
        Some(p) => p.to_path_buf(),
        None => env::current_dir()?,
    };

    let is_toml = base
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));

    if is_toml {
        Ok(base)
    } else if base.is_file() {
        Err(ArcError::ConfigValidation(format!(
            "path '{}' exists and is not a .toml file",
            base.display()
        )))
    } else {
        // Directory, or a path we will treat as a directory to create.
        Ok(base.join(CONFIG_NAME))
    }
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

    #[test]
    fn init_config_writes_template() {
        let dir = env::temp_dir().join(format!("arc-init-config-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        init_config(Some(&dir)).unwrap();
        let written = fs::read_to_string(dir.join(CONFIG_NAME)).unwrap();
        assert_eq!(written, ARC_TOML_TEMPLATE);
        assert!(validate(&toml::from_str::<ArcConfig>(&written).unwrap()).is_ok());

        let err = init_config(Some(&dir)).unwrap_err().to_string();
        assert!(err.contains("refusing to overwrite"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_config_accepts_explicit_toml_path() {
        let dir = env::temp_dir().join(format!("arc-init-config-file-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("custom.toml");

        init_config(Some(&file)).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), ARC_TOML_TEMPLATE);

        let _ = fs::remove_dir_all(&dir);
    }
}

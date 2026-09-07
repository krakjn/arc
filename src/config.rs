use crate::defs::{ArcConfig, ArcPath, Repo, Workspace};
use crate::error::{ArcError, Result};
use crate::utils::{self, CONFIG_NAME};
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
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

    let parsed: ArcConfig = toml::from_str(&contents).map_err(|e| {
        let msg = e.to_string();
        if (msg.contains("project") || msg.contains("unknown field"))
            && contents.contains("[[project")
        {
            ArcError::ConfigParseError("use [[repo]] (not [[project]] or [[projects]])".into())
        } else {
            ArcError::ConfigParseError(format!("invalid TOML: {e}"))
        }
    })?;

    validate(&parsed)?;

    Ok(Workspace {
        path: ArcPath {
            tld,
            config: config_path,
        },
        repos: parsed.repos,
    })
}

fn validate(config: &ArcConfig) -> Result<()> {
    if config.repos.is_empty() {
        return Err(ArcError::ConfigValidation(
            "no [[repo]] entries found (use singular [[repo]], not [[repos]])".into(),
        ));
    }

    let mut seen = HashSet::new();

    for Repo {
        path, url, remote, ..
    } in &config.repos
    {
        let path = path.trim();
        let url = url.trim();
        let remote = remote.trim();

        if path.is_empty() {
            return Err(ArcError::ConfigValidation(
                "repo path must not be empty".into(),
            ));
        }
        if url.is_empty() {
            return Err(ArcError::ConfigValidation(format!(
                "repo '{path}' has empty url"
            )));
        }
        if remote.is_empty() {
            return Err(ArcError::ConfigValidation(format!(
                "repo '{path}' has empty remote"
            )));
        }
        if !seen.insert(path.to_string()) {
            return Err(ArcError::ConfigValidation(format!(
                "duplicate repo path '{path}'"
            )));
        }
    }

    for repo in &config.repos {
        for dep in &repo.deps {
            if !seen.contains(dep.as_str()) {
                return Err(ArcError::ConfigValidation(format!(
                    "repo '{}' depends on unknown path '{dep}'",
                    repo.path
                )));
            }
        }
    }

    if let Some(cycle) = detect_cycle(&config.repos) {
        return Err(ArcError::ConfigValidation(format!(
            "dependency cycle: {cycle}"
        )));
    }

    Ok(())
}

fn detect_cycle(repos: &[Repo]) -> Option<String> {
    let index: HashMap<&str, usize> = repos
        .iter()
        .enumerate()
        .map(|(i, r)| (r.path.as_str(), i))
        .collect();

    const UNVISITED: u8 = 0;
    const VISITING: u8 = 1;
    const DONE: u8 = 2;
    let mut mark = vec![UNVISITED; repos.len()];
    let mut stack = Vec::new();

    fn dfs(
        i: usize,
        repos: &[Repo],
        index: &HashMap<&str, usize>,
        mark: &mut [u8],
        stack: &mut Vec<usize>,
    ) -> Option<String> {
        mark[i] = VISITING;
        stack.push(i);
        for dep in &repos[i].deps {
            let Some(&j) = index.get(dep.as_str()) else {
                continue;
            };
            if mark[j] == VISITING {
                let start = stack.iter().position(|&x| x == j).unwrap_or(0);
                let mut names: Vec<&str> = stack[start..]
                    .iter()
                    .map(|&k| repos[k].path.as_str())
                    .collect();
                names.push(repos[j].path.as_str());
                return Some(names.join(" -> "));
            }
            if mark[j] == UNVISITED
                && let Some(cycle) = dfs(j, repos, index, mark, stack)
            {
                return Some(cycle);
            }
        }
        stack.pop();
        mark[i] = DONE;
        None
    }

    for i in 0..repos.len() {
        if mark[i] == UNVISITED
            && let Some(cycle) = dfs(i, repos, &index, &mut mark, &mut stack)
        {
            return Some(cycle);
        }
    }
    None
}

/// Write a starter `arc.toml` to cwd or `path`.
///
/// - `None` → `./arc.toml`
/// - directory (or path without `.toml`) → `path/arc.toml`
/// - path ending in `.toml` → that file
pub fn init(path: Option<&Path>) -> Result<()> {
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
        Ok(base.join(CONFIG_NAME))
    }
}

/// Display loaded configuration.
pub fn show(ws: &Workspace) {
    let path_col = utils::line_layout(ws.repos.iter().map(|r| r.path.as_str())).0;
    for repo in &ws.repos {
        print!("{} ", repo.url.cyan());
        print!(
            "{}",
            utils::format_path_column(&repo.path, path_col).yellow()
        );
        print!("{}", "deps:".magenta());
        print!(" [ ");
        if !repo.deps.is_empty() {
            let joined = repo
                .deps
                .iter()
                .map(|d| d.bright_magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            print!("{joined} ");
        }
        println!("]");
    }
}

/// Repo whose resolved directory contains `cwd` (longest prefix wins).
pub fn repo_for_cwd<'a>(ws: &'a Workspace, cwd: &Path) -> Option<&'a Repo> {
    let cwd = cwd.canonicalize().ok()?;
    ws.repos
        .iter()
        .filter(|repo| {
            ws.repo_dir(repo)
                .canonicalize()
                .ok()
                .is_some_and(|dir| cwd.starts_with(&dir))
        })
        .max_by_key(|repo| ws.repo_dir(repo).as_os_str().len())
}

/// Exact `path` match from `[[repo]]`.
pub fn repo_by_path<'a>(ws: &'a Workspace, path: &str) -> Option<&'a Repo> {
    ws.repos.iter().find(|r| r.path == path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> ArcConfig {
        toml::from_str(raw).unwrap()
    }

    #[test]
    fn toml_requires_remote() {
        let raw = r#"
[[repo]]
path = "path/arc"
url = "http://domain"
remote = "origin"
dependencies = []
"#;
        let cfg = parse(raw);
        assert_eq!(cfg.repos.len(), 1);
        assert_eq!(cfg.repos[0].remote, "origin");
        assert_eq!(cfg.repos[0].path, "path/arc");
        validate(&cfg).unwrap();
    }

    #[test]
    fn repo_by_path_exact() {
        let ws = Workspace {
            path: ArcPath::default(),
            repos: vec![Repo {
                path: "path/arc".into(),
                remote: "origin".into(),
                url: "http://x".into(),
                deps: vec![],
            }],
        };
        assert!(repo_by_path(&ws, "path/arc").is_some());
        assert!(repo_by_path(&ws, "arc").is_none());
    }

    #[test]
    fn rejects_missing_remote() {
        let raw = r#"
[[repo]]
path = "a"
url = "http://x"
dependencies = []
"#;
        assert!(toml::from_str::<ArcConfig>(raw).is_err());
    }

    #[test]
    fn rejects_unknown_dep() {
        let raw = r#"
[[repo]]
path = "a"
url = "http://x"
remote = "origin"
dependencies = ["missing"]
"#;
        let cfg = parse(raw);
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn rejects_cycle() {
        let raw = r#"
[[repo]]
path = "a"
url = "http://x"
remote = "origin"
dependencies = ["b"]

[[repo]]
path = "b"
url = "http://y"
remote = "origin"
dependencies = ["a"]
"#;
        let cfg = parse(raw);
        let err = validate(&cfg).unwrap_err().to_string();
        assert!(err.contains("cycle"), "{err}");
    }

    #[test]
    fn parent_dir_path_is_valid_config() {
        let raw = r#"
[[repo]]
path = "../sibling"
url = "http://x/s.git"
remote = "origin"
dependencies = []
"#;
        validate(&parse(raw)).unwrap();
    }

    #[test]
    fn allows_absolute_paths() {
        let raw = r#"
[[repo]]
path = "/opt/other"
url = "http://x/other.git"
remote = "origin"
dependencies = []
"#;
        validate(&parse(raw)).unwrap();
    }

    #[test]
    fn rejects_old_project_key() {
        let raw = r#"
[[project]]
path = "a"
url = "http://x"
remote = "origin"
dependencies = []
"#;
        let err = toml::from_str::<ArcConfig>(raw).unwrap_err().to_string();
        assert!(
            err.contains("project") || err.contains("unknown"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_empty_config() {
        let cfg: ArcConfig = toml::from_str("").unwrap();
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn init_writes_template() {
        let dir = tempfile::tempdir().unwrap();
        init(Some(dir.path())).unwrap();
        let written = fs::read_to_string(dir.path().join(CONFIG_NAME)).unwrap();
        assert_eq!(written, ARC_TOML_TEMPLATE);
        assert!(validate(&toml::from_str::<ArcConfig>(&written).unwrap()).is_ok());

        let err = init(Some(dir.path())).unwrap_err().to_string();
        assert!(err.contains("refusing to overwrite"));
    }

    #[test]
    fn init_accepts_explicit_toml_path() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("custom.toml");
        init(Some(&file)).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), ARC_TOML_TEMPLATE);
    }
}

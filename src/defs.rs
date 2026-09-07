use serde::Deserialize;

use crate::utils::{looks_like_full_repo_url, short_name};

/// ASCII banner shown in `--help`.
pub const BANNER: &str = "   _____ ___________________\n  /  _  \\\\______   \\_   ____\\\n /  /_\\  \\|       _/    \\       A_ggregate\n/    |    \\    |   \\     \\____  R_epository\n\\____|____/____|___/\\________/  C_ontrol";

/// Deserialized `arc.toml` root
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcConfig {
    #[serde(default, rename = "repo")]
    pub repos: Vec<Repo>,
}

/// A single `[[repo]]` table and runtime entry
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub path: String,
    pub remote: String,
    pub url: String,
    #[serde(default, rename = "dependencies")]
    pub deps: Vec<String>,
}

/// File paths for the loaded workspace
#[derive(Debug, Default, Clone)]
pub struct ArcPath {
    /// Directory containing `arc.toml`
    pub tld: std::path::PathBuf,
    /// Full path to `arc.toml`
    pub config: std::path::PathBuf,
}

/// Loaded workspace configuration and roots
#[derive(Debug, Default)]
pub struct Workspace {
    pub path: ArcPath,
    pub repos: Vec<Repo>,
}

impl Workspace {
    pub fn repo_dir(&self, repo: &Repo) -> std::path::PathBuf {
        crate::utils::resolve_repo_path(&self.path.tld, &repo.path)
    }

    /// Full clone URL.
    ///
    /// - Ends with `.git`, or looks like a full repo URL → use as-is.
    /// - Otherwise treat as a host/base and append `/{short_name}.git`.
    pub fn clone_url(repo: &Repo) -> String {
        let url = repo.url.trim_end_matches('/');
        if url.ends_with(".git") || looks_like_full_repo_url(url) {
            url.to_string()
        } else {
            format!("{url}/{}.git", short_name(&repo.path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_url_git_suffix() {
        let repo = Repo {
            path: "ignored".into(),
            remote: "origin".into(),
            url: "https://host/org/repo.git".into(),
            deps: vec![],
        };
        assert_eq!(Workspace::clone_url(&repo), "https://host/org/repo.git");
    }

    #[test]
    fn clone_url_full_https() {
        let repo = Repo {
            path: "local/name".into(),
            remote: "origin".into(),
            url: "https://host/org/repo".into(),
            deps: vec![],
        };
        assert_eq!(Workspace::clone_url(&repo), "https://host/org/repo");
    }

    #[test]
    fn clone_url_base_uses_short_name() {
        let repo = Repo {
            path: "path/arc".into(),
            remote: "origin".into(),
            url: "http://domain".into(),
            deps: vec![],
        };
        assert_eq!(Workspace::clone_url(&repo), "http://domain/arc.git");
    }

    #[test]
    fn clone_url_ssh() {
        let repo = Repo {
            path: "arc".into(),
            remote: "origin".into(),
            url: "git@host:org/repo".into(),
            deps: vec![],
        };
        assert_eq!(Workspace::clone_url(&repo), "git@host:org/repo");
    }
}

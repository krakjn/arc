use serde::Deserialize;

fn default_remote() -> String {
    "origin".into()
}

/// Deserialized `arc.toml` root
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcConfig {
    #[serde(default, rename = "project")]
    pub projects: Vec<ProjectConfig>,
}

/// A single `[[project]]` table
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub path: String,
    #[serde(default = "default_remote")]
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

/// Runtime repository entry
#[derive(Debug, Clone)]
pub struct Repo {
    pub path: String,
    pub remote: String,
    pub url: String,
    pub deps: Vec<String>,
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

    /// Full clone URL. If `url` already looks like a repo URL (ends with `.git`), use it
    /// as-is; otherwise treat it as a base and append `/{path}.git`.
    pub fn clone_url(repo: &Repo) -> String {
        let url = repo.url.trim_end_matches('/');
        if url.ends_with(".git") {
            url.to_string()
        } else {
            format!("{url}/{}.git", repo.path.trim_matches('/'))
        }
    }
}

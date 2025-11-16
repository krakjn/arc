use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use std::path::PathBuf;

/// Represents the file paths for arc configuration
#[derive(Debug, Default)]
pub struct ArcPath {
    pub tld: PathBuf,
    pub arc: PathBuf,
}

/// Represents a repository with its dependencies
#[derive(Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub deps: Vec<String>,
}

/// Main arc configuration structure
#[derive(Debug, Default)]
pub struct Arc {
    pub path: ArcPath,
    pub remote: Remote,
    pub repos: Vec<Repo>,
}

impl Arc {
    pub fn new() -> Self {
        Self::default()
    }
}

//--------------------
// J S O N
//--------------------
/// Remote repository configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

/// arc.json file structure
#[derive(Debug, Deserialize)]
pub struct ArcJson {
    pub remote: Remote,
    pub projects: Map<String, Vec<String>>,
}

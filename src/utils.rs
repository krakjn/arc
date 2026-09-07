use crate::error::{ArcError, Result};
use std::env;
use std::io::IsTerminal;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static USE_COLOR: AtomicBool = AtomicBool::new(true);

pub const CONFIG_NAME: &str = "arc.toml";

/// Resolve a project path against the config directory (`tld`).
/// Absolute paths are kept as-is; relative paths join onto `tld`.
pub fn resolve_repo_path(tld: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        tld.join(p)
    }
}

/// Parent directory and short name for cloning into `path` (absolute or relative).
pub fn clone_destination(tld: &Path, path: &str) -> (PathBuf, String) {
    let full = resolve_repo_path(tld, path);
    let short = short_name(path).to_string();
    let parent = full
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| tld.to_path_buf());
    (parent, short)
}

/// Short name of a path (`"a/b/c"` → `"c"`).
pub fn short_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}

/// True if `path` contains a `..` component.
pub fn path_has_parent_dir(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

/// True if dest looks like an existing git repository.
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// `url` already names a repo (ssh or http(s) with a path beyond the host).
pub fn looks_like_full_repo_url(url: &str) -> bool {
    let url = url.trim();
    if url.starts_with("git@") || url.starts_with("ssh://") {
        return true;
    }
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    if let Some(rest) = rest {
        return rest.split_once('/').is_some_and(|(_, p)| !p.is_empty());
    }
    false
}

/// Walk parents from `start` looking for `arc.toml`.
pub fn find_config(start: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut cwd = start.to_path_buf();
    loop {
        let candidate = cwd.join(CONFIG_NAME);
        if candidate.is_file() {
            return Ok((cwd, candidate));
        }
        if !cwd.pop() {
            return Err(ArcError::ConfigNotFound);
        }
    }
}

/// Resolve config from `--config` or upward search from CWD.
/// A directory `--config` means `dir/arc.toml`.
pub fn resolve_config(config_override: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    if let Some(path) = config_override {
        let mut path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir()?.join(path)
        };
        if path.is_dir() {
            path = path.join(CONFIG_NAME);
        }
        if !path.is_file() {
            return Err(ArcError::ConfigNotFound);
        }
        let tld = path.parent().ok_or(ArcError::ConfigNotFound)?.to_path_buf();
        return Ok((tld, path));
    }

    find_config(&env::current_dir()?)
}

/// True if a `.git` entry exists at or above `start`.
pub fn find_git(start: &Path) -> bool {
    let mut cwd = start.to_path_buf();
    loop {
        if cwd.join(".git").exists() {
            return true;
        }
        if !cwd.pop() {
            return false;
        }
    }
}

/// Extra dots after the longest path when the terminal is wide enough.
pub const PATH_DOT_PAD: usize = 10;

/// Minimum path+dots column when the terminal is tight.
pub const PATH_MIN_COL: usize = 6;

/// Max width for the branch / status label field.
pub const BRANCH_MAX_COL: usize = 25;

/// Space reserved for ahead/behind / dirty markers after the branch.
pub const MARKER_BUDGET: usize = 8;

/// Detect terminal width (`COLUMNS`, then TTY size, else 80).
pub fn terminal_columns() -> usize {
    if let Ok(cols) = env::var("COLUMNS")
        && let Ok(n) = cols.parse::<usize>()
        && n > 0
    {
        return n;
    }
    terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .filter(|&w| w > 0)
        .unwrap_or(80)
}

/// `(path_col, branch_col)` shared across the terminal width.
pub fn line_layout<'a, I>(paths: I) -> (usize, usize)
where
    I: IntoIterator<Item = &'a str>,
{
    line_layout_with(paths, terminal_columns())
}

fn line_layout_with<'a, I>(paths: I, term_cols: usize) -> (usize, usize)
where
    I: IntoIterator<Item = &'a str>,
{
    let max_path = paths
        .into_iter()
        .map(|p| p.chars().count())
        .max()
        .unwrap_or(0);
    let desired = max_path.saturating_add(PATH_DOT_PAD);

    let branch_reserve = BRANCH_MAX_COL.min(term_cols / 3).max(4);
    let path_cap = term_cols.saturating_sub(branch_reserve + MARKER_BUDGET);
    let path_col = desired
        .min(path_cap)
        .max(PATH_MIN_COL.min(term_cols.saturating_sub(MARKER_BUDGET + 4)));

    let branch_col = term_cols
        .saturating_sub(path_col + MARKER_BUDGET)
        .clamp(4, BRANCH_MAX_COL);

    (path_col, branch_col)
}

/// Format `path` into `col` width.
/// - Fits: pad with dots (longest path gets ~10).
/// - Too narrow: keep path suffix + exactly two dots (`../som..STATUS`).
pub fn format_path_column(path: &str, col: usize) -> String {
    let col = col.max(3);
    let path_len = path.chars().count();
    if path_len + 2 <= col {
        format!("{path}{}", ".".repeat(col - path_len))
    } else {
        format!("{}..", suffix_chars(path, col - 2))
    }
}

/// Branch / status label immediately after the path column (no leading pad).
pub fn format_branch_column(label: &str, col: usize) -> String {
    let col = col.max(1);
    let len = label.chars().count();
    if len > col {
        format!("{} ", suffix_chars(label, col))
    } else {
        format!("{label} ")
    }
}

fn suffix_chars(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    s.chars().skip(count - max_chars).collect()
}

/// Apply color override from `--color` / `NO_COLOR` / TTY.
pub fn apply_color(choice: clap::ColorChoice) {
    let on = match choice {
        clap::ColorChoice::Always => true,
        clap::ColorChoice::Never => false,
        clap::ColorChoice::Auto => {
            env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
        }
    };
    USE_COLOR.store(on, Ordering::Relaxed);
    owo_colors::set_override(on);
}

pub fn use_color() -> bool {
    USE_COLOR.load(Ordering::Relaxed)
}

/// Apply a style only when color is enabled.
pub fn paint(s: &str, style: impl FnOnce(&str) -> String) -> String {
    if use_color() { style(s) } else { s.to_string() }
}

/// Default `--jobs` (available parallelism, at least 1).
pub fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn resolve_relative_and_absolute() {
        let tld = PathBuf::from("/ws/arc");
        assert_eq!(
            resolve_repo_path(&tld, "path/repo"),
            PathBuf::from("/ws/arc/path/repo")
        );
        assert_eq!(
            resolve_repo_path(&tld, "/opt/repo"),
            PathBuf::from("/opt/repo")
        );
    }

    #[test]
    fn parent_dir_detection() {
        assert!(path_has_parent_dir("../jetsam"));
        assert!(path_has_parent_dir("foo/../bar"));
        assert!(!path_has_parent_dir("path/arc"));
        assert!(!path_has_parent_dir("/opt/repo"));
    }

    #[test]
    fn clone_destination_nested() {
        let tld = PathBuf::from("/ws");
        let (parent, short) = clone_destination(&tld, "path/arc");
        assert_eq!(parent, PathBuf::from("/ws/path"));
        assert_eq!(short, "arc");
    }

    #[test]
    fn full_repo_url_detection() {
        assert!(looks_like_full_repo_url("https://host/org/repo"));
        assert!(looks_like_full_repo_url("git@host:org/repo"));
        assert!(looks_like_full_repo_url("ssh://git@host/org/repo"));
        assert!(!looks_like_full_repo_url("http://domain"));
        assert!(!looks_like_full_repo_url("https://github.com"));
    }

    #[test]
    fn path_pads_when_wide() {
        let (col, _) = line_layout_with(["a", "abcd"], 100);
        assert_eq!(col, 14); // longest 4 + 10 pad
        assert_eq!(format_path_column("abcd", col), "abcd..........");
        assert_eq!(format_path_column("a", col), "a.............");
    }

    #[test]
    fn path_two_dots_only_when_truncated() {
        assert_eq!(format_path_column("../some/path", 8), "e/path..");
        assert_eq!(format_path_column("abcdefghij", 10), "cdefghij..");
        assert_eq!(format_path_column("ab", 6), "ab....");
    }

    #[test]
    fn narrow_terminal_truncates_with_two_dots() {
        let (col, branch_col) = line_layout_with(["../arc", "../reflect"], 30);
        assert!(col >= PATH_MIN_COL);
        assert!(branch_col >= 4);
        let rendered = format_path_column("../reflect", col);
        assert!(rendered.ends_with(".."));
        assert_eq!(rendered.chars().count(), col);
    }

    #[test]
    fn find_config_walks_up() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();
        let mut f = fs::File::create(tmp.path().join(CONFIG_NAME)).unwrap();
        writeln!(
            f,
            "[[repo]]\npath = \"x\"\nurl = \"http://x\"\nremote = \"origin\"\ndependencies = []"
        )
        .unwrap();

        let (tld, cfg) = find_config(&nested).unwrap();
        assert_eq!(tld, tmp.path());
        assert_eq!(cfg, tmp.path().join(CONFIG_NAME));
    }

    #[test]
    fn resolve_config_accepts_directory() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join(CONFIG_NAME),
            "[[repo]]\npath = \"x\"\nurl = \"http://x\"\nremote = \"origin\"\n",
        )
        .unwrap();
        let (tld, cfg) = resolve_config(Some(tmp.path())).unwrap();
        assert_eq!(tld, tmp.path());
        assert_eq!(cfg, tmp.path().join(CONFIG_NAME));
    }

    #[test]
    fn color_never_disables_paint() {
        apply_color(clap::ColorChoice::Never);
        assert!(!use_color());
        assert_eq!(paint("x", |s| format!("styled-{s}")), "x");
        apply_color(clap::ColorChoice::Always);
        assert!(use_color());
        assert_eq!(paint("x", |s| format!("styled-{s}")), "styled-x");
    }
}

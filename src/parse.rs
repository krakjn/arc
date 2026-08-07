//! Parsers for git porcelain / message output.

use std::sync::LazyLock;

use regex::Regex;

static BRANCH_TRACKING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^## ([^.]+)(?:\.\.\.|$)").expect("valid regex"));
static BRANCH_PLAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^## (.+)$").expect("valid regex"));
static AHEAD_BEHIND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[ahead (\d+)(?:, behind (\d+))?\]|\[behind (\d+)\]").expect("valid regex")
});
static SWITCHED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"Switch(?:ed)?.*?['"](.+?)['"]"#).expect("valid regex"));
static BRANCH_QUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"['"](.+?)['"]"#).expect("valid regex"));
static FETCHED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"remote branch ['"](.+?)['"]"#).expect("valid regex"));
static RESET_AT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"at [[:alnum:]]+ (.+)").expect("valid regex"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchHeader {
    pub name: String,
    pub tracked: bool,
    pub ahead: u32,
    pub behind: u32,
}

/// Parse the first `##` line of `git status --porcelain -b` / `-bs`.
pub fn parse_branch_header(status: &str) -> Option<BranchHeader> {
    let line = status.lines().find(|l| l.starts_with("## "))?;

    let tracked = line.contains("...");
    let name = if let Some(caps) = BRANCH_TRACKING.captures(line) {
        caps.get(1)?.as_str().to_string()
    } else if let Some(caps) = BRANCH_PLAIN.captures(line) {
        caps.get(1)?.as_str().to_string()
    } else {
        return None;
    };

    let mut ahead = 0;
    let mut behind = 0;
    if let Some(caps) = AHEAD_BEHIND.captures(line) {
        if let Some(a) = caps.get(1) {
            ahead = a.as_str().parse().unwrap_or(0);
            behind = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
        } else if let Some(b) = caps.get(3) {
            behind = b.as_str().parse().unwrap_or(0);
        }
    }

    Some(BranchHeader {
        name,
        tracked,
        ahead,
        behind,
    })
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PorcelainFiles {
    pub staged_added: Vec<String>,
    pub staged_modified: Vec<String>,
    pub staged_deleted: Vec<String>,
    pub unstaged_modified: Vec<String>,
    pub unstaged_deleted: Vec<String>,
    pub untracked: Vec<String>,
}

impl PorcelainFiles {
    pub fn has_changes(&self) -> bool {
        !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
            || !self.unstaged_modified.is_empty()
            || !self.unstaged_deleted.is_empty()
            || !self.untracked.is_empty()
    }

    pub fn has_staged(&self) -> bool {
        !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
    }

    pub fn has_unstaged(&self) -> bool {
        !self.unstaged_modified.is_empty() || !self.unstaged_deleted.is_empty()
    }
}

/// Parse file lines from porcelain status (skips `##` header lines).
pub fn parse_porcelain_files(status: &str) -> PorcelainFiles {
    let mut files = PorcelainFiles::default();

    for line in status.lines() {
        if line.starts_with("##") || line.len() < 3 {
            continue;
        }
        let xy = &line[..2];
        let path = line[3..].to_string();
        let (x, y) = (xy.as_bytes()[0], xy.as_bytes()[1]);

        match (x, y) {
            (b'?', b'?') => files.untracked.push(path),
            (b'A', _) => files.staged_added.push(path),
            (b'M', _) => files.staged_modified.push(path),
            (b'D', _) => files.staged_deleted.push(path),
            (_, b'M') => files.unstaged_modified.push(path),
            (_, b'D') => files.unstaged_deleted.push(path),
            _ => {}
        }
    }

    files
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullResult {
    UpToDate,
    Updating,
    Conflict,
    Unable,
    Ok,
}

pub fn classify_pull(stdout: &str, stderr: &str) -> PullResult {
    let out = stdout.to_ascii_lowercase();
    if out.contains("up to date") || out.contains("up-to-date") {
        PullResult::UpToDate
    } else if out.contains("updating") {
        PullResult::Updating
    } else if out.contains("conflict") {
        PullResult::Conflict
    } else if !stderr.is_empty() {
        PullResult::Unable
    } else {
        PullResult::Ok
    }
}

pub fn extract_switched(text: &str) -> Option<String> {
    SWITCHED
        .captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

pub fn already_on_branch(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("already")
}

pub fn extract_quoted_branch(text: &str) -> Option<String> {
    BRANCH_QUOTE
        .captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

pub fn extract_fetched_branch(text: &str) -> Option<String> {
    FETCHED
        .captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

pub fn extract_reset_target(text: &str) -> Option<String> {
    RESET_AT
        .captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_ahead_behind() {
        let h = parse_branch_header("## main...origin/main [ahead 2, behind 1]\n").unwrap();
        assert_eq!(h.name, "main");
        assert!(h.tracked);
        assert_eq!(h.ahead, 2);
        assert_eq!(h.behind, 1);
    }

    #[test]
    fn branch_untracked() {
        let h = parse_branch_header("## feature\n").unwrap();
        assert_eq!(h.name, "feature");
        assert!(!h.tracked);
    }

    #[test]
    fn porcelain_files() {
        let status = "## main\n M dirty.rs\nA  new.rs\n?? untracked.txt\n";
        let files = parse_porcelain_files(status);
        assert_eq!(files.unstaged_modified, vec!["dirty.rs"]);
        assert_eq!(files.staged_added, vec!["new.rs"]);
        assert_eq!(files.untracked, vec!["untracked.txt"]);
        assert!(files.has_changes());
    }

    #[test]
    fn pull_classify() {
        assert_eq!(
            classify_pull("Already up to date.\n", ""),
            PullResult::UpToDate
        );
        assert_eq!(
            classify_pull("Updating abc..def\n", ""),
            PullResult::Updating
        );
    }
}

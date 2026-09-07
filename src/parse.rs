//! Parsers for git porcelain / message output.

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
    let rest = line[3..].trim();

    if rest.starts_with("HEAD") && rest.contains("no branch") {
        return Some(BranchHeader {
            name: "HEAD".into(),
            tracked: false,
            ahead: 0,
            behind: 0,
        });
    }

    let tracked = rest.contains("...");
    let name = if let Some((name, _)) = rest.split_once("...") {
        name.trim().to_string()
    } else {
        rest.split_whitespace().next()?.to_string()
    };

    let mut ahead = 0;
    let mut behind = 0;
    if let Some(start) = rest.find('[')
        && let Some(inside) = rest[start + 1..].split(']').next()
    {
        for part in inside.split(',') {
            let part = part.trim();
            if let Some(n) = part.strip_prefix("ahead ") {
                ahead = n.trim().parse().unwrap_or(0);
            } else if let Some(n) = part.strip_prefix("behind ") {
                behind = n.trim().parse().unwrap_or(0);
            }
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
    pub renamed: Vec<String>,
    pub copied: Vec<String>,
    pub typechanged: Vec<String>,
}

impl PorcelainFiles {
    pub fn has_changes(&self) -> bool {
        self.has_staged()
            || self.has_unstaged()
            || !self.untracked.is_empty()
            || !self.renamed.is_empty()
            || !self.copied.is_empty()
            || !self.typechanged.is_empty()
    }

    pub fn has_staged(&self) -> bool {
        !self.staged_added.is_empty()
            || !self.staged_modified.is_empty()
            || !self.staged_deleted.is_empty()
            || !self.renamed.is_empty()
            || !self.copied.is_empty()
            || !self.typechanged.is_empty()
    }

    pub fn has_unstaged(&self) -> bool {
        !self.unstaged_modified.is_empty() || !self.unstaged_deleted.is_empty()
    }
}

fn porcelain_path(line: &str) -> String {
    let raw = if line.len() >= 3 { &line[3..] } else { "" };
    let raw = raw.trim_start();
    if let Some(rest) = raw.strip_prefix('"')
        && let Some(end) = rest.rfind('"')
    {
        return rest[..end].to_string();
    }
    if let Some((_, dest)) = raw.split_once(" -> ") {
        return dest.trim_matches('"').to_string();
    }
    raw.to_string()
}

/// Parse file lines from porcelain status (skips `##` header lines).
pub fn parse_porcelain_files(status: &str) -> PorcelainFiles {
    let mut files = PorcelainFiles::default();

    for line in status.lines() {
        if line.starts_with("##") || line.len() < 2 {
            continue;
        }
        let xy = line.as_bytes();
        let (x, y) = (xy[0], xy.get(1).copied().unwrap_or(b' '));
        let path = porcelain_path(line);

        match (x, y) {
            (b'?', b'?') => files.untracked.push(path),
            (b'R', _) => files.renamed.push(path),
            (b'C', _) => files.copied.push(path),
            (b'T', _) | (_, b'T') => files.typechanged.push(path),
            (b'A', _) => files.staged_added.push(path),
            (b'D', b'D') => {
                files.staged_deleted.push(path.clone());
                files.unstaged_deleted.push(path);
            }
            (b'M', b'M') => {
                files.staged_modified.push(path.clone());
                files.unstaged_modified.push(path);
            }
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

pub fn classify_pull(success: bool, stdout: &str, stderr: &str) -> PullResult {
    let out = stdout.to_ascii_lowercase();
    let err = stderr.to_ascii_lowercase();
    if out.contains("conflict") || err.contains("conflict") {
        return PullResult::Conflict;
    }
    if !success {
        return PullResult::Unable;
    }
    if out.contains("up to date") || out.contains("up-to-date") {
        PullResult::UpToDate
    } else if out.contains("updating") {
        PullResult::Updating
    } else {
        PullResult::Ok
    }
}

fn extract_quoted(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let start = text.find(['\'', '"'])?;
    let quote = bytes[start];
    let rest = &text[start + 1..];
    let end = rest.find(quote as char)?;
    Some(rest[..end].to_string())
}

pub fn extract_switched(text: &str) -> Option<String> {
    if !text.to_ascii_lowercase().contains("switch") {
        return None;
    }
    extract_quoted(text)
}

pub fn already_on_branch(text: &str) -> bool {
    text.to_ascii_lowercase().contains("already on")
}

pub fn extract_quoted_branch(text: &str) -> Option<String> {
    extract_quoted(text)
}

pub fn extract_fetched_branch(text: &str) -> Option<String> {
    let idx = text
        .find("remote branch")
        .or_else(|| text.find("Remote branch"))?;
    extract_quoted(&text[idx..])
}

pub fn extract_reset_target(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.split(" at ").nth(1) {
            let mut parts = rest.splitn(2, ' ');
            let _hash = parts.next()?;
            return Some(parts.next().unwrap_or(rest).trim().to_string());
        }
    }
    None
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
    fn branch_behind_only() {
        let h = parse_branch_header("## main...origin/main [behind 3]\n").unwrap();
        assert_eq!(h.behind, 3);
        assert_eq!(h.ahead, 0);
    }

    #[test]
    fn branch_with_dot_in_name() {
        let h = parse_branch_header("## release/1.0...origin/release/1.0 [ahead 1]\n").unwrap();
        assert_eq!(h.name, "release/1.0");
        assert!(h.tracked);
        assert_eq!(h.ahead, 1);
    }

    #[test]
    fn branch_untracked() {
        let h = parse_branch_header("## feature\n").unwrap();
        assert_eq!(h.name, "feature");
        assert!(!h.tracked);
    }

    #[test]
    fn branch_detached() {
        let h = parse_branch_header("## HEAD (no branch)\n").unwrap();
        assert_eq!(h.name, "HEAD");
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
    fn porcelain_mm_and_rename() {
        let status = "## main\nMM both.rs\nR  old.rs -> new.rs\nT  file.rs\n";
        let files = parse_porcelain_files(status);
        assert_eq!(files.staged_modified, vec!["both.rs"]);
        assert_eq!(files.unstaged_modified, vec!["both.rs"]);
        assert_eq!(files.renamed, vec!["new.rs"]);
        assert_eq!(files.typechanged, vec!["file.rs"]);
        assert!(files.has_staged());
        assert!(files.has_unstaged());
    }

    #[test]
    fn pull_classify() {
        assert_eq!(
            classify_pull(true, "Already up to date.\n", "From github.com:org/repo\n"),
            PullResult::UpToDate
        );
        assert_eq!(
            classify_pull(true, "Updating abc..def\n", "From github.com:org/repo\n"),
            PullResult::Updating
        );
        assert_eq!(
            classify_pull(false, "", "fatal: refusing to merge unrelated histories\n"),
            PullResult::Unable
        );
        assert_eq!(
            classify_pull(false, "CONFLICT (content): Merge conflict\n", ""),
            PullResult::Conflict
        );
    }

    #[test]
    fn already_on_is_specific() {
        assert!(already_on_branch("Already on 'main'\n"));
        assert!(!already_on_branch("error: already exists\n"));
    }
}

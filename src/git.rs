use crate::error::{ArcError, CommandOp, Result};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::thread;

/// How to attach the child process to the terminal.
pub enum StdioMode<'a> {
    Capture,
    Inherit,
    Prefix {
        label: &'a str,
        width: usize,
        lock: &'a Mutex<()>,
    },
}

/// Run `git` with the given args in `dir`.
pub fn run(dir: &Path, args: &[&str], mode: StdioMode<'_>) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    command(&mut cmd, mode)
}

/// Run an arbitrary command in `dir`.
pub fn run_cmd(dir: &Path, program: &str, args: &[String], mode: StdioMode<'_>) -> Result<Output> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(dir);
    command(&mut cmd, mode)
}

fn command(cmd: &mut Command, mode: StdioMode<'_>) -> Result<Output> {
    match mode {
        StdioMode::Capture => cmd
            .output()
            .map_err(|e| ArcError::command(CommandOp::Git, None, e.to_string())),
        StdioMode::Inherit => cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| ArcError::command(CommandOp::Git, None, e.to_string())),
        StdioMode::Prefix { label, width, lock } => run_prefixed(cmd, label, width, lock),
    }
}

fn run_prefixed(cmd: &mut Command, label: &str, width: usize, lock: &Mutex<()>) -> Result<Output> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ArcError::command(CommandOp::Git, None, e.to_string()))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let prefix = stream_prefix(label, width);
    let out_buf = Mutex::new(Vec::new());
    let err_buf = Mutex::new(Vec::new());

    thread::scope(|s| {
        if let Some(out) = stdout {
            s.spawn(|| pump_lines(out, &prefix, lock, &out_buf, false));
        }
        if let Some(err) = stderr {
            s.spawn(|| pump_lines(err, &prefix, lock, &err_buf, true));
        }
    });

    let status = child
        .wait()
        .map_err(|e| ArcError::command(CommandOp::Git, None, e.to_string()))?;

    Ok(Output {
        status,
        stdout: out_buf.into_inner().unwrap_or_else(|e| e.into_inner()),
        stderr: err_buf.into_inner().unwrap_or_else(|e| e.into_inner()),
    })
}

fn pump_lines(
    reader: impl std::io::Read,
    prefix: &str,
    lock: &Mutex<()>,
    buf: &Mutex<Vec<u8>>,
    stderr: bool,
) {
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if let Ok(mut b) = buf.lock() {
            b.extend_from_slice(line.as_bytes());
            b.push(b'\n');
        }
        let _g = lock.lock().unwrap_or_else(|e| e.into_inner());
        let formatted = format!("{prefix}{line}");
        if stderr {
            let _ = writeln!(std::io::stderr(), "{formatted}");
        } else {
            let _ = writeln!(std::io::stdout(), "{formatted}");
        }
    }
}

pub fn stream_prefix(label: &str, width: usize) -> String {
    use owo_colors::OwoColorize;
    let pad = width.max(label.chars().count());
    let name = format!("{label:<pad$}");
    format!(
        "{} | ",
        crate::utils::paint(&name, |s| s.cyan().to_string())
    )
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Current branch name (`rev-parse --abbrev-ref HEAD`). Errors on detached HEAD.
pub fn current_branch(dir: &Path) -> Result<String> {
    let output = run(
        dir,
        &["rev-parse", "--abbrev-ref", "HEAD"],
        StdioMode::Capture,
    )?;
    if !output.status.success() {
        return Err(ArcError::command(CommandOp::Git, None, stderr(&output)));
    }
    let name = stdout(&output).trim().to_string();
    if name.is_empty() || name == "HEAD" {
        return Err(ArcError::command(
            CommandOp::Checkout,
            None,
            "detached HEAD; pass a branch name to --deps-track",
        ));
    }
    Ok(name)
}

/// True if `git rev-parse --verify --quiet <spec>` succeeds.
pub fn ref_exists(dir: &Path, spec: &str) -> bool {
    run(
        dir,
        &["rev-parse", "--verify", "--quiet", spec],
        StdioMode::Capture,
    )
    .map(|o| o.status.success())
    .unwrap_or(false)
}

/// Default branch from `{remote}/HEAD` only (one git call). None if unset.
pub fn remote_head_branch(dir: &Path, remote: &str) -> Option<String> {
    let spec = format!("refs/remotes/{remote}/HEAD");
    let output = run(dir, &["symbolic-ref", "--short", &spec], StdioMode::Capture).ok()?;
    if !output.status.success() {
        return None;
    }
    let s = stdout(&output).trim().to_string();
    let prefix = format!("{remote}/");
    if let Some(branch) = s.strip_prefix(&prefix) {
        return Some(branch.to_string());
    }
    if s.is_empty() { None } else { Some(s) }
}

/// Detect the default branch: `{remote}/HEAD` → `main` if present → `master`.
pub fn detect_default_branch(dir: &Path, remote: &str) -> String {
    if let Some(branch) = remote_head_branch(dir, remote) {
        return branch;
    }

    if let Ok(output) = run(dir, &["branch", "-r"], StdioMode::Capture) {
        let text = stdout(&output);
        let main = format!("{remote}/main");
        let master = format!("{remote}/master");
        if text.contains(&main) {
            return "main".to_string();
        }
        if text.contains(&master) {
            return "master".to_string();
        }
    }

    "master".to_string()
}

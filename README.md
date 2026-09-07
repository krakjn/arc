```
   _____ ___________________
  /  _  \\______   \_   ____\
 /  /_\  \|       _/    \       A_ggregate
/    |    \    |   \     \____  R_epository
\____|____/____|___/\________/  C_ontrol
```
---
# The bridge across repositories

## Why?
1. Have a bunch of repos you maintain?
1. Do you jump around them a bunch? 
1. Wish you could get a bird's eye view of all of them?
1. Are some dependent on others? 

If any one of those answers is a "yes" I suggest you check out `arc`.

## Install

**Linux (musl amd64 / arm64):**

```sh
curl -fsSL https://gitlab.com/ballast-dev/arc/-/raw/main/install/get_arc.sh | sh
```

Or download `arc-linux-amd64` / `arc-linux-arm64` from [releases](https://gitlab.com/ballast-dev/arc/-/releases).

macOS and Windows binaries are not published from CI; build from source with `cargo build --release`.

## Features:

1. Multi-repo workspace — manage many git repos from one `arc.toml`
1. TOML config — per-repo `path`, `url`, required `remote`, and `dependencies`
1. Config discovery — walks up from CWD for `arc.toml`, or `--config <PATH>` (file or directory)
1. Paths next door or below `arc.toml` — relative descendants or absolute; `..` is skipped with a warning
1. Per-repo remotes — each repo names the remote used for fetch/reset/default-branch
1. Dependency-aware checkout — `arc checkout` is status for CWD + deps; `arc checkout BRANCH` aligns that tree (`git checkout --track`). A dep without BRANCH stays on HEAD or its default branch. `--repo PATH` selects another tree from any CWD.
1. Bird’s-eye status — ahead/behind, dirty/staged/untracked markers; `-v` for file detail; `--fetch` to refresh
1. Terminal-aware layout — path columns pad when wide, truncate with `..` when narrow
1. Bulk pull / fetch / sync — across every configured repo
1. Clone — clone all repos from config; skips worktrees that already exist
1. Init — write a starter `arc.toml` (`arc init [PATH]`)
1. Reset — hard reset to `{remote}/<default>`; `-y/--yes` skips the prompt; `-s/--sanitize` clears the index cache
1. Cross-repo shell — `-c <cmd>...` runs any command in every repo
1. Cross-repo git — `-g <args>...` runs `git …` in every repo
1. `--only PATH [PATH...]` — exact `path` values from `arc.toml`
1. `--sort` — alphanumeric by `path` (default: listed order in `arc.toml`)
1. `--jobs N` / `-j` — parallel work (default: CPU count). Streaming commands prefix lines like Compose (`name | …`). Use `--jobs 1` if git asks for credentials.
1. Show config — pretty-print loaded repos, URLs, and deps
1. Completions — `arc completions bash|zsh|fish`
1. Live config — re-read `arc.toml` every run

> NOTE: arc anchors on the location of `arc.toml` (or the path given by `--config`). Bare `arc` runs `status`.

```text
USAGE:
    arc [OPTIONS] [SUBCOMMAND]

OPTIONS:
    --config <PATH>        Path to arc.toml or the directory that contains it
    --only <PATH>...       Only these repo paths (exact match)
    --repo <PATH>          Select a repo by exact path (checkout tree root)
    --sort                 Alphanumeric by path (default: arc.toml order)
    -j, --jobs <N>         Parallel jobs (default: CPU count)
    --color <WHEN>         auto | always | never (also honors NO_COLOR)
    -c <COMMAND>...        Invoke any command across all repos
    -g <COMMAND>...        Invoke any git command across all repos
    -h, --help             Print help
    -V, --version          Print version

SUBCOMMANDS:
    checkout      Tree status (no args) or align CWD/deps onto BRANCH
    clone         Clone all repos defined in arc.toml
    completions   Generate shell completions
    fetch         Fetch from each repo's configured remote
    help          Print this message or the help of the given subcommand(s)
    init          Write a starter arc.toml to cwd or PATH
    pull          Fetch and merge on current branch
    reset         Reset worktree to remote default branch (-s/--sanitize, -y/--yes)
    show          Display current configuration
    status        Show status of all repos (-v/--verbose, --fetch)
    sync          Checkout default branch and pull
```

## Credential Manager

Pick your poison

  - Windows
    - install the latest [Git Credential Manager](https://github.com/microsoft/Git-Credential-Manager-for-Windows/releases)
  - Linux/Mac

          git config --global credential.helper cache
          git config --global credential.helper "store --file=~/.my-credentials"

  - The credential helper command will store credentials for 15 mins. Passing
    the `--file` argument will store your credentials permanently, however they
    are in ***plain text***. For more info look at the
    [git-docs](https://git-scm.com/book/id/v2/Git-Tools-Credential-Storage)


Example:

```toml
[[repo]]
path = "path/arc"
url = "http://domain"
remote = "origin"
dependencies = []

[[repo]]
path = "path/other"
remote = "upstream"
url = "http://other.domain"
dependencies = ["path/arc"]
```

### Workflow

Along with easing development by automating synchronization of the entire project, `arc` can prove useful as a workflow tool. Developers working on a project push to their specific branches and send a merge request to the maintainer when the issue is complete.

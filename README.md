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

**Linux / macOS:**

```sh
curl -fsSL https://gitlab.com/ballast-dev/arc/-/raw/main/install/get_arc.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://gitlab.com/ballast-dev/arc/-/raw/main/install/get_arc.ps1 | iex
```

Or download a binary from [releases](https://gitlab.com/ballast-dev/arc/-/releases):
`arc-linux-amd64`, `arc-macos-arm64`, `arc-macos-amd64`, `arc-windows-amd64.exe`.

## Features:

1. Multi-repo workspace — manage many git repos from one arc.toml
1. TOML config — per-project path, url, optional remote (default origin), and dependencies
1. Config discovery — walks up from CWD for arc.toml, or use --config <PATH>
1. Relative or absolute paths — including ../sibling layouts
1. Per-project remotes — each project can point at a different host/URL
1. Dependency-aware checkout — checkout / checkout -b switches deps onto the same branch first
1. Bird’s-eye status — ahead/behind, dirty/staged/untracked markers; -v for file detail
1. Terminal-aware layout — path columns pad when wide, truncate with .. when narrow
1. Bulk pull — pull across every configured repo
1. Sync to default branch — detect main/master (or origin/HEAD), checkout, pull; clone if missing
1. Init — clone all projects from config in one shot
1. Init-config — write a starter `arc.toml` (`arc init-config [PATH]`)
1. Reset — hard reset to origin/<default>; optional --sanitize clears the index cache
1. Cross-repo shell — -c <cmd>... runs any command in every repo
1. Cross-repo git — -g <args>... runs git … in every repo
1. Show config — pretty-print loaded projects, URLs, and deps
1. Live config — re-read arc.toml every run (add/remove projects without restart)

> NOTE: arc anchors on the location of `arc.toml` (or the path given by `--config`).

```text
USAGE:
    arc [OPTIONS] [SUBCOMMAND]

OPTIONS:
    --config <PATH>        Path to arc.toml (skips upward search)
    -c <COMMAND>...        Invoke any command across all repos
    -g <COMMAND>...        Invoke any git command across all repos
    -h, --help             Print help
    -V, --version          Print version

SUBCOMMANDS:
    checkout      Checks out all dependent repos on same branch
    help          Print this message or the help of the given subcommand(s)
    info          Print contact info and purpose of arc
    init          Initialize all repos defined in arc.toml
    init-config   Write a starter arc.toml to cwd or PATH
    pull          Fetch and merge on current branch
    reset         Reset worktree to origin default branch (-s/--sanitize)
    show          Display current configuration
    status        Show status of all repos (-v/--verbose)
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
[[project]]
path = "path/arc"
url = "http://domain"
dependencies = []

[[project]]
path = "path/other"
remote = "upstream"          # optional; default "origin"
url = "http://other.domain"
dependencies = ["path/arc"]
```

### Workflow

Along with easing development by automating synchronization of the entire project, `arc` can prove useful as a workflow tool. Developers working on a project push to their specific branches and send a merge request to the maintainer when the issue is complete.

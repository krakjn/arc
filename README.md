# arc - Aggregate Repository Control

## _The bridge across repositories_

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
    checkout    Checks out all dependent repos on same branch
    help        Print this message or the help of the given subcommand(s)
    info        Print contact info and purpose of arc
    init        Initialize all repos defined in arc.toml
    pull        Fetch and merge on current branch
    reset       Reset worktree to origin default branch (-s/--sanitize)
    show        Display current configuration
    status      Show status of all repos (-v/--verbose)
    sync        Checkout default branch and pull
```

### 1) Credential Manager

- Windows
  - install the latest [Git Credential Manager](https://github.com/microsoft/Git-Credential-Manager-for-Windows/releases)
- Linux/Mac

        git config --global credential.helper cache
        git config --global credential.helper "store --file=~/.my-credentials"

- The credential helper command will store credentials for 15 mins. Passing
  the `--file` argument will store your credentials permanently, however they
  are in ***plain text***. For more info look at the
  [git-docs](https://git-scm.com/book/id/v2/Git-Tools-Credential-Storage)

### 2) Place [this](config/arc.toml) configuration file in the top-level folder

**Parsed on the fly** — adding or removing `[[project]]` entries is picked up on every run.

> NOTE: arc anchors on the location of `arc.toml` (or the path given by `--config`).

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

Hard cutover from older releases: `arc.json` is no longer read — convert to `arc.toml`.

### 3) Download latest [**release**](http://gitlab.com/krakjn/arc/-/releases) for your system

### 4) Place `arc` on PATH

- Windows: create directory `C:\bin`, append to system PATH
- Mac: `/usr/local/bin`
- Linux: `/usr/bin/` for all users or `~/bin` just for you

### 5) Initialize

- run `arc init`
- arc will clone all repos defined in `arc.toml` using each project's `url` + `path`

### Workflow

Along with easing development by automating synchronization of the entire project, `arc` can prove useful as a workflow tool. Developers working on a project push to their specific branches and send a merge request to the maintainer when the issue is complete.

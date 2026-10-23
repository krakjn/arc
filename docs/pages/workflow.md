# Workflow

`arc` keeps a set of repositories in step, and it is also a way to move a change across the ones that depend on each other. Developers push their own branches and open a merge request for the maintainer when the issue is done.

For `arc.toml` fields, see [Configuration](config.md). For every flag, see [Commands](commands.md).

## A tree of repos

From a repo in the workspace, status covers that repo and its dependencies. Checking out a branch aligns the same tree. `--repo` starts from a different `path` without changing directory first.

```bash
arc checkout
arc checkout feature/bridge
arc checkout feature/bridge --repo path/web
```

A dependency that does not have the branch stays on HEAD or its default branch. The checkout uses `git checkout --track`.

## Stay current

`sync` checks out the default branch and pulls. `pull` fetches and merges whatever branch is already checked out. `fetch` only updates the configured remote.

```bash
arc sync
arc pull
arc fetch
arc status --fetch
```

`arc status` is the bird’s-eye view: ahead/behind, and dirty, staged, and untracked markers. `-v` adds file detail.

## Run a command everywhere

`-c` runs a shell command in every repo. `-g` runs git. Streaming output is prefixed like Compose (`name | …`). Use `--jobs 1` when git asks for credentials, and `--only` when the command should not cross the whole bridge.

```bash
arc -c cargo test
arc -g status -sb
arc -g diff --only path/api path/web
arc --sort -j 4 -c make check
```

## Reset

Reset is a hard reset to `{remote}/<default>`. It asks first. `-y` skips the prompt. `-s` clears the index cache.

```bash
arc reset
arc reset --yes --sanitize
```

## Credentials

Git still needs a credential helper. `arc` does not store one.

- **Windows.** Install the latest [Git Credential Manager](https://github.com/microsoft/Git-Credential-Manager-for-Windows/releases).
- **Linux and macOS.** Cache for 15 minutes, or store them in a file.

```bash
git config --global credential.helper cache
git config --global credential.helper "store --file=~/.my-credentials"
```

The `--file` form keeps credentials permanently, in plain text. See [Git’s credential storage](https://git-scm.com/book/id/v2/Git-Tools-Credential-Storage).

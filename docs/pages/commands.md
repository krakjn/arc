# Commands

`arc` anchors on the location of `arc.toml`, or the path given by `--config`. Bare `arc` runs `status`.

```text
arc [OPTIONS] [SUBCOMMAND]
```

## Global options

| Option | What it does |
|--------|----------------|
| `--config <PATH>` | Path to `arc.toml`, or the directory that contains it |
| `--only <PATH>...` | Only these repo paths (exact `path` from `arc.toml`) |
| `--repo <PATH>` | Select a repo by exact path (checkout tree root) |
| `--sort` | Alphanumeric by `path` (default: order in `arc.toml`) |
| `-j`, `--jobs <N>` | Parallel jobs (default: CPU count). Use `1` if git asks for credentials |
| `--color <WHEN>` | `auto`, `always`, or `never` (also honors `NO_COLOR`) |
| `-c <COMMAND>...` | Invoke any command across all repos |
| `-g <COMMAND>...` | Invoke any git command across all repos |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

`-c` and `-g` cannot be used together. Streaming commands prefix lines like Compose (`name | …`).

## checkout

Status of the working directory and its dependencies, or align that tree onto a branch.

```bash
arc checkout            # status for this tree
arc checkout BRANCH     # git checkout --track onto BRANCH
arc checkout BRANCH --repo path/web
```

`arc checkout` with no branch is status for the current repo and its dependencies. `arc checkout BRANCH` aligns that tree. A dependency that does not have `BRANCH` stays on HEAD or its default branch. `--repo PATH` selects another tree from any working directory. The path is an exact `path` value from `arc.toml`.

## clone

Clone every repo defined in `arc.toml`. Worktrees that already exist are skipped.

```bash
arc clone
```

## completions

Generate shell completions.

```bash
arc completions bash
arc completions zsh
arc completions fish
```

## fetch

Fetch from each repo's configured remote.

```bash
arc fetch
```

## init

Write a starter `arc.toml` to the working directory, or to `PATH`.

```bash
arc init
arc init PATH
```

## pull

Fetch and merge on the current branch, in every configured repo.

```bash
arc pull
```

## reset

Hard reset each worktree to `{remote}/<default>`.

```bash
arc reset                 # asks before resetting
arc reset -y              # skip the prompt
arc reset -s              # also clear the index cache
arc reset --yes --sanitize
```

`-y` / `--yes` skips the confirmation prompt. `-s` / `--sanitize` clears the index cache.

## show

Pretty-print the loaded repos, URLs, and dependencies.

```bash
arc show
```

## status

Show the status of every repo in `arc.toml`. This is also what bare `arc` runs.

```bash
arc status
arc status -v        # file detail
arc status --fetch   # fetch remotes before ahead/behind
```

The table shows ahead/behind, and dirty, staged, and untracked markers. Path columns pad when the terminal is wide and truncate with `..` when it is narrow.

## sync

Check out the default branch and pull, across the configured repos.

```bash
arc sync
```

## Cross-repo commands

Run one command in every repo. Use `--only` to limit the set, `--sort` to order by path, and `--jobs` to cap parallelism.

```bash
arc -c cargo test
arc -g status -sb
arc -g pull --only path/api
```

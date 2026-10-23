# Configuration

`arc` reads `arc.toml` on every run. It walks up from the working directory until it finds that file, unless you pass `--config <PATH>` with the file or the directory that contains it.

## Fields

Each repo is a `[[repo]]` table.

| Field | Required | Purpose |
|-------|----------|---------|
| `path` | yes | Worktree path. Exact value used by `--only` and `--repo` |
| `url` | yes | Clone URL, or a host/base that clone turns into a repo URL |
| `remote` | yes | Remote used for fetch, reset, and the default branch |
| `dependencies` | no | Other `path` values this repo depends on. Default: none |

## Paths

Paths may be relative descendants of the directory that holds `arc.toml`, or absolute. A path that climbs with `..` is skipped, with a warning. The anchor is the `arc.toml` location, not whatever directory you happened to start in.

## Dependencies

`dependencies` lists other `path` values. `arc checkout` uses that list: with no branch it reports status for the current repo and its dependencies, and with a branch it aligns that tree. A dependency that does not have the requested branch stays on HEAD or its default branch.

## Example

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

`arc init` writes a starter file. `arc show` prints the repos, URLs, and dependencies that loaded.

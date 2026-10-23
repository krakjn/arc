# The bridge across repositories

Have a bunch of repos you maintain? Do you jump around them? Wish you could see all of them at once, including the ones that depend on each other? That is what `arc` is for.

## Why?

- A bunch of repos, one place to stand.
- A bird’s-eye view instead of `cd` and `git status` in each one.
- Dependencies between those repos, checked out as a tree.

## Features

- **One config.** Every repo’s `path`, `url`, `remote`, and `dependencies` live in `arc.toml`.
- **One status.** Ahead, behind, and dirty, for the whole tree.
- **Checkout follows dependencies.** A branch moves the repos that have it. A repo without that branch stays.
- **One command, every repo.** `-c` runs a shell command. `-g` runs git.

The full command list is on [Commands](commands.md). `arc.toml` is documented in [Configuration](config.md). A working day is in [Workflow](workflow.md).

> arc anchors on the location of `arc.toml`, or the path given by `--config`. Bare `arc` runs `status`.

## Install

Linux, macOS, and Windows each have a one-line install. The shell script covers Linux (musl) and macOS. PowerShell covers Windows.

```bash
curl -fsSL https://raw.githubusercontent.com/krakjn/arc/main/install/get_arc.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/krakjn/arc/main/install/get_arc.ps1 | iex
```

## Download latest

Files on the release are `arc-<version>-<platform>`. The cards fill in the current version from GitHub. On Linux and macOS, mark the file executable and put it on your `PATH`: `chmod +x arc && mv arc ~/.local/bin/`.

<div class="fittings">
<a class="fitting" data-platform="linux-amd64" data-asset="linux-amd64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">Linux</span><span class="fitting-arch">amd64</span><code>arc-&lt;version&gt;-linux-amd64</code></a>
<a class="fitting" data-platform="linux-arm64" data-asset="linux-arm64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">Linux</span><span class="fitting-arch">arm64</span><code>arc-&lt;version&gt;-linux-arm64</code></a>
<a class="fitting" data-platform="macos-arm64" data-asset="macos-arm64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">macOS</span><span class="fitting-arch">Apple silicon</span><code>arc-&lt;version&gt;-macos-arm64</code></a>
<a class="fitting" data-platform="macos-amd64" data-asset="macos-amd64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">macOS</span><span class="fitting-arch">Intel</span><code>arc-&lt;version&gt;-macos-amd64</code></a>
<a class="fitting" data-platform="windows-amd64" data-asset="windows-amd64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">Windows</span><span class="fitting-arch">amd64</span><code>arc-&lt;version&gt;-windows-amd64.exe</code></a>
<a class="fitting" data-platform="windows-arm64" data-asset="windows-arm64" href="https://github.com/krakjn/arc/releases/latest"><span class="fitting-os">Windows</span><span class="fitting-arch">arm64</span><code>arc-&lt;version&gt;-windows-arm64.exe</code></a>
</div>

## Quick start

```bash
arc init          # write a starter arc.toml
arc clone         # clone every repo in the config
arc               # status of every repo (bare arc runs status)
arc checkout main # align this tree and its dependencies onto main
```

Next: [Commands](commands.md) for the full reference, [Configuration](config.md) for `arc.toml`, or [Workflow](workflow.md) for a day on the bridge.

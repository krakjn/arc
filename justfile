# Site tasks. `just` lists them. `just show` opens the docs site.

set quiet

# List the recipes
default:
    @just --list

# Serve the docs site and open it
show:
    #!/usr/bin/env bash
    set -euo pipefail
    url="http://127.0.0.1:8000/"
    printf 'docs  %s\n' "$url"
    sh "{{justfile_directory()}}/docs/build.sh" serve &
    pid=$!
    trap 'kill "$pid" 2>/dev/null || true' EXIT INT TERM
    for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do
        curl -sf -o /dev/null "$url" && break
        sleep 0.25
    done
    open "$url" || true
    wait "$pid"

# Serve the docs site
docs:
    sh "{{justfile_directory()}}/docs/build.sh" serve

# Build the docs site into docs/site
docs-build:
    sh "{{justfile_directory()}}/docs/build.sh" build

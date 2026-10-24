_:
    @just --list
    
build:
    bump update Cargo.toml
    cargo build --release

test:
    cargo test

lint:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt --all

# Serve the docs site
docs:
    sh "{{justfile_directory()}}/docs/build.sh" serve

# Build the docs site into docs/site
docs-build:
    sh "{{justfile_directory()}}/docs/build.sh" build

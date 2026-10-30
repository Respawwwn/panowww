# Panowww tasks. `just` lists them.

default:
    @just --list

# Everything CI checks: format, lints, typos, dependency policy.
check: fmt-check lint
    typos
    cargo deny check

# Clippy on every crate and target, warnings as errors.
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Unit, integration and doc tests.
test:
    cargo nextest run --workspace --all-features --no-tests=warn
    cargo test --workspace --all-features --doc

# Fail if any file is not rustfmt-formatted.
fmt-check:
    cargo fmt --all --check

# Format every crate.
fmt:
    cargo fmt --all

# Run the app in development (the Tauri app arrives with issue 2).
dev:
    @echo "No app yet: issue 2 adds the Tauri app and wires 'just dev'."

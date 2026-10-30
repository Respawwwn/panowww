# Panowww tasks. `just` lists them.

default:
    @just --list

# Everything CI checks: format, lints, typos, dependency policy, UI lint and types.
check: fmt-check generate lint
    typos
    cargo deny check
    pnpm lint
    pnpm check

# Clippy on every crate and target, warnings as errors.
lint: ui-build
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Rust unit, integration and doc tests, then the UI tests.
test: ui-build generate
    cargo nextest run --workspace --all-features --no-tests=warn
    cargo test --workspace --all-features --doc
    pnpm test

# Write the TypeScript of every `#[ts(export)]` type into ui/src/lib/generated.
generate:
    cargo test --workspace --all-features export_bindings

# Build the UI bundle the app crate embeds; the app does not compile without it.
ui-build:
    pnpm --dir ui build

# Fail if any file is not rustfmt-formatted.
fmt-check:
    cargo fmt --all --check

# Format every crate and the UI.
fmt:
    cargo fmt --all
    pnpm format

# Run the app with hot reload.
dev: generate
    pnpm tauri dev

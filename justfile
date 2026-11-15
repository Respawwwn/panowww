set positional-arguments

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

# Render app/icons from assets/icon/panowww.svg (needs rsvg-convert).
icons:
    assets/icon/build.sh

# Write fake Games and Sessions into the app's folders for the UI pass
# (`just fixtures --game "My Game"` adds Sessions to an existing Game,
# `--clean` removes the fixture Games first).
fixtures *ARGS:
    cargo run -q -p panowww-store --example fixtures -- "$@"

# Stitch the real Sessions of the sibling repository panowww-stitch-fixtures
# again and hold them to their approved Panoramas (crates/store/tests/
# real_sessions.rs). Expects the checkout next to this one, or
# `PANOWWW_STITCH_FIXTURES` pointing at it. Pass a filter to run one Session.
stitch-regress *ARGS:
    cargo nextest run --release -p panowww-store --test real_sessions --run-ignored only --no-tests=fail "$@"

# Approve the current code's output as this platform's reference (the
# floating point differs per CPU, so macos-aarch64, windows-x86_64 and
# linux-x86_64 each keep their own): rewrites the Panoramas in the fixtures'
# expected/<platform>/ and the report snapshots here. Review the Panoramas by
# eye before committing either.
stitch-regress-accept *ARGS:
    PANOWWW_STITCH_FIXTURES_WRITE=1 INSTA_UPDATE=always just stitch-regress "$@"

# Copy a Session folder (its Shots and session.json, not its Panorama) into
# the fixtures as `sessions/SLUG`; then add a `real_session!` line to the test
# and run `just stitch-regress-accept SLUG`.
stitch-fixture-add SESSION SLUG FIXTURES="../panowww-stitch-fixtures":
    #!/usr/bin/env bash
    set -euo pipefail
    dest="$3/sessions/$2"
    mkdir -p "$dest"
    cp "$1"/r*_c*.png "$dest/"
    jq 'del(.panorama)' "$1/session.json" > "$dest/session.json"
    echo "$(find "$dest" -name 'r*_c*.png' | wc -l | tr -d ' ') Shots copied to $dest"


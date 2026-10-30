# Panowww

Captures 360° panorama source Shots inside video games by driving the in-game camera and capturing a zone of the screen. Rust + Tauri 2, Svelte UI, macOS first, then Windows.

- Spec: `docs/spec.md`. Backlog: `docs/implementation-plan.md`. Decisions: `docs/adr/`.
- Read `GLOSSARY.md` before writing code or prose.

## Workflow

- **One branch per issue, never commit on `main`.** Start from up-to-date `main`, name the branch `feat/NN-short-slug` (for example `feat/03-game-schema`), commit on it in small steps with clear messages, and stop there. The human reviews, squashes or reorders, and merges. Open no PR unless asked.
- If an issue needs work from an unmerged branch, say so and wait; do not branch off it.
- Done = the issue's acceptance list passes. Manual checks go into `docs/manual-test.md`.

## Vocabulary

Use the glossary terms in identifiers, comments, strings and commit messages: Game, Input device, Capture zone, Session, Shot, Test (Turn test, Pitch test), Step, Step once, Row, Overlap, Sample, Check overlap, Display preview, Nadir / Zenith. Never use the `_Avoid_` words (profile, config, preset, region, screenshot, dry-run, live view…).

## Crates

| Crate | Path | Role |
|---|---|---|
| `panowww-core` | `crates/core` | Game schema, capture plan, Runner, screen-unchanged comparison, duration estimate, ports. Fakes behind `test-support`. |
| `panowww-input` | `crates/input` | `Camera` adapters (CGEvent, SendInput, ViGEm), input prerequisite checks. |
| `panowww-capture` | `crates/capture` | `Grabber` / `Stream` adapters (ScreenCaptureKit, WGC), displays, Display preview. |
| `panowww-store` | `crates/store` | Everything on disk: Games, settings, Session folders, PNG worker, zip. Implements `ShotSink`. |
| `panowww-app` | `app` | Thin Tauri shell: commands (`src/commands.rs`, each listed in `build.rs` and allowed in `capabilities/default.json`), the single Runner owner, overlay, shortcuts. |
| UI | `ui` | Svelte 5 + TypeScript + Vite, pnpm, Biome, Vitest. `src/lib/api.ts` wraps every command; `src/lib/strings.ts` holds every user-visible string; `src/lib/generated/` is written by ts-rs, never by hand. |

Rules:
- **No OS code in `core` or `store`.** Both `#![forbid(unsafe_code)]`. `store` touches paths only.
- Movement and timing logic lives in `core`; OS adapters only perform primitives (ADR 0001). Need something new from an OS? Add a primitive.
- `input` and `capture` expose an `unsupported` module on other OSes so the workspace builds on Linux.
- Errors: `thiserror` per crate, no `anyhow`. Logs: `tracing`.
- Every crate starts with a `//!` doc stating its role and interface.
- Lints live in `[workspace.lints]` in `Cargo.toml`; each crate opts in with `[lints] workspace = true`. Do not silence pedantic lints locally without a reason in a comment.
- The version lives only in the workspace `Cargo.toml`.

## Checks

Tools: `just`, `cargo-nextest`, `cargo-deny`, `typos` (`brew install just cargo-nextest cargo-deny typos-cli`), Node 24 and pnpm (`corepack enable pnpm`, then `pnpm install` at the root). The toolchain is pinned in `rust-toolchain.toml`; the Tauri CLI is an npm dev dependency (`pnpm tauri`).

- `just check`: rustfmt, clippy `-D warnings`, typos, cargo-deny, Biome, svelte-check.
- `just test`: nextest, doc tests, Vitest.
- `just generate`: writes `ui/src/lib/generated/` from every `#[ts(export)]` type. Run it after changing a type the UI sees.
- `just dev`: the app with hot reload. `just lint`, `just fmt`, `just ui-build`. `just icons` renders `app/icons/` from `assets/icon/panowww.svg`.

The app crate embeds `ui/dist`, so `just lint` and `just test` build the UI first. CI (`.github/workflows/ci.yml`) runs the same on macOS and Windows, plus `core` + `store` and a library-crate build on Linux. Actions are pinned by SHA; run `actionlint` after editing workflows.

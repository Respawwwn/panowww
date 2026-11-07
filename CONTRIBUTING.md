# Contributing to Panowww

Thanks for helping. Bug reports, Games that don't work, ideas and pull requests are all welcome.

## Before you start

- **Found a bug or a game that doesn't work?** Open an [issue](https://github.com/Respawwwn/panowww/issues/new/choose) with the matching template. For a game, attach the Game file (*Share* in the Game's header): it lets us reproduce your setup.
- **Want to change something big?** Open an issue first so we can agree on the approach before you spend time on it.
- **Read [GLOSSARY.md](GLOSSARY.md).** Panowww has its own words (Game, Capture zone, Session, Shot, Step, Row…) and we use them everywhere: code, UI, commit messages.

## Set up the project

Panowww is a Rust + Tauri 2 app with a Svelte UI. It builds on macOS (Apple Silicon) and Windows (x64).

1. Install [Rust](https://rustup.rs). The toolchain version is pinned in `rust-toolchain.toml` and installs itself on the first build. On Windows, also install Visual Studio Build Tools with *Desktop development with C++*.
2. Install Node 24, then enable pnpm: `corepack enable pnpm`.
3. Install the tools: `cargo install just cargo-nextest cargo-deny typos-cli` (or `brew install just cargo-nextest cargo-deny typos-cli` on macOS).
4. At the repository root: `pnpm install`.
5. Run the app with hot reload: `just dev`.

`just fixtures` fills the app with fake Games and Sessions, handy to work on the UI.

## Before you open a pull request

```sh
just check   # rustfmt, clippy, typos, cargo-deny, Biome, svelte-check
just test    # Rust tests, doc tests, Vitest
```

Both must pass: clippy runs with warnings as errors. If you touched a Rust type the UI uses, run `just generate` and commit the result.

Anything only a human with a game can verify (camera moves, the overlay, captures) goes through the checklist in [docs/manual-test.md](docs/manual-test.md). Tell us in the pull request which rows you ran, on which OS and with which game.

## Where things live

| Folder | What |
|---|---|
| `crates/core` | Game files, capture plan, the Session runner. No OS code. |
| `crates/input` | Moving the camera: CGEvent on macOS, SendInput and ViGEm on Windows. |
| `crates/capture` | Capturing the zone: ScreenCaptureKit on macOS, Windows Graphics Capture on Windows. |
| `crates/store` | Everything on disk: Games, settings, Session folders. No OS code. |
| `app` | The Tauri shell: commands, overlay, shortcuts. |
| `ui` | The Svelte interface. Every visible string is in `ui/src/lib/strings.ts`. |

The full design is in [docs/spec.md](docs/spec.md), and the reasons behind the big choices in [docs/adr/](docs/adr/).

## License

By contributing, you agree that your contribution is released under the [MIT License](LICENSE).

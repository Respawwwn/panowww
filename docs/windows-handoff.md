# Windows handoff

The macOS pass (issue 19) is done on a Mac; the Windows work (issues 23 to
26) runs on a Windows machine with its own Claude Code session. This note
is what that session needs beyond `CLAUDE.md`, `GLOSSARY.md`, `docs/spec.md`
and `docs/implementation-plan.md`. Delete it when issue 26 is closed.

## Where things stand (2026-11-01)

- Issues 1 to 20 and the macOS pass (19) are merged in `main`: every macOS
  row of `docs/manual-test.md` passes, two displays included, except I3,
  which needs a mouse-look game, and P6, out of scope by decision.
- The desk research the spec was written from is in `docs/research/`; the
  Windows sections of `app-shell.md`, `input-injection.md` and
  `screen-capture.md` are the starting point of issues 23 to 25.
- No Windows code exists yet: `crates/input` and `crates/capture` have a
  `macos` module and an `unsupported` module, nothing else. The `app` crate
  builds on Windows but has no overlay flags, manifest or NSIS specifics.
- The Windows facts W1 to W10 (`docs/spec.md` §15) are verified except W9;
  the spike that produced them is `spike/` on
  `prototype/overlay-shortcut-spike`, kept until issue 25 ports its window
  code.
- GitHub Actions is switched off on `Respawwwn/panowww`; `just check` and
  `just test` run locally instead.

## Machine setup

Native Windows, not WSL: the adapters call Win32 and the game runs on the
Windows desktop.

1. Git for Windows (`winget install Git.Git`), `git config --global
   core.autocrlf false`.
2. Rust: rustup with the MSVC host; the pinned toolchain in
   `rust-toolchain.toml` installs itself on the first `cargo` call. Visual
   Studio Build Tools with "Desktop development with C++" and a Windows 11
   SDK.
3. Node 24, `corepack enable pnpm`, `pnpm install` at the repository root.
4. `cargo install just cargo-nextest cargo-deny typos-cli`.
5. ViGEmBus 1.22.0 for the gamepad Game.
6. Clone into a Windows folder (`C:\www\panowww`), never under `\\wsl$`.
7. `just check`, then `just dev`: the app must open before any Windows work.

Games for the pass: NoClip (noclip.website in a browser) for keyboard and
mouse drag, one mouse-look game, one gamepad game.

## Order of work

1. **Spike verification.** Done on 2026-11-01 (W9 left to issue 26).
2. **Issue 23** `feat/23-input-windows`: `SendInput` Camera, ViGEm adapter,
   `BusNotFound` check, per-OS sub-move constants.
3. **Issue 24** `feat/24-capture-windows`: WGC Stream, preview, display list,
   `WDA_EXCLUDEFROMCAPTURE` on our windows, DXGI fallback only if W6 fails.
4. **Issue 25** `feat/25-app-windows`: overlay or in-app fallback, raw-input
   shortcut listener if W3 asks for it, manifest, NSIS, WebView2 memory
   target, Windows strings and chrome.
5. **Issue 26**: the Windows column of `docs/manual-test.md`, same protocol as
   the macOS pass, from the installer when issue 21 exists, from
   `pnpm tauri build` otherwise.

Rules do not change on Windows: one branch per issue from up-to-date `main`,
small commits, no PR, the human merges. The macOS crates must keep building:
`cargo check` for the whole workspace stays green on both OSes.

## First prompt on the Windows machine

> Read CLAUDE.md, GLOSSARY.md and docs/windows-handoff.md. Set up the
> machine per the handoff, confirm `just check` and `just dev` work, then
> start the spike verification (W1 to W10).

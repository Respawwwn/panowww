# Keeping a Rust desktop app clean and regression-free

Research for Panowww: what tools and practices keep a Rust desktop app (macOS + Windows, probably Tauri 2, maybe pure Rust) clean and free of regressions, and which of them Panowww should adopt.

Researched 2026-10-29. Versions come from crates.io, the npm registry and GitHub Releases on that date. "Real repo" facts come from the default branch of each repo on that date.

---

## 1. Facts

### 1.1 Current versions (2026-10-29)

| Thing | Version | Source |
|---|---|---|
| Rust stable | 1.99.1 (1.100 due 2026-11-12) | [releases.rs](https://releases.rs/) |
| Tauri (crate) | 2.12.5; `@tauri-apps/api` / `@tauri-apps/cli` 2.12.4 | [crates.io](https://crates.io/crates/tauri), [npm](https://www.npmjs.com/package/@tauri-apps/api) |
| cargo-nextest | 0.9.150 | [crates.io](https://crates.io/crates/cargo-nextest) |
| cargo-llvm-cov | 0.9.1 | [crates.io](https://crates.io/crates/cargo-llvm-cov) |
| cargo-deny | 0.20.2 (action `EmbarkStudios/cargo-deny-action` v2.1.1) | [crates.io](https://crates.io/crates/cargo-deny), [GH](https://github.com/EmbarkStudios/cargo-deny-action) |
| cargo-audit | 0.22.2 | [crates.io](https://crates.io/crates/cargo-audit) |
| cargo-machete / cargo-shear / cargo-udeps | 0.9.2 / (Zed pins 1.13.4) / 0.1.61 | crates.io; [Zed run_tests.yml](https://github.com/zed-industries/zed/blob/main/.github/workflows/run_tests.yml) |
| cargo-hack / cargo-msrv | 0.6.45 / 0.19.6 | crates.io |
| proptest / insta / mockall | 1.12.0 / 1.49.0 / 0.15.0 | crates.io |
| typos-cli | 1.51.1 | crates.io |
| tauri-action | v1.0.0 (2026-06-29) | [GH](https://github.com/tauri-apps/tauri-action) |
| Swatinem/rust-cache | v2.9.3 | [GH](https://github.com/Swatinem/rust-cache) |
| taiki-e/install-action | v2.88.0 | [GH](https://github.com/taiki-e/install-action) |
| release-please-action / release-plz / dist (cargo-dist) | v5.0.0 / 0.4.2 / 0.33.1 | GH releases |
| prek / lefthook | 0.5.7 / 2.2.2 | GH releases |
| `@wdio/tauri-service` / `tauri-plugin-wdio-webdriver` | 1.5.1 / 1.5.1 | npm, crates.io |
| Biome / oxlint / ESLint / Prettier / Vitest / Playwright | 2.6.4 / 1.90.0 / 10.13.0 / 3.9.10 / 5.0.5 / 1.64.1 | npm |

GitHub-hosted runners: `macos-latest` is macOS 26 on arm64, and `windows-latest` is Windows Server 2025 on x64. Intel macOS is still available as `macos-15-intel` and `macos-26-intel`, and Windows on Arm as `windows-11-arm`. macOS 14 images are deprecated. ([actions/runner-images](https://github.com/actions/runner-images))

### 1.2 Tests

**Unit and integration.** `cargo test` with `#[cfg(test)]` modules, plus `tests/` for integration tests. This is standard and needs no citation.

**nextest.** nextest runs each test in its own process and supports parallelism, retries, per-test timeouts, test groups and JUnit output. It does **not run doctests**, so projects run `cargo test --doc` next to it. ([nexte.st coverage docs](https://nexte.st/docs/integrations/test-coverage/)) Real use:
- GitButler runs `cargo nextest run --profile ci`, then `cargo test --doc`. Its `.config/nextest.toml` sets `fail-fast = false` and writes JUnit to `junit.xml`. ([push.yaml](https://github.com/gitbutlerapp/gitbutler/blob/master/.github/workflows/push.yaml), [nextest.toml](https://github.com/gitbutlerapp/gitbutler/blob/master/.config/nextest.toml))
- Zed runs `cargo nextest run --workspace --no-fail-fast` on Windows, Linux and macOS. Its nextest config uses `slow-timeout`, test groups to run DB tests one at a time, and priorities. ([run_tests.yml](https://github.com/zed-industries/zed/blob/main/.github/workflows/run_tests.yml), [nextest.toml](https://github.com/zed-industries/zed/blob/main/.config/nextest.toml))

**Property-based (proptest).** proptest stores failing seeds under `proptest-regressions/`, and the docs say "It is recommended to check these files in to your source control." ([proptest book](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html)) For Panowww, proptest fits pure maths: step/row/overlap planning, angle wrap-around, and Game file round-trips (serialize then parse gives the same value).

**Snapshot (insta).** Macros such as `assert_snapshot!`, `assert_debug_snapshot!` and `assert_json_snapshot!`/`assert_yaml_snapshot!`, with redactions. Changes are reviewed with `cargo insta review` or `cargo insta test --review`. When the `CI` env var is set, `INSTA_UPDATE=auto` resolves to `no`, so CI never writes snapshots. ([insta quickstart](https://insta.rs/docs/quickstart/), [insta docs](https://insta.rs/docs/)) For Panowww, insta fits the serialized Game files, the generated capture plan (the list of moves and Shots for a Game), and error messages.

**Fakes for OS-bound code.** The usual pattern is a narrow trait at each OS boundary (for example `InputSink` for key, mouse and pad events, `ScreenGrabber` for capturing a rect, `Clock` for sleeping), real adapters per OS, and a hand-written in-memory fake that records calls for tests. `mockall` 0.15 can generate mocks from traits ([docs.rs](https://docs.rs/mockall)). Hand-written fakes are usually clearer for "record what was sent" assertions, and they also give you a dry-run mode. GitButler also enforces hermetic tests: it points the app-data dir at a guard path in CI and fails the job if any test wrote there. ([push.yaml](https://github.com/gitbutlerapp/gitbutler/blob/master/.github/workflows/push.yaml), "Assert test hermeticity" step)

**Tauri commands (Rust side).** `tauri::test` (crate feature `test`) provides `mock_builder`, `mock_context`, `mock_app`, `noop_assets`, `get_ipc_response` and `assert_ipc_response`. These run commands through a `MockRuntime` with no real window. ([docs.rs tauri::test, 2.12.5](https://docs.rs/tauri/latest/tauri/test/index.html)) The more robust practice is still to keep command bodies thin and call plain functions that are tested directly.

**Frontend (JS side).** `@tauri-apps/api/mocks` provides `mockIPC`, `mockWindows` and `clearMocks`. Since 2.7.0 it can also mock events (`shouldMockEvents: true`). The docs use Vitest with jsdom and remind you to call `clearMocks()` after each test. ([Tauri mocking docs](https://v2.tauri.app/develop/tests/mocking/))

**Webview end-to-end and macOS.**
- Driven directly, `tauri-driver` supports only Windows and Linux: "macOS has no WKWebView driver tool available". ([Tauri WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/), updated 2026-06-29)
- The Tauri docs now recommend **WebdriverIO with `@wdio/tauri-service`**. It supports Windows, Linux **and macOS**. By default it uses an **embedded WebDriver server** that runs inside the app through the `tauri-plugin-wdio-webdriver` Rust plugin. The docs recommend registering that plugin only in debug builds and granting the `wdio-webdriver:default` capability. An optional `tauri-plugin-wdio` adds `browser.tauri.execute()`, IPC mocking and log capture. A "browser mode" runs the frontend in Chrome against Vite with `invoke()` mocked. ([Tauri WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/), [webdriver.io platform support](https://webdriver.io/docs/desktop-testing/tauri/platform-support))
- CrabNebula's fork of `tauri-driver` also works on macOS but needs a **paid API key** there. (Tauri WebDriver docs)
- Caveat: the embedded plugin is a community fork of Choochmeque's `tauri-plugin-webdriver`, and docs.rs failed to build some versions (for example 1.3.0). Treat it as young. ([docs.rs](https://docs.rs/crate/tauri-plugin-wdio-webdriver/1.3.0))

**What only a human with a game can check.** Input injection into a real game (anti-cheat, raw-input games ignoring `SendInput`/`CGEvent`), capture of exclusive-fullscreen or HDR surfaces, overlay z-order, and macOS permission prompts (Accessibility, Screen Recording) cannot be automated on CI runners. Real projects handle this with a checked-in manual test script plus a PR/release checklist. For example, GitButler and Zed each run a separate smoke or "bundle" workflow, and GitButler notes tests that "fail on macOS CI for unclear reasons. Keep running them locally." ([push.yaml](https://github.com/gitbutlerapp/gitbutler/blob/master/.github/workflows/push.yaml)) No tool replaces this. The practice is a `docs/manual-test.md` matrix (OS × input device × reference game) that the release issue template links to.

### 1.3 Lint and format

- **rustfmt.** `cargo fmt --all -- --check` in CI (Tauri [fmt.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/fmt.yml), GitButler `cargo fmt --check --all`). Tauri also checks TOML formatting with `taplo fmt --check`.
- **Workspace `[lints]`.** `[workspace.lints.<tool>]` in the root manifest, and `[lints] workspace = true` in each member (MSRV 1.74). `priority` is a signed integer, and lower numbers are overridden by higher ones. Set a group at `priority = -1` so individual lints can override it. Lints apply only to the current package, not to dependencies. ([Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Cargo manifest `[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html))
- **Clippy groups.** correctness is deny by default. suspicious, complexity, perf and style warn by default. pedantic, restriction, nursery and cargo are allow by default. The docs say to enable **pedantic** whole only if you expect "generous `#[allow]` use", or to cherry-pick from it. Never enable **restriction** whole ("It is **not** recommended to enable the whole group"), and Clippy warns if you do. **nursery**: cherry-pick only. ([Clippy lint groups](https://doc.rust-lang.org/stable/clippy/lints.html))
- **What real repos do:**
  - Tauri: clippy `--all-targets --all-features -- -D warnings` on a matrix that includes Windows and macOS, plus a few cherry-picked lints (`redundant_clone`, `assigning_clones`, `or_fun_call`) in `[workspace.lints.clippy]`. ([lint-rust.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/lint-rust.yml), [Cargo.toml](https://github.com/tauri-apps/tauri/blob/dev/Cargo.toml))
  - GitButler: `all = { level = "deny", priority = -1 }` with some lints relaxed to warn, `redundant_clone = "deny"` from nursery, and clippy run three times (default, `--no-default-features`, `--all-features`). The manifest comment says "You can try `pedantic = deny` to see those extra lints". Pedantic is not on. ([Cargo.toml](https://github.com/gitbutlerapp/gitbutler/blob/master/Cargo.toml))
  - Zed: denies `dbg_macro`, `todo`, `redundant_clone` and `disallowed_methods` (configured in `clippy.toml`), and sets `style = { level = "allow", priority = -1 }` because style nits "slow down shipping". Clippy runs separately on Windows, Linux and macOS. ([Cargo.toml](https://github.com/zed-industries/zed/blob/main/Cargo.toml), [clippy.toml](https://github.com/zed-industries/zed/blob/main/clippy.toml))
- **Why clippy needs the OS matrix.** Code under `#[cfg(target_os = "windows")]` is only compiled, and so only linted, on Windows. All three repos run clippy per OS. For Panowww that matters, because the input and capture adapters are cfg-gated.
- **Frontend (if Tauri).** GitButler uses oxlint/oxfmt (`.oxlintrc.json`, `.oxfmtrc.jsonc`) alongside ESLint/Prettier configs and `pnpm check`. Tauri runs `lint-js.yml` and `pnpm audit`. Biome 2.x is the single-binary alternative (lint and format in one). ([GitButler repo root](https://github.com/gitbutlerapp/gitbutler), [Tauri workflows](https://github.com/tauri-apps/tauri/tree/dev/.github/workflows))

### 1.4 Static analysis and supply chain

- **cargo-deny** runs four checks. **advisories** (RustSec: vulnerable, unmaintained, yanked), **licenses** (allow-list of SPDX licences), **bans** (forbidden crates and duplicate versions) and **sources** (only allowed registries and git sources). ([cargo-deny checks](https://embarkstudios.github.io/cargo-deny/checks/index.html)) GitButler runs it with `cargo-deny-action` v2.1.1, pinned by SHA. For an MIT app that ships binaries, the licences check is what stops a GPL dependency from slipping in.
- **cargo-audit** reads the same RustSec database. Tauri runs `rustsec/audit-check@v2` nightly (cron), with ignores in `.cargo/audit.toml`. ([audit.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/audit.yml), [audit.toml](https://github.com/tauri-apps/tauri/blob/dev/.cargo/audit.toml)) If you already run cargo-deny's advisories check, cargo-audit is mostly redundant. The one thing it adds is a scheduled run that opens issues.
- **cargo-vet.** Tauri runs `cargo vet suggest` nightly. ([supply-chain.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/supply-chain.yml)) It is heavy for a small project.
- **Unused dependencies:**
  - cargo-udeps compiles the project and "needs Rust nightly to actually run". ([README](https://github.com/est31/cargo-udeps)) Tauri runs it on pushes to `dev`.
  - cargo-machete is regex-based, "fast (yet imprecise)", and has a GitHub Action. ([README](https://github.com/bnjbvr/cargo-machete)) GitButler runs it.
  - cargo-shear parses source with rust-analyzer's parser, can `--fix`, catches misplaced dev-deps and unlinked files, and runs on one platform. ([README](https://github.com/Boshen/cargo-shear)) Zed runs `cargo shear --locked --deny-warnings --check-test-targets`, and also `cargo update --locked --workspace` to verify the lockfile.
- **MSRV policy.** Cargo docs: "The simplest policy to support is to always use the latest Rust version". N-2 is a common alternative. Verify with CI or `cargo-hack`. Resolver v3 (edition 2024) makes the resolver take `rust-version` into account. ([Cargo rust-version chapter](https://doc.rust-lang.org/cargo/reference/rust-version.html)) What real apps do:
  - Tauri: `rust-version = "1.90"`, tested on 1.90.
  - GitButler: `rust-version = "1.95"`, with `rust-toolchain.toml` pinned to `1.98`.
  - Zed: `rust-toolchain.toml` pinned to `1.98.1`.

  Desktop apps (as opposed to libraries) mostly **pin the toolchain** with `rust-toolchain.toml` and bump it on purpose.
- **`unsafe` policy:**
  - In edition 2024, `unsafe_op_in_unsafe_fn` warns by default. ([Edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html))
  - `[lints.rust] unsafe_code = "forbid"` is the documented example. ([Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html))
  - `clippy::undocumented_unsafe_blocks` (restriction group, so cherry-pick it) requires a `// SAFETY:` comment on each block. ([Clippy lint list](https://clippy.rust-lang.org/master/index.html))
  - Panowww's Windows `SendInput` and macOS `CGEvent`/ScreenCaptureKit calls go through FFI, so `forbid` can only apply to the platform-independent crates.
- **Miri** detects UB (out-of-bounds access, use-after-free, invalid uninitialized data, data races, aliasing) but has "no access to most platform-specific APIs or FFI". It stops with "unsupported operation: can't call foreign function", and Windows is "supported less well". ([Miri README](https://github.com/rust-lang/miri)) Zed runs Miri only on one pure-Rust crate (`cargo +nightly-2026-09-09 miri test -p scheduler`). ([run_tests.yml](https://github.com/zed-industries/zed/blob/main/.github/workflows/run_tests.yml)) **Miri cannot test Panowww's OS adapters.** It is only useful if Panowww writes its own non-FFI `unsafe`.

### 1.5 CI

- **Toolchain, caching and tool installs:**
  - `dtolnay/rust-toolchain@stable` (or `@master` with `toolchain:`), `Swatinem/rust-cache@v2` keyed per target with `save-if: github.ref == 'refs/heads/<default>'`, and `CARGO_PROFILE_DEV_DEBUG: 0` to shrink the cache. `concurrency: cancel-in-progress` and `permissions: contents: read`. (Tauri [lint-rust.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/lint-rust.yml) / [test-core.yml](https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/test-core.yml))
  - Tool installs via `taiki-e/install-action` (prebuilt binaries for nextest, cargo-llvm-cov, cargo-shear and so on). ([cargo-llvm-cov README](https://github.com/taiki-e/cargo-llvm-cov))
- **Matrix.** `fail-fast: false`, with `windows-latest` and `macos-latest` (arm64). Add `macos-*-intel` only if you ship an x86_64 macOS build and want to test it. GitButler tests Rust on `ubuntu-latest` and `macos-15` only. Zed tests on all three.
- **Hardening.**
  - GitButler pins every action **by commit SHA** with a version comment, sets `persist-credentials: false`, and runs `zizmor` (a workflow security linter) and `osv-scanner`. ([workflows](https://github.com/gitbutlerapp/gitbutler/tree/master/.github/workflows))
  - Zed adds `step-security/harden-runner` and `actions/dependency-review-action`.
- **Coverage.** `cargo llvm-cov nextest`, merged with a doctest run that needs nightly, then `--lcov` output for Codecov. `--fail-under-lines` exists. It works on Windows MSVC and macOS. ([nextest coverage docs](https://nexte.st/docs/integrations/test-coverage/), [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)) None of the three repos above gates on coverage.
- **Pre-commit hooks.** `prek` is a Rust drop-in for pre-commit. It reads `.pre-commit-config.yaml` or a native `prek.toml`, and CPython, Ruff, Godot and Home Assistant use it. ([prek README](https://github.com/j178/prek)) `lefthook` is the other common choice. Keep hooks fast (fmt, typos, clippy on staged crates) and leave the full test run to CI.
- **Release automation:**
  - **tauri-action v1**: builds bundles per OS, creates or updates a (draft) GitHub Release, uploads assets, and generates the updater's `latest.json` and `.sig` files. The official example matrix uses `macos-latest` with `--target aarch64-apple-darwin` and `--target x86_64-apple-darwin`, plus `windows-latest`. Without an Apple certificate, the docs recommend ad-hoc signing so Apple Silicon builds aren't flagged as "damaged". ([tauri-action README](https://github.com/tauri-apps/tauri-action), [Tauri GitHub pipeline docs](https://v2.tauri.app/distribute/pipelines/github/))
  - **dist (cargo-dist)**: shell, PowerShell, npm, Homebrew and MSI installers, Windows signing and attestations. Its docs don't mention `.app`/`.dmg` bundles, macOS signing or Tauri. ([dist book](https://axodotdev.github.io/cargo-dist/book/)) **It is not a fit for a Tauri GUI app.** It could fit a pure-Rust app shipped as a plain binary.
  - **release-please**: has a `rust` release type (workspaces need manifest mode and the `cargo-workspace` plugin) and assumes Conventional Commits. ([README](https://github.com/googleapis/release-please))
  - **release-plz**: built around publishing to a cargo registry (git-cliff changelog, cargo-semver-checks). ([docs](https://release-plz.dev/docs)) It is less natural for an app nobody `cargo install`s.
  - **Tauri itself** uses `covector` (change files). That is overkill for a single app.

---

## 2. Recommended toolset for Panowww

### Adopt from day one

1. **Architecture for testability.** Keep a pure core crate (Game schema, capture-plan maths, Session state machine) with `unsafe_code = "forbid"`. Put OS adapters (input, capture, overlay) behind narrow traits in their own crates. The Tauri layer is thin, and its commands call the core. Ship hand-written fakes (`RecordingInput`, `FakeGrabber` returning generated images, a fake `Clock`) so a whole Session can run in tests without a game.
2. **Tests.**
   - `cargo nextest run` plus `cargo test --doc`, with a `.config/nextest.toml` `ci` profile (`fail-fast = false`, JUnit).
   - **insta** for Game file serialization and generated capture plans.
   - **proptest** for step/row/overlap maths and Game round-trips. Commit `proptest-regressions/`.
3. **Lint and format.**
   - `rust-toolchain.toml` pinned to a recent stable (for example `1.99`), with `rustfmt` and `clippy` components. Set `edition = "2024"` and `resolver = "3"`.
   - Root `[workspace.lints]`, with `[lints] workspace = true` in every crate:
     ```toml
     [workspace.lints.rust]
     unsafe_code = "deny"            # override per OS-adapter crate; forbid in core
     unsafe_op_in_unsafe_fn = "deny"
     [workspace.lints.clippy]
     all = { level = "deny", priority = -1 }   # CI-enforced; GitButler-style
     pedantic = { level = "warn", priority = -1 }
     # pedantic opt-outs, decided as they come up, e.g.:
     module_name_repetitions = "allow"
     # restriction cherry-picks
     undocumented_unsafe_blocks = "deny"
     dbg_macro = "deny"
     todo = "warn"
     ```
     Pedantic at warn level on a greenfield codebase is cheap, which is why the Clippy docs allow it.
   - CI runs `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` **on both macOS and Windows**.
   - Add `typos` for spelling, since it's cheap.
4. **Supply chain.** **cargo-deny**, with a `deny.toml` that allows MIT-compatible licences, denies unknown registries and git sources, and checks advisories. Run it on every PR and on a weekly cron. Skip cargo-audit, because cargo-deny already checks advisories.
5. **CI (GitHub Actions).**
   - One `ci.yml` with `fail-fast: false` over `macos-latest` and `windows-latest`. Steps: fmt (one OS), clippy and nextest (both OSes), cargo-deny (Linux or one OS).
   - `Swatinem/rust-cache@v2` with `save-if` on `main`, `taiki-e/install-action` for nextest and cargo-deny, `permissions: contents: read`, and `concurrency` with cancel-in-progress.
   - Pin actions by SHA (Dependabot or Renovate keeps the pins current).
6. **Manual non-regression.**
   - A checked-in `docs/manual-test.md`: OS × input device (keyboard/mouse/gamepad) × 1–2 reference games. Cover permissions prompts, capture-zone overlay, a Test run, a full Session, and checking Overlap in a stitcher.
   - A release-checklist issue template that links to it.
   - A `--dry-run`/fake-backend mode, so contributors without the game can still run the full flow.
7. **Frontend (if Tauri).** Use one tool for lint and format, either Biome or oxlint+oxfmt. Add Vitest with `@tauri-apps/api/mocks` (`mockIPC`) for UI logic.

### Later

- **Coverage** with `cargo llvm-cov nextest`, report-only (Codecov or a job summary), once the core exists. Gate only on the core crate, if at all.
- **Unused dependencies**: `cargo-shear`, on one platform, once there are several crates.
- **Pre-commit hooks** with `prek` (fmt + typos + clippy), once there are contributors.
- **Release automation.** If Tauri: **tauri-action v1**, triggered by a version tag, with release notes from **release-please** (`rust` type with the `extra-files` mechanism for `tauri.conf.json`/`package.json`, to be verified). Add signing and notarization secrets once an Apple Developer ID and a Windows signing route exist. If pure Rust: dist for the binaries, plus a custom `.app` step.
- **Webview E2E** with WebdriverIO and `@wdio/tauri-service` (embedded driver, so it works on macOS). Add it only once the UI has flows worth protecting, and register the plugin in debug builds only.
- **Workflow hardening**: `zizmor`, and the `dependency-review-action`.
- **MSRV verification** with `cargo-hack`, only if Panowww ever publishes library crates. For an app, the pinned toolchain *is* the policy.
- **Miri**, only for a crate that contains hand-written non-FFI `unsafe` (for example a pixel-buffer conversion). It cannot exercise the OS adapters.
- **cargo-vet / cargo-udeps**: skip unless the project grows a lot.

---

## 3. Open questions

1. **Tauri or pure Rust?** This decides the frontend tooling, whether WebDriver E2E is relevant, and tauri-action vs dist. (Ticket 001-app-shell.)
2. **Do we ship x86_64 macOS?** If yes, add a `macos-*-intel` (or cross-compiled `x86_64-apple-darwin`) build to CI and release.
3. **Code signing.** An Apple Developer ID (notarization) and a Windows signing route (for example Azure Trusted Signing) are needed before releases stop triggering Gatekeeper/SmartScreen warnings. Who pays and owns them?
4. **How strict should pedantic be?** Warn (recommended above) or deny with a curated allow-list? Decide after the first few hundred lines.
5. **Gamepad virtual device on CI.** ViGEm on Windows needs a driver install, which hosted runners may not allow. Will gamepad adapters only ever be tested by hand?
6. **Reference games for the manual checklist.** Which free or demo titles (one per input device) do we standardise on, so that results can be compared across testers?
7. **release-please `extra-files` for `tauri.conf.json`.** Not confirmed from the README. Check `docs/customizing.md` before relying on it.
8. **Maturity of `tauri-plugin-wdio-webdriver`.** It is a young community fork. Re-check stability before betting macOS E2E on it.

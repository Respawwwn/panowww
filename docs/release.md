# Releasing Panowww

Written for the maintainer, who is not a Rust developer. Nothing here needs Rust knowledge: the CI builds, you tag and publish.

## How versions work

- Panowww follows [SemVer](https://semver.org/): `MAJOR.MINOR.PATCH`.
- The version lives in **one place**: the `version` field of the workspace `Cargo.toml` at the repo root. `tauri.conf.json` reads it from there, so never edit the version anywhere else.
- Git tags are `v` + version: `v0.1.0`, `v1.0.0-beta.1`, `v1.0.0`.
- Roadmap: the repo starts at `0.0.1` and iterates freely without releases. The first CI-built release is `v0.1.0`. Everything `0.x` is a pre-release. The repo goes public with `v1.0.0-beta.1`. `v1.0.0` is the first stable release.

## What the CI does on a tag

Pushing a tag `v*` runs the release workflow (`.github/workflows/release.yml`):

1. Builds on `macos-latest` (Apple Silicon only) and `windows-latest` (x64 only).
2. Signs the macOS app with our self-signed certificate (see below), after importing it into a temporary keychain and trusting it for code signing.
3. Bundles `Panowww.dmg` and `Panowww-setup.exe`.
4. Creates a **draft** GitHub Release with GitHub's auto-generated notes and attaches both files, marked as a pre-release for any `0.x` or `-beta` tag.
5. Prints the app's designated requirement in the macOS job log: it must name our certificate (`certificate leaf = H"…"`), never a `cdhash`.

It never publishes by itself. A failed build leaves no release.

While the repo is private, GitHub Actions minutes are billed and macOS counts ×10 (2 000 free minutes ≈ 200 macOS minutes ≈ 15 builds a month). Tag only what you want to test.

## Making a release

1. Make sure `main` is green (the CI workflow passed on the last commit).
2. Bump the version in the root `Cargo.toml`, then run `cargo update -w --offline` so `Cargo.lock` follows (it touches only Panowww's own crates). The crates depend on each other by path only, so nothing else carries the version.
3. Commit on a branch (`chore/release-X.Y.Z`): `git commit -am "Release vX.Y.Z"`, then merge it into `main`.
4. Tag and push:
   ```sh
   git tag vX.Y.Z
   git push origin main vX.Y.Z
   ```
5. Wait for the *Release* workflow in the **Actions** tab (10–20 minutes).
6. Open **Releases**: a draft `vX.Y.Z` exists with the two files attached.
7. Read the generated notes, add two or three human lines at the top (what changed for users), remove noise.
8. Check that **Set as a pre-release** is ticked for any `0.x` or `-beta` version (the workflow ticks it).
9. **Publish release**.
10. Install the published DMG and setup.exe on a real machine each and run the manual test checklist (`docs/manual-test.md`) before announcing.

If something is wrong after publishing, don't edit the tag: fix, bump the patch version and release again.

## The macOS signing certificate

Panowww is not notarized (no Apple Developer account in v1), but it **is** signed with a self-signed certificate we own. Without that, macOS would tie the Accessibility and Screen Recording permissions to each exact build, and users would have to re-grant them after every update. With a stable certificate the permissions survive updates. Gatekeeper still shows "Apple could not verify" on first launch; that is expected and documented in the README.

Rules:

- The bundle identifier `com.respawwwn.panowww` must **never** change: macOS remembers permissions by identifier + certificate.
- The certificate is valid 10 years. When it is replaced (expiry or loss), every user re-grants the two permissions once. Announce it in the release notes.

Where things are:

- 1Password item **"Panowww macOS signing certificate"**: the `panowww-signing.p12`, its password, the certificate name (CN), the expiry date, the openssl command that created it, and the original `.key` / `.crt` files.
- GitHub repository secrets (Settings → Secrets and variables → Actions):
  - `MACOS_SIGNING_P12_BASE64`: the `.p12` file, base64-encoded (`base64 -i panowww-signing.p12 | pbcopy`)
  - `MACOS_SIGNING_P12_PASSWORD`: its password
  - `APPLE_SIGNING_IDENTITY`: the certificate name (CN), exactly as in 1Password

The CI imports the certificate into a temporary keychain, trusts it for code signing (`codesign` refuses an untrusted self-signed identity with "no identity found"), and passes `APPLE_SIGNING_IDENTITY` to the Tauri build. (Tauri's own `APPLE_CERTIFICATE` variables only accept Apple-issued certificates, which is why we don't use them.)

### Rotating the certificate

Only when it expires (10 years after 2026-11-08), leaks or is lost. Every user then grants the two permissions again: say so in the release notes.

1. Write `panowww-signing.cnf`:
   ```ini
   [req]
   distinguished_name = dn
   x509_extensions = ext
   prompt = no
   [dn]
   CN = Respawwwn Panowww Code Signing
   O = Respawwwn
   [ext]
   basicConstraints = critical, CA:false
   keyUsage = critical, digitalSignature
   extendedKeyUsage = critical, codeSigning
   ```
2. Make the certificate and the `.p12` (`-legacy`: macOS's `security import` cannot read OpenSSL 3's default `.p12` cipher):
   ```sh
   openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -config panowww-signing.cnf \
     -keyout panowww-signing.key -out panowww-signing.crt
   openssl pkcs12 -export -legacy -inkey panowww-signing.key -in panowww-signing.crt \
     -name "Respawwwn Panowww Code Signing" -out panowww-signing.p12 -passout pass:<new password>
   ```
3. Replace the files, the password, the expiry date and the SHA-1 (`openssl x509 -in panowww-signing.crt -noout -fingerprint -sha1`) in the 1Password item.
4. Set the three secrets:
   ```sh
   base64 < panowww-signing.p12 | tr -d '\n' | gh secret set MACOS_SIGNING_P12_BASE64
   gh secret set MACOS_SIGNING_P12_PASSWORD   # paste the password
   gh secret set APPLE_SIGNING_IDENTITY        # the CN, if it changed
   ```
5. Delete the local files.

## Going public

Before flipping `Respawwwn/panowww` to public (planned with `v1.0.0-beta.1`):

- [ ] Search the whole history for secrets and personal paths: `git log -p | grep -iE "password|secret|token|/Users/"`
- [ ] `.wayfinder/` is git-ignored and absent from history
- [ ] `LICENSE` (MIT, Kevin Wenger (Respawwwn)), `NOTICE` (Respawwwn marks), `README.md`, `CONTRIBUTING.md` are in place with real screenshots
- [ ] Issue templates exist: Bug report, Game doesn't work, Feature request
- [ ] Labels `bug`, `game` and `enhancement` exist (the issue templates apply them; GitHub ignores a missing one)
- [ ] Repository settings: Issues and Discussions enabled, default branch `main` protected
- [ ] Flip to public, then tag and publish `v1.0.0-beta.1` as a pre-release
- [ ] The in-app update check starts working only now (it reads the public Releases API)

## Troubleshooting

- **The Release workflow failed on macOS at the signing step**: a secret is missing or the `.p12` password is wrong. Re-check the three secrets against 1Password.
- **No draft release appeared**: the tag doesn't start with `v`, or the workflow didn't run (check Actions).
- **Users say a permission is ticked but Panowww does nothing**: the certificate changed between their two installs. Tell them to remove Panowww from the permission list and add it again (also in the README's Troubleshooting).

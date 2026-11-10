# Releasing Cloister

Releases are built by GitHub Actions. Nobody needs a Mac or a Linux machine to ship a version.

## One-time setup

1. **Pages:** in the repository go to Settings › Pages › Build and deployment and set Source to
   **GitHub Actions**. The landing page in `site/` then deploys on every push to `main` that touches it, to
   <https://muntahi-safwan.github.io/Cloister/>.
2. **Update signing key:** installed copies of Cloister only accept updates signed with the private key that
   matches the public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`). In the repository go to
   Settings › Secrets and variables › Actions and add:
   - `TAURI_SIGNING_PRIVATE_KEY`: the full contents of the private key file;
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password (leave the secret out, or empty, if the key has none).

   Keep a backup of the private key somewhere safe and never commit it. If it is lost, existing installs can no
   longer update themselves and users must download the next version by hand. To make a new pair:
   `pnpm tauri signer generate -w ~/.tauri/cloister-updater.key`, then put the new public key in
   `tauri.conf.json`.

3. **Actions permissions:** Settings › Actions › General › Workflow permissions should allow
   **Read and write** so the release workflow can create releases.

## Shipping a version

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Move the `[Unreleased]` notes in `CHANGELOG.md` under a new `## [x.y.z] - date` heading. The release
   notes are taken from that section.
3. Commit, then tag and push:

   ```bash
   git tag v0.3.0
   git push origin main v0.3.0
   ```

The **Release** workflow then:

1. checks that the tag matches the version in `tauri.conf.json`;
2. creates a draft release;
3. builds on Windows, macOS (one universal app for Apple Silicon and Intel) and Linux in parallel, and
   uploads every installer to the draft, together with signed update packages and `latest.json`;
4. publishes the release once all three succeed.

The landing page reads the latest published release from the GitHub API, so its download buttons update by
themselves, and installed copies find the new version through `latest.json` within a few hours. If a build fails, the draft stays unpublished: fix the problem, delete the draft and the tag, and
tag again.

## Signing (later)

The installers are not signed yet, so Windows SmartScreen and macOS Gatekeeper warn on first launch. The
landing page and README explain the workaround. To remove the warnings:

- **Windows:** buy a code-signing certificate (or use Azure Trusted Signing) and add it to `bundle.windows`
  in `tauri.conf.json`. See <https://v2.tauri.app/distribute/sign/windows/>.
- **macOS:** join the Apple Developer Program, then add the `APPLE_CERTIFICATE`, `APPLE_ID`, `APPLE_PASSWORD`
  and `APPLE_TEAM_ID` secrets; `tauri-action` signs and notarises automatically.
  See <https://v2.tauri.app/distribute/sign/macos/>.

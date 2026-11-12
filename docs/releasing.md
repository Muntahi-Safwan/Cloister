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
2. Move the `[Unreleased]` notes in `CHANGELOG.md` under a new `## [x.y.z] - date` heading, and add a
   `[x.y.z]: https://github.com/Muntahi-Safwan/Cloister/releases/tag/vx.y.z` line at the bottom. The release
   notes are taken from that section, and the website's changelog page is built from the same file.
3. Commit, then tag and push:

   ```bash
   pnpm site:changelog   # rebuild site/changelog.html from CHANGELOG.md
   git tag v1.2.3        # the version you just set
   git push origin main v1.2.3
   ```

The **Release** workflow then:

1. checks that the tag matches the version in `tauri.conf.json`;
2. creates a draft release;
3. builds on Windows, macOS (one universal app for Apple Silicon and Intel) and Linux in parallel, and
   uploads every installer to the draft, together with signed update packages and `latest.json`;
4. publishes the release once all three succeed.

The landing page reads the latest published release from the GitHub API, so its download buttons update by
themselves, and installed copies find the new version through `latest.json` within a few hours.

If a build fails, the draft stays unpublished: fix the problem, delete the draft and the tag, and tag again.

If only one build failed for a reason outside the code, such as Ubuntu's package mirrors being unreachable
("Temporary failure resolving 'azure.archive.ubuntu.com'"), you don't need a new tag. Open the run on the
Actions page and click **Re-run failed jobs**; the builds that already finished are kept. Every build job
stops after an hour at most, and the Linux package step after 15 minutes, so a stuck mirror fails the run
instead of leaving it hanging.

## Windows code signing (SignPath Foundation)

Windows installers are signed for free through the [SignPath Foundation](https://signpath.org) programme for
open source projects. The policy it requires is published at `site/code-signing.html`.

**Apply.** Use **Apply** on <https://signpath.org> and fill in with the repository URL, the release
workflow and the policy page. Turn on two-factor authentication for GitHub first; SignPath requires it for
everyone who can change the code or approve signing.

**Once approved, set up SignPath:**

1. In SignPath, create the project (for example `cloister`) and link it to the GitHub repository as a
   trusted build system.
2. Set its artifact configuration to the contents of `.signpath/artifact-configuration.xml`.
3. Use the `release-signing` signing policy, with yourself as approver. Add the metadata restriction
   product name `Cloister`.
4. Create a CI user and copy its API token.

**Then connect GitHub** (Settings › Secrets and variables › Actions):

| Kind     | Name                           | Value                                     |
| -------- | ------------------------------ | ----------------------------------------- |
| Secret   | `SIGNPATH_API_TOKEN`           | The CI user's API token                   |
| Variable | `SIGNPATH_ORGANIZATION_ID`     | Your organisation ID (in SignPath's URLs) |
| Variable | `SIGNPATH_PROJECT_SLUG`        | The project slug, for example `cloister`  |
| Variable | `SIGNPATH_SIGNING_POLICY_SLUG` | Only if it is not `release-signing`       |

From the next tag on, the release workflow uploads the two Windows installers to SignPath and waits. Approve
the request in SignPath (you get an email); the workflow then puts the signed files into the draft release,
signs them again for the updater (signing changes the files, so the old `.sig` would no longer match),
updates `latest.json` and publishes. If nobody approves within 80 minutes the run fails and nothing is
published; re-run the failed job to try again. Finally, change the status note at the top of
`site/code-signing.html` to name the first signed version.

Without the `SIGNPATH_ORGANIZATION_ID` variable the signing job is skipped and releases are published unsigned,
as before.

## winget

Cloister is packaged for the Windows Package Manager as `MuntahiSafwan.Cloister`.

**First version, by hand.** The manifests for 0.4.2 are in `packaging/winget/`. Submit them once:

1. Install [wingetcreate](https://github.com/microsoft/winget-create) on Windows: `winget install wingetcreate`.
2. Check them locally: `winget validate --manifest packaging\winget\manifests\m\MuntahiSafwan\Cloister\0.4.2`,
   then try `winget install --manifest packaging\winget\manifests\m\MuntahiSafwan\Cloister\0.4.2`.
3. Submit: `wingetcreate submit --token <a GitHub token with public_repo> packaging\winget\manifests\m\MuntahiSafwan\Cloister\0.4.2`.
   This opens a pull request on `microsoft/winget-pkgs`. Automated checks run, then a moderator merges it,
   usually within a few days.

**Later versions, automatically.** After the first one is merged, fork `microsoft/winget-pkgs`, save a classic
token with the `public_repo` scope as the `WINGET_TOKEN` secret, and set the variable `WINGET_PUBLISH` to
`true`. The `winget` workflow then opens the pull request for every release you publish.

## macOS signing (later)

The macOS app is not notarised yet, so Gatekeeper warns on first launch. To remove the warning:

- **macOS:** join the Apple Developer Program, then add the `APPLE_CERTIFICATE`, `APPLE_ID`, `APPLE_PASSWORD`
  and `APPLE_TEAM_ID` secrets; `tauri-action` signs and notarises automatically.
  See <https://v2.tauri.app/distribute/sign/macos/>.

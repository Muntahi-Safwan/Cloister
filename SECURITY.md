# Security and privacy

## What Cloister does with your data

- Cloister has no account, no server and no telemetry.
- Each service stores its cookies and site data in its own browser profile under
  `%LOCALAPPDATA%\app.cloister.desktop\profiles\`.
- Cloister's own settings (`services.json`, `settings.json`) contain service names, addresses and preferences
  only. They live in `%APPDATA%\app.cloister.desktop\`.
- Notification text passes through Cloister only to be shown as a Windows toast, or kept in memory for the
  focus summary and the recent notifications panel (last 50). It is never written to disk.
- "Sign out and clear data" deletes a service's whole profile folder: cookies, local storage and cache.

## Updates

Cloister checks `latest.json` on this repository's latest GitHub release. Every update package is signed with a
private key held only by the maintainer, and the app refuses packages whose signature does not match the public
key built into it. Nothing installs without the user clicking "Install".

## App lock

- The lock uses a Cloister PIN that you choose. Only an Argon2id hash of it (with a random salt) is stored, in
  `lock.json` in Cloister's settings folder. It is never sent to the interface or anywhere else. After five
  wrong guesses in a row, each further guess waits 30 seconds, doubling up to five minutes.
- On Windows, Windows Hello can also unlock, through `UserConsentVerifier`. Cloister never sees your Windows
  PIN or biometrics; it only learns whether verification succeeded. Hello is never the only way in, so a
  broken Hello setup cannot lock you out.
- Changing or removing the PIN needs the current PIN, and cannot be done from the lock screen.
- While locked, every service page is hidden (not just covered), shortcuts are ignored and toasts never show
  message text.
- The lock protects against someone glancing at or using your unlocked PC. It does not encrypt data on disk:
  anyone with access to your user account can read browser profiles. Use full-disk encryption (BitLocker,
  FileVault or LUKS) for that.

## Camera and microphone

Cloister does not grant camera or microphone access itself. When a site asks, WebView2 shows its own
permission prompt and the choice is yours.

## Network requests Cloister itself makes

- **Service icons.** Once a service is open, its own page reports its icon and Cloister loads it from that
  site. Catalog entries that have never been opened show icons from DuckDuckGo's favicon service
  (`icons.duckduckgo.com`). The catalog requests every entry at once, so this does not reveal which services
  you use, and custom websites are never sent there.
- **Nothing else.** No analytics, update checks or crash reports.

## How service pages are isolated

Service pages run third-party code, so they are treated as untrusted:

- Each page can call exactly four backend commands, `bridge_notify`, `bridge_icon`, `bridge_shortcut` and
  `bridge_open_link`, through a runtime capability scoped to that webview and its own domains
  (`src-tauri/src/runtime/capabilities.rs`).
- `bridge_open_link` opens only `https`, `http`, `mailto` and `tel` addresses in the default browser, never
  files or other app schemes, and each service can open at most five links in ten seconds. The same limit
  applies to new-window requests the engine passes to Cloister. Logs record only the link's host.
- The backend identifies the caller from the webview label, so a page cannot act as another service.
- Shortcut actions are checked against an allow-list. Notification text is length-limited. Icon URLs must be
  `https` or inline images under 64 KB.
- Cloister's own UI commands are granted only to the main UI webview (`src-tauri/capabilities/main.json`).

The injected bridge (`src-tauri/scripts/bridge.js`) never sends page content to Cloister or anywhere else, and
never clicks, types or sends anything. The find bar (`Ctrl F`) reads the page's text only while you search, and
only inside the page.

With "Hide message previews" on for a service, its notifications show only the service name and "New
message". The sender and text are not kept in the notification history or the focus summary either.

## Reporting a vulnerability

Please do not open a public issue for security problems. Report them privately through GitHub's
"Report a vulnerability" button on the repository's Security tab. We aim to respond within a week.

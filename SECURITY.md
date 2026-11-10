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

- The lock uses Windows Hello through `UserConsentVerifier`. Cloister never sees your PIN or biometrics; it
  only learns whether verification succeeded.
- While locked, every service page is hidden (not just covered), shortcuts are ignored and toasts never show
  message text.
- The lock protects against someone glancing at or using your unlocked PC. It does not encrypt data on disk:
  anyone with access to your Windows account can read browser profiles. Use BitLocker for that.

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

- Each page can call exactly three backend commands, `bridge_notify`, `bridge_icon` and `bridge_shortcut`, through a runtime
  capability scoped to that webview and its own domains (`src-tauri/src/runtime/capabilities.rs`).
- The backend identifies the caller from the webview label, so a page cannot act as another service.
- Shortcut actions are checked against an allow-list. Notification text is length-limited. Icon URLs must be
  `https` or inline images under 64 KB.
- Cloister's own UI commands are granted only to the main UI webview (`src-tauri/capabilities/main.json`).

The injected bridge (`src-tauri/scripts/bridge.js`) never reads page content, and never clicks, types or sends
anything.

## Reporting a vulnerability

Please do not open a public issue for security problems. Report them privately through GitHub's
"Report a vulnerability" button on the repository's Security tab. We aim to respond within a week.

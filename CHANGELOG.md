# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-11-09

### Added

- App lock with Windows Hello (PIN, face or fingerprint): on start, after a chosen time away, or on demand
  with `Ctrl Shift L`, the title bar or the tray. While locked, service pages are hidden and notifications
  hide their text.
- Automatic Veil: when another window takes focus, or after a chosen time away.
- Clicking a notification opens Cloister on the service it came from.
- Recent notifications panel (bell in the title bar): the last 50, including held and silenced ones. Kept in
  memory only and forgotten when Cloister quits; hidden while Veil is on.
- Memory use per service in Settings › Services.
- "Sign out and clear data" for one service (right-click menu or Settings) or for every service at once.

- Installers for every platform from one tag: Windows (`.exe`, `.msi`), macOS (universal `.dmg`) and Linux
  (`.AppImage`, `.deb`, `.rpm`), built and published by the release workflow.
- A landing page with downloads that match the visitor's system (`site/`, deployed to GitHub Pages).
- macOS support: Cmd shortcuts and a separate sign in per service on macOS 14 and later.
- Branded Windows installer.

### Changed

- Linux: service pages now sit beside the sidebar instead of taking half the window.
- CI checks the backend on Windows, macOS and Linux.
- The background sweep runs every 30 seconds instead of every minute so away actions apply promptly.

## [0.2.0] - 2026-11-09

### Added

- Native right-click menu on sidebar services: open, mute (1 hour, until 08:00 tomorrow, until unmuted),
  rename, reload, sleep now, open in browser, copy link, remove.
- Multiple accounts: add the same catalog service again ("Add another"); each copy has its own profile.
- Quiet hours: a daily window when notifications are silenced (VIPs still get through).
- System-wide shortcuts: Ctrl+Alt+Space shows or hides Cloister; Ctrl+Alt+H turns on Veil and hides it.
- Start with Windows, minimised to the tray.
- Per-service zoom with Ctrl+Plus, Ctrl+Minus and Ctrl+0, remembered per service.
- "Asleep" state with a wake button when the open service has been put to sleep.
- Command palette actions for the current service: mute, rename, sleep.

## [0.1.0] - 2026-11-09

### Added

- Theme picker with six themes (Quiet Room, Midnight, Forest, Mono, Paper, Daylight) and a "Match Windows"
  option. Previews are drawn with each theme's real colours; tests check WCAG AA contrast for every theme.
- Catalog entries show service icons before they are added (via DuckDuckGo's favicon service), with the
  site's own icon used once a service is opened.

- Site icons in the sidebar, reported by each page; switch between site icon and initials in Settings.
- Drag to reorder services in the sidebar, with Alt+Up/Down as a keyboard alternative.
- Low-memory mode for background pages using WebView2's memory target level (on by default).
- "Send test" notification button in Settings, and logging of how each notification was routed.
- About section crediting the author.

### Fixed

- The service bridge did not run after formatting changed how `bridge.js` ended, which disabled
  notifications and Veil inside service pages. The bridge now calls itself and is covered by tests.
- Veil blurs chats, messages and other list rows individually, so hovering reveals only the row under the
  pointer (after a short delay). The open WhatsApp chat's header is blurred too.
- Veil now uses constructable stylesheets so a site's Content Security Policy cannot block it, and blurs
  the whole page if a recipe's regions are no longer found.

- Main window with a service rail, custom title bar and command palette.
- Built-in recipes for WhatsApp, Messenger, Discord, Gmail, Outlook, Telegram, Slack, Microsoft Teams,
  Facebook, LinkedIn and Proton Mail.
- Custom websites with initials or site icons, tile colours and per-service behaviour.
- Isolated browser profile per service; lazy loading; optional unloading of idle services.
- Native notifications, unread counts from page titles, taskbar overlay badge and tray tooltip.
- Veil privacy mode with per-recipe blur regions and redacted notifications.
- Focus sessions with held notifications, a VIP list and an end-of-session summary.
- Settings for Veil, focus, tray behaviour and services.

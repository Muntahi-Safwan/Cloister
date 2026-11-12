# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.2] - 2026-11-11

### Added

- **Find in page.** `Ctrl F` (`⌘ F` on macOS) opens a search bar inside the current service. Every match is
  highlighted, `Enter` and `Shift Enter` (or `F3`) move between them, and `Esc` closes it. It also works from
  the sidebar and the command palette. It finds text the page has loaded; chat apps load older messages as
  you scroll up.
- **Hide message previews per service.** Right-click a service and turn on "Hide message previews": its
  notifications then show only the service name and "New message", while other services keep showing the
  sender and text. Those notifications are kept in the history without their text too. Also available when
  adding a custom website.

### Fixed

- Shortcut labels in Settings and the command palette show `⌘` on macOS.
- Settings no longer label app lock as Windows Hello only.

## [0.4.1] - 2026-11-10

### Changed

- **App lock now uses a Cloister PIN.** You choose a PIN (four or more characters) in Settings › Lock, and
  it always unlocks Cloister. Windows Hello is now an optional quicker way in. Before, the lock relied on
  Windows Hello alone, so when Hello failed (for example error `0xD000A002` from a damaged Windows PIN) there
  was no way to unlock. If you had the lock on, Cloister now starts unlocked; set a PIN in Settings › Lock and
  the lock is back on.
- App lock now works on macOS and Linux too.

### Security

- The PIN is stored only as an Argon2id hash in `lock.json`, never in `settings.json`. After five wrong
  guesses in a row, Cloister makes you wait 30 seconds, doubling up to five minutes.

## [0.4.0] - 2026-11-10

### Changed

- **Windows: one shared browser engine.** Every service now runs as a separate WebView2 profile inside the
  same browser process group as Cloister's window, instead of starting its own browser, GPU and network
  processes. Logins stay separate per service. **Upgrading signs you out of every service once**: the old
  per-service browser folders cannot be carried over (each encrypted its cookies with its own key) and are
  removed to free disk space.
- Memory in Settings › Services is now measured per renderer process, and the shared part is shown on its own
  line.

### Added

- **Pause in background** (Windows): a hidden service is suspended so the system can reclaim its memory, and is
  woken every few minutes (configurable) to catch up on messages. Opening it wakes it instantly. Choose it per
  service from the right-click menu under "In the background".
- **Refresh pages that grow too large**: a page hidden for 10 minutes that passes a size limit (800 MB by
  default) is reloaded quietly to undo slow memory leaks.
- "In the background" submenu to change a service's background mode at any time.
- **Automatic updates.** Cloister checks GitHub on start and every six hours (can be turned off in Settings ›
  About) and shows an "Update to x.y.z" button in the title bar. Updates are signed and only install when you
  click; on Windows the installer runs in passive mode and reopens Cloister.

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

The first release.

### Added

- Main window with a service rail, custom title bar and command palette.
- Built-in recipes for WhatsApp, Messenger, Discord, Gmail, Outlook, Telegram, Slack, Microsoft Teams,
  Facebook, LinkedIn and Proton Mail.
- Custom websites with initials or site icons, tile colours and per-service behaviour.
- Isolated browser profile per service; lazy loading; optional unloading of idle services.
- Native notifications, unread counts from page titles, taskbar overlay badge and tray tooltip.
- Veil privacy mode with per-recipe blur regions and redacted notifications.
- Focus sessions with held notifications, a VIP list and an end-of-session summary.
- Settings for Veil, focus, tray behaviour and services.
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

[Unreleased]: https://github.com/Muntahi-Safwan/Cloister/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/Muntahi-Safwan/Cloister/releases/tag/v0.4.2
[0.4.1]: https://github.com/Muntahi-Safwan/Cloister/releases/tag/v0.4.1
[0.4.0]: https://github.com/Muntahi-Safwan/Cloister/releases/tag/v0.4.0

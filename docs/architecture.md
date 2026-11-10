# Architecture

Cloister is a Tauri 2 app. A Svelte UI runs in the main webview; each service runs in its own **child
webview** inside the same window, positioned by the Rust backend over the UI's content area.

```
┌──────────────────────── Main window ────────────────────────┐
│ Title bar (Svelte)                                          │
├──────┬──────────────────────────────────────────────────────┤
│ Rail │  Content area (Svelte measures it)                   │
│      │  ┌────────────────────────────────────────────────┐  │
│      │  │ Service child webview (WebView2)               │  │
│      │  │ own profile · bridge.js injected               │  │
│      │  └────────────────────────────────────────────────┘  │
└──────┴──────────────────────────────────────────────────────┘
```

## Layers

The backend is split so that decisions are testable without a running app.

| Layer         | Depends on Tauri? | Responsibility                                                           |
| ------------- | ----------------- | ------------------------------------------------------------------------ |
| `services/`   | No                | Service config, catalog recipes, persistence, unread and host rules      |
| `features/`   | No                | Focus sessions and notification routing                                  |
| `runtime/`    | Yes               | Creates webviews, applies Veil, updates badges and tray                  |
| `commands/`   | Yes               | IPC entry points for the UI (`services`, `session`) and pages (`bridge`) |
| `snapshot.rs` | Yes               | Builds the single view model the UI renders                              |

The UI mirrors this: `lib/api/backend.ts` is the only file that calls `invoke`, `lib/stores/app.svelte.ts` holds
all state, and components read from the store and call its methods.

## Data flow

1. The UI calls a command, for example `activate_service`.
2. The command updates `AppState` and asks `runtime/` to apply the change (create, show or hide webviews).
3. The backend pushes a fresh `Snapshot` event. The UI replaces its state with it.

There is one view model rather than many fine-grained events, so the UI can never drift out of sync.

## State and locking

`AppState` holds three mutexes: the service registry, the settings store and the volatile session. Code
never holds two of them at once, which rules out deadlocks by construction. Locks recover from poisoning so a
panic in one command cannot take the app down.

## Service webviews

- **Creation** is lazy: a webview is created the first time a service is opened, from an `async` command
  (WebView2 can deadlock if webviews are created from synchronous commands).
- **Placement**: the UI's `ContentArea` reports its rectangle through `set_content_bounds` whenever it resizes.
  The backend moves every service webview there.
- **Dialogs**: native child webviews always draw above the main UI, so while a dialog is open the UI calls
  `set_overlay_open(true)` and the backend hides the active page.
- **Profiles**: each service gets `%LOCALAPPDATA%\app.cloister.desktop\profiles\<service-id>`.
- **Links**: top-level navigation stays in the service so single sign-on redirects work. New windows to the
  service's own domains open as in-app popups (sharing the session); every other link opens in the default
  browser. Non-web schemes such as `mailto:` go to the OS.
- **Unread counts** are read from page title changes (`on_document_title_changed`), so pages do not need to
  call the backend for badges at all.
- **Background memory**: pages that are not on screen get WebView2's low memory target level
  (`platform/webview2.rs`). Scripts keep running, so notifications still arrive.
- **Idle unloading**: a background thread closes "unload when idle" services after the configured number of
  minutes off screen.

## The bridge

`src-tauri/scripts/bridge.js` is injected into every service page before the site's own scripts. It:

1. Replaces `window.Notification` and `ServiceWorkerRegistration.showNotification` and forwards calls to
   `bridge_notify`.
2. Adds or removes the Veil stylesheet (`window.__CLOISTER__.setVeil`), falling back to a whole-page blur
   if the recipe's regions are not found.
3. Reports the site's icon once per page load to `bridge_icon`.
4. Forwards Cloister's shortcuts to `bridge_shortcut`.

The backend fills in the bridge's configuration by replacing the `__CLOISTER_CONFIG__` argument on its last
line. `bridge.test.ts` runs the bridge in a simulated DOM.

Each service webview receives a **runtime capability** scoped to its own label and its own domains that
allows only those three commands (`runtime/capabilities.rs`). The backend identifies the calling service from
the webview label, never from the payload.

## Notification routing

`features/notify.rs` decides, as a pure function, whether a notification is shown, held for the focus summary
or dropped, and whether Veil redacts it. The runtime only executes the decision.

## Privacy runtime

- `runtime/lock.rs` locks and unlocks. Locking sets `session.locked`; `sync_visibility` then hides every
  service page, and the UI draws the lock screen. `platform/hello.rs` asks Windows Hello.
- `runtime/away.rs` records when the main window lost focus. The background sweep (`lifecycle.rs`) calls
  `away::check`, which uses the pure `features/away.rs` to decide whether to lock or veil.
- `features/history.rs` is a bounded, in-memory list fed by `notifications::deliver`.
- `platform/toast.rs` shows toasts that call `activation::activate` when clicked. Debug builds borrow
  PowerShell's app id, because Windows only shows toasts for registered apps.
- `runtime/memory.rs` sums process memory by matching each profile folder on browser command lines.
- `runtime/erase.rs` closes pages and deletes profile folders, retrying while WebView2 releases files.

## Platform notes

- Windows is the primary target. The taskbar badge uses an overlay icon (`icons/badge.png`).
- Tauri's multi-webview support sits behind the `unstable` feature flag.
- On Linux, Tauri packs every webview of a window into one vertical `GtkBox` and ignores `set_bounds`, so
  `platform/gtk_layer.rs` wraps the main UI in a `GtkOverlay` and moves service pages into a positioned
  `GtkFixed` layer above it.
- On macOS, WKWebView cannot use data folders, so each service gets a WebKit data store named by a stable hash
  of its id (`services/profile.rs`, macOS 14+). Erasing data goes through the webview for the same reason.

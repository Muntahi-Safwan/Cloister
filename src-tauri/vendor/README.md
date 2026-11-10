# Vendored crates

## tauri-runtime-wry 2.12.3

An unmodified copy of [`tauri-runtime-wry`](https://crates.io/crates/tauri-runtime-wry) 2.12.3 with one
small patch, marked `CLOISTER PATCH` in `src/lib.rs`, applied through `[patch.crates-io]` in
`../Cargo.toml`.

**What it does.** On Windows, when a webview is created with `WebviewBuilder::with_environment` _and_
`WebviewBuilder::data_directory`, the data directory's folder name becomes the webview's
[WebView2 profile](https://learn.microsoft.com/microsoft-edge/webview2/concepts/multi-profile-support).
Without an environment nothing changes.

**Why.** Tauri does not expose WebView2 profiles yet, although `wry` supports them
(`WebViewBuilderExtWindows::with_profile_name`). Profiles let every service share one browser process
group while keeping its own cookies and storage. Giving each service its own data folder instead starts a
separate browser, GPU and network process per service, which costs far more memory.

**Upgrading Tauri.** Copy the new `tauri-runtime-wry` release here, re-apply the patch (it is one `if let`
block next to `with_environment`), and update the version in this file. Drop the vendored copy once
Tauri exposes `profile_name` on `WebviewBuilder`.

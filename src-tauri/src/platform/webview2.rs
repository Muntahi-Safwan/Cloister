//! WebView2 memory target level.
//!
//! WebView2 lets an app ask for a "low" memory target on inactive views. The
//! engine then trims caches and may page memory out, while scripts keep
//! running (unlike `TrySuspend`, which would stop notifications). Microsoft
//! advises not to mix the two approaches, so Cloister only uses this one.
//! See <https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/icorewebview2_19>.

use tauri::webview::PlatformWebview;
use tauri::{Runtime, Webview};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use windows_core::Interface;

/// Sets the memory target of `webview` to low (`true`) or normal (`false`).
pub fn set_memory_target<R: Runtime>(webview: &Webview<R>, low: bool) {
    let label = webview.label().to_string();
    let scheduled = webview.with_webview(move |platform| {
        if let Err(err) = apply(&platform, low) {
            // Older WebView2 runtimes lack ICoreWebView2_19; nothing else breaks.
            log::debug!("memory target not applied to {label}: {err}");
        }
    });
    if let Err(err) = scheduled {
        log::debug!("could not reach webview for memory target: {err}");
    }
}

fn apply(platform: &PlatformWebview, low: bool) -> windows_core::Result<()> {
    let level = if low {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    // SAFETY: plain COM calls on a live WebView2 controller. `with_webview`
    // runs this closure on the UI thread that owns the controller.
    unsafe {
        let core = platform.controller().CoreWebView2()?;
        core.cast::<ICoreWebView2_19>()?.SetMemoryUsageTargetLevel(level)
    }
}

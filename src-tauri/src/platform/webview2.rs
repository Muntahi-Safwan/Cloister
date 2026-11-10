//! WebView2 background memory controls.
//!
//! Two tools, used on different services because Microsoft advises not to mix
//! them on one view:
//! * a "low" memory target for hidden services that keep notifying: the engine
//!   trims caches while scripts keep running
//!   (<https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/icorewebview2_19>);
//! * `TrySuspend` / `Resume` for services in pause mode: scripts stop and the
//!   system may reuse the renderer's memory until the page is resumed
//!   (<https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/icorewebview2_3>).

use tauri::webview::PlatformWebview;
use tauri::{Runtime, Webview};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, ICoreWebView2_3, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use webview2_com::TrySuspendCompletedHandler;
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

/// Suspends (`true`) or resumes (`false`) a hidden page. Suspending only works
/// while the webview is hidden; showing it resumes it automatically.
pub fn set_suspended<R: Runtime>(webview: &Webview<R>, suspended: bool) {
    let label = webview.label().to_string();
    let scheduled = webview.with_webview(move |platform| {
        if let Err(err) = suspend_or_resume(&platform, suspended, label.clone()) {
            log::debug!("could not {} {label}: {err}", if suspended { "suspend" } else { "resume" });
        }
    });
    if let Err(err) = scheduled {
        log::debug!("could not reach webview to suspend it: {err}");
    }
}

fn suspend_or_resume(platform: &PlatformWebview, suspended: bool, label: String) -> windows_core::Result<()> {
    // SAFETY: plain COM calls on a live controller, on its UI thread (`with_webview`).
    unsafe {
        let core = platform.controller().CoreWebView2()?.cast::<ICoreWebView2_3>()?;
        if suspended {
            let handler = TrySuspendCompletedHandler::create(Box::new(move |result, done| {
                if result.is_err() || !done {
                    // Pages playing media or holding locks refuse; they stay awake.
                    log::debug!("{label} declined to suspend");
                }
                Ok(())
            }));
            core.TrySuspend(&handler)
        } else {
            core.Resume()
        }
    }
}

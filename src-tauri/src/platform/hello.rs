//! Windows Hello (fingerprint, face or PIN) via `UserConsentVerifier`.
//!
//! Desktop apps must use the window-aware interop call so the prompt is
//! owned by Cloister's window and appears in front of it.

use tauri::{Runtime, Window};
use windows::Security::Credentials::UI::{
    UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
use windows_core::HSTRING;
use windows_future::IAsyncOperation;

use super::AuthOutcome;

/// Whether Windows Hello is set up and usable. Blocks briefly; call off the UI thread.
pub fn available() -> bool {
    UserConsentVerifier::CheckAvailabilityAsync()
        .and_then(|operation| operation.join())
        .map(|availability| availability == UserConsentVerifierAvailability::Available)
        .unwrap_or(false)
}

/// Shows the Windows Hello prompt over `window` and waits for the answer.
/// Blocks until the user responds; call from an async command, never the UI thread.
pub fn verify<R: Runtime>(window: &Window<R>, reason: &str) -> AuthOutcome {
    match request(window, reason) {
        Ok(UserConsentVerificationResult::Verified) => AuthOutcome::Verified,
        Ok(_) => AuthOutcome::Rejected,
        Err(err) => {
            log::warn!("Windows Hello failed: {err}");
            AuthOutcome::Unavailable
        }
    }
}

fn request<R: Runtime>(
    window: &Window<R>,
    reason: &str,
) -> windows_core::Result<UserConsentVerificationResult> {
    let hwnd = window.hwnd().map_err(|_| windows_core::Error::from_thread())?;
    let interop = windows_core::factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;
    // SAFETY: `hwnd` is Cloister's live top-level window and `reason` outlives the call.
    let operation: IAsyncOperation<UserConsentVerificationResult> =
        unsafe { interop.RequestVerificationForWindowAsync(hwnd, &HSTRING::from(reason))? };
    operation.join()
}

//! Stable identifiers for each service's browser profile.
//!
//! Windows and Linux keep each profile in its own folder. WKWebView on macOS
//! cannot use folders, so each service instead gets a data store named by a
//! 16-byte identifier derived from its id. The derivation must never change,
//! or every macOS user would be signed out of every service after an update.

/// Name of a service's profile folder.
///
/// On Windows every service is a WebView2 profile inside one shared browser,
/// and the folder name becomes the profile name. The `svc-` prefix keeps it
/// clear of WebView2's own profile names such as "Default".
pub fn folder_name(service_id: &str) -> String {
    if cfg!(target_os = "windows") {
        let safe: String = service_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .take(60)
            .collect();
        format!("svc-{safe}")
    } else {
        service_id.to_string()
    }
}

/// FNV-1a (128-bit) of the service id: stable across Rust versions and platforms.
pub fn store_id(service_id: &str) -> [u8; 16] {
    const OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let hash = service_id.bytes().fold(OFFSET, |hash, byte| (hash ^ u128::from(byte)).wrapping_mul(PRIME));
    hash.to_be_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names_are_safe_profile_names() {
        let name = folder_name("whatsapp-k3x9");
        if cfg!(target_os = "windows") {
            assert_eq!(name, "svc-whatsapp-k3x9");
            assert!(folder_name(&"x".repeat(200)).len() <= 64);
            assert_eq!(folder_name("a/b"), "svc-a_b");
        } else {
            assert_eq!(name, "whatsapp-k3x9");
        }
    }

    #[test]
    fn is_stable_and_distinct() {
        // Pinned value: changing the algorithm would sign macOS users out.
        assert_eq!(store_id(""), 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d_u128.to_be_bytes());
        assert_eq!(store_id("whatsapp"), store_id("whatsapp"));
        assert_ne!(store_id("whatsapp"), store_id("whatsapp-2"));
        assert_ne!(store_id("discord"), store_id("gmail"));
    }
}

//! User agent strings for services that reject the default engine.

use crate::services::UserAgentPreset;

const CHROME: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
const FIREFOX: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0";

/// Returns the user agent override for a preset, or `None` to keep the engine default.
pub fn for_preset(preset: UserAgentPreset) -> Option<&'static str> {
    match preset {
        UserAgentPreset::Default => None,
        UserAgentPreset::Chrome => Some(CHROME),
        UserAgentPreset::Firefox => Some(FIREFOX),
    }
}

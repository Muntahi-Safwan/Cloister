//! The Cloister PIN: which PINs are accepted, how they are stored, and how
//! long to wait after repeated wrong guesses.
//!
//! The PIN is never stored. Only an Argon2id hash (with a random salt) is kept,
//! in the PHC string format, so a copied settings folder does not reveal it.

use std::time::Duration;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Shortest accepted PIN.
pub const MIN_LEN: usize = 4;
/// Longest accepted PIN (it may also be a longer password).
pub const MAX_LEN: usize = 64;
/// Wrong guesses allowed before Cloister starts making you wait.
pub const FREE_ATTEMPTS: u32 = 5;
/// The longest wait between guesses.
const MAX_WAIT: Duration = Duration::from_secs(5 * 60);

/// Checks that a new PIN is acceptable. Returns a message for the user if not.
pub fn validate(pin: &str) -> Result<(), String> {
    let len = pin.chars().count();
    if len < MIN_LEN {
        return Err(format!("Use at least {MIN_LEN} characters."));
    }
    if len > MAX_LEN {
        return Err(format!("Use at most {MAX_LEN} characters."));
    }
    if pin.trim() != pin {
        return Err("The PIN can't start or end with a space.".into());
    }
    Ok(())
}

/// Hashes a PIN for storage. `None` only if the system has no random source.
pub fn hash(pin: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(pin.as_bytes(), &salt).ok().map(|h| h.to_string())
}

/// Whether `pin` matches a hash made by [`hash`]. A damaged hash never matches.
pub fn verify(pin: &str, stored: &str) -> bool {
    PasswordHash::new(stored)
        .is_ok_and(|parsed| Argon2::default().verify_password(pin.as_bytes(), &parsed).is_ok())
}

/// How long to wait before the next guess after `failures` wrong guesses in a row.
/// The first few are free; after that the wait starts at 30 seconds and doubles.
pub fn wait_after(failures: u32) -> Duration {
    if failures < FREE_ATTEMPTS {
        return Duration::ZERO;
    }
    let doublings = (failures - FREE_ATTEMPTS).min(10);
    (Duration::from_secs(30) * 2u32.pow(doublings)).min(MAX_WAIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_reasonable_pins() {
        assert!(validate("1234").is_ok());
        assert!(validate("correct horse").is_ok());
        assert!(validate("১২৩৪").is_ok(), "length counts characters, not bytes");
    }

    #[test]
    fn rejects_short_long_or_padded_pins() {
        assert!(validate("123").is_err());
        assert!(validate(&"9".repeat(MAX_LEN + 1)).is_err());
        assert!(validate(" 1234").is_err());
        assert!(validate("1234 ").is_err());
    }

    #[test]
    fn hash_verifies_only_the_same_pin() {
        let stored = hash("2580").unwrap();
        assert!(!stored.contains("2580"), "the PIN itself is never stored");
        assert!(verify("2580", &stored));
        assert!(!verify("2581", &stored));
        assert!(!verify("", &stored));
    }

    #[test]
    fn same_pin_hashes_differently_each_time() {
        assert_ne!(hash("2580").unwrap(), hash("2580").unwrap());
    }

    #[test]
    fn damaged_hash_never_matches() {
        assert!(!verify("2580", ""));
        assert!(!verify("2580", "not a hash"));
    }

    #[test]
    fn waits_grow_after_free_attempts_and_are_capped() {
        assert_eq!(wait_after(0), Duration::ZERO);
        assert_eq!(wait_after(FREE_ATTEMPTS - 1), Duration::ZERO);
        assert_eq!(wait_after(FREE_ATTEMPTS), Duration::from_secs(30));
        assert_eq!(wait_after(FREE_ATTEMPTS + 1), Duration::from_secs(60));
        assert_eq!(wait_after(FREE_ATTEMPTS + 2), Duration::from_secs(120));
        assert_eq!(wait_after(100), MAX_WAIT);
    }
}

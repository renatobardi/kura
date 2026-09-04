//! NIP-49 identity lock for the headless daemon.
//!
//! Thin wrappers around the same `nostr::nips::nip49` primitive `kura-desktop`
//! already uses for its portable key backups (`desktop/src-tauri/src/key_backup.rs`):
//! `EncryptedSecretKey::new` to encrypt, `EncryptedSecretKey::from_bech32` +
//! `decrypt` to recover. This module exists so `kurad` can ship an identity
//! "locked" (encrypted at rest under a passphrase) instead of the plaintext
//! `identity.key` file — see [`crate::headless_identity`] for the resolution
//! flow that uses it.

use nostr::nips::nip49::{EncryptedSecretKey, KeySecurity};
use nostr::{FromBech32, Keys, ToBech32};

/// scrypt cost for locking a headless identity. Matches the desktop's
/// `key_backup::BACKUP_LOG_N` (2^18, Gossip's desktop default) so the two
/// paths share one durability/cost tradeoff instead of inventing a second one.
pub const LOCK_LOG_N: u8 = 18;

/// Encrypt `keys` under `passphrase`, returning the bech32 `ncryptsec1…`
/// string. Never panics; a `KeySecurity::Unknown` tag is used because the
/// daemon has no way to assert stronger guarantees about how the secret was
/// generated or handled before reaching this function.
pub fn lock_keys_to_ncryptsec(keys: &Keys, passphrase: &str) -> Result<String, String> {
    lock_keys_to_ncryptsec_with_log_n(keys, passphrase, LOCK_LOG_N)
}

/// Same as [`lock_keys_to_ncryptsec`] with an explicit scrypt cost. Exposed
/// at `pub(crate)` visibility purely so tests elsewhere in this crate (e.g.
/// [`crate::headless_identity`]'s) can use a cheap cost and stay fast; every
/// non-test call site must go through [`lock_keys_to_ncryptsec`], which pins
/// the real [`LOCK_LOG_N`].
pub(crate) fn lock_keys_to_ncryptsec_with_log_n(
    keys: &Keys,
    passphrase: &str,
    log_n: u8,
) -> Result<String, String> {
    let encrypted =
        EncryptedSecretKey::new(keys.secret_key(), passphrase, log_n, KeySecurity::Unknown)
            .map_err(|e| format!("encrypt identity: {e}"))?;

    encrypted
        .to_bech32()
        .map_err(|e| format!("encode ncryptsec: {e}"))
}

/// Parse and decrypt an `ncryptsec1…` string with `passphrase` into identity
/// keys. Returns a clear `Err` — never panics — on a malformed string or a
/// wrong passphrase.
pub fn unlock_ncryptsec(ncryptsec: &str, passphrase: &str) -> Result<Keys, String> {
    let encrypted = EncryptedSecretKey::from_bech32(ncryptsec.trim())
        .map_err(|e| format!("invalid ncryptsec: {e}"))?;

    let secret_key = encrypted
        .decrypt(passphrase)
        .map_err(|_| "wrong passphrase or corrupted identity.ncryptsec".to_string())?;

    Ok(Keys::new(secret_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fast scrypt tier for tests. `LOCK_LOG_N` (18, ~256 MiB) is exercised
    /// once below to prove the production constant actually works; every
    /// other test uses this cheaper tier to keep the suite fast — same
    /// pattern as the desktop's `key_backup_tests.rs::FAST_LOG_N`.
    const FAST_LOG_N: u8 = 12;

    #[test]
    fn round_trip_recovers_the_same_keypair() {
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "correct horse battery staple", FAST_LOG_N)
                .unwrap();
        assert!(ncryptsec.starts_with("ncryptsec1"));

        let recovered = unlock_ncryptsec(&ncryptsec, "correct horse battery staple").unwrap();
        assert_eq!(recovered.public_key(), keys.public_key());
        assert_eq!(recovered.secret_key(), keys.secret_key());
    }

    #[test]
    fn production_log_n_round_trips() {
        let keys = Keys::generate();
        let ncryptsec = lock_keys_to_ncryptsec(&keys, "production cost tier check").unwrap();
        let recovered = unlock_ncryptsec(&ncryptsec, "production cost tier check").unwrap();
        assert_eq!(recovered.public_key(), keys.public_key());
    }

    #[test]
    fn wrong_passphrase_fails_cleanly() {
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "right passphrase", FAST_LOG_N).unwrap();

        let result = unlock_ncryptsec(&ncryptsec, "wrong passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn malformed_ncryptsec_fails_cleanly() {
        let result = unlock_ncryptsec("not-an-ncryptsec-string", "whatever");
        assert!(result.is_err());

        let result = unlock_ncryptsec("ncryptsec1garbage", "whatever");
        assert!(result.is_err());
    }
}

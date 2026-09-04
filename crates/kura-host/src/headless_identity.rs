//! Headless (`kurad`-only) identity resolution over a locked, NIP-49
//! encrypted identity file.
//!
//! This is a *separate* resolution path from
//! [`crate::app_state::resolve_persisted_identity`], which desktop also
//! calls — that function is left completely untouched so desktop behavior is
//! unaffected. [`resolve_headless_identity`] only ever runs when
//! `<data_dir>/identity.ncryptsec` exists; the caller (`kurad`) falls back to
//! `resolve_persisted_identity` when it does not.
//!
//! File layout under `data_dir`:
//! - `identity.ncryptsec` — the locked ciphertext (bech32 string, plain text
//!   file; safe to read since it is ciphertext).
//! - `identity.autounlock` — `0600`, plaintext passphrase, present only when
//!   the operator opted into unattended unlock (`kurad identity unlock
//!   --remember`).

use std::path::{Path, PathBuf};

use crate::identity_lock::unlock_ncryptsec;
use crate::identity_storage::{IdentityStorage, RecoveryState, ResolvedIdentity};

/// Filename of the locked identity ciphertext.
pub const NCRYPTSEC_FILE_NAME: &str = "identity.ncryptsec";

/// Filename of the opt-in stored passphrase for unattended unlock.
pub const AUTOUNLOCK_FILE_NAME: &str = "identity.autounlock";

/// Env var carrying the passphrase for scripted/`systemd EnvironmentFile=`
/// use. Same trust tier as the existing `KURA_PRIVATE_KEY` override.
pub const PASSPHRASE_ENV_VAR: &str = "KURA_IDENTITY_PASSPHRASE";

/// Path of the locked identity ciphertext under `data_dir`.
pub fn ncryptsec_path(data_dir: &Path) -> PathBuf {
    data_dir.join(NCRYPTSEC_FILE_NAME)
}

/// Path of the opt-in autounlock passphrase file under `data_dir`.
pub fn autounlock_path(data_dir: &Path) -> PathBuf {
    data_dir.join(AUTOUNLOCK_FILE_NAME)
}

/// A source of an interactive passphrase prompt. Abstracted so tests never
/// have to touch a real terminal.
pub trait PassphraseSource {
    /// Prompt with `message` and return the entered passphrase. Must return a
    /// clear `Err` — never panic or hang — when no interactive prompt is
    /// possible (e.g. stdin is not a TTY).
    fn prompt(&self, message: &str) -> Result<String, String>;
}

/// Real, no-echo terminal prompt backed by `rpassword`.
pub struct TerminalPassphraseSource;

impl PassphraseSource for TerminalPassphraseSource {
    fn prompt(&self, message: &str) -> Result<String, String> {
        rpassword::prompt_password(message)
            .map_err(|e| format!("could not read passphrase from terminal: {e}"))
    }
}

/// Resolve the identity from a locked `identity.ncryptsec` file.
///
/// This function is ONLY for the locked path: it returns `Err` when
/// `<data_dir>/identity.ncryptsec` does not exist. Callers fall back to
/// [`crate::app_state::resolve_persisted_identity`] in that case rather than
/// this function merging the two flows.
///
/// Passphrase precedence:
/// 1. `KURA_IDENTITY_PASSPHRASE` env var, if set and non-empty.
/// 2. `<data_dir>/identity.autounlock`, if present (trimmed of whitespace).
/// 3. `passphrase.prompt(...)`, interactively.
///
/// A decrypt failure at any stage is a clear `Err` — this function never
/// falls back to generating or returning an ephemeral identity. Signing under
/// a phantom identity while the real, locked one sits right there would be a
/// security/correctness bug, not graceful degradation.
pub fn resolve_headless_identity(
    data_dir: &Path,
    passphrase: &dyn PassphraseSource,
) -> Result<ResolvedIdentity, String> {
    let ncryptsec_path = ncryptsec_path(data_dir);
    if !ncryptsec_path.exists() {
        return Err(format!(
            "no locked identity at {}",
            ncryptsec_path.display()
        ));
    }
    let ncryptsec = std::fs::read_to_string(&ncryptsec_path)
        .map_err(|e| format!("read {}: {e}", ncryptsec_path.display()))?;

    let (source, secret) = resolve_passphrase(data_dir, passphrase)?;

    let keys = unlock_ncryptsec(ncryptsec.trim(), &secret)
        .map_err(|e| format!("unlock identity ({source}): {e}"))?;

    Ok(ResolvedIdentity {
        keys,
        recovery: RecoveryState::None,
        storage: IdentityStorage::LockedLocalFile,
    })
}

/// Where the passphrase came from, for error messages only.
fn resolve_passphrase(
    data_dir: &Path,
    passphrase: &dyn PassphraseSource,
) -> Result<(&'static str, String), String> {
    match std::env::var(PASSPHRASE_ENV_VAR) {
        Ok(value) if !value.is_empty() => return Ok(("env var", value)),
        Ok(_) => {} // empty: fall through
        Err(std::env::VarError::NotPresent) => {}
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(format!("{PASSPHRASE_ENV_VAR} contains invalid UTF-8"));
        }
    }

    let autounlock = autounlock_path(data_dir);
    if autounlock.exists() {
        let content = std::fs::read_to_string(&autounlock)
            .map_err(|e| format!("read {}: {e}", autounlock.display()))?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(format!("{} is empty", autounlock.display()));
        }
        return Ok(("autounlock file", trimmed.to_string()));
    }

    let entered = passphrase.prompt("Passphrase to unlock the Kura identity: ")?;
    Ok(("interactive prompt", entered))
}

/// Write `passphrase` to `<data_dir>/identity.autounlock` at `0600`.
pub fn write_autounlock_file(data_dir: &Path, passphrase: &str) -> Result<(), String> {
    use atomic_write_file::AtomicWriteFile;
    use std::io::Write;

    let path = autounlock_path(data_dir);
    let mut file =
        AtomicWriteFile::open(&path).map_err(|e| format!("open {}: {e}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("set permissions on {}: {e}", path.display()))?;
    }

    file.write_all(passphrase.as_bytes())
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    file.commit()
        .map_err(|e| format!("commit {}: {e}", path.display()))
}

/// Delete `<data_dir>/identity.autounlock` if present. A missing file is not
/// an error.
pub fn forget_autounlock(data_dir: &Path) -> Result<bool, String> {
    let path = autounlock_path(data_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("delete {}: {e}", path.display())),
    }
}

/// Write the locked ciphertext to `<data_dir>/identity.ncryptsec`. Refuses to
/// overwrite an existing file unless `force` is set.
pub fn write_ncryptsec_file(data_dir: &Path, ncryptsec: &str, force: bool) -> Result<(), String> {
    let path = ncryptsec_path(data_dir);
    if path.exists() && !force {
        return Err(format!(
            "{} already exists; pass --force to overwrite it",
            path.display()
        ));
    }
    std::fs::write(&path, ncryptsec).map_err(|e| format!("write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity_lock::lock_keys_to_ncryptsec_with_log_n;

    /// Cheap scrypt tier so this module's tests stay fast; production always
    /// goes through `lock_keys_to_ncryptsec`'s pinned `LOCK_LOG_N`.
    const FAST_LOG_N: u8 = 12;
    use nostr::Keys;

    struct FixedPassphrase(&'static str);
    impl PassphraseSource for FixedPassphrase {
        fn prompt(&self, _message: &str) -> Result<String, String> {
            Ok(self.0.to_string())
        }
    }

    struct FailingPassphrase;
    impl PassphraseSource for FailingPassphrase {
        fn prompt(&self, _message: &str) -> Result<String, String> {
            Err("no TTY available".to_string())
        }
    }

    struct PanicIfCalled;
    impl PassphraseSource for PanicIfCalled {
        fn prompt(&self, _message: &str) -> Result<String, String> {
            panic!("prompt() must not be called when a non-interactive passphrase is available");
        }
    }

    /// Serializes env-var mutation across tests in this module: `std::env::set_var`
    /// is process-global and `cargo test` runs tests in the same process.
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    fn clear_env() {
        std::env::remove_var(PASSPHRASE_ENV_VAR);
    }

    #[test]
    fn no_ncryptsec_file_returns_err() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let result = resolve_headless_identity(dir.path(), &PanicIfCalled);
        assert!(result.is_err());
    }

    #[test]
    fn autounlock_file_takes_precedence_over_interactive_prompt() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "s3cret-phrase", FAST_LOG_N).unwrap();
        write_ncryptsec_file(dir.path(), &ncryptsec, false).unwrap();
        write_autounlock_file(dir.path(), "s3cret-phrase").unwrap();

        // PanicIfCalled proves the interactive prompt is never invoked.
        let resolved = resolve_headless_identity(dir.path(), &PanicIfCalled).unwrap();
        assert_eq!(resolved.keys.public_key(), keys.public_key());
        assert_eq!(resolved.storage, IdentityStorage::LockedLocalFile);
        assert_eq!(resolved.recovery, RecoveryState::None);
    }

    #[test]
    fn env_var_takes_precedence_over_autounlock_file() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let keys = Keys::generate();
        let ncryptsec = lock_keys_to_ncryptsec_with_log_n(&keys, "env-phrase", FAST_LOG_N).unwrap();
        write_ncryptsec_file(dir.path(), &ncryptsec, false).unwrap();
        // Autounlock file holds the WRONG passphrase — if it were consulted
        // first, decryption would fail.
        write_autounlock_file(dir.path(), "wrong-phrase").unwrap();
        std::env::set_var(PASSPHRASE_ENV_VAR, "env-phrase");

        let resolved = resolve_headless_identity(dir.path(), &PanicIfCalled).unwrap();
        assert_eq!(resolved.keys.public_key(), keys.public_key());

        clear_env();
    }

    #[test]
    fn wrong_passphrase_in_autounlock_file_is_a_clean_err() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "right-phrase", FAST_LOG_N).unwrap();
        write_ncryptsec_file(dir.path(), &ncryptsec, false).unwrap();
        write_autounlock_file(dir.path(), "wrong-phrase").unwrap();

        let result = resolve_headless_identity(dir.path(), &PanicIfCalled);
        assert!(result.is_err());
    }

    #[test]
    fn interactive_prompt_used_when_no_env_and_no_autounlock() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "typed-phrase", FAST_LOG_N).unwrap();
        write_ncryptsec_file(dir.path(), &ncryptsec, false).unwrap();

        let resolved =
            resolve_headless_identity(dir.path(), &FixedPassphrase("typed-phrase")).unwrap();
        assert_eq!(resolved.keys.public_key(), keys.public_key());
    }

    #[test]
    fn no_tty_and_no_autounlock_surfaces_prompt_error() {
        let _guard = env_lock();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let keys = Keys::generate();
        let ncryptsec =
            lock_keys_to_ncryptsec_with_log_n(&keys, "typed-phrase", FAST_LOG_N).unwrap();
        write_ncryptsec_file(dir.path(), &ncryptsec, false).unwrap();

        let result = resolve_headless_identity(dir.path(), &FailingPassphrase);
        assert!(result.is_err());
    }

    #[test]
    fn write_ncryptsec_file_refuses_to_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        write_ncryptsec_file(dir.path(), "ncryptsec1first", false).unwrap();
        let result = write_ncryptsec_file(dir.path(), "ncryptsec1second", false);
        assert!(result.is_err());

        write_ncryptsec_file(dir.path(), "ncryptsec1second", true).unwrap();
        let on_disk = std::fs::read_to_string(ncryptsec_path(dir.path())).unwrap();
        assert_eq!(on_disk, "ncryptsec1second");
    }

    #[test]
    fn forget_autounlock_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!forget_autounlock(dir.path()).unwrap());

        write_autounlock_file(dir.path(), "whatever").unwrap();
        assert!(forget_autounlock(dir.path()).unwrap());
        assert!(!autounlock_path(dir.path()).exists());
        // Second call: still not an error, just reports nothing was there.
        assert!(!forget_autounlock(dir.path()).unwrap());
    }
}

use nostr::Keys;

use crate::app_state::AppState;

/// Durable location of the active human identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IdentityStorage {
    Ephemeral = 0,
    SystemKeyring = 1,
    LocalFile = 2,
    Environment = 3,
    /// NIP-49 encrypted `identity.ncryptsec`, resolved through
    /// `headless_identity::resolve_headless_identity`. Headless (`kurad`) only —
    /// desktop never produces or reads this variant.
    LockedLocalFile = 4,
}

impl IdentityStorage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ephemeral => "ephemeral",
            Self::SystemKeyring => "system-keyring",
            Self::LocalFile => "local-file",
            Self::Environment => "environment",
            Self::LockedLocalFile => "locked-local-file",
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::SystemKeyring,
            2 => Self::LocalFile,
            3 => Self::Environment,
            4 => Self::LockedLocalFile,
            _ => Self::Ephemeral,
        }
    }
}

impl AppState {
    pub fn identity_storage(&self) -> IdentityStorage {
        IdentityStorage::from_u8(
            self.identity_storage
                .load(std::sync::atomic::Ordering::Acquire),
        )
    }

    pub fn set_identity_storage(&self, storage: IdentityStorage) {
        self.identity_storage
            .store(storage as u8, std::sync::atomic::Ordering::Release);
    }
}

/// Recovery state produced by identity resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    None,
    Lost,
    KeyringLocked,
}

/// Identity and persistence metadata produced by startup resolution.
pub struct ResolvedIdentity {
    pub keys: Keys,
    pub recovery: RecoveryState,
    pub storage: IdentityStorage,
}

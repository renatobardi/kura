//! Relay egress guard for NIP-49 key-backup material.
//!
//! The guard itself now lives in [`kura_host::egress_guard`] — every relay
//! egress boundary in this crate and in `kura-host` calls the same function.
//! What stays here is the boundary-completeness test: it walks BOTH source
//! trees and asserts that every `/events` URL-construction site calls the
//! guard, so a new submission path in either crate fails the build until it is
//! wired.

pub use kura_host::egress_guard::{assert_no_key_backup, assert_no_key_backup_bytes};

#[cfg(test)]
#[path = "egress_guard_tests.rs"]
mod tests;

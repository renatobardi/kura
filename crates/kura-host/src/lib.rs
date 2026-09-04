//! Host-agnostic Kura desktop backend.
//!
//! This crate holds the parts of the Kura desktop backend that have nothing to
//! do with Tauri: managed agents (spawn, discovery, personas, teams, nest,
//! retention), the local archive database and its relay sync, the relay
//! client, the identity/secret storage, and the shared [`app_state::AppState`]
//! those need.
//!
//! Everything that used to take a `tauri::AppHandle` now takes a
//! [`host::Host`]. The desktop crate implements it (`TauriHost`) and keeps the
//! `#[tauri::command]` wrappers; nothing here depends on `tauri`.

pub mod host;

pub mod app_state;
pub mod archive;
pub mod egress_guard;
pub mod events;
pub mod identity_archive;
pub mod identity_lock;
pub mod identity_storage;
pub mod link_preview_tags;
pub mod managed_agents;
pub mod native_relay_client;
pub mod profile_reconcile;
pub mod relay;
pub mod relay_admission;
pub mod secret_store;
pub mod util;

#[cfg(test)]
mod test_host;

pub use host::{Host, HostHandle};

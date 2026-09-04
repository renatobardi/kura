//! Desktop application state.
//!
//! The host-agnostic half lives in [`kura_host::app_state::AppState`]; this
//! struct wraps it and adds the fields only the Tauri desktop has (huddle,
//! mesh-llm, the app handle, the media proxy, the pending-channel overlay).
//!
//! [`Deref`] to the host state is what keeps the existing `State<'_,
//! AppState>` call sites compiling unchanged: `state.keys`, `state.archive_db`
//! and friends resolve through it, and `&AppState` coerces to
//! `&kura_host::app_state::AppState` wherever a `kura_host` function wants one.

use std::sync::{atomic::AtomicU16, Arc, Mutex};

use tauri::AppHandle;
#[cfg(feature = "mesh-llm")]
use tokio::sync::Mutex as AsyncMutex;

use crate::huddle::HuddleState;

pub use kura_host::app_state::{
    keyring_service, persist_imported_identity, resolve_persisted_identity,
};

/// Build the no-redirect HTTP client used for authenticated relay media
/// fetches (download / copy).
///
/// This client is a security boundary, not a convenience: it carries a minted
/// media `Authorization` header, so it MUST NOT follow redirects. A relay 3xx
/// to an off-origin or private host would otherwise forward that header across
/// origins (a redirect-hop SSRF). `redirect::Policy::none()` returns the 3xx
/// verbatim so the caller can reject it.
///
/// Returned as a `Result` so the fail-closed invariant is testable — callers
/// must never substitute a redirect-following client on build failure. Shares
/// the localhost `resolve`/pool config with the app-wide `http_client`.
pub fn build_media_fetch_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .resolve("localhost", std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
        .pool_idle_timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(1)
        .redirect(reqwest::redirect::Policy::none())
        .build()
}
#[allow(unused_imports)]
pub(crate) use kura_host::identity_storage::{IdentityStorage, RecoveryState, ResolvedIdentity};

pub struct AppState {
    /// The host-agnostic backend state (identity, relay, managed agents,
    /// archive DB). Reachable directly through [`Deref`].
    pub host: kura_host::app_state::AppState,
    /// A no-redirect client for authenticated relay media fetches (download,
    /// clipboard copy, snapshot, editor). Every caller pre-validates the URL
    /// origin, but the app-wide `http_client` follows redirects by default, so
    /// a relay `/media/` URL returning a 3xx to an off-origin or private host
    /// would forward the minted media Authorization header across origins —
    /// a redirect-hop SSRF. This client treats any 3xx as a non-success
    /// response (surfaced as an error) so the auth token never leaves the
    /// validated relay origin.
    pub media_fetch_client: reqwest::Client,
    pub channel_templates_store_lock: Mutex<()>,
    pub huddle_state: Mutex<HuddleState>,
    pub huddle_audio: crate::huddle::tts_settings::HuddleAudioSettingsState,
    /// Tauri app handle — stored after setup so huddle commands can emit
    /// `huddle-state-changed` events without needing the handle threaded
    /// through every call site.
    ///
    /// Set once during `setup()` in `lib.rs`; never cleared.
    pub app_handle: Mutex<Option<AppHandle>>,
    /// Port of the localhost media streaming proxy (set during setup).
    pub media_proxy_port: AtomicU16,
    /// IOKit power assertion state — prevents idle sleep while agents run.
    pub prevent_sleep: Arc<Mutex<crate::prevent_sleep::PreventSleepState>>,
    /// In-process mesh-llm node started by Kura Desktop.
    #[cfg(feature = "mesh-llm")]
    pub mesh_llm_runtime: AsyncMutex<Option<crate::mesh_llm::DesktopMeshRuntime>>,
    #[cfg(feature = "mesh-llm")]
    pub mesh_recovery: crate::mesh_llm::MeshRecoveryState,
    /// Runtime-owned shared-compute coordinator. It publishes member-signed
    /// discovery status and reconciles MeshLLM's admission roster; MeshLLM
    /// itself owns direct QUIC/iroh connection establishment.
    #[cfg(feature = "mesh-llm")]
    pub mesh_coordinator: AsyncMutex<Option<crate::mesh_llm::MeshCoordinator>>,
    /// `(creator_pubkey_hex, channel_id)` pairs for channels the *named*
    /// identity created via `create_channel` and has not yet observed its own
    /// kind:39002 membership entry for. The relay provisions that entry
    /// asynchronously (#1761), so without this overlay a freshly created
    /// channel's owner reads back as `is_member=false` until the snapshot
    /// propagates, disabling their own composer. Entries are bound to the
    /// creating identity so an in-process identity swap (`import_identity`,
    /// workspace apply) can never inherit another identity's stale
    /// membership. Populated only by this process's own `create_channel`
    /// calls — a relay can never write into it — so it carries no
    /// trust-boundary risk. `get_channels` clears an entry once the real
    /// kind:39002 is observed for the current identity, keeping the set
    /// bounded and letting a later leave correctly flip the channel back to
    /// `is_member=false`.
    pub pending_owned_channels: Mutex<std::collections::HashSet<(String, String)>>,
}

impl std::ops::Deref for AppState {
    type Target = kura_host::app_state::AppState;

    fn deref(&self) -> &Self::Target {
        &self.host
    }
}

impl std::ops::DerefMut for AppState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.host
    }
}

pub fn build_app_state() -> AppState {
    AppState {
        host: kura_host::app_state::build_app_state(),
        media_fetch_client: build_media_fetch_client().expect(
            "media_fetch_client must build with redirect::Policy::none(); a \
             redirect-following fallback would forward the minted media auth \
             header across origins (redirect-hop SSRF)",
        ),
        channel_templates_store_lock: Mutex::new(()),
        huddle_state: Mutex::new(HuddleState::default()),
        huddle_audio: Default::default(),
        app_handle: Mutex::new(None),
        media_proxy_port: AtomicU16::new(0),
        prevent_sleep: Default::default(),
        #[cfg(feature = "mesh-llm")]
        mesh_llm_runtime: AsyncMutex::new(None),
        #[cfg(feature = "mesh-llm")]
        mesh_recovery: crate::mesh_llm::MeshRecoveryState::default(),
        #[cfg(feature = "mesh-llm")]
        mesh_coordinator: AsyncMutex::new(None),
        pending_owned_channels: Mutex::new(std::collections::HashSet::new()),
    }
}

impl AppState {
    /// Lock the huddle state mutex, converting a poisoned-lock error to a String.
    ///
    /// Convenience wrapper — replaces 15+ instances of
    /// `state.huddle_state.lock().map_err(|e| e.to_string())?` throughout the
    /// huddle module.
    pub fn huddle(&self) -> Result<std::sync::MutexGuard<'_, crate::huddle::HuddleState>, String> {
        self.huddle_state.lock().map_err(|e| e.to_string())
    }

    /// Emit the current huddle state to the frontend via Tauri event.
    ///
    /// Acquires both locks (app_handle + huddle_state), clones a snapshot,
    /// releases both, then emits. Best-effort — no-op if either lock is
    /// poisoned or the app_handle hasn't been set yet.
    pub fn emit_huddle_state_changed(&self) {
        let app = match self.app_handle.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => return,
        };
        let Some(app) = app else { return };
        let snapshot = match self.huddle_state.lock() {
            Ok(hs) => hs.clone(),
            Err(_) => return,
        };
        crate::huddle::state::emit_huddle_state(&app, &snapshot);
    }
}

#[path = "app_state_pending_channels.rs"]
mod pending_channels;

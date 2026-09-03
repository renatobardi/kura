//! The host abstraction.
//!
//! Everything in this crate used to take a `tauri::AppHandle`. It now takes a
//! [`Host`] instead: the small set of capabilities the backend actually needs
//! from its embedder (directories, shared state, event emission, task spawn,
//! instance identity, dev-vs-prod). The desktop crate provides a `TauriHost`
//! implementation; tests provide their own.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use crate::app_state::AppState;

/// A boxed future a [`Host`] can spawn.
pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// The embedder-provided capabilities this crate needs.
///
/// Implementations must be cheap to clone through the [`HostHandle`] `Arc` and
/// safe to use from any thread.
pub trait Host: Send + Sync + 'static {
    /// Per-user application data directory (identity key, agent stores, DBs).
    fn app_data_dir(&self) -> Result<PathBuf, String>;

    /// Per-user application config directory.
    fn app_config_dir(&self) -> Result<PathBuf, String>;

    /// The shared application state. The returned reference must live as long
    /// as the host itself, so implementations hold it behind an `Arc` rather
    /// than looking it up per call.
    fn state(&self) -> &AppState;

    /// Emit an event to the frontend. Best-effort; errors are surfaced as
    /// `Err` so callers can log them.
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String>;

    /// Spawn a detached background task.
    ///
    /// This is deliberately a host capability rather than a bare
    /// `tokio::spawn`: several call sites (`try_regenerate_nest`,
    /// `spawn_warm_init`) run synchronously from the desktop's `setup()`,
    /// outside any tokio runtime context, where `tokio::spawn` panics. The
    /// Tauri host routes this to `tauri::async_runtime::spawn`, which does
    /// not.
    fn spawn(&self, future: BoxFuture);

    /// Stable identifier for this application instance (the bundle
    /// identifier on the desktop). Used to scope runtime receipts so two
    /// installs never adopt each other's processes.
    fn instance_id(&self) -> String;

    /// `true` for a development build. Drives the dev nest directory name and
    /// other dev-only affordances.
    fn is_dev(&self) -> bool;
}

/// Shared, cloneable handle to the [`Host`].
pub type HostHandle = Arc<dyn Host>;

/// Convenience: spawn an ordinary future on the host.
pub fn spawn<F>(host: &HostHandle, future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    host.spawn(Box::pin(future));
}

static GLOBAL_HOST: std::sync::OnceLock<HostHandle> = std::sync::OnceLock::new();

/// Register the process-wide host.
///
/// The desktop calls this once, first thing in `setup()`. It exists for the
/// handful of call sites buried under `Default`-constructed managed state
/// (`NativeRelayClient`) that cannot be handed a handle through their
/// signature. Everything else takes a [`HostHandle`] argument.
pub fn install(host: HostHandle) {
    let _ = GLOBAL_HOST.set(host);
}

/// The process-wide host, if one was installed.
pub fn global() -> Option<&'static HostHandle> {
    GLOBAL_HOST.get()
}

/// Spawn on the installed host.
///
/// Falls back to `tokio::spawn` when no host is installed — that is the unit
/// test configuration, where the caller is already inside a tokio runtime. In
/// the desktop the host is always installed before any relay session starts,
/// so this routes to `tauri::async_runtime::spawn`.
pub fn spawn_detached<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    match global() {
        Some(host) => host.spawn(Box::pin(future)),
        None => {
            tokio::spawn(future);
        }
    }
}

//! The Tauri implementation of [`kura_host::Host`].
//!
//! `kura-host` knows nothing about Tauri: everything it needs from the
//! embedder — directories, the shared state, event emission, task spawn,
//! instance identity, dev-vs-prod — arrives through this adapter.
//!
//! [`AsHost`] is the ergonomic half. Every `kura_host` entry point takes a
//! [`HostHandle`]; the desktop's call sites hold a `tauri::AppHandle`. `AsHost`
//! bridges the two so `crate::managed_agents::load_managed_agents(&app)` keeps
//! working verbatim.

use std::path::PathBuf;

use kura_host::host::{BoxFuture, Host, HostHandle};
use tauri::{Emitter, Manager, Runtime};

/// A [`Host`] backed by a Tauri app handle.
pub struct TauriHost<R: Runtime> {
    app: tauri::AppHandle<R>,
}

impl<R: Runtime> TauriHost<R> {
    pub fn new(app: tauri::AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> Host for TauriHost<R> {
    fn app_data_dir(&self) -> Result<PathBuf, String> {
        self.app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())
    }

    fn app_config_dir(&self) -> Result<PathBuf, String> {
        self.app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())
    }

    fn state(&self) -> &kura_host::app_state::AppState {
        // `State::inner` hands back a reference borrowed from the app handle
        // this adapter owns, so the returned `&AppState` lives as long as
        // `&self` — no `Arc<AppState>` outside Tauri's managed state needed.
        // The desktop `AppState` derefs to the host one.
        self.app.state::<crate::app_state::AppState>().inner()
    }

    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        self.app.emit(event, payload).map_err(|e| e.to_string())
    }

    fn spawn(&self, future: BoxFuture) {
        // MUST be `tauri::async_runtime::spawn`, not `tokio::spawn`.
        // `try_regenerate_nest` and `spawn_warm_init` are called synchronously
        // from `setup()`, outside any tokio runtime context, where a bare
        // `tokio::spawn` panics.
        tauri::async_runtime::spawn(future);
    }

    fn instance_id(&self) -> String {
        self.app.config().identifier.clone()
    }

    fn is_dev(&self) -> bool {
        self.app_data_dir()
            .ok()
            .as_deref()
            .and_then(|dir| dir.file_name())
            .and_then(|name| name.to_str())
            .is_some_and(crate::migration::is_dev_data_dir_name)
    }
}

/// Anything the desktop can turn into a [`HostHandle`].
///
/// Implemented for `AppHandle` (owned and by reference) so existing call sites
/// that pass `app` or `&app` need no change, and for `HostHandle` itself so a
/// handle already in hand is passed straight through.
pub trait AsHost {
    fn as_host(&self) -> HostHandle;
}

impl<R: Runtime> AsHost for tauri::AppHandle<R> {
    fn as_host(&self) -> HostHandle {
        std::sync::Arc::new(TauriHost::new(self.clone()))
    }
}

impl AsHost for HostHandle {
    fn as_host(&self) -> HostHandle {
        HostHandle::clone(self)
    }
}

impl<T: AsHost + ?Sized> AsHost for &T {
    fn as_host(&self) -> HostHandle {
        (**self).as_host()
    }
}

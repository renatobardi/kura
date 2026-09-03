//! The headless implementation of [`kura_host::Host`].
//!
//! This is the whole daemon-shape proof: `kura-host` needs six capabilities
//! from its embedder, none of which require a window, a webview, or Tauri. The
//! desktop supplies them from an `AppHandle` (`TauriHost`); here they come from
//! a `--data-dir`, a `tracing` logger and a tokio runtime.

use std::path::{Path, PathBuf};

use kura_host::app_state::AppState;
use kura_host::host::{BoxFuture, Host, HostHandle};

/// A [`Host`] backed by a data directory and the ambient tokio runtime.
pub struct HeadlessHost {
    data_dir: PathBuf,
    state: AppState,
    instance_id: String,
    dev: bool,
}

impl HeadlessHost {
    /// Build a host over `data_dir`, taking ownership of `state`.
    ///
    /// The directory is created eagerly: `resolve_persisted_identity` and the
    /// managed-agent store both expect to be able to write under it, and a
    /// daemon whose data dir does not exist yet is an ordinary first boot.
    pub fn new(data_dir: PathBuf, state: AppState, dev: bool) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir).map_err(|error| {
            format!("failed to create data dir {}: {error}", data_dir.display())
        })?;
        let data_dir = data_dir
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize data dir: {error}"))?;
        let instance_id = instance_id_for(&data_dir);
        Ok(Self {
            data_dir,
            state,
            instance_id,
            dev,
        })
    }

    /// Wrap into the `Arc<dyn Host>` every `kura-host` entry point takes.
    pub fn into_handle(self) -> HostHandle {
        std::sync::Arc::new(self)
    }
}

/// The instance id stamped into every spawned agent's `KURA_MANAGED_AGENT`
/// env var and into its runtime receipt.
///
/// It must be stable across restarts (so a relaunched `kurad` reclaims its own
/// children) and distinct per install (so two daemons — or a daemon and a
/// desktop app, whose ids are bundle identifiers like `pro.oute.kura.app` —
/// never adopt or reap each other's processes). The canonical data-dir path
/// satisfies both: one daemon per data dir, same path every boot. The value is
/// only ever compared as an env-var string, never used as a filename, so an
/// embedded path is safe here.
fn instance_id_for(data_dir: &Path) -> String {
    format!("pro.oute.kurad:{}", data_dir.display())
}

impl Host for HeadlessHost {
    fn app_data_dir(&self) -> Result<PathBuf, String> {
        Ok(self.data_dir.clone())
    }

    fn app_config_dir(&self) -> Result<PathBuf, String> {
        // A headless install is one self-contained tree: the daemon is pointed
        // at a single directory and everything it owns lives under it. Nothing
        // in `kura-host` reads this today; splitting it off into an XDG config
        // dir would only invent a second location to keep in sync.
        Ok(self.data_dir.clone())
    }

    fn state(&self) -> &AppState {
        // The host owns the state, so the borrow lives as long as `&self` —
        // the contract `Host::state` asks for. Unlike `TauriHost` there is no
        // managed-state registry to look it up in.
        &self.state
    }

    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        // There is no frontend. Events are the backend's own progress
        // narration, so a log line is the honest headless equivalent; dropping
        // them silently would make agent lifecycle transitions invisible.
        tracing::info!(event, payload = %payload, "emit");
        Ok(())
    }

    fn spawn(&self, future: BoxFuture) {
        // Safe here, unlike at the `kura-host` call sites the trait doc warns
        // about: `main` is `#[tokio::main]`, so every path that reaches this
        // impl already runs inside a runtime context.
        tokio::spawn(future);
    }

    fn instance_id(&self) -> String {
        self.instance_id.clone()
    }

    fn is_dev(&self) -> bool {
        self.dev
    }
}

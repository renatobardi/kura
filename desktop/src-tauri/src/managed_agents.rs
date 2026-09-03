//! Thin desktop adapter over [`kura_host::managed_agents`].
//!
//! The managed-agent backend itself lives in `kura-host` and knows nothing
//! about Tauri. This module re-exports all of it and adds two things the
//! desktop needs on top:
//!
//! 1. Wrappers that accept whatever the call site already holds — an
//!    `AppHandle`, a reference to one, or a `HostHandle` — via
//!    [`crate::host::AsHost`], so the ~290 existing `crate::managed_agents::…(&app, …)`
//!    call sites keep compiling verbatim.
//! 2. The `#[tauri::command]` entry points, whose names, parameter names and
//!    return types are byte-identical to what the frontend already invokes.

use std::collections::HashMap;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::host::AsHost;

pub use kura_host::managed_agents::*;

pub fn load_global_agent_config(
    app: impl crate::host::AsHost,
) -> Result<GlobalAgentConfig, String> {
    kura_host::managed_agents::load_global_agent_config(&app.as_host())
}

pub fn save_global_agent_config(
    app: impl crate::host::AsHost,
    config: &GlobalAgentConfig,
) -> Result<(), String> {
    kura_host::managed_agents::save_global_agent_config(&app.as_host(), config)
}

pub fn try_regenerate_nest(app: impl crate::host::AsHost) {
    kura_host::managed_agents::try_regenerate_nest(&app.as_host())
}

pub fn load_personas(app: impl crate::host::AsHost) -> Result<Vec<AgentDefinition>, String> {
    kura_host::managed_agents::load_personas(&app.as_host())
}

pub fn save_personas(
    app: impl crate::host::AsHost,
    records: &[AgentDefinition],
) -> Result<(), String> {
    kura_host::managed_agents::save_personas(&app.as_host(), records)
}

pub async fn restore_managed_agents_on_launch(
    app: impl crate::host::AsHost,
    shutdown_started: &std::sync::atomic::AtomicBool,
    hooks: &dyn RestoreHooks,
) -> Result<(), String> {
    kura_host::managed_agents::restore_managed_agents_on_launch(
        &app.as_host(),
        shutdown_started,
        hooks,
    )
    .await
}

pub fn backfill_persona_snapshots(app: impl crate::host::AsHost) -> Result<(), String> {
    kura_host::managed_agents::backfill_persona_snapshots(&app.as_host())
}

pub fn build_managed_agent_summary(
    app: impl crate::host::AsHost,
    record: &ManagedAgentRecord,
    runtimes: &HashMap<ManagedAgentRuntimeKey, ManagedAgentPairRuntime>,
    personas: &[AgentDefinition],
    teams: &[crate::managed_agents::TeamRecord],
    global_config: &crate::managed_agents::GlobalAgentConfig,
) -> Result<ManagedAgentSummary, String> {
    kura_host::managed_agents::build_managed_agent_summary(
        &app.as_host(),
        record,
        runtimes,
        personas,
        teams,
        global_config,
    )
}

pub fn start_managed_agent_process(
    app: impl crate::host::AsHost,
    record: &mut ManagedAgentRecord,
    runtimes: &mut HashMap<ManagedAgentRuntimeKey, ManagedAgentPairRuntime>,
    owner_hex: Option<&str>,
    workspace_relay: &crate::relay::ScopedWorkspaceRelay,
) -> Result<(), String> {
    kura_host::managed_agents::start_managed_agent_process(
        &app.as_host(),
        record,
        runtimes,
        owner_hex,
        workspace_relay,
    )
}

pub fn sweep_orphaned_agent_processes(app: impl crate::host::AsHost, skip_pids: &[u32]) {
    kura_host::managed_agents::sweep_orphaned_agent_processes(&app.as_host(), skip_pids)
}

pub fn current_instance_id(app: impl crate::host::AsHost) -> String {
    kura_host::managed_agents::current_instance_id(&app.as_host())
}

pub fn stop_managed_agent_workspace_pair(
    app: impl crate::host::AsHost,
    record: &mut ManagedAgentRecord,
    runtimes: &mut HashMap<ManagedAgentRuntimeKey, ManagedAgentPairRuntime>,
) -> Result<(), String> {
    kura_host::managed_agents::stop_managed_agent_workspace_pair(&app.as_host(), record, runtimes)
}

pub fn stop_managed_agent_process(
    app: impl crate::host::AsHost,
    record: &mut ManagedAgentRecord,
    runtimes: &mut HashMap<ManagedAgentRuntimeKey, ManagedAgentPairRuntime>,
) -> Result<(), String> {
    kura_host::managed_agents::stop_managed_agent_process(&app.as_host(), record, runtimes)
}

pub fn start_managed_agent_runtime_pair_lazy(
    pubkey: String,
    relay_url: String,
    app: impl crate::host::AsHost,
) -> Result<ManagedAgentRuntimeStatus, String> {
    kura_host::managed_agents::runtime_commands::start_managed_agent_runtime_pair_lazy(
        pubkey,
        relay_url,
        app.as_host(),
    )
}

pub fn managed_agents_base_dir(app: impl crate::host::AsHost) -> Result<PathBuf, String> {
    kura_host::managed_agents::managed_agents_base_dir(&app.as_host())
}

pub fn managed_agents_store_path(app: impl crate::host::AsHost) -> Result<PathBuf, String> {
    kura_host::managed_agents::managed_agents_store_path(&app.as_host())
}

pub fn latest_managed_agent_log_path(
    app: impl crate::host::AsHost,
    pubkey: &str,
) -> Result<PathBuf, String> {
    kura_host::managed_agents::latest_managed_agent_log_path(&app.as_host(), pubkey)
}

pub fn load_managed_agents(
    app: impl crate::host::AsHost,
) -> Result<Vec<ManagedAgentRecord>, String> {
    kura_host::managed_agents::load_managed_agents(&app.as_host())
}

pub fn load_agent_definitions(
    app: impl crate::host::AsHost,
) -> Result<Vec<ManagedAgentRecord>, String> {
    kura_host::managed_agents::load_agent_definitions(&app.as_host())
}

pub fn save_managed_agents(
    app: impl crate::host::AsHost,
    records: &[ManagedAgentRecord],
) -> Result<(), String> {
    kura_host::managed_agents::save_managed_agents(&app.as_host(), records)
}

pub fn migrate_agent_keys_to_dev_service(app: impl crate::host::AsHost) {
    kura_host::managed_agents::migrate_agent_keys_to_dev_service(&app.as_host())
}

pub fn teams_store_path(app: impl crate::host::AsHost) -> Result<PathBuf, String> {
    kura_host::managed_agents::teams_store_path(&app.as_host())
}

pub fn load_teams(app: impl crate::host::AsHost) -> Result<Vec<TeamRecord>, String> {
    kura_host::managed_agents::load_teams(&app.as_host())
}

pub fn save_teams(app: impl crate::host::AsHost, records: &[TeamRecord]) -> Result<(), String> {
    kura_host::managed_agents::save_teams(&app.as_host(), records)
}

pub fn delete_team_with_cascade(
    app: impl crate::host::AsHost,
    team_id: &str,
) -> Result<Vec<String>, String> {
    kura_host::managed_agents::delete_team_with_cascade(&app.as_host(), team_id)
}

// ── Tauri command wrappers ───────────────────────────────────────────────────
//
// Signatures (names, parameter names, order and types) are unchanged from
// before the backend moved, so `shared/api/*.ts` and every `invokeTauri` call
// site are unaffected.

#[tauri::command]
pub fn put_managed_agent_runtime_lifecycle(
    outer_pubkey: String,
    payload: ManagedAgentRuntimeLifecycleObserverPayload,
    app: AppHandle,
) -> Result<ManagedAgentRuntimeStatus, String> {
    kura_host::managed_agents::runtime_commands::put_managed_agent_runtime_lifecycle(
        outer_pubkey,
        payload,
        app.as_host(),
    )
}

#[tauri::command]
pub async fn list_managed_agent_runtimes(
    app: AppHandle,
) -> Result<Vec<ManagedAgentRuntimeStatus>, String> {
    kura_host::managed_agents::runtime_commands::list_managed_agent_runtimes(app.as_host()).await
}

#[tauri::command]
pub fn start_managed_agent_runtime(
    pubkey: String,
    relay_url: String,
    app: AppHandle,
) -> Result<ManagedAgentRuntimeStatus, String> {
    kura_host::managed_agents::runtime_commands::start_managed_agent_runtime(
        pubkey,
        relay_url,
        app.as_host(),
    )
}

#[tauri::command]
pub fn stop_managed_agent_runtime(
    pubkey: String,
    relay_url: String,
    app: AppHandle,
) -> Result<ManagedAgentRuntimeStatus, String> {
    kura_host::managed_agents::runtime_commands::stop_managed_agent_runtime(
        pubkey,
        relay_url,
        app.as_host(),
    )
}

#[tauri::command]
pub fn restart_managed_agent_runtime(
    pubkey: String,
    relay_url: String,
    app: AppHandle,
) -> Result<ManagedAgentRuntimeStatus, String> {
    kura_host::managed_agents::runtime_commands::restart_managed_agent_runtime(
        pubkey,
        relay_url,
        app.as_host(),
    )
}

#[tauri::command]
pub async fn reconcile_managed_agent_runtimes(
    communities: Vec<ManagedAgentCommunityTarget>,
    app: AppHandle,
) -> Result<Vec<ManagedAgentRuntimeStatus>, String> {
    kura_host::managed_agents::runtime_commands::reconcile_managed_agent_runtimes(
        communities,
        app.as_host(),
    )
    .await
}

// ── Module-qualified shims ───────────────────────────────────────────────────
//
// Call sites that reach these through their module path (rather than the
// re-export at this level) need the same `AsHost` treatment.

pub mod persona_events {
    use crate::host::AsHost;
    pub use kura_host::managed_agents::persona_events::*;

    pub fn active_pending_event(
        app: impl AsHost,
        state: &kura_host::app_state::AppState,
        kind: u32,
        d_tag: &str,
    ) -> Result<bool, String> {
        kura_host::managed_agents::persona_events::active_pending_event(
            &app.as_host(),
            state,
            kind,
            d_tag,
        )
    }

    pub async fn flush_active_pending_events(
        app: impl AsHost,
        state: &kura_host::app_state::AppState,
    ) -> Result<u32, String> {
        kura_host::managed_agents::persona_events::flush_active_pending_events(
            &app.as_host(),
            state,
        )
        .await
    }
}

pub mod retention {
    use crate::host::AsHost;
    pub use kura_host::managed_agents::retention::*;

    pub fn active_retention_scope(
        app: impl AsHost,
        state: &kura_host::app_state::AppState,
    ) -> Result<RetentionScope, String> {
        kura_host::managed_agents::retention::active_retention_scope(&app.as_host(), state)
    }

    pub fn arrival_retention_scope(
        app: impl AsHost,
        state: &kura_host::app_state::AppState,
        arrival_relay_url: &str,
    ) -> Result<Option<RetentionScope>, String> {
        kura_host::managed_agents::retention::arrival_retention_scope(
            &app.as_host(),
            state,
            arrival_relay_url,
        )
    }
}

pub mod reconcile {
    use crate::host::AsHost;
    pub use kura_host::managed_agents::reconcile::*;

    pub fn reconcile_agents_to_events(
        app: impl AsHost,
        keys: &nostr::Keys,
        db_path: &std::path::Path,
    ) {
        kura_host::managed_agents::reconcile::reconcile_agents_to_events(
            &app.as_host(),
            keys,
            db_path,
        )
    }
}

pub mod storage {
    use crate::host::AsHost;
    pub use kura_host::managed_agents::storage::*;

    pub fn install_log_path(
        app: impl AsHost,
        runtime_id: &str,
    ) -> Result<std::path::PathBuf, String> {
        kura_host::managed_agents::storage::install_log_path(&app.as_host(), runtime_id)
    }

    pub fn managed_agents_store_path(app: impl AsHost) -> Result<std::path::PathBuf, String> {
        kura_host::managed_agents::storage::managed_agents_store_path(&app.as_host())
    }
}

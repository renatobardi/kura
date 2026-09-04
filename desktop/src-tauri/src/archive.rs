//! Thin desktop adapter over [`kura_host::archive`].
//!
//! The local-save archive (SQLite store, pipeline, retention, relay sync) lives
//! in `kura-host`. This module re-exports it and keeps the `#[tauri::command]`
//! entry points here, converting Tauri's `State`/`AppHandle` into the plain
//! references and [`kura_host::HostHandle`] the backend takes.
//!
//! Command names, parameter names, parameter order and return types are
//! byte-identical to before the move, so the frontend needs no change.

use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::host::AsHost;
use crate::native_relay_client::NativeRelayClient;

pub use kura_host::archive::*;

/// Warm the archive DB init barrier on a background task. See
/// [`kura_host::archive::spawn_warm_init`].
pub fn spawn_warm_init(app: impl AsHost) {
    kura_host::archive::spawn_warm_init(app.as_host())
}

#[tauri::command]
pub async fn archive_events(
    state: State<'_, AppState>,
    candidates: Vec<ArchiveCandidate>,
) -> Result<ArchiveBatchResult, String> {
    kura_host::archive::archive_events(&state, candidates).await
}

#[tauri::command]
pub async fn create_save_subscription(
    state: State<'_, AppState>,
    sync_state: State<'_, sync::ArchiveSyncState>,
    scope_type: ScopeType,
    scope_value: String,
    kinds: Vec<u32>,
) -> Result<(), String> {
    kura_host::archive::create_save_subscription(
        &state,
        &sync_state,
        scope_type,
        scope_value,
        kinds,
    )
    .await
}

#[tauri::command]
pub async fn merge_save_subscription_kinds(
    state: State<'_, AppState>,
    sync_state: State<'_, sync::ArchiveSyncState>,
    kind: u32,
) -> Result<(), String> {
    kura_host::archive::merge_save_subscription_kinds(&state, &sync_state, kind).await
}

#[tauri::command]
pub async fn remove_save_subscription_kind(
    state: State<'_, AppState>,
    sync_state: State<'_, sync::ArchiveSyncState>,
    kind: u32,
) -> Result<(), String> {
    kura_host::archive::remove_save_subscription_kind(&state, &sync_state, kind).await
}

#[tauri::command]
pub async fn list_save_subscriptions(
    state: State<'_, AppState>,
) -> Result<Vec<store::SaveSubscription>, String> {
    kura_host::archive::list_save_subscriptions(&state).await
}

#[tauri::command]
pub async fn delete_save_subscription(
    state: State<'_, AppState>,
    sync_state: State<'_, sync::ArchiveSyncState>,
    scope_type: ScopeType,
    scope_value: String,
) -> Result<bool, String> {
    kura_host::archive::delete_save_subscription(&state, &sync_state, scope_type, scope_value).await
}

#[tauri::command]
pub async fn read_archived_observer_events_for_channel(
    state: State<'_, AppState>,
    channel_id: String,
    before_created_at: Option<i64>,
    before_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<String>, String> {
    kura_host::archive::read_archived_observer_events_for_channel(
        &state,
        channel_id,
        before_created_at,
        before_id,
        limit,
    )
    .await
}

#[tauri::command]
pub async fn index_observer_channel_id(
    state: State<'_, AppState>,
    entries: Vec<ObserverChannelIndexEntry>,
) -> Result<(), String> {
    kura_host::archive::index_observer_channel_id(&state, entries).await
}

#[tauri::command]
pub async fn read_unindexed_observer_rows(
    state: State<'_, AppState>,
) -> Result<Vec<RawObserverRow>, String> {
    kura_host::archive::read_unindexed_observer_rows(&state).await
}

#[tauri::command]
pub async fn read_archived_events(
    state: State<'_, AppState>,
    scope_type: ScopeType,
    scope_value: String,
    kinds: Option<Vec<i64>>,
    before_created_at: Option<i64>,
    before_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<String>, String> {
    kura_host::archive::read_archived_events(
        &state,
        scope_type,
        scope_value,
        kinds,
        before_created_at,
        before_id,
        limit,
    )
    .await
}

#[tauri::command]
pub async fn get_agent_usage_series(
    state: State<'_, AppState>,
    request: agent_usage::AgentUsageSeriesRequest,
) -> Result<agent_usage::AgentUsageSeries, String> {
    kura_host::archive::get_agent_usage_series(&state, request).await
}

#[tauri::command]
pub async fn get_observer_retention_days(state: State<'_, AppState>) -> Result<i64, String> {
    kura_host::archive::get_observer_retention_days(&state).await
}

#[tauri::command]
pub async fn set_observer_retention_days(
    state: State<'_, AppState>,
    days: i64,
) -> Result<(), String> {
    kura_host::archive::set_observer_retention_days(&state, days).await
}

#[tauri::command]
pub async fn archive_size_stats(
    state: State<'_, AppState>,
) -> Result<retention::ArchiveSizeStats, String> {
    kura_host::archive::archive_size_stats(&state).await
}

#[tauri::command]
pub async fn announce_archive_sync_epoch(
    sync_state: State<'_, sync::ArchiveSyncState>,
) -> Result<u64, String> {
    kura_host::archive::sync::announce_archive_sync_epoch(&sync_state).await
}

#[tauri::command]
pub async fn start_archive_sync(
    app: AppHandle,
    state: State<'_, AppState>,
    sync_state: State<'_, sync::ArchiveSyncState>,
    relay_client: State<'_, NativeRelayClient>,
    epoch: u64,
    lease: u64,
) -> Result<(), String> {
    kura_host::archive::sync::start_archive_sync(
        app.as_host(),
        &state,
        &sync_state,
        &relay_client,
        epoch,
        lease,
    )
    .await
}

#[tauri::command]
pub async fn stop_archive_sync(
    sync_state: State<'_, sync::ArchiveSyncState>,
    epoch: u64,
    lease: u64,
) -> Result<(), String> {
    kura_host::archive::sync::stop_archive_sync(&sync_state, epoch, lease).await
}

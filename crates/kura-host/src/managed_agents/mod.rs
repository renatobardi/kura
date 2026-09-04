pub mod access_policy;
pub mod agent_env;
pub mod agent_events;
pub mod agent_snapshot;
pub mod agent_snapshot_envelope;
pub mod team_snapshot;
pub use access_policy::{owner_only, owner_only_access_build, projected_access_with_policy};
pub use agent_env::{
    baked_build_env, build_kura_agent_provider_defaults, discovery_env_with_baked_floor,
};
pub mod backend;
pub mod claude_config;
pub mod config_bridge;
pub mod custom_harnesses;
pub mod definition_validation;
pub mod discovery;
pub mod effective_config;
pub mod env_vars;
pub mod git_bash;
pub mod global_config;
pub mod managed_node_paths;
pub mod nest;
pub mod parallelism;
pub mod persona_avatars;
pub mod persona_events;
pub mod personas;
#[cfg(windows)]
mod process_lifecycle;
pub mod readiness;
pub mod reconcile;
pub mod relay_mesh;
pub mod repos;
pub mod restore;
pub mod retention;
pub mod runtime;
pub mod runtime_commands;
pub mod runtime_types;
pub mod snapshot_avatar;
pub mod spawn_snapshot;
pub mod storage;
pub mod team_catalog;
pub mod team_events;
pub mod team_repair;
pub use team_repair::team_persona_key;
pub mod teams;
pub mod types;

// Shared guard for tests that mutate or read process-global PATH.
#[cfg(any(test, feature = "test-support"))]
static PATH_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(any(test, feature = "test-support"))]
pub fn lock_path_mutex() -> std::sync::MutexGuard<'static, ()> {
    PATH_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

pub use backend::*;
pub use definition_validation::{
    validate_agent_definition_text, validate_managed_agent_definition_text, validate_visible_text,
};
pub use discovery::*;
pub use env_vars::*;
#[cfg(windows)]
pub use git_bash::git_bash_available;
pub use git_bash::{discover_git_bash, GitBashPrerequisite};
pub use global_config::{
    load_global_agent_config, resolve_effective_model_provider, save_global_agent_config,
    validate_global_config, GlobalAgentConfig,
};
pub use managed_node_paths::*;
pub use nest::*;
pub use parallelism::{acp_agents_value, effective_parallelism, harness_max_parallelism};
pub use personas::*;
#[cfg(windows)]
pub use process_lifecycle::*;
pub use readiness::{
    agent_readiness, resolve_effective_agent_env, resolve_effective_harness_descriptor,
    AgentReadiness, Requirement,
};
pub use relay_mesh::*;
pub use repos::{
    effective_repos_dir, ensure_repos_symlink, resolve_repos_at_boot, validate_repos_dir,
    write_persisted_repos_dir,
};
pub use restore::*;
pub use runtime::*;
pub use runtime_commands::*;
pub use runtime_types::*;
pub use storage::*;
pub use teams::*;
pub use types::*;

#[cfg(test)]
pub use teams::delete_catalog_team_at;

/// Returns the Kura nest directory (`~/.kura`) if it exists as a real
/// directory (not a symlink), falling back to the user's home directory.
///
/// Used as the default working directory for spawned agent processes.
/// `ensure_nest()` must be called during app setup before this is first
/// invoked, so that `~/.kura` exists and gets cached.
///
/// Cached for the process lifetime via `OnceLock`.
/// Returns `None` in sandboxed/containerized environments where `$HOME` is
/// unset or points to a non-existent path; callers fall back to inheriting
/// the parent's CWD.
pub fn default_agent_workdir() -> Option<std::path::PathBuf> {
    use std::sync::OnceLock;
    static WORKDIR: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
    WORKDIR
        .get_or_init(|| {
            // Prefer ~/.kura if it exists (created by ensure_nest()).
            // Reject symlinks to prevent redirect attacks — is_dir()
            // follows symlinks, so check symlink_metadata() first.
            // Fall back to $HOME for resilience.
            nest_dir()
                .filter(|p| is_real_dir(p))
                .or_else(|| dirs::home_dir().filter(|p| p.is_dir()))
        })
        .clone()
}

/// Returns `true` if `path` is a real directory (not a symlink).
fn is_real_dir(path: &std::path::Path) -> bool {
    path.symlink_metadata().map(|m| m.is_dir()).unwrap_or(false)
}

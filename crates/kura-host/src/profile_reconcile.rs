use std::path::Path;

/// Relay profile reconciliation queue.
///
/// An agent renamed while it is stopped still carries the old name in its
/// relay profile: the runtime is not there to publish the update. A rename
/// records the agent here and the next workspace apply republishes the
/// profile, marking the relay it repaired. Entries survive success on purpose
/// — Desktop does not keep its community list in Rust, so a community that is
/// inactive now must still get its one repair when it is next applied.
///
/// The queue lives beside the agent store so it survives restarts. Nothing
/// enqueues today: the Bumble→Pollen rename that wrote it was retired with the
/// bee starter team. The mechanism stays because it is what any future rename
/// of a stopped agent needs, and the consuming commands already speak it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProfileReconcileQueueEntry {
    pub pubkey: String,
    pub expected_name: String,
    /// Canonical relay identities already repaired for this agent.
    #[serde(default)]
    pub reconciled_relays: Vec<String>,
}

pub const PROFILE_RECONCILE_QUEUE_MAX_BYTES: usize = 1024 * 1024;

pub fn profile_reconcile_queue_path(agent_store_path: &Path) -> std::path::PathBuf {
    agent_store_path.with_file_name("profile-reconcile-pending.json")
}

pub fn read_profile_reconcile_queue(
    path: &Path,
) -> Result<Vec<ProfileReconcileQueueEntry>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("failed to inspect profile reconcile queue: {error}"))?;
    if metadata.len() > PROFILE_RECONCILE_QUEUE_MAX_BYTES as u64 {
        return Err("profile reconcile queue exceeds its size limit".to_string());
    }
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read profile reconcile queue: {error}"))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse profile reconcile queue: {error}"))
}

pub fn write_profile_reconcile_queue(
    path: &Path,
    entries: &[ProfileReconcileQueueEntry],
) -> Result<(), String> {
    if entries.is_empty() {
        return match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "failed to remove empty profile reconcile queue {}: {error}",
                path.display()
            )),
        };
    }
    let bytes = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("failed to serialize profile reconcile queue: {error}"))?;
    if bytes.len() > PROFILE_RECONCILE_QUEUE_MAX_BYTES {
        return Err("profile reconcile queue exceeds its size limit".to_string());
    }
    crate::managed_agents::atomic_write_json_restricted(path, &bytes)
}

pub fn profile_reconcile_relay_key(relay_url: &str) -> Result<String, String> {
    kura_core_pkg::relay::normalize_relay_url(relay_url)
        .map_err(|error| format!("invalid profile reconcile relay: {error}"))
}

pub fn record_profile_reconciled(
    entries: &mut [ProfileReconcileQueueEntry],
    pubkey: &str,
    relay_key: String,
) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.pubkey == pubkey) {
        if !entry.reconciled_relays.contains(&relay_key) {
            entry.reconciled_relays.push(relay_key);
            entry.reconciled_relays.sort();
        }
    }
}

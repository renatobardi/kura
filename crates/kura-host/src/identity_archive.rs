//! NIP-IA relay-side reads shared by the desktop's identity-archive commands
//! and the backend nest regeneration.
//!
//! Split out of `commands/identity_archive.rs` when the backend moved into
//! this crate: `managed_agents::nest` needs the archived-pubkey read, and the
//! `#[tauri::command]` half stays in the desktop crate.

use serde::Deserialize;

use crate::app_state::AppState;
use crate::relay::{
    classify_request_error, query_relay_at, relay_api_base_url, relay_http_base_url, relay_ws_url,
    relay_ws_url_with_override, workspace_relay_override,
};

/// A relay target resolved from a single workspace-override read, so a caller
/// that performs several relay requests cannot mix two relays if the workspace
/// override changes mid-flight.
///
/// `relay_ws_url_with_override` and `relay_api_base_url_with_override` each read
/// the override independently; a workspace switch between two such reads can
/// pair one relay's NIP-11 signer with another relay's snapshot query.
/// Capturing both fields from one read — matching those two functions' exact
/// precedence, including the standalone `KURA_RELAY_HTTP` path when no override
/// is set — guarantees the pair is internally consistent.
pub struct RelayTarget {
    /// Relay WebSocket URL (drives the NIP-11 fetch and the rendered footer).
    pub ws_url: String,
    /// Relay HTTP API base URL (drives `/query`).
    pub api_base_url: String,
}

/// Capture the effective relay target once, before any network work.
pub fn capture_relay_target(state: &AppState) -> RelayTarget {
    match workspace_relay_override(state) {
        Some(url) => RelayTarget {
            api_base_url: relay_http_base_url(&url),
            ws_url: url,
        },
        None => RelayTarget {
            ws_url: relay_ws_url(),
            api_base_url: relay_api_base_url(),
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct RelayInformationDocument {
    #[serde(default, rename = "self")]
    pub self_: Option<String>,
}

pub async fn fetch_relay_self(state: &AppState) -> Result<Option<String>, String> {
    fetch_relay_self_at(state, &relay_ws_url_with_override(state)).await
}

/// Like [`fetch_relay_self`] but reads NIP-11 from an explicit relay WS URL
/// instead of re-resolving the workspace override. Used by
/// [`fetch_archived_pubkeys_at`] so the advertised signer and the snapshot
/// query belong to the same captured relay target.
pub async fn fetch_relay_self_at(
    state: &AppState,
    relay_url: &str,
) -> Result<Option<String>, String> {
    let http_url = relay_http_base_url(relay_url);
    let response = state
        .http_client
        .get(&http_url)
        .header("Accept", "application/nostr+json")
        .send()
        .await
        .map_err(|e| classify_request_error(&e))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let doc = response
        .json::<RelayInformationDocument>()
        .await
        .map_err(|_| "relay returned malformed NIP-11 document".to_string())?;

    let Some(relay_self) = doc.self_.map(|value| value.to_ascii_lowercase()) else {
        return Ok(None);
    };

    if relay_self.len() == 64 && relay_self.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(Some(relay_self))
    } else {
        Ok(None)
    }
}

pub fn archived_pubkeys_from_snapshot(snapshot: &nostr::Event) -> Vec<String> {
    snapshot
        .tags
        .iter()
        .filter_map(|t| {
            let slice = t.as_slice();
            if slice.first().map(String::as_str) == Some("p") && slice.len() >= 2 {
                let pk = slice[1].to_ascii_lowercase();
                if pk.len() == 64 && pk.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(pk);
                }
            }
            None
        })
        .collect()
}

/// Read the relay's latest valid `kind:13535` archive snapshot as lowercase
/// hex pubkeys. Shared by the `list_archived_identities` command (frontend
/// flair) and the backend nest regen (excluding archived agents from
/// `AGENTS.md`).
///
/// Per NIP-IA §Client Behavior and §Snapshot and Delta Consistency, only a
/// snapshot signed by the relay identity advertised in NIP-11 `self` can affect
/// archive state. Every failure path — no stable `self`, no snapshot, a bad
/// signature or wrong author, or a query error — **fails open** with an empty
/// set rather than trusting unauthenticated relay-authoritative state.
pub async fn fetch_archived_pubkeys(state: &AppState) -> Vec<String> {
    fetch_archived_pubkeys_at(state, &capture_relay_target(state)).await
}

/// Like [`fetch_archived_pubkeys`] but resolves both the NIP-11 signer and the
/// snapshot query against one captured [`RelayTarget`] instead of re-reading
/// the workspace override for each. This keeps a regeneration's advertised
/// signer and its snapshot query on the same relay even if the workspace
/// override changes between the two awaits.
pub async fn fetch_archived_pubkeys_at(state: &AppState, target: &RelayTarget) -> Vec<String> {
    let Ok(Some(relay_self)) = fetch_relay_self_at(state, &target.ws_url).await else {
        return vec![];
    };

    let query = query_relay_at(
        state,
        &target.api_base_url,
        &[serde_json::json!({
            "authors": [relay_self.clone()],
            "kinds": [13535],
            "limit": 1,
        })],
    )
    .await;
    let Ok(events) = query else {
        return vec![];
    };

    let Some(snapshot) = events.into_iter().next() else {
        return vec![];
    };

    // Defense-in-depth: the filter should already restrict author, but the
    // client must still reject malformed or wrongly signed relay state.
    if !snapshot.verify_id() || !snapshot.verify_signature() {
        return vec![];
    }
    if !snapshot.pubkey.to_hex().eq_ignore_ascii_case(&relay_self) {
        return vec![];
    }

    archived_pubkeys_from_snapshot(&snapshot)
}

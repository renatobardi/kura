//! `kurad` — the headless Kura daemon.
//!
//! A plain Rust binary, with no Tauri and no UI, that runs the *same*
//! `kura-host` backend the desktop app runs. Its only job in this phase is to
//! keep the configured managed agents alive and let you inspect them:
//!
//! ```text
//! kurad run    --data-dir <path> [--relay <url>] [--dev]
//! kurad status --data-dir <path>
//! ```
//!
//! Deliberately absent (later phases): the JSON-RPC/WebSocket API, the web
//! console, any HTTP listener, and any service-manager integration. `kurad` is
//! a foreground process you run in a terminal or under a supervisor.
//!
//! It also does not *create* agents. The agent set is whatever
//! `<data-dir>/agents/managed-agents.json` already holds — the same store the
//! desktop writes when the user adds an agent through the UI.

mod host;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use kura_host::app_state::{build_app_state, resolve_persisted_identity};
use kura_host::host::HostHandle;
use kura_host::managed_agents::runtime_commands::{
    list_managed_agent_runtimes, start_managed_agent_runtime, stop_managed_agent_runtime,
};
use kura_host::managed_agents::{
    load_managed_agents, process_is_running, read_all_agent_runtime_receipts,
};
use tracing_subscriber::EnvFilter;

use crate::host::HeadlessHost;

#[derive(Debug, Parser)]
#[command(
    name = "kurad",
    version,
    about = "Headless Kura daemon: keeps managed agents running without a UI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start every configured managed agent and stay in the foreground.
    Run(RunArgs),
    /// Print the managed-agent runtime status as JSON and exit.
    Status(StatusArgs),
}

/// Flags shared by every subcommand: they all need a host over the same tree.
#[derive(Debug, Args)]
struct HostArgs {
    /// Application data directory (identity key, agent store, logs).
    /// Defaults to the platform data dir plus `kura`.
    #[arg(long, env = "KURA_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Mark this as a development instance (`Host::is_dev`).
    #[arg(long)]
    dev: bool,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[command(flatten)]
    host: HostArgs,

    /// Relay URL to run every agent against. Overrides the per-agent
    /// `relay_url` in the store; without it each agent uses its own.
    #[arg(long, env = "KURA_RELAY_URL")]
    relay: Option<String>,
}

#[derive(Debug, Args)]
struct StatusArgs {
    #[command(flatten)]
    host: HostArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("kurad=info")),
        )
        .with_writer(std::io::stderr)
        .compact()
        .init();

    match Cli::parse().command {
        Command::Run(args) => run(args).await,
        Command::Status(args) => status(args).await,
    }
}

/// Resolve the data directory: the flag wins, else the platform default.
fn resolve_data_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = explicit {
        return Ok(dir);
    }
    // macOS and Linux are the platforms this phase targets; `dirs` gives
    // `~/Library/Application Support` and `$XDG_DATA_HOME` respectively.
    let base = dirs::data_dir()
        .context("could not resolve a platform data dir; pass --data-dir explicitly")?;
    Ok(base.join("kura"))
}

/// Build the daemon's host: data dir, `AppState`, identity.
///
/// Identity resolution is the desktop's own, unchanged:
/// `build_app_state` honours `KURA_PRIVATE_KEY`, and `resolve_persisted_identity`
/// then walks keyring → `<data-dir>/identity.key` → generate-and-save. A
/// failure there is logged and tolerated: a headless box booting for the first
/// time with no identity anywhere is a legitimate state, and
/// `build_app_state` has already seeded an ephemeral key, so refusing to boot
/// would be strictly worse than running degraded.
fn build_host(args: &HostArgs) -> Result<HostHandle> {
    let data_dir = resolve_data_dir(args.data_dir.clone())?;
    let state = build_app_state();
    let host = HeadlessHost::new(data_dir.clone(), state, args.dev)
        .map_err(anyhow::Error::msg)?
        .into_handle();

    if let Err(error) = resolve_persisted_identity(&host, host.state()) {
        tracing::warn!(%error, "identity resolution failed; continuing on the boot key");
    }
    let pubkey = host
        .state()
        .keys
        .lock()
        .map(|keys| keys.public_key().to_hex())
        .unwrap_or_else(|_| "<poisoned>".into());
    tracing::info!(
        data_dir = %data_dir.display(),
        instance_id = %host.instance_id(),
        %pubkey,
        "kurad host ready"
    );
    Ok(host)
}

/// The relay this agent should run against: `--relay` if given, else the
/// agent's own stored `relay_url`. Mirrors the store's shape instead of
/// inventing a workspace-relay concept the daemon has no UI to configure.
fn agent_relay_url(override_url: Option<&str>, record_relay: &str) -> Option<String> {
    let candidate = override_url.unwrap_or(record_relay).trim();
    (!candidate.is_empty()).then(|| candidate.to_string())
}

async fn run(args: RunArgs) -> Result<()> {
    let host = build_host(&args.host)?;
    let records = load_managed_agents(&host).map_err(anyhow::Error::msg)?;
    if records.is_empty() {
        tracing::warn!("no managed agents configured; kurad has nothing to start");
    }

    // Started pairs, so shutdown stops exactly what this process spawned.
    let mut started: Vec<(String, String)> = Vec::new();
    for record in &records {
        let Some(relay_url) = agent_relay_url(args.relay.as_deref(), &record.relay_url) else {
            tracing::warn!(
                agent = %record.name,
                pubkey = %record.pubkey,
                "no relay URL for this agent; pass --relay or set relay_url in the store"
            );
            continue;
        };
        match start_managed_agent_runtime(record.pubkey.clone(), relay_url.clone(), host.clone()) {
            Ok(status) => {
                tracing::info!(
                    agent = %record.name,
                    pubkey = %status.pubkey,
                    relay_url = %status.relay_url,
                    pid = ?status.pid,
                    lifecycle = ?status.lifecycle,
                    "started managed agent"
                );
                started.push((status.pubkey, status.relay_url));
            }
            Err(error) => tracing::error!(
                agent = %record.name,
                pubkey = %record.pubkey,
                %relay_url,
                %error,
                "failed to start managed agent"
            ),
        }
    }

    println!(
        "kurad running ({} agent(s) started), Ctrl+C to stop",
        started.len()
    );
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for Ctrl+C")?;
    println!("kurad shutting down");

    // Best effort: a failed stop is logged, never fatal — the remaining agents
    // still deserve their turn at a clean teardown.
    for (pubkey, relay_url) in started {
        match stop_managed_agent_runtime(pubkey.clone(), relay_url.clone(), host.clone()) {
            Ok(_) => tracing::info!(%pubkey, %relay_url, "stopped managed agent"),
            Err(error) => tracing::error!(%pubkey, %relay_url, %error, "failed to stop agent"),
        }
    }
    Ok(())
}

async fn status(args: StatusArgs) -> Result<()> {
    let host = build_host(&args.host)?;

    // Three views, because no single one is the honest answer:
    //   - `agents`: what the store configures (this process starts nothing).
    //   - `runtimes`: the live runtimes *this* process tracks — always empty
    //     for a bare `status`, but it is the same call the desktop makes.
    //   - `receipts`: what some other `kurad run` left on disk, with a
    //     liveness check, so status across processes is not silently blind.
    let records = load_managed_agents(&host).map_err(anyhow::Error::msg)?;
    let agents: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "pubkey": record.pubkey,
                "name": record.name,
                "relayUrl": record.relay_url,
                "backend": record.backend,
                "acpCommand": record.acp_command,
                "lastStartedAt": record.last_started_at,
                "lastStoppedAt": record.last_stopped_at,
            })
        })
        .collect();

    let runtimes = list_managed_agent_runtimes(host.clone())
        .await
        .map_err(anyhow::Error::msg)?;

    let receipts: Vec<_> = read_all_agent_runtime_receipts(&host)
        .into_iter()
        .map(|(_, receipt)| {
            serde_json::json!({
                "pubkey": receipt.key.pubkey,
                "relayUrl": receipt.key.relay_url,
                "pid": receipt.pid,
                "instanceId": receipt.desktop_instance_id,
                "startedAt": receipt.started_at,
                "running": process_is_running(receipt.pid),
            })
        })
        .collect();

    let report = serde_json::json!({
        "dataDir": host.app_data_dir().map_err(anyhow::Error::msg)?,
        "instanceId": host.instance_id(),
        "agents": agents,
        "runtimes": runtimes,
        "receipts": receipts,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_run_and_status() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn relay_override_wins_over_the_stored_url() {
        assert_eq!(
            agent_relay_url(Some("wss://cli.example"), "wss://stored.example").as_deref(),
            Some("wss://cli.example")
        );
    }

    #[test]
    fn stored_relay_is_the_fallback_and_blank_means_skip() {
        assert_eq!(
            agent_relay_url(None, "wss://stored.example").as_deref(),
            Some("wss://stored.example")
        );
        assert_eq!(agent_relay_url(None, "   "), None);
        assert_eq!(agent_relay_url(Some("  "), "wss://stored.example"), None);
    }

    #[test]
    fn explicit_data_dir_wins_over_the_platform_default() {
        let dir = PathBuf::from("/tmp/kurad-explicit");
        assert_eq!(resolve_data_dir(Some(dir.clone())).unwrap(), dir);
    }
}

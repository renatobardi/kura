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
//!
//! `run` boots agents through `restore_managed_agents_on_launch`, the exact
//! function the desktop calls at launch. That buys the orphan sweep for free: a
//! `kurad` killed with SIGKILL leaves its `kura-acp` children running, and the
//! next `kurad run` reconciles them (adopting what is still tracked, reaping
//! what is stale) instead of blindly spawning a second competing child. It also
//! means `kurad` honours the store's `start_on_app_launch` flag, exactly like
//! the desktop does.

mod host;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use kura_host::app_state::{
    build_app_state, remove_identity_from_keyring, resolve_persisted_identity,
};
use kura_host::headless_identity::{
    self, forget_autounlock, resolve_headless_identity, write_autounlock_file,
    write_ncryptsec_file, TerminalPassphraseSource,
};
use kura_host::host::HostHandle;
use kura_host::identity_lock::lock_keys_to_ncryptsec;
use kura_host::identity_storage::IdentityStorage;
use kura_host::managed_agents::runtime_commands::{
    list_managed_agent_runtimes, stop_managed_agent_runtime,
};
use kura_host::managed_agents::{
    backfill_persona_snapshots, load_managed_agents, process_is_running,
    read_all_agent_runtime_receipts, restore_managed_agents_on_launch,
    ManagedAgentRuntimeLifecycle, NoRestoreHooks,
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
    /// Manage the locked (NIP-49 encrypted) owner identity.
    Identity(IdentityArgs),
}

#[derive(Debug, Args)]
struct IdentityArgs {
    #[command(subcommand)]
    command: IdentityCommand,
}

#[derive(Debug, Subcommand)]
enum IdentityCommand {
    /// Encrypt the current identity to `identity.ncryptsec` under a
    /// passphrase, then remove the plaintext copy.
    Lock(IdentityLockArgs),
    /// Verify a passphrase against `identity.ncryptsec`, optionally storing
    /// it for unattended future boots.
    Unlock(IdentityUnlockArgs),
    /// Delete `identity.autounlock`, disabling unattended unlock.
    ForgetAutounlock(HostArgs),
    /// Print whether the identity is locked and/or set to auto-unlock.
    Status(HostArgs),
}

#[derive(Debug, Args)]
struct IdentityLockArgs {
    #[command(flatten)]
    host: HostArgs,

    /// Read the passphrase as a single line from stdin instead of an
    /// interactive no-echo prompt (for scripting). The caller is responsible
    /// for keeping the passphrase out of shell history and process listings.
    #[arg(long)]
    passphrase_stdin: bool,

    /// Overwrite an existing `identity.ncryptsec`.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct IdentityUnlockArgs {
    #[command(flatten)]
    host: HostArgs,

    /// Store the verified passphrase in `identity.autounlock` (0600) so
    /// future `kurad run`/`kurad status` decrypt without a prompt. This does
    /// NOT unlock a currently running daemon — there is no supervisor/IPC in
    /// this phase — it only verifies the passphrase and, with this flag,
    /// enables unattended future boots.
    #[arg(long)]
    remember: bool,
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

    /// Relay URL to run every agent against, set as the workspace relay
    /// override. Without it the relay comes from `KURA_RELAY_URL`, then the
    /// build-time default. The per-agent `relay_url` in the store is *not* a
    /// fallback: `kura-host` resolves every agent to the workspace relay
    /// (agents-everywhere), and `kurad` does not diverge from that.
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
        Command::Identity(args) => identity(args).await,
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

/// Construct the host shell (data dir, `AppState`, `HeadlessHost`) without
/// resolving identity. Callers finish setup with whichever identity path
/// fits their command.
fn new_headless_host(args: &HostArgs) -> Result<HostHandle> {
    let data_dir = resolve_data_dir(args.data_dir.clone())?;
    let state = build_app_state();
    Ok(HeadlessHost::new(data_dir, state, args.dev)
        .map_err(anyhow::Error::msg)?
        .into_handle())
}

fn log_ready(host: &HostHandle) {
    let pubkey = host
        .state()
        .keys
        .lock()
        .map(|keys| keys.public_key().to_hex())
        .unwrap_or_else(|_| "<poisoned>".into());
    tracing::info!(
        data_dir = %host.app_data_dir().map(|d| d.display().to_string()).unwrap_or_default(),
        instance_id = %host.instance_id(),
        %pubkey,
        storage = %host.state().identity_storage().as_str(),
        "kurad host ready"
    );
}

/// Build the daemon's host for `run`/`status`: data dir, `AppState`, identity.
///
/// If `<data-dir>/identity.ncryptsec` exists, the identity is locked: it is
/// resolved through [`resolve_headless_identity`] (env var →
/// `identity.autounlock` → interactive prompt), and a failure there — wrong
/// or missing passphrase, no TTY and no autounlock file, a corrupted file —
/// is a **hard error that aborts startup**. Continuing on an ephemeral key
/// while the real, locked identity sits right there unrecognized would run
/// agents under the wrong pubkey without any indication something is wrong;
/// that is worse than refusing to start.
///
/// Otherwise, behavior is unchanged from before this feature: `build_app_state`
/// honours `KURA_PRIVATE_KEY`, and `resolve_persisted_identity` walks keyring →
/// `<data-dir>/identity.key` → generate-and-save. A failure there is logged
/// and tolerated — a headless box booting for the first time with no identity
/// anywhere is a legitimate state, and `build_app_state` has already seeded an
/// ephemeral key, so refusing to boot would be strictly worse than running
/// degraded.
fn build_host(args: &HostArgs) -> Result<HostHandle> {
    let host = new_headless_host(args)?;
    let data_dir = host.app_data_dir().map_err(anyhow::Error::msg)?;

    if headless_identity::ncryptsec_path(&data_dir).exists() {
        let resolved = resolve_headless_identity(&data_dir, &TerminalPassphraseSource)
            .map_err(|error| anyhow::anyhow!("locked identity: {error}"))?;
        let mut active_keys = host
            .state()
            .keys
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        *active_keys = resolved.keys;
        host.state().set_identity_storage(resolved.storage);
        drop(active_keys);
    } else if let Err(error) = resolve_persisted_identity(&host, host.state()) {
        tracing::warn!(%error, "identity resolution failed; continuing on the boot key");
    }

    log_ready(&host);
    Ok(host)
}

/// Normalise a `--relay` value: blank means "not given".
fn relay_override(value: Option<&str>) -> Option<String> {
    let candidate = value?.trim();
    (!candidate.is_empty()).then(|| candidate.to_string())
}

/// Pairs `stop_managed_agent_runtime` should be called on at shutdown: every
/// runtime this process still tracks as live. `Stopped` rows are the ones
/// `list_managed_agent_runtimes` reports for a child it just reaped, so
/// stopping them again would be pure noise.
fn live_pairs(
    runtimes: &[kura_host::managed_agents::ManagedAgentRuntimeStatus],
) -> Vec<(String, String)> {
    runtimes
        .iter()
        .filter(|status| !matches!(status.lifecycle, ManagedAgentRuntimeLifecycle::Stopped))
        .map(|status| (status.pubkey.clone(), status.relay_url.clone()))
        .collect()
}

async fn run(args: RunArgs) -> Result<()> {
    let host = build_host(&args.host)?;

    // `--relay` is applied as the *workspace* relay override, the single knob
    // `kura-host` resolves every agent's relay through
    // (`relay_ws_url_with_override` → `effective_agent_relay_url`). Mutating
    // each record's `relay_url` would have been a no-op: that field is parsed
    // and persisted but deliberately ignored at spawn time.
    if let Some(relay) = relay_override(args.relay.as_deref()) {
        match host.state().relay_url_override.lock() {
            Ok(mut guard) => {
                tracing::info!(relay_url = %relay, "workspace relay override from --relay");
                *guard = Some(relay);
            }
            Err(error) => {
                tracing::error!(%error, "could not set the relay override; using default")
            }
        }
    }

    let records = load_managed_agents(&host).map_err(anyhow::Error::msg)?;
    if records.is_empty() {
        tracing::warn!("no managed agents configured; kurad has nothing to start");
    }

    // The desktop runs this before its own restore, and so must `kurad`: a data
    // dir copied from an older desktop install can hold agents with a
    // `persona_id` but no snapshot, and `spawn_agent_child` would boot them from
    // an empty config. `kurad` never *creates* an agent, so it cannot produce
    // that shape itself — but it can very well inherit it. Best effort: a
    // backfill failure is not a reason to refuse to start the healthy agents.
    if let Err(error) = backfill_persona_snapshots(&host) {
        tracing::warn!(%error, "persona snapshot backfill failed; continuing");
    }

    // The desktop's launch path, verbatim: it syncs tracked runtimes against
    // on-disk PID receipts, sweeps orphans left by a previous process, and
    // spawns only what is not already alive — which is exactly the crash
    // recovery a bare `start_managed_agent_runtime` loop never did.
    restore_managed_agents_on_launch(&host, &host.state().shutdown_started, &NoRestoreHooks)
        .await
        .map_err(anyhow::Error::msg)?;

    // Restore returns no per-agent list, so ask the runtime map what is live.
    let started = live_pairs(
        &list_managed_agent_runtimes(host.clone())
            .await
            .map_err(anyhow::Error::msg)?,
    );
    for (pubkey, relay_url) in &started {
        tracing::info!(%pubkey, %relay_url, "managed agent running");
    }

    println!(
        "kurad running ({} agent(s) live), Ctrl+C to stop",
        started.len()
    );
    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for Ctrl+C")?;
    println!("kurad shutting down");
    host.state()
        .shutdown_started
        .store(true, std::sync::atomic::Ordering::SeqCst);

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

    let identity_storage = host.state().identity_storage();
    let report = serde_json::json!({
        "dataDir": host.app_data_dir().map_err(anyhow::Error::msg)?,
        "instanceId": host.instance_id(),
        "agents": agents,
        "runtimes": runtimes,
        "receipts": receipts,
        "identityStorage": identity_storage.as_str(),
        "identityLocked": identity_storage == IdentityStorage::LockedLocalFile,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn identity(args: IdentityArgs) -> Result<()> {
    match args.command {
        IdentityCommand::Lock(args) => identity_lock(args),
        IdentityCommand::Unlock(args) => identity_unlock(args),
        IdentityCommand::ForgetAutounlock(args) => identity_forget_autounlock(&args),
        IdentityCommand::Status(args) => identity_status(&args),
    }
}

/// Resolve the *current* (unlocked) identity the normal way — the same
/// keyring/plaintext-file/env resolution `run`/`status` use when there is no
/// `identity.ncryptsec` yet. `identity lock` needs this, not the locked path,
/// since its whole job is to take an unlocked identity and lock it.
fn build_host_for_unlocked_identity(args: &HostArgs) -> Result<HostHandle> {
    let host = new_headless_host(args)?;
    resolve_persisted_identity(&host, host.state()).map_err(|error| {
        anyhow::anyhow!("could not resolve the current (unlocked) identity: {error}")
    })?;
    Ok(host)
}

/// Read exactly one line from stdin as a passphrase (for `--passphrase-stdin`
/// scripting). Trims the trailing newline only.
fn read_passphrase_line_from_stdin() -> Result<String> {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read passphrase from stdin")?;
    let trimmed = line.trim_end_matches(['\n', '\r']);
    if trimmed.is_empty() {
        bail!("stdin passphrase was empty");
    }
    Ok(trimmed.to_string())
}

fn identity_lock(args: IdentityLockArgs) -> Result<()> {
    let host = build_host_for_unlocked_identity(&args.host)?;
    let data_dir = host.app_data_dir().map_err(anyhow::Error::msg)?;

    if headless_identity::ncryptsec_path(&data_dir).exists() && !args.force {
        bail!(
            "{} already exists; pass --force to overwrite it",
            headless_identity::ncryptsec_path(&data_dir).display()
        );
    }

    let passphrase = if args.passphrase_stdin {
        read_passphrase_line_from_stdin()?
    } else {
        let entered = rpassword::prompt_password("New passphrase to lock the Kura identity: ")
            .context("read passphrase")?;
        let confirm =
            rpassword::prompt_password("Confirm passphrase: ").context("read passphrase")?;
        if entered != confirm {
            bail!("passphrases did not match; identity was not locked");
        }
        entered
    };

    let keys = host
        .state()
        .keys
        .lock()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .clone();
    let pubkey = keys.public_key().to_hex();

    let ncryptsec = lock_keys_to_ncryptsec(&keys, &passphrase).map_err(|e| anyhow::anyhow!(e))?;
    write_ncryptsec_file(&data_dir, &ncryptsec, args.force).map_err(|e| anyhow::anyhow!(e))?;

    // Remove the plaintext copy so the encrypted file becomes the only source
    // of truth. A previously-keyring-resident identity is also removed from
    // the keyring, reusing the existing delete plumbing.
    let legacy_key_path = data_dir.join("identity.key");
    let mut removed_plaintext_file = false;
    if legacy_key_path.exists() {
        std::fs::remove_file(&legacy_key_path)
            .with_context(|| format!("remove {}", legacy_key_path.display()))?;
        removed_plaintext_file = true;
    }
    match remove_identity_from_keyring() {
        Ok(()) => {}
        Err(error) => {
            eprintln!(
                "warning: could not remove any keyring-resident copy of the identity ({error}); \
                 if it was stored in the OS keyring, remove it manually"
            );
        }
    }

    println!(
        "identity locked ({}): {}",
        pubkey,
        headless_identity::ncryptsec_path(&data_dir).display()
    );
    if removed_plaintext_file {
        println!("removed plaintext {}", legacy_key_path.display());
    }
    println!(
        "note: any keyring-resident copy was also targeted for removal; \
         see any warning above if that could not be confirmed"
    );
    Ok(())
}

fn identity_unlock(args: IdentityUnlockArgs) -> Result<()> {
    let host = new_headless_host(&args.host)?;
    let data_dir = host.app_data_dir().map_err(anyhow::Error::msg)?;

    if !headless_identity::ncryptsec_path(&data_dir).exists() {
        bail!(
            "{} does not exist; nothing to unlock",
            headless_identity::ncryptsec_path(&data_dir).display()
        );
    }

    let passphrase = rpassword::prompt_password("Passphrase to unlock the Kura identity: ")
        .context("read passphrase")?;

    let ncryptsec = std::fs::read_to_string(headless_identity::ncryptsec_path(&data_dir))
        .context("read identity.ncryptsec")?;
    let keys = kura_host::identity_lock::unlock_ncryptsec(ncryptsec.trim(), &passphrase)
        .map_err(|error| anyhow::anyhow!("passphrase verification failed: {error}"))?;
    let pubkey = keys.public_key().to_hex();

    if args.remember {
        write_autounlock_file(&data_dir, &passphrase).map_err(|e| anyhow::anyhow!(e))?;
        println!(
            "passphrase verified for {pubkey} and stored in {} (0600).",
            headless_identity::autounlock_path(&data_dir).display()
        );
        println!(
            "warning: this stores the passphrase in plaintext at 0600 on this host — the same \
             threat model as the old unlocked identity.key. This is the deliberate opt-in \
             tradeoff for unattended restarts (e.g. via systemd)."
        );
    } else {
        println!(
            "passphrase verified for {pubkey}. This only verifies the passphrase is correct — \
             it does not unlock a running daemon (there is no supervisor/IPC yet) and stores \
             nothing. Pass --remember to enable unattended future boots."
        );
    }
    Ok(())
}

fn identity_forget_autounlock(args: &HostArgs) -> Result<()> {
    let host = new_headless_host(args)?;
    let data_dir = host.app_data_dir().map_err(anyhow::Error::msg)?;
    let removed = forget_autounlock(&data_dir).map_err(|e| anyhow::anyhow!(e))?;
    if removed {
        println!(
            "removed {} — unattended unlock is now disabled",
            headless_identity::autounlock_path(&data_dir).display()
        );
    } else {
        println!("no autounlock file was present; nothing to do");
    }
    Ok(())
}

fn identity_status(args: &HostArgs) -> Result<()> {
    let host = new_headless_host(args)?;
    let data_dir = host.app_data_dir().map_err(anyhow::Error::msg)?;
    let report = serde_json::json!({
        "locked": headless_identity::ncryptsec_path(&data_dir).exists(),
        "autoUnlock": headless_identity::autounlock_path(&data_dir).exists(),
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

    fn status(
        pubkey: &str,
        relay_url: &str,
        lifecycle: ManagedAgentRuntimeLifecycle,
    ) -> kura_host::managed_agents::ManagedAgentRuntimeStatus {
        kura_host::managed_agents::ManagedAgentRuntimeStatus {
            pubkey: pubkey.into(),
            relay_url: relay_url.into(),
            requested_relay_url: None,
            local_setup: true,
            lifecycle,
            pid: None,
            error: None,
            log_path: None,
        }
    }

    #[test]
    fn relay_override_is_trimmed_and_blank_means_unset() {
        assert_eq!(
            relay_override(Some(" wss://cli.example ")).as_deref(),
            Some("wss://cli.example")
        );
        assert_eq!(relay_override(Some("   ")), None);
        assert_eq!(relay_override(None), None);
    }

    #[test]
    fn shutdown_targets_every_runtime_that_is_not_stopped() {
        let runtimes = vec![
            status("aa", "wss://a.example", ManagedAgentRuntimeLifecycle::Ready),
            status(
                "bb",
                "wss://b.example",
                ManagedAgentRuntimeLifecycle::Starting,
            ),
            status(
                "cc",
                "wss://c.example",
                ManagedAgentRuntimeLifecycle::Stopped,
            ),
            status(
                "dd",
                "wss://d.example",
                ManagedAgentRuntimeLifecycle::Failed,
            ),
        ];
        assert_eq!(
            live_pairs(&runtimes),
            vec![
                ("aa".to_string(), "wss://a.example".to_string()),
                ("bb".to_string(), "wss://b.example".to_string()),
                ("dd".to_string(), "wss://d.example".to_string()),
            ]
        );
    }

    #[test]
    fn explicit_data_dir_wins_over_the_platform_default() {
        let dir = PathBuf::from("/tmp/kurad-explicit");
        assert_eq!(resolve_data_dir(Some(dir.clone())).unwrap(), dir);
    }
}

// Compile-time build capabilities for `kura-host`.
//
// `cargo:rustc-env` emitted by a build script reaches ONLY the crate that
// printed it — never that crate's dependencies. The readers of these
// `KURA_DESKTOP_BUILD_*` values (`relay.rs`, `managed_agents::agent_env`,
// `managed_agents::access_policy`) now live in this crate, so this crate must
// bake them itself; `desktop/src-tauri/build.rs` doing it only ever configured
// the `kura-desktop` crate. Keep the emitted names exactly as they are: they
// are the `option_env!` keys the runtime reads.
//
// The `KURA_BUILD_*` names on the input side stay identical to the desktop
// build script's, so release packaging sets one set of variables for both
// crates.

// Same source of truth the runtime filters with, so a baked build env cannot
// carry a reserved key the runtime believes it already rejected.
include!("src/managed_agents/reserved_env_keys.rs");

use base64::Engine as _;

fn main() {
    // Without these, cargo will NOT rebuild this crate when the capability
    // flips between two builds — it would reuse a stale artifact baked with
    // the previous value.
    println!("cargo:rerun-if-env-changed=KURA_RELAY_URL");
    println!("cargo:rerun-if-env-changed=KURA_RELAY_HTTP");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_KURA_AGENT_PROVIDER");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_KURA_AGENT_MODEL");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_AGENT_ENV");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_AGENT_ACCESS_OWNER_ONLY");

    // Explicit owner-only agent-access capability. Release packaging sets this
    // presence-only marker; OSS/custom builds leave agent access configurable.
    if std::env::var("KURA_BUILD_AGENT_ACCESS_OWNER_ONLY").is_ok() {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_AGENT_ACCESS_OWNER_ONLY=1");
    }

    if let Ok(relay_url) = std::env::var("KURA_RELAY_URL") {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_RELAY_URL={relay_url}");
    }

    if let Ok(relay_http) = std::env::var("KURA_RELAY_HTTP") {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_RELAY_HTTP={relay_http}");
    }

    if let Ok(provider) = std::env::var("KURA_BUILD_KURA_AGENT_PROVIDER") {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_KURA_AGENT_PROVIDER={provider}");
    }

    if let Ok(model) = std::env::var("KURA_BUILD_KURA_AGENT_MODEL") {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_KURA_AGENT_MODEL={model}");
    }

    // Generic KEY=VALUE pairs to inject into every spawned agent process.
    // Newline-delimited; each line must be non-empty and contain exactly one
    // `=` separator with a non-empty key.  OSS builds leave this unset.
    // The validated value is base64-encoded before emitting so the single-line
    // Cargo build-script output carries all pairs (Cargo output is line-oriented;
    // a raw multiline value would be silently truncated to the first line).
    if let Ok(raw) = std::env::var("KURA_BUILD_AGENT_ENV") {
        for (line_no, line) in raw.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let eq = line.find('=').unwrap_or_else(|| {
                panic!(
                    "KURA_BUILD_AGENT_ENV line {}: missing '=' separator in {:?}",
                    line_no + 1,
                    line
                )
            });
            let key = &line[..eq];
            if key.is_empty() {
                panic!(
                    "KURA_BUILD_AGENT_ENV line {}: key must not be empty in {:?}",
                    line_no + 1,
                    line
                );
            }
            // The baked env is written into every spawned agent's environment
            // LAST (see `managed_agents/runtime.rs`), after Kura sets the
            // access gates and identity vars. A baked reserved key would
            // therefore silently override the gate the UI promises, so reject
            // it at build time instead of shipping a binary that bypasses its
            // own enforcement.
            if is_reserved_env_key(key) {
                panic!(
                    "KURA_BUILD_AGENT_ENV line {}: `{}` is reserved by Kura and cannot be baked \
                     into a build (it would override Kura's own identity/access env)",
                    line_no + 1,
                    key
                );
            }
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_AGENT_ENV={encoded}");
    }
}

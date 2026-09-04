// Shared schema, included from the same source the runtime command parses with,
// so the build-time validation below and the runtime parse cannot drift.
include!("src/commands/reconnect_hook_config.rs");

fn main() {
    // Only the capabilities this crate itself reads via `option_env!` belong
    // here. `cargo:rustc-env` never reaches a dependency, so the relay URL/HTTP,
    // agent provider/model, baked agent env and owner-only access markers — all
    // read from `kura-host` — are baked by `crates/kura-host/build.rs` instead.
    println!("cargo:rerun-if-env-changed=KURA_UPDATER_PUBLIC_KEY");
    println!("cargo:rerun-if-env-changed=KURA_UPDATER_ENDPOINT");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_RELAY_RECONNECT_CMD");
    println!("cargo:rerun-if-env-changed=KURA_BUILD_AUTO_CONNECT_DEFAULT_RELAY");
    println!("cargo:rustc-check-cfg=cfg(kura_updater_enabled)");

    if let Ok(val) = std::env::var("KURA_BUILD_RELAY_RECONNECT_CMD") {
        let parsed: serde_json::Value = serde_json::from_str(&val)
            .unwrap_or_else(|e| panic!("KURA_BUILD_RELAY_RECONNECT_CMD is not valid JSON: {e}"));
        serde_json::from_value::<ReconnectHookConfig>(parsed).unwrap_or_else(|e| {
            panic!("KURA_BUILD_RELAY_RECONNECT_CMD doesn't match ReconnectHookConfig: {e}")
        });
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_RELAY_RECONNECT_CMD={val}");
    }

    // Presence-only release capability: internal desktop builds opt into
    // auto-connecting their configured default relay on first run. OSS builds
    // leave this unset and retain explicit community selection.
    if std::env::var("KURA_BUILD_AUTO_CONNECT_DEFAULT_RELAY").is_ok() {
        println!("cargo:rustc-env=KURA_DESKTOP_BUILD_AUTO_CONNECT_DEFAULT_RELAY=1");
    }

    let updater_public_key = std::env::var("KURA_UPDATER_PUBLIC_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let updater_endpoint = std::env::var("KURA_UPDATER_ENDPOINT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if updater_public_key.is_some() && updater_endpoint.is_some() {
        println!("cargo:rustc-cfg=kura_updater_enabled");
    }

    // Cargo test executables get no embedded Windows manifest (tauri_build
    // attaches one to bin targets only), so the loader binds comctl32 v5, which
    // lacks TaskDialogIndirect (statically imported via tauri-plugin-dialog/rfd)
    // and debug test exes die at load with STATUS_ENTRYPOINT_NOT_FOUND. Declaring
    // the Common Controls v6 dependency makes link.exe emit a side-by-side
    // <exe>.manifest that the loader honors for manifest-less executables;
    // binaries with an embedded manifest (the real app) ignore it.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }

    tauri_build::try_build(
        tauri_build::Attributes::new().plugin(
            "websocket",
            tauri_build::InlinedPlugin::new()
                .commands(&["connect", "send", "disconnect", "disconnect_all"])
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to build Tauri application");
}

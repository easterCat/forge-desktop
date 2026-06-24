// End-to-end smoke test for the install_plugin dispatch logic.
//
// The previous install_plugin only handled the `local` source kind, so 80%+
// of marketplace plugins (those declared as `git-subdir` or `url`) failed
// with "Plugin 'X' not found in marketplace source 'Y'". This test verifies
// that the `url` source kind actually clones the external repo into
// forge/plugins/cache/<source>/<name>/.
//
// We use the real `superpowers` plugin from `obra/superpowers.git` — the
// same plugin the FEAT-009 bug report was about — as the test fixture so
// the regression is impossible to miss in the future.
//
// Note: this test shells out to `git clone` and so requires network access
// and a working `git` binary. It's marked `#[ignore]` by default to keep
// CI fast; run with `cargo test --test install_superpowers -- --ignored`.

use std::path::PathBuf;

use forge_desktop_lib::models::plugin_marketplace::{
    MarketplacePlugin, PluginInstallSource,
};

/// Build a temp FORGE_HOME for this test run so we never touch the
/// real `~/.forge` used by the desktop app.
fn test_forge_home() -> PathBuf {
    if let Ok(v) = std::env::var("FORGE_HOME_TEST") {
        return PathBuf::from(v);
    }
    let dir = std::env::temp_dir().join(format!("forge-test-install-{}", std::process::id()));
    std::env::set_var("FORGE_HOME_TEST", &dir);
    // Mirror onto the env var the lib actually reads.
    std::env::set_var("FORGE_HOME", &dir);
    dir
}

#[test]
#[ignore = "requires network + git; run with --ignored"]
fn install_superpowers_url_source_creates_cache_dir() {
    let forge_home = test_forge_home();
    std::fs::create_dir_all(&forge_home).expect("create FORGE_HOME");
    std::env::set_var("FORGE_HOME", &forge_home);

    let plugin = MarketplacePlugin {
        id: "superpowers".to_string(),
        source_id: "anthropics".to_string(),
        name: "superpowers".to_string(),
        description: "Superpowers teaches Claude brainstorming, …".to_string(),
        long_description: None,
        author: None,
        version: None,
        latest_version: None,
        has_update: Some(false),
        categories: vec!["development".to_string()],
        tags: vec![],
        install_command: None,
        install_path: None,
        repository: Some("https://github.com/obra/superpowers.git".to_string()),
        homepage: Some("https://github.com/obra/superpowers.git".to_string()),
        license: None,
        stars: None,
        downloads: None,
        last_updated: None,
        is_installed: false,
        disabled: false,
        install_source: Some(PluginInstallSource {
            kind: "url".to_string(),
            url: "https://github.com/obra/superpowers.git".to_string(),
            path: String::new(),
            r#ref: String::new(),
            sha: String::new(),
        }),
        cli_tool_key: None,
        cli_tool_keys: vec![],
    };

    let plugins_dir = forge_home.join("plugins");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    // First install: should clone from obra/superpowers.git.
    let result = rt.block_on(forge_desktop_lib::services::plugin_marketplace::install_plugin(
        &plugin,
        "https://github.com/anthropics/claude-plugins-official",
        &plugins_dir,
    ))
    .expect("install_plugin returned Err");

    assert!(
        result.success,
        "install_plugin should succeed, got error: {:?}",
        result.error
    );
    let installed_path = result.path.expect("install_plugin returned no path");
    let dest = PathBuf::from(&installed_path);
    assert!(
        dest.exists(),
        "expected plugin dir at {}",
        dest.display()
    );
    // Sanity: a real superpowers repo has a SKILL.md or skills/ directory.
    let has_payload =
        dest.join("SKILL.md").exists() || dest.join("skills").exists();
    assert!(
        has_payload,
        "installed superpowers dir looks empty: {}",
        dest.display()
    );

    // Idempotency: second invocation should also succeed and report the
    // same path without re-cloning.
    let second = rt.block_on(forge_desktop_lib::services::plugin_marketplace::install_plugin(
        &plugin,
        "https://github.com/anthropics/claude-plugins-official",
        &plugins_dir,
    ))
    .expect("second install_plugin returned Err");
    assert!(
        second.success,
        "second install should be idempotent, got error: {:?}",
        second.error
    );
    assert_eq!(second.path.as_deref(), Some(installed_path.as_str()));

    // Cleanup: remove the cloned cache so the next run starts fresh.
    // Disabled by default; set KEEP_TEST_ARTIFACTS=1 to inspect the
    // installed plugin dir for debugging.
    if std::env::var("KEEP_TEST_ARTIFACTS").is_err() {
        let _ = std::fs::remove_dir_all(forge_home.join("plugins").join("cache"));
        let _ = std::fs::remove_file(forge_home.join("plugins").join("marketplace.json"));
        let _ = std::fs::remove_file(forge_home.join("plugins").join("installed_plugins.json"));
        let _ = std::fs::remove_file(forge_home.join("plugins").join("sync_records.json"));
        let _ = std::fs::remove_dir_all(forge_home.join("plugins").join("marketplace"));
        let _ = std::fs::remove_dir_all(forge_home.join(".tmp"));
    } else {
        eprintln!(
            "KEEP_TEST_ARTIFACTS=1; inspect cache at: {}",
            forge_home.join("plugins").join("cache").display()
        );
    }
}

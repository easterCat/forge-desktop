// Smoke test for BUG-005: Plugin Details dialog fails for plugins installed
// under `plugins/cache/<source>/<name>/` when `marketplace.json` is missing.
//
// Reproduces the user's reported scenario: the `superpowers` plugin lives at
// `~/.forge/plugins/cache/anthropics/superpowers/.claude-plugin/plugin.json`
// and there is no top-level `marketplace.json`. Before the fix, the Details
// dialog would error with:
//   "Plugin 'superpowers' not found in source 'anthropics' of marketplace.json"
//
// This test points `FORGE_HOME` at the user's real home so we exercise the
// exact on-disk layout. The test is gated behind an env var so it doesn't
// fire by accident on developer machines that don't have superpowers
// installed. Run with:
//   FORGE_HOME=/Users/rhino/.forge cargo test --test plugin_details_cache_layout -- --ignored --nocapture

use forge_desktop_lib::services::plugin_capabilities as caps;

#[test]
#[ignore = "requires the user's real FORGE_HOME; run with FORGE_HOME=~/.forge and --ignored"]
fn get_capabilities_finds_plugin_under_cache_layout() {
    // The test env var is set by the runner. We just sanity-check that the
    // function finds the plugin under the cache/ layout.
    let result = caps::parse_local_capabilities("anthropics", "superpowers");
    assert!(
        result.is_ok(),
        "parse_local_capabilities should find superpowers under cache/ — got: {:?}",
        result.err()
    );

    let capabilities = result.unwrap();
    assert_eq!(capabilities.name, "superpowers");
    assert_eq!(
        capabilities.version.as_deref(),
        Some("5.1.0"),
        "expected plugin.json version 5.1.0 to be read from cache layout"
    );
    assert!(
        !capabilities.skills.is_empty(),
        "expected at least one skill to be discovered from the cache layout"
    );
    println!(
        "OK — parsed {} skills, {} commands, {} hooks from cache layout",
        capabilities.capabilities.skills,
        capabilities.capabilities.commands,
        capabilities.capabilities.hooks
    );
}

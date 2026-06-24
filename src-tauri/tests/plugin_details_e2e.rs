// End-to-end smoke test for the public `get_plugin_capabilities` command
// against the user's reported scenario. Verifies that the Details dialog
// (which invokes this exact function) returns a populated `PluginCapabilities`
// for the `superpowers` plugin installed under `cache/anthropics/`.
//
// Run with:  FORGE_HOME=/Users/rhino/.forge cargo test --test plugin_details_e2e -- --ignored --nocapture

use forge_desktop_lib::commands::plugin_capabilities::get_plugin_capabilities;

#[tokio::test]
#[ignore = "requires the user's real FORGE_HOME; run with FORGE_HOME=~/.forge and --ignored"]
async fn details_dialog_e2e_for_superpowers_under_cache_layout() {
    // This is the exact call the PluginDetailsDialog.vue component makes.
    let result = get_plugin_capabilities(
        "anthropics".to_string(),
        "superpowers".to_string(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Details dialog should load for superpowers — got: {:?}",
        result.err()
    );

    let capabilities = result.unwrap();
    assert_eq!(capabilities.name, "superpowers");
    assert_eq!(capabilities.version.as_deref(), Some("5.1.0"));
    let skills_count = capabilities.capabilities.skills;
    assert!(
        skills_count >= 10,
        "expected at least 10 skills (superpowers ships 14), got {}",
        skills_count
    );
    println!(
        "OK — Details dialog would render: name={}, version={}, skills={}, commands={}, hooks={}",
        capabilities.name,
        capabilities.version.unwrap_or_default(),
        skills_count,
        capabilities.capabilities.commands,
        capabilities.capabilities.hooks,
    );
}

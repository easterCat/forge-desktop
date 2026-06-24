pub mod file_service;
pub mod software_scanner;
pub mod software_installer;
pub mod cli_tools;
pub mod skill_marketplace;
pub mod plugin_marketplace;
pub mod plugin_capabilities;
pub mod mcp_bridge;
pub mod mcp_protocol;
pub mod plugin_repo_sync;

pub use file_service::*;
pub use software_scanner::*;
pub use software_installer::*;
pub use cli_tools::*;
pub use skill_marketplace::{
    get_preset_sources as get_skill_preset_sources,
    fetch_skills_from_api,
    fetch_github_skills,
    install_skill,
    sync_skill_to_target,
    validate_sync_target,
    get_local_skills,
};
pub use plugin_marketplace::{
    get_preset_sources as get_plugin_preset_sources,
    fetch_plugins_from_source,
    install_plugin,
    uninstall_plugin,
    update_plugin,
    add_source,
    get_installed_plugins,
    InstalledRegistry,
    InstalledEntry,
    read_installed_registry,
    write_installed_registry,
    update_plugin_inuse,
    sweep_inuse,
};

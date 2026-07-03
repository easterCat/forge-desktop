use tauri::Manager;

pub mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod utils;

pub use db::Database;

// Re-export InstalledRegistry/InstalledEntry from services layer
pub use services::{InstalledRegistry, InstalledEntry};

// Re-export AllAgents types
pub use services::allagents_service::{
    AllAgentsConfig, AllAgentsService, WorkspaceConfig,
};

// Re-export model types with explicit paths to avoid ambiguity
pub use models::{
    Software, Plugin, Skill, McpService, Rule, BackupRecord, ConfigTemplate, FileEntry, Agent,
    SkillSource, MarketplaceSkill, PaginatedSkills, SyncTarget, SyncConfig,
    SyncProgress, InstallResult, SyncResult, FetchSkillsRequest,
    MCPSource, EnvVar, MCPServer, PaginatedMCPServers, MCPSyncTarget,
    MCPInstallProgress, MCPSyncProgress, MCPInstallResult, MCPSyncResult,
    MCPSyncConfig, FetchServersRequest,
    PluginSource, MarketplacePlugin, PluginInstallResult, PluginUpdateResult,
    SourceStatus, SourceInstallResult, SourceInstallProgress,
    // Aliased types to avoid name collisions
    SkillInstallProgress,
    PluginInstallProgress,
    PluginCapabilities,
    HookExecutionResult,
    ValidationIssue,
    ValidationReport,
    ValidationCapabilityCounts,
    McpProbeResult,
};
// Anthropic skills types
pub use commands::anthropic_skills::{
    AnthropicSkill, InstallProgress, InstallVerification,
};
pub use models::skill_marketplace::GitHubContent as SkillGitHubContent;
pub use models::skill_marketplace::GitHubReadme as SkillGitHubReadme;
pub use models::plugin_marketplace::GitHubContent as PluginGitHubContent;
pub use models::plugin_marketplace::GitHubReadme as PluginGitHubReadme;

// Explicitly re-export command types to avoid glob re-export ambiguity
pub use commands::{
    software::{get_software_list, detect_software, get_software_by_id, get_software_by_key,
              sync_software, install_software, uninstall_software, update_software,
              UpdateCheckResult},
    plugin::{get_plugins, install_plugin, uninstall_plugin, update_plugin, toggle_plugin},
    plugin_marketplace::{get_marketplace_sources, fetch_marketplace_plugins, get_marketplace_plugins,
                         get_marketplace_manifest,
                         install_marketplace_plugin, uninstall_marketplace_plugin, update_marketplace_plugin,
                         add_marketplace_source, is_plugin_installed, set_plugin_disabled,
                         get_marketplace_source_status, install_marketplace_source, install_all_marketplace_sources,
                         get_user_marketplace_sources, add_user_marketplace_source, remove_user_marketplace_source,
                         update_source_repo_type,
                         get_source_notes, save_source_note},
    plugin_capabilities::{get_plugin_capabilities, execute_plugin_hook, validate_plugin_path},
    mcp_bridge::{probe_plugin_mcp},
    installed_registry::{get_installed_registry, update_plugin_inuse_cmd, sweep_inuse_cmd},
    skill::{get_skills, create_skill, update_skill, delete_skill},
    skill_import::{unzip_skill_package, scan_local_skills, import_local_skill,
                   detect_cli_skills_paths, get_default_skills_dir},
    skill_repository::{get_repositories, add_repository, remove_repository, validate_repository,
                       sync_repository, get_repository_skills, sync_all_repositories,
                       update_repository, download_skill_from_repository},
    anthropic_skills::{list_anthropic_skills, install_anthropic_skill, verify_anthropic_skill_install,
                       uninstall_anthropic_skill, get_local_anthropic_skills,
                       list_remote_skills, install_remote_skill, verify_remote_skill_install,
                       uninstall_remote_skill, get_local_remote_skills, get_remote_skill_sources,
                       list_remote_skills_cached_only, CachedSkillsResult},
    skill_marketplace::{get_skill_sources, fetch_marketplace_skills, get_skill_categories,
                        install_marketplace_skill, get_local_marketplace_skills,
                        sync_skill_to_target, get_sync_targets, add_sync_target,
                        remove_sync_target, is_skill_installed, get_skill_details},
    mcp::{get_mcp_services, add_mcp_service, update_mcp_service, delete_mcp_service,
          check_mcp_service_health},
    mcp_marketplace::{get_mcp_sources, fetch_mcp_servers, get_local_mcp_servers,
                      install_mcp_server, sync_mcp_to_target, get_mcp_sync_targets,
                      add_mcp_sync_target, remove_mcp_sync_target},
    rule::{get_rules, create_rule, update_rule, delete_rule, toggle_rule},
    agent::{get_agents, search_agents, create_agent, update_agent, delete_agent,
            import_agents_from_repo, install_agent_to_target, uninstall_agent_from_target,
            get_agents_marketplace_path, has_agents_marketplace,
            ImportResult as AgentImportResult},
    backup::{create_backup, get_backups, restore_backup, delete_backup, get_backup_contents},
    file::{read_file, write_file, list_directory},
    cli_tools::{get_cli_tools, check_cli_tool_status, check_all_cli_tools_status,
                check_all_cli_tools_status_parallel, upgrade_cli_tool},
    settings::{has_github_token, get_github_token_preview, set_github_token, clear_github_token},
    mcp_manager::{get_mcp_service_detail, invoke_mcp_tool, discover_mcp_service,
                  export_mcp_services, import_mcp_services, get_mcp_health_history,
                  get_mcp_groups, create_mcp_group, update_mcp_group, delete_mcp_group,
                  get_mcp_audit_log},
    version_manager::{get_version_list, get_available_versions, install_version, switch_version,
                      set_global_version, remove_version,
                      VersionListResult, VersionOperationResult, VersionInfo, AvailableVersion},
};
// Skills.sh types
pub use commands::skills_sh::{
    SkillsShSkill, SkillsShPage, SkillsShPagination,
    SkillsShCuratedResponse, SkillsShCuratedOwner,
    SkillsShSkillDetail, SkillsShSkillFile,
    SkillsShAuditResponse, SkillsShAuditEntry,
    InstallResult as SkillsShInstallResult,
};

pub struct AppState {
    pub db: Database,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // Initialize file-based logging asynchronously
            let log_dir = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("forge")
                .join("logs");

            // Create log directory synchronously (required for setup)
            if let Err(e) = std::fs::create_dir_all(&log_dir) {
                eprintln!("Failed to create log directory: {}", e);
            }

            let log_file = log_dir.join(format!("app_{}.log", chrono::Local::now().format("%Y%m%d_%H%M%S")));

            // Initialize logger (deferred, won't block startup)
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
            {
                env_logger::Builder::from_default_env()
                    .target(env_logger::Target::Pipe(Box::new(file)))
                    .filter_level(log::LevelFilter::Debug)
                    .format_timestamp_secs()
                    .init();
            } else {
                env_logger::Builder::from_default_env()
                    .filter_level(log::LevelFilter::Debug)
                    .format_timestamp_secs()
                    .init();
            }

            log::info!("Starting Forge...");
            log::info!("Log file: {:?}", log_file);

            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");

            let db_path = app_data_dir.join("forge.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");
            Database::set_global(db);  // moves db into OnceLock for KvStore access
            let db = Database::new(&db_path).expect("Failed to initialize database");

            app.manage(AppState { db });

            log::info!("Forge initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Software commands
            commands::software::get_software_list,
            commands::software::detect_software,
            commands::software::get_software_by_id,
            commands::software::get_software_by_key,
            commands::software::sync_software,
            commands::software::install_software,
            commands::software::uninstall_software,
            commands::software::update_software,
            // CLI Tools commands
            commands::cli_tools::get_cli_tools,
            commands::cli_tools::check_cli_tool_status,
            commands::cli_tools::check_all_cli_tools_status,
            commands::cli_tools::check_all_cli_tools_status_parallel,
            commands::cli_tools::upgrade_cli_tool,
            // Settings commands
            commands::settings::has_github_token,
            commands::settings::get_github_token_preview,
            commands::settings::set_github_token,
            commands::settings::clear_github_token,
            // Plugin commands
            commands::plugin::get_plugins,
            commands::plugin::install_plugin,
            commands::plugin::uninstall_plugin,
            commands::plugin::update_plugin,
            commands::plugin::toggle_plugin,
            // Plugin Marketplace commands
            commands::plugin_marketplace::get_marketplace_sources,
            commands::plugin_marketplace::fetch_marketplace_plugins,
            commands::plugin_marketplace::get_marketplace_plugins,
            commands::plugin_marketplace::get_marketplace_manifest,
            commands::plugin_marketplace::install_marketplace_plugin,
            commands::plugin_marketplace::uninstall_marketplace_plugin,
            commands::plugin_marketplace::update_marketplace_plugin,
            commands::plugin_marketplace::add_marketplace_source,
            commands::plugin_marketplace::is_plugin_installed,
            commands::plugin_marketplace::set_plugin_disabled,
            commands::plugin_marketplace::get_marketplace_source_status,
            commands::plugin_marketplace::install_marketplace_source,
            commands::plugin_marketplace::install_all_marketplace_sources,
            commands::plugin_marketplace::get_user_marketplace_sources,
            commands::plugin_marketplace::add_user_marketplace_source,
            commands::plugin_marketplace::remove_user_marketplace_source,
            commands::plugin_marketplace::update_source_repo_type,
            commands::plugin_marketplace::get_source_notes,
            commands::plugin_marketplace::save_source_note,
            commands::plugin_marketplace::resolve_plugin_version,
            crate::commands::plugin_capabilities::get_plugin_capabilities,
            crate::commands::plugin_capabilities::execute_plugin_hook,
            crate::commands::plugin_capabilities::validate_plugin_path,
            crate::commands::mcp_bridge::probe_plugin_mcp,
            crate::commands::installed_registry::get_installed_registry,
            crate::commands::installed_registry::update_plugin_inuse_cmd,
            crate::commands::installed_registry::sweep_inuse_cmd,
            // Plugin Sync commands
            crate::commands::plugin_sync::sync_plugin_to_cli_tool,
            crate::commands::plugin_sync::unsync_plugin_from_cli_tool,
            crate::commands::plugin_sync::get_plugin_sync_status,
            // Skill commands
            commands::skill::get_skills,
            commands::skill::create_skill,
            commands::skill::update_skill,
            commands::skill::delete_skill,
            // Skill Import commands
            commands::skill_import::unzip_skill_package,
            commands::skill_import::scan_local_skills,
            commands::skill_import::import_local_skill,
            commands::skill_import::detect_cli_skills_paths,
            commands::skill_import::get_default_skills_dir,
            // Skill Repository commands
            commands::skill_repository::get_repositories,
            commands::skill_repository::add_repository,
            commands::skill_repository::remove_repository,
            commands::skill_repository::validate_repository,
            commands::skill_repository::sync_repository,
            commands::skill_repository::get_repository_skills,
            commands::skill_repository::sync_all_repositories,
            commands::skill_repository::update_repository,
            commands::skill_repository::download_skill_from_repository,
            // Anthropic Skills commands
            commands::anthropic_skills::list_anthropic_skills,
            commands::anthropic_skills::install_anthropic_skill,
            commands::anthropic_skills::verify_anthropic_skill_install,
            commands::anthropic_skills::uninstall_anthropic_skill,
            commands::anthropic_skills::get_local_anthropic_skills,
            // Remote Skills commands (source-aware)
            commands::anthropic_skills::get_remote_skill_sources,
            commands::anthropic_skills::list_remote_skills,
            commands::anthropic_skills::list_remote_skills_cached_only,
            commands::anthropic_skills::install_remote_skill,
            commands::anthropic_skills::verify_remote_skill_install,
            commands::anthropic_skills::uninstall_remote_skill,
            commands::anthropic_skills::get_local_remote_skills,
            // Skill Marketplace commands
            commands::skill_marketplace::get_skill_sources,
            commands::skill_marketplace::fetch_marketplace_skills,
            commands::skill_marketplace::get_skill_categories,
            commands::skill_marketplace::install_marketplace_skill,
            commands::skill_marketplace::get_local_marketplace_skills,
            commands::skill_marketplace::sync_skill_to_target,
            commands::skill_marketplace::get_sync_targets,
            commands::skill_marketplace::add_sync_target,
            commands::skill_marketplace::remove_sync_target,
            commands::skill_marketplace::is_skill_installed,
            commands::skill_marketplace::get_skill_details,
            // MCP commands
            commands::mcp::get_mcp_services,
            commands::mcp::add_mcp_service,
            commands::mcp::update_mcp_service,
            commands::mcp::delete_mcp_service,
            commands::mcp::check_mcp_service_health,
            // MCP Marketplace commands
            commands::mcp_marketplace::get_mcp_sources,
            commands::mcp_marketplace::fetch_mcp_servers,
            commands::mcp_marketplace::get_local_mcp_servers,
            commands::mcp_marketplace::install_mcp_server,
            commands::mcp_marketplace::sync_mcp_to_target,
            commands::mcp_marketplace::get_mcp_sync_targets,
            commands::mcp_marketplace::add_mcp_sync_target,
            commands::mcp_marketplace::remove_mcp_sync_target,
            // Rule commands
            commands::rule::get_rules,
            commands::rule::create_rule,
            commands::rule::update_rule,
            commands::rule::delete_rule,
            commands::rule::toggle_rule,
            // Agent commands
            commands::agent::get_agents,
            commands::agent::search_agents,
            commands::agent::create_agent,
            commands::agent::update_agent,
            commands::agent::delete_agent,
            commands::agent::import_agents_from_repo,
            commands::agent::install_agent_to_target,
            commands::agent::uninstall_agent_from_target,
            commands::agent::get_agents_marketplace_path,
            commands::agent::has_agents_marketplace,
            // Backup commands
            commands::backup::create_backup,
            commands::backup::get_backups,
            commands::backup::restore_backup,
            commands::backup::delete_backup,
            commands::backup::get_backup_contents,
            // Data transfer commands
            commands::data_transfer::export_all_data,
            commands::data_transfer::import_all_data,
            // File commands
            commands::file::read_file,
            commands::file::write_file,
            commands::file::list_directory,
            // Skills.sh commands
            commands::skills_sh::fetch_skills_sh_leaderboard,
            commands::skills_sh::search_skills_sh,
            commands::skills_sh::fetch_skills_sh_curated,
            commands::skills_sh::fetch_skills_sh_skill_detail,
            commands::skills_sh::fetch_skills_sh_audit,
            commands::skills_sh::install_skill_via_skills_sh,
            // FEAT-022: MCP Manager commands
            commands::mcp_manager::get_mcp_service_detail,
            commands::mcp_manager::invoke_mcp_tool,
            commands::mcp_manager::discover_mcp_service,
            commands::mcp_manager::export_mcp_services,
            commands::mcp_manager::import_mcp_services,
            commands::mcp_manager::get_mcp_health_history,
            commands::mcp_manager::get_mcp_groups,
            commands::mcp_manager::create_mcp_group,
            commands::mcp_manager::update_mcp_group,
            commands::mcp_manager::delete_mcp_group,
            commands::mcp_manager::get_mcp_audit_log,
            // Version Manager commands
            commands::version_manager::get_version_list,
            commands::version_manager::get_available_versions,
            commands::version_manager::install_version,
            commands::version_manager::switch_version,
            commands::version_manager::set_global_version,
            commands::version_manager::remove_version,
            // AllAgents commands
            commands::allagents_commands::allagents_init,
            commands::allagents_commands::allagents_update,
            commands::allagents_commands::allagents_status,
            commands::allagents_commands::allagents_plugin_install,
            commands::allagents_commands::allagents_plugin_uninstall,
            commands::allagents_commands::allagents_plugin_list,
            commands::allagents_commands::allagents_skill_list,
            commands::allagents_commands::allagents_skill_add,
            commands::allagents_commands::allagents_skill_remove,
            commands::allagents_commands::allagents_mcp_add,
            commands::allagents_commands::allagents_mcp_remove,
            commands::allagents_commands::allagents_mcp_list,
            commands::allagents_commands::allagents_mcp_update,
            commands::allagents_commands::allagents_marketplace_add,
            commands::allagents_commands::allagents_marketplace_remove,
            commands::allagents_commands::allagents_marketplace_list,
            commands::allagents_commands::allagents_generate_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

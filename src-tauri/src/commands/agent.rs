use crate::models::Agent;
use crate::AppState;
use tauri::State;
use std::path::PathBuf;

/// Default marketplace directory for agency-agents-zh
const DEFAULT_MARKETPLACE_DIR: &str = ".forge/agents/marketplace/agency-agents-zh";

#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub imported: u32,
    pub skipped: u32,
}

#[tauri::command]
pub fn get_agents(state: State<AppState>, department: Option<String>) -> Result<Vec<Agent>, String> {
    log::info!("Getting agents, department: {:?}", department);
    match department {
        Some(dept) => state.db.get_agents_by_department(&dept).map_err(|e| e.to_string()),
        None => state.db.get_all_agents().map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn search_agents(state: State<AppState>, query: String) -> Result<Vec<Agent>, String> {
    log::info!("Searching agents: {}", query);
    state.db.search_agents(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_agent(state: State<AppState>, agent: Agent) -> Result<(), String> {
    log::info!("Creating agent: {}", agent.name);
    state.db.upsert_agent(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_agent(state: State<AppState>, agent: Agent) -> Result<(), String> {
    log::info!("Updating agent: {}", agent.name);
    state.db.upsert_agent(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent(state: State<AppState>, agent_id: String) -> Result<(), String> {
    log::info!("Deleting agent: {}", agent_id);
    state.db.delete_agent(&agent_id).map_err(|e| e.to_string())
}

/// Get the default marketplace path for agency-agents-zh
#[tauri::command]
pub fn get_agents_marketplace_path() -> Result<String, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;
    let path = home.join(DEFAULT_MARKETPLACE_DIR);
    Ok(path.to_string_lossy().to_string())
}

/// Check if the default marketplace exists
#[tauri::command]
pub fn has_agents_marketplace() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    home.join(DEFAULT_MARKETPLACE_DIR).exists()
}

/// Import agents from a local clone of agency-agents-zh repository
#[tauri::command]
pub fn import_agents_from_repo(state: State<AppState>, source_dir: String) -> Result<ImportResult, String> {
    log::info!("Importing agents from: {}", source_dir);
    let repo_path = PathBuf::from(&source_dir);
    if !repo_path.exists() {
        return Err(format!("Directory does not exist: {}", source_dir));
    }

    let departments = vec![
        "academic", "design", "engineering", "finance", "game-development",
        "hr", "legal", "marketing", "paid-media", "product",
        "project-management", "sales", "spatial-computing", "specialized",
        "strategy", "supply-chain", "support", "testing",
    ];

    let mut imported = 0u32;
    let mut skipped = 0u32;

    for dept in departments {
        let dept_dir = repo_path.join(dept);
        if !dept_dir.exists() {
            continue;
        }

        let entries = match std::fs::read_dir(&dept_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => { skipped += 1; continue; }
            };

            match parse_agent_file(&content, dept) {
                Some(agent) => {
                    if let Err(e) = state.db.upsert_agent(&agent) {
                        log::warn!("Failed to upsert agent {}: {}", agent.name, e);
                        skipped += 1;
                    } else {
                        imported += 1;
                    }
                }
                None => {
                    skipped += 1;
                }
            }
        }
    }

    log::info!("Import complete: {} imported, {} skipped", imported, skipped);
    Ok(ImportResult { imported, skipped })
}

/// Install an agent's content to a target tool's directory
#[tauri::command]
pub fn install_agent_to_target(
    state: State<AppState>,
    agent_id: String,
    target: String,
) -> Result<String, String> {
    log::info!("Installing agent {} to target: {}", agent_id, target);

    let agents = state.db.get_all_agents().map_err(|e| e.to_string())?;
    let agent = agents.iter().find(|a| a.id == agent_id)
        .ok_or_else(|| format!("Agent not found: {}", agent_id))?;

    let home = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let (install_dir, file_name, file_content) = match target.as_str() {
        "claude-code" => {
            let dir = home.join(".claude/agents");
            let name = format!("{}.md", slugify(&agent.name));
            (dir, name, agent.content.clone())
        }
        "cursor" => {
            let dir = std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(".cursor/rules");
            let name = format!("{}.mdc", slugify(&agent.name));
            let mdc_content = format!("---\nname: {}\ndescription: {}\n---\n\n{}", agent.name, agent.description, agent.content);
            (dir, name, mdc_content)
        }
        "copilot" => {
            let dir = home.join(".github/agents");
            let name = format!("{}.md", slugify(&agent.name));
            (dir, name, agent.content.clone())
        }
        _ => {
            return Err(format!("Unsupported target: {}", target));
        }
    };

    std::fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create directory {:?}: {}", install_dir, e))?;

    let file_path = install_dir.join(&file_name);
    std::fs::write(&file_path, &file_content)
        .map_err(|e| format!("Failed to write file {:?}: {}", file_path, e))?;

    // Update installed_targets in DB
    let mut updated_agent = agent.clone();
    let mut targets: Vec<String> = updated_agent.installed_targets
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    if !targets.contains(&target) {
        targets.push(target);
        updated_agent.installed_targets = Some(serde_json::to_string(&targets).unwrap_or_default());
        updated_agent.updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        state.db.upsert_agent(&updated_agent).map_err(|e| e.to_string())?;
    }

    let path_str = file_path.to_string_lossy().to_string();
    log::info!("Agent installed to: {}", path_str);
    Ok(path_str)
}

/// Uninstall an agent from a target tool's directory
#[tauri::command]
pub fn uninstall_agent_from_target(
    state: State<AppState>,
    agent_id: String,
    target: String,
) -> Result<(), String> {
    log::info!("Uninstalling agent {} from target: {}", agent_id, target);

    let agents = state.db.get_all_agents().map_err(|e| e.to_string())?;
    let agent = agents.iter().find(|a| a.id == agent_id)
        .ok_or_else(|| format!("Agent not found: {}", agent_id))?;

    let home = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;

    let file_path = match target.as_str() {
        "claude-code" => home.join(".claude/agents").join(format!("{}.md", slugify(&agent.name))),
        "cursor" => std::env::current_dir().map_err(|e| e.to_string())?.join(".cursor/rules").join(format!("{}.mdc", slugify(&agent.name))),
        "copilot" => home.join(".github/agents").join(format!("{}.md", slugify(&agent.name))),
        _ => return Err(format!("Unsupported target: {}", target)),
    };

    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove file {:?}: {}", file_path, e))?;
    }

    // Update installed_targets in DB
    let mut updated_agent = agent.clone();
    let mut targets: Vec<String> = updated_agent.installed_targets
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    targets.retain(|t| t != &target);
    updated_agent.installed_targets = Some(serde_json::to_string(&targets).unwrap_or_default());
    updated_agent.updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    state.db.upsert_agent(&updated_agent).map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Helpers ───

fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

/// Parse YAML frontmatter + markdown body from an agency-agents-zh file
fn parse_agent_file(content: &str, department: &str) -> Option<Agent> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let yaml_str = &after_first[..end];
    let body = after_first[end + 3..].trim();

    let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;

    let name = frontmatter.get("name")?.as_str()?.to_string();
    let description = frontmatter.get("description")?.as_str()?.to_string();
    let emoji = frontmatter.get("emoji").and_then(|v| v.as_str()).map(|s| s.to_string());
    let color = frontmatter.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    Some(Agent {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description,
        emoji,
        color,
        department: department.to_string(),
        content: body.to_string(),
        source: "builtin".to_string(),
        tags: None,
        installed_targets: None,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

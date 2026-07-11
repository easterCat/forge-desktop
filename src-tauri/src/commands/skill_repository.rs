use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepository {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "type")]
    pub repo_type: String,
    pub last_sync_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub skill_count: i32,
    pub added_at: String,
    pub skills: Vec<DiscoveredSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub repository_id: String,
    pub repository_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryValidation {
    pub valid: bool,
    pub name: Option<String>,
    pub branch_count: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub skill_count: i32,
    pub message: String,
}

const REPOSITORIES_CONFIG: &str = ".env-manager/repositories.json";

fn get_config_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("forge")
        .join("repositories.json")
}

fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(config_path)
}

/// Load repositories from KvStore (with JSON file fallback)
fn load_repositories() -> Result<Vec<SkillRepository>, String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(db.conn.clone());
        if let Some(data) = kv.get::<serde_json::Value>("skill_repositories") {
            let repos = data.get("repositories")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| serde_json::from_value(item.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();
            return Ok(repos);
        }
        return Ok(vec![]);
    }
    // Fallback
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;
    let repos = data.get("repositories")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    Ok(repos)
}

/// Save repositories to KvStore (with JSON file fallback)
fn save_repositories(repos: &[SkillRepository]) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(db.conn.clone());
        let data = serde_json::json!({
            "version": "1.0",
            "repositories": repos
        });
        return kv.put("skill_repositories", &data);
    }
    // Fallback
    let config_path = ensure_config_dir()
        .map_err(|e| format!("无法创建配置目录: {}", e))?;
    let data = serde_json::json!({
        "version": "1.0",
        "repositories": repos
    });
    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

/// Get all skill repositories
#[tauri::command]
pub fn get_repositories() -> Result<Vec<SkillRepository>, String> {
    log::info!("Getting all skill repositories");
    load_repositories()
}

/// Add a new repository
#[tauri::command]
pub fn add_repository(url: String, name: Option<String>) -> Result<SkillRepository, String> {
    log::info!("Adding repository: {}", url);
    
    // Parse repository type from URL
    let repo_type = if url.contains("github.com") {
        "github"
    } else if url.contains("gitlab.com") {
        "gitlab"
    } else if url.contains("gitee.com") {
        "gitee"
    } else {
        "custom"
    };
    
    // Generate repository name from URL if not provided
    let repo_name = name.unwrap_or_else(|| {
        url.split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git")
            .to_string()
    });
    
    let now = chrono::Utc::now().to_rfc3339();
    
    let repository = SkillRepository {
        id: uuid::Uuid::new_v4().to_string(),
        name: repo_name,
        url: url.clone(),
        repo_type: repo_type.to_string(),
        last_sync_at: None,
        status: "pending".to_string(),
        error_message: None,
        skill_count: 0,
        added_at: now,
        skills: vec![],
    };
    
    // Load existing repos and add new one
    let mut repos = load_repositories()?;
    
    // Check for duplicate
    if repos.iter().any(|r| r.url == url) {
        return Err("该仓库已存在".to_string());
    }
    
    repos.push(repository.clone());
    save_repositories(&repos)?;
    
    log::info!("Repository added: {} ({})", repository.name, repository.id);
    Ok(repository)
}

/// Remove a repository
#[tauri::command]
pub fn remove_repository(repo_id: String) -> Result<(), String> {
    log::info!("Removing repository: {}", repo_id);
    
    let mut repos = load_repositories()?;
    let initial_len = repos.len();
    repos.retain(|r| r.id != repo_id);
    
    if repos.len() == initial_len {
        return Err("仓库不存在".to_string());
    }
    
    save_repositories(&repos)?;
    Ok(())
}

/// Validate a repository URL
#[tauri::command]
pub async fn validate_repository(url: String) -> Result<RepositoryValidation, String> {
    log::info!("Validating repository: {}", url);
    
    let clean_url = url.trim().trim_end_matches(".git").to_string();
    
    // Try git ls-remote to validate
    let output = tokio::process::Command::new("git")
        .args(["ls-remote", "--heads", &clean_url])
        .output()
        .await
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(RepositoryValidation {
            valid: false,
            name: None,
            branch_count: 0,
            error_message: Some(format!("无法连接到仓库: {}", stderr.trim())),
        });
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branch_count = stdout.lines().count() as i32;
    
    let name = clean_url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .to_string();
    
    log::info!("Repository valid: {} ({} branches)", name, branch_count);
    
    Ok(RepositoryValidation {
        valid: true,
        name: Some(name),
        branch_count,
        error_message: None,
    })
}

/// Sync a repository and fetch skills
#[tauri::command]
pub async fn sync_repository(repo_id: String) -> Result<SyncResult, String> {
    log::info!("Syncing repository: {}", repo_id);
    
    let mut repos = load_repositories()?;
    let repo_index = repos.iter().position(|r| r.id == repo_id)
        .ok_or("仓库不存在")?;
    
    // Clone the repo data we need before mutating
    let clean_url = repos[repo_index].url.trim().trim_end_matches(".git").to_string();
    let repo_type = repos[repo_index].repo_type.clone();
    
    // Update status
    repos[repo_index].status = "syncing".to_string();
    repos[repo_index].error_message = None;
    save_repositories(&repos)?;
    
    // Fetch skills from repository
    let skills = if repo_type == "github" {
        fetch_github_skills(&clean_url).await.unwrap_or_default()
    } else {
        fetch_git_skills(&clean_url).await.unwrap_or_default()
    };
    
    // Reload and update
    let mut repos = load_repositories()?;
    if let Some(repo) = repos.iter_mut().find(|r| r.id == repo_id) {
        repo.skills = skills.clone();
        repo.skill_count = skills.len() as i32;
        repo.last_sync_at = Some(chrono::Utc::now().to_rfc3339());
        repo.status = "synced".to_string();
    }
    
    save_repositories(&repos)?;
    
    log::info!("Repository synced: {} ({} skills)", repo_id, skills.len());
    
    Ok(SyncResult {
        success: true,
        skill_count: skills.len() as i32,
        message: format!("同步成功，发现 {} 个技能", skills.len()),
    })
}

/// Fetch skills from GitHub repository using API
async fn fetch_github_skills(url: &str) -> Result<Vec<DiscoveredSkill>, String> {
    // Extract owner and repo from URL
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 2 {
        return Ok(vec![]);
    }
    
    let owner = parts.get(parts.len() - 2).unwrap_or(&"");
    let repo = parts.get(parts.len() - 1).unwrap_or(&"");
    
    let client = reqwest::Client::new();
    let mut all_skills = Vec::new();
    
    // Search in root directory and skills/ subdirectory
    let search_paths = vec!["", "skills"];
    
    for search_path in search_paths {
        let api_url = if search_path.is_empty() {
            format!("https://api.github.com/repos/{}/{}/contents", owner, repo)
        } else {
            format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, search_path)
        };
        
        let response = match client
            .get(&api_url)
            .header("User-Agent", "forge-desktop")
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        
        if !response.status().is_success() {
            continue;
        }
        
        let contents: Vec<serde_json::Value> = match response.json().await {
            Ok(json) => json,
            Err(_) => continue,
        };
        
        for item in &contents {
            // Look for directories with SKILL.md or skill directories
            if item.get("type").and_then(|t| t.as_str()) == Some("dir") {
                let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let path = item.get("path").and_then(|p| p.as_str()).unwrap_or("");
                
                // Check if this directory contains SKILL.md
                let skill_md_url = format!(
                    "https://api.github.com/repos/{}/{}/contents/{}/SKILL.md",
                    owner, repo, path
                );
                
                let has_skill_md = client
                    .get(&skill_md_url)
                    .header("User-Agent", "forge-desktop")
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);
                
                if has_skill_md || name == "skills" || search_path == "skills" {
                    all_skills.push(DiscoveredSkill {
                        name: name.to_string(),
                        path: path.to_string(),
                        description: None,
                        version: Some("1.0.0".to_string()),
                        repository_id: String::new(),
                        repository_name: String::new(),
                    });
                }
            }
        }
    }
    
    // Deduplicate by path
    all_skills.sort_by_key(|s| s.path.clone());
    all_skills.dedup_by(|a, b| a.path == b.path);
    
    Ok(all_skills)
}

/// Fetch skills by cloning git repository
async fn fetch_git_skills(url: &str) -> Result<Vec<DiscoveredSkill>, String> {
    let temp_dir = std::env::temp_dir()
        .join(format!("skill-repo-{}", uuid::Uuid::new_v4()));
    
    // Clone repository
    let output = tokio::process::Command::new("git")
        .args(["clone", "--depth", "1", url, temp_dir.to_str().unwrap()])
        .output()
        .await
        .map_err(|e| format!("克隆失败: {}", e))?;
    
    if !output.status.success() {
        // Cleanup on failure
        fs::remove_dir_all(&temp_dir).ok();
        return Ok(vec![]);
    }
    
    // Scan for skills directory
    let mut skills = vec![];
    
    // Check skills/ directory
    let skills_dir = temp_dir.join("skills");
    if skills_dir.exists() {
        scan_skills_directory(&skills_dir, "", &mut skills);
    }
    
    // Also scan root directory for SKILL.md files
    scan_skills_directory(&temp_dir, "", &mut skills);
    
    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    
    Ok(skills)
}

/// Scan a directory for skill packages
fn scan_skills_directory(
    dir: &PathBuf,
    prefix: &str,
    skills: &mut Vec<DiscoveredSkill>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    let name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let description = fs::read_to_string(&skill_md)
                        .ok()
                        .and_then(|c| {
                            c.lines()
                                .find(|l| l.trim().starts_with("description:"))
                                .map(|l| l.replace("description:", "").trim().to_string())
                        });
                    
                    skills.push(DiscoveredSkill {
                        name,
                        path: format!("{}{}", prefix, path.file_name().unwrap().to_string_lossy()),
                        description,
                        version: Some("1.0.0".to_string()),
                        repository_id: String::new(),
                        repository_name: String::new(),
                    });
                }
            }
        }
    }
}

/// Get skills from a specific repository
#[tauri::command]
pub fn get_repository_skills(repo_id: String) -> Result<Vec<DiscoveredSkill>, String> {
    let repos = load_repositories()?;
    let repo = repos.iter().find(|r| r.id == repo_id)
        .ok_or("仓库不存在")?;
    
    Ok(repo.skills.clone())
}

/// Sync all repositories (for auto-sync on page load)
#[tauri::command]
pub async fn sync_all_repositories() -> Result<Vec<SyncResult>, String> {
    log::info!("Syncing all repositories");
    
    let mut repos = load_repositories()?;
    let mut results = vec![];
    
    for repo in &mut repos {
        // Skip if recently synced (within 5 minutes)
        if let Some(last_sync) = &repo.last_sync_at {
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last_sync) {
                let now = chrono::Utc::now();
                if (now - last_time.with_timezone(&chrono::Utc)).num_minutes() < 5 {
                    results.push(SyncResult {
                        success: true,
                        skill_count: repo.skill_count,
                        message: "最近已同步".to_string(),
                    });
                    continue;
                }
            }
        }
        
        // Sync this repository
        let sync_result = sync_repository(repo.id.clone()).await;
        match sync_result {
            Ok(r) => results.push(r),
            Err(e) => results.push(SyncResult {
                success: false,
                skill_count: 0,
                message: e,
            }),
        }
    }
    
    Ok(results)
}

/// Update repository info
#[tauri::command]
pub fn update_repository(
    repo_id: String,
    name: Option<String>,
) -> Result<SkillRepository, String> {
    let mut repos = load_repositories()?;
    let repo_index = repos.iter().position(|r| r.id == repo_id)
        .ok_or("仓库不存在")?;
    
    if let Some(new_name) = name {
        repos[repo_index].name = new_name;
    }
    
    let result = repos[repo_index].clone();
    save_repositories(&repos)?;
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSkillResult {
    pub success: bool,
    pub skill_name: String,
    pub skill_path: String,
    pub message: String,
}

/// Download and install a skill from repository
#[tauri::command]
pub async fn download_skill_from_repository(
    repo_id: String,
    skill_path: String,
    target_dir: String,
) -> Result<DownloadSkillResult, String> {
    log::info!("Downloading skill '{}' from repo '{}'", skill_path, repo_id);
    
    let repos = load_repositories()?;
    let repo = repos.iter().find(|r| r.id == repo_id)
        .ok_or("仓库不存在")?;
    
    let repo_url = repo.url.trim().trim_end_matches(".git");
    
    // Create target directory
    let target_dir = PathBuf::from(&target_dir);
    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建目标目录: {}", e))?;
    
    // Extract skill name from path
    let skill_name = skill_path.split('/')
        .last()
        .unwrap_or(&skill_path)
        .to_string();
    
    let skill_target_dir = target_dir.join(&skill_name);
    if skill_target_dir.exists() {
        return Ok(DownloadSkillResult {
            success: false,
            skill_name: skill_name.clone(),
            skill_path: skill_target_dir.to_string_lossy().to_string(),
            message: format!("技能 '{}' 已存在", skill_name),
        });
    }
    
    // Create skill directory
    fs::create_dir_all(&skill_target_dir).map_err(|e| format!("无法创建技能目录: {}", e))?;
    
    // Download skill files using git archive or direct download
    // For GitHub, we can use the raw content API
    if repo.repo_type == "github" {
        let parts: Vec<&str> = repo_url.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts.get(parts.len() - 2).unwrap_or(&"");
            let repo_name = parts.get(parts.len() - 1).unwrap_or(&"");
            
            // Try to download SKILL.md first to verify skill structure
            let _skill_md_url = format!(
                "https://raw.githubusercontent.com/{}/{}/main/{}",
                owner, repo_name, skill_path
            );
            
            // Try main branch first, then master
            let branches = ["main", "master"];
            let mut downloaded = false;
            
            for _branch in branches {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/contents/{}",
                    owner, repo_name, skill_path
                );
                
                let client = reqwest::Client::new();
                match client
                    .get(&url)
                    .header("User-Agent", "forge-desktop")
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.json::<Vec<serde_json::Value>>().await {
                                Ok(items) => {
                                    // It's a directory, download each file
                                    for item in items {
                                        if let (Some(name), Some(download_url), Some("file")) = (
                                            item.get("name").and_then(|v| v.as_str()),
                                            item.get("download_url").and_then(|v| v.as_str()),
                                            item.get("type").and_then(|v| v.as_str()),
                                        ) {
                                            let file_path = skill_target_dir.join(name);
                                            if let Err(e) = download_file(download_url, &file_path).await {
                                                log::warn!("Failed to download {}: {}", name, e);
                                            }
                                        }
                                    }
                                    downloaded = true;
                                    break;
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
            
            if downloaded {
                log::info!("Successfully downloaded skill '{}'", skill_name);
                return Ok(DownloadSkillResult {
                    success: true,
                    skill_name: skill_name.clone(),
                    skill_path: skill_target_dir.to_string_lossy().to_string(),
                    message: format!("技能 '{}' 安装成功", skill_name),
                });
            }
        }
    }
    
    // Fallback: Use git clone with sparse checkout
    let temp_dir = std::env::temp_dir()
        .join(format!("skill-download-{}", uuid::Uuid::new_v4()));
    
    let output = tokio::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            repo_url,
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("git clone 失败: {}", e))?;
    
    if !output.status.success() {
        fs::remove_dir_all(&temp_dir).ok();
        return Ok(DownloadSkillResult {
            success: false,
            skill_name: skill_name.clone(),
            skill_path: String::new(),
            message: format!("无法下载技能: {}", String::from_utf8_lossy(&output.stderr)),
        });
    }
    
    // Copy the skill directory
    let source_dir = temp_dir.join(&skill_path);
    if source_dir.exists() {
        copy_dir_recursive(&source_dir, &skill_target_dir)
            .map_err(|e| format!("复制文件失败: {}", e))?;
    } else {
        fs::remove_dir_all(&temp_dir).ok();
        return Ok(DownloadSkillResult {
            success: false,
            skill_name: skill_name.clone(),
            skill_path: String::new(),
            message: format!("仓库中未找到技能 '{}'", skill_path),
        });
    }
    
    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
    
    log::info!("Successfully downloaded skill '{}'", skill_name);
    Ok(DownloadSkillResult {
        success: true,
        skill_name: skill_name.clone(),
        skill_path: skill_target_dir.to_string_lossy().to_string(),
        message: format!("技能 '{}' 安装成功", skill_name),
    })
}

async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "forge-desktop")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    let content = response
        .bytes()
        .await
        .map_err(|e| format!("读取内容失败: {}", e))?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    
    Ok(())
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillImportResult {
    pub success: bool,
    pub skill_name: String,
    pub skill_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkill {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub skill_type: String,
    pub has_skill_md: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub skill_name: String,
    pub method: String,
    pub target_path: String,
    pub message: String,
}

/// Unzip and import a skill package from a ZIP file
#[tauri::command]
pub async fn unzip_skill_package(
    zip_path: String,
    target_dir: String,
) -> Result<SkillImportResult, String> {
    log::info!("Unzipping skill package: {} to {}", zip_path, target_dir);

    // Validate ZIP file exists
    let zip_path = PathBuf::from(&zip_path);
    if !zip_path.exists() {
        return Ok(SkillImportResult {
            success: false,
            skill_name: String::new(),
            skill_path: String::new(),
            message: "ZIP 文件不存在".to_string(),
        });
    }

    // Open ZIP file
    let file = fs::File::open(&zip_path).map_err(|e| format!("无法打开 ZIP 文件: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("无效的 ZIP 文件: {}", e))?;

    // Create target directory
    let target_dir = PathBuf::from(&target_dir);
    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建目标目录: {}", e))?;

    // Find skill directory in ZIP
    let mut skill_dir_name: Option<String> = None;
    let mut skill_md_found = false;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| format!("读取 ZIP 内容失败: {}", e))?;
        let path = file.name().to_string();
        
        // Look for SKILL.md to identify skill package
        if path.ends_with("SKILL.md") {
            skill_md_found = true;
            // Extract skill directory name
            if let Some(pos) = path.strip_suffix("SKILL.md") {
                let dir = pos.trim_end_matches('/').trim_end_matches('\\');
                if !dir.is_empty() {
                    skill_dir_name = Some(dir.to_string());
                }
            }
            break;
        }
    }

    if !skill_md_found {
        return Ok(SkillImportResult {
            success: false,
            skill_name: String::new(),
            skill_path: String::new(),
            message: "压缩包中未找到有效的技能包（缺少 SKILL.md）".to_string(),
        });
    }

    let skill_name = skill_dir_name.unwrap_or_else(|| {
        // Fallback: use ZIP filename without extension
        zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-skill")
            .to_string()
    });

    // Create skill target directory
    let skill_target_dir = target_dir.join(&skill_name);
    if skill_target_dir.exists() {
        return Ok(SkillImportResult {
            success: false,
            skill_name: skill_name.clone(),
            skill_path: skill_target_dir.to_string_lossy().to_string(),
            message: format!("技能 '{}' 已存在，请先删除或重命名", skill_name),
        });
    }

    fs::create_dir_all(&skill_target_dir).map_err(|e| format!("无法创建技能目录: {}", e))?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取文件失败: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => skill_target_dir.join(path),
            None => continue,
        };

        // Security check: prevent path traversal
        if !outpath.starts_with(&skill_target_dir) {
            log::warn!("Skipping file outside target directory: {:?}", file.name());
            continue;
        }

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(|e| format!("无法创建目录: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("无法创建父目录: {}", e))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("无法创建文件: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    log::info!("Successfully extracted skill '{}' to {:?}", skill_name, skill_target_dir);

    Ok(SkillImportResult {
        success: true,
        skill_name: skill_name.clone(),
        skill_path: skill_target_dir.to_string_lossy().to_string(),
        message: format!("技能 '{}' 安装成功", skill_name),
    })
}

/// Scan local skills directory and return list of available skills
#[tauri::command]
pub async fn scan_local_skills(base_path: String) -> Result<Vec<LocalSkill>, String> {
    log::info!("Scanning local skills in: {}", base_path);

    let base_path = PathBuf::from(&base_path);
    if !base_path.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();

    let entries = fs::read_dir(&base_path).map_err(|e| format!("无法读取目录: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_md_path = path.join("SKILL.md");
            let has_skill_md = skill_md_path.exists();

            let description = if has_skill_md {
                read_skill_description(&skill_md_path)
            } else {
                None
            };

            let skill_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            skills.push(LocalSkill {
                name: skill_name,
                path: path.to_string_lossy().to_string(),
                skill_type: "directory".to_string(),
                has_skill_md,
                description,
            });
        }
    }

    // Sort by name
    skills.sort_by(|a, b| a.name.cmp(&b.name));

    log::info!("Found {} skills in local directory", skills.len());
    Ok(skills)
}

/// Read skill description from SKILL.md file
fn read_skill_description(skill_md_path: &Path) -> Option<String> {
    let content = fs::read_to_string(skill_md_path).ok()?;
    
    // Try to extract description from frontmatter or first paragraph
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("description:") {
            return Some(line.replace("description:", "").trim().to_string());
        }
        if line.starts_with("# ") {
            // Use first heading as description
            return Some(line.replace("# ", "").to_string());
        }
    }
    
    // Fallback: first non-empty line
    content.lines()
        .find(|l| !l.trim().is_empty() && !l.starts_with("---"))
        .map(|l| l.trim().to_string())
}

/// Import a local skill to target directory
#[tauri::command]
pub async fn import_local_skill(
    source_path: String,
    target_dir: String,
    import_method: String,
) -> Result<ImportResult, String> {
    log::info!(
        "Importing local skill from {} to {} using method: {}",
        source_path,
        target_dir,
        import_method
    );

    let source_path = PathBuf::from(&source_path);
    let target_dir = PathBuf::from(&target_dir);

    if !source_path.exists() {
        return Ok(ImportResult {
            success: false,
            skill_name: String::new(),
            method: import_method.clone(),
            target_path: String::new(),
            message: "源技能目录不存在".to_string(),
        });
    }

    let skill_name = source_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown-skill")
        .to_string();

    let target_path = target_dir.join(&skill_name);

    if target_path.exists() {
        return Ok(ImportResult {
            success: false,
            skill_name: skill_name.clone(),
            method: import_method.clone(),
            target_path: target_path.to_string_lossy().to_string(),
            message: format!("技能 '{}' 已存在，请先删除", skill_name),
        });
    }

    // Create target directory
    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建目标目录: {}", e))?;

    match import_method.as_str() {
        "symlink" => {
            // Create symbolic link
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source_path, &target_path)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
            
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&source_path, &target_path)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
        }
        _ => {
            // Copy directory recursively
            copy_dir_recursive(&source_path, &target_path)
                .map_err(|e| format!("复制失败: {}", e))?;
        }
    }

    log::info!("Successfully imported skill '{}' to {:?}", skill_name, target_path);

    Ok(ImportResult {
        success: true,
        skill_name: skill_name.clone(),
        method: import_method.clone(),
        target_path: target_path.to_string_lossy().to_string(),
        message: format!("技能 '{}' 导入成功", skill_name),
    })
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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

/// Detect common CLI skills directories
#[tauri::command]
pub fn detect_cli_skills_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Always include the Forge-managed skills directory.
    paths.push(
        crate::services::plugin_marketplace::forge_home()
            .join("skills")
            .to_string_lossy()
            .to_string(),
    );

    if let Some(home) = dirs::home_dir() {
        // Other CLI / IDE paths users may also have on disk
        let cli_paths = vec![
            home.join(".cursor/skills"),
            home.join(".agents/skills"),
            home.join(".cursor/plugins/skills"),
        ];

        for path in cli_paths {
            if path.exists() {
                paths.push(path.to_string_lossy().to_string());
            }
        }
    }

    #[cfg(windows)]
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let windows_paths = vec![
            PathBuf::from(&user_profile).join(".cursor/skills"),
            PathBuf::from(&user_profile).join(".agents/skills"),
        ];

        for path in windows_paths {
            if path.exists() {
                paths.push(path.to_string_lossy().to_string());
            }
        }
    }

    log::info!("Detected {} CLI skills paths", paths.len());
    paths
}

/// Get the default skills installation directory
#[tauri::command]
pub fn get_default_skills_dir() -> String {
    // Resolve the user-level Forge home (default: ~/.forge, overridable
    // via FORGE_HOME). All Forge-managed skills live under
    // <FORGE_HOME>/skills by default.
    let dir = crate::services::plugin_marketplace::forge_home().join("skills");
    std::fs::create_dir_all(&dir).ok();
    dir.to_string_lossy().to_string()
}

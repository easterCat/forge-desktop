// Skill Marketplace Services - HTTP client and data fetching

use crate::models::{
    MarketplaceSkill, PaginatedSkills, SkillSource, SyncResult, SyncTarget,
};
use base64::Engine;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;

// Preset skill sources configuration
pub fn get_preset_sources() -> Vec<SkillSource> {
    vec![
        SkillSource {
            id: "skillmp".to_string(),
            name: "SkillMP".to_string(),
            name_zh: None,
            region: "international".to_string(),
            url: "https://skillsmp.com".to_string(),
            api_endpoint: "https://api.skillsmp.com/v1/skills".to_string(),
            description: "International AI skills marketplace".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "skillzwave".to_string(),
            name: "SkillzWave".to_string(),
            name_zh: None,
            region: "international".to_string(),
            url: "https://skillzwave.ai".to_string(),
            api_endpoint: "https://api.skillzwave.ai/v1/skills".to_string(),
            description: "AI skills platform for developers".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "agensi".to_string(),
            name: "Agensi".to_string(),
            name_zh: None,
            region: "international".to_string(),
            url: "https://agensi.io".to_string(),
            api_endpoint: "https://agensi.io/skills/api/v1".to_string(),
            description: "Enterprise skills library".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "skills-marketplace".to_string(),
            name: "Skills Marketplace".to_string(),
            name_zh: None,
            region: "international".to_string(),
            url: "https://skills.marketplace".to_string(),
            api_endpoint: "https://skills.marketplace/api/v1".to_string(),
            description: "Community-driven skills marketplace".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "clawhub".to_string(),
            name: "ClawHub".to_string(),
            name_zh: Some("爪哇市场".to_string()),
            region: "china".to_string(),
            url: "https://clawhub.ai".to_string(),
            api_endpoint: "https://clawhub.ai/api/skills".to_string(),
            description: "国内 AI 技能聚合平台".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "skill-cn".to_string(),
            name: "Skill Hub 中国".to_string(),
            name_zh: None,
            region: "china".to_string(),
            url: "https://skill-cn.com".to_string(),
            api_endpoint: "https://skill-cn.com/api/skills".to_string(),
            description: "中文 AI 技能市场".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "agskills".to_string(),
            name: "agskills.dev".to_string(),
            name_zh: None,
            region: "china".to_string(),
            url: "https://agskills.dev".to_string(),
            api_endpoint: "https://agskills.dev/api/skills".to_string(),
            description: "开发者技能平台".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
        SkillSource {
            id: "awesome-skills".to_string(),
            name: "Awesome-Skills".to_string(),
            name_zh: Some("技能聚合".to_string()),
            region: "github".to_string(),
            url: "https://github.com/Sec-Dome/Awesome-Skills".to_string(),
            api_endpoint: "https://api.github.com/repos/Sec-Dome/Awesome-Skills/contents".to_string(),
            description: "GitHub aggregated skills collection".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            skill_count: None,
        },
    ]
}

// Fetch skills from a standard API source
pub async fn fetch_skills_from_api(
    source: &SkillSource,
    page: u32,
    page_size: u32,
    category: Option<&str>,
    keyword: Option<&str>,
) -> Result<PaginatedSkills, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let mut url = format!("{}?page={}&size={}", source.api_endpoint, page, page_size);
    if let Some(cat) = category {
        url.push_str(&format!("&category={}", cat));
    }
    if let Some(kw) = keyword {
        url.push_str(&format!("&search={}", kw));
    }

    log::info!("Fetching skills from: {}", url);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("HTTP request failed: {}", e);
            format!("网络请求失败: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("API 返回错误状态码: {}", status));
    }

    #[derive(Deserialize)]
    struct ApiResponse {
        items: Vec<MarketplaceSkill>,
        total: u32,
        page: u32,
        page_size: u32,
    }

    let api_resp: ApiResponse = response.json().await.map_err(|e| {
        log::error!("Failed to parse JSON: {}", e);
        format!("解析响应失败: {}", e)
    })?;

    let total_pages = ((api_resp.total as f64) / (page_size as f64)).ceil() as u32;

    Ok(PaginatedSkills {
        items: api_resp.items,
        total: api_resp.total,
        page: api_resp.page,
        page_size: api_resp.page_size,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    })
}

// Fetch skills from GitHub Awesome-Skills repository
pub async fn fetch_github_skills(
    _source: &SkillSource,
    page: u32,
    page_size: u32,
    category: Option<&str>,
    keyword: Option<&str>,
) -> Result<PaginatedSkills, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Fetch directory listing
    let url = "https://api.github.com/repos/Sec-Dome/Awesome-Skills/contents/README.md";
    
    log::info!("Fetching GitHub skills from: {}", url);

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header("User-Agent", "Forge-Desktop")
        .send()
        .await
        .map_err(|e| format!("GitHub API 请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    #[derive(Deserialize)]
    struct GitHubFileResponse {
        content: String,
        encoding: String,
    }

    let file_resp: GitHubFileResponse = response.json().await.map_err(|e| e.to_string())?;

    // Decode base64 content
    let content = if file_resp.encoding == "base64" {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(file_resp.content.replace('\n', ""))
            .map_err(|e| format!("Base64 解码失败: {}", e))?;
        String::from_utf8(decoded).map_err(|e| format!("UTF-8 解码失败: {}", e))?
    } else {
        file_resp.content
    };

    // Parse skills from README content
    let all_skills = parse_readme_skills(&content);
    
    // Apply filters
    let filtered: Vec<MarketplaceSkill> = all_skills
        .into_iter()
        .filter(|skill| {
            let matches_category = category.map_or(true, |cat| {
                skill.categories.iter().any(|c| c.to_lowercase().contains(&cat.to_lowercase()))
                    || skill.tags.iter().any(|t| t.to_lowercase().contains(&cat.to_lowercase()))
            });
            
            let matches_keyword = keyword.map_or(true, |kw| {
                skill.name.to_lowercase().contains(&kw.to_lowercase())
                    || skill.description.to_lowercase().contains(&kw.to_lowercase())
            });
            
            matches_category && matches_keyword
        })
        .collect();

    let total = filtered.len() as u32;
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(filtered.len());
    
    let items = if start < filtered.len() {
        filtered[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(PaginatedSkills {
        items,
        total,
        page,
        page_size,
        total_pages: total_pages.max(1),
        has_next: page < total_pages.max(1),
        has_prev: page > 1,
    })
}

// Parse skills from README markdown content
fn parse_readme_skills(content: &str) -> Vec<MarketplaceSkill> {
    let mut skills = Vec::new();
    
    // Parse markdown sections for skills
    let lines: Vec<&str> = content.lines().collect();
    let mut current_category = String::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        // Check for category headers (## Category)
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            current_category = trimmed.trim_start_matches("## ").trim().to_string();
        }
        
        // Look for skill entries - patterns like:
        // - [SkillName](url) - description
        // - [SkillName](url): description
        if trimmed.starts_with("- [") || trimmed.starts_with("* [") {
            if let Some(skill) = parse_skill_entry(trimmed, &current_category) {
                skills.push(skill);
            }
        }
    }
    
    // If no skills found, return sample data for demonstration
    if skills.is_empty() {
        skills = get_sample_skills();
    }
    
    skills
}

fn parse_skill_entry(line: &str, category: &str) -> Option<MarketplaceSkill> {
    // Pattern: - [Name](url) - description
    let content = line.trim_start_matches('-').trim_start_matches('*').trim();
    
    if !content.starts_with("[") {
        return None;
    }
    
    // Extract name and URL
    let bracket_end = content.find("](")?;
    let name = content[1..bracket_end].to_string();
    let url_end = content.find(") ")?; // Find the end of URL
    let url_start = content.find("](")? + 2;
    let url = content[url_start..url_end].to_string();
    
    // Extract description (after - or :)
    let desc_start = if content.find(" - ").is_some() {
        content.find(" - ").map(|p| p + 3)
    } else if content.find(": ").is_some() {
        content.find(": ").map(|p| p + 2)
    } else {
        None
    };
    
    let description = desc_start
        .map(|pos| content[pos..].trim().to_string())
        .unwrap_or_else(|| "No description".to_string());
    
    Some(MarketplaceSkill {
        id: name.to_lowercase().replace(' ', "-"),
        source_id: "awesome-skills".to_string(),
        name,
        description,
        long_description: None,
        author: None,
        version: None,
        categories: vec![category.to_string()],
        tags: vec![],
        install_command: Some(format!("git clone {}", url)),
        install_path: None,
        repository: Some(url),
        homepage: None,
        license: None,
        stars: None,
        downloads: None,
        last_updated: None,
    })
}

// Sample skills for demonstration when API is unavailable
fn get_sample_skills() -> Vec<MarketplaceSkill> {
    vec![
        MarketplaceSkill {
            id: "frontend-design".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "frontend-design".to_string(),
            description: "Premium frontend UI design skill with modern aesthetics".to_string(),
            long_description: Some("Creates distinctive, production-grade frontend interfaces with high design quality. Optimized for landing pages, dashboards, and web applications.".to_string()),
            author: Some("Design Team".to_string()),
            version: Some("2.0.0".to_string()),
            categories: vec!["Development".to_string(), "Tools".to_string()],
            tags: vec!["frontend".to_string(), "ui".to_string(), "design".to_string()],
            install_command: Some("cp -r skills/frontend-design ~/.cursor/skills/".to_string()),
            install_path: Some("skills/frontend-design/".to_string()),
            repository: Some("https://github.com/example/frontend-design".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(1250),
            downloads: Some(5000),
            last_updated: Some("2026-06-01".to_string()),
        },
        MarketplaceSkill {
            id: "canvas".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "canvas".to_string(),
            description: "Live React canvas builder for interactive visualizations".to_string(),
            long_description: Some("Build interactive React canvases for data visualization, charts, and analytical artifacts. Supports real-time updates and complex layouts.".to_string()),
            author: Some("Canvas Team".to_string()),
            version: Some("1.5.0".to_string()),
            categories: vec!["AI".to_string(), "Tools".to_string()],
            tags: vec!["react".to_string(), "canvas".to_string(), "visualization".to_string()],
            install_command: Some("cp -r skills/canvas ~/.cursor/skills/".to_string()),
            install_path: Some("skills/canvas/".to_string()),
            repository: Some("https://github.com/example/canvas".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(890),
            downloads: Some(3200),
            last_updated: Some("2026-05-28".to_string()),
        },
        MarketplaceSkill {
            id: "brandkit".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "brandkit".to_string(),
            description: "Premium brand guidelines and visual identity generator".to_string(),
            long_description: Some("Generate comprehensive brand guidelines including color palettes, typography systems, logo usage rules, and visual identity documentation.".to_string()),
            author: Some("Brand Studio".to_string()),
            version: Some("3.1.0".to_string()),
            categories: vec!["Business".to_string(), "Writing".to_string()],
            tags: vec!["brand".to_string(), "design".to_string(), "identity".to_string()],
            install_command: Some("cp -r skills/brandkit ~/.cursor/skills/".to_string()),
            install_path: Some("skills/brandkit/".to_string()),
            repository: Some("https://github.com/example/brandkit".to_string()),
            homepage: None,
            license: Some("CC-BY-4.0".to_string()),
            stars: Some(2100),
            downloads: Some(7800),
            last_updated: Some("2026-06-05".to_string()),
        },
        MarketplaceSkill {
            id: "imagegen-frontend-web".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "imagegen-frontend-web".to_string(),
            description: "Elite frontend image direction skill for premium websites".to_string(),
            long_description: Some("Generate high-end website design references with conversion-focused imagery. Creates section-specific images for landing pages and marketing sites.".to_string()),
            author: Some("Web Design Co".to_string()),
            version: Some("1.8.0".to_string()),
            categories: vec!["Development".to_string(), "Tools".to_string()],
            tags: vec!["frontend".to_string(), "image".to_string(), "web".to_string()],
            install_command: Some("cp -r skills/imagegen-frontend-web ~/.cursor/skills/".to_string()),
            install_path: Some("skills/imagegen-frontend-web/".to_string()),
            repository: Some("https://github.com/example/imagegen-frontend-web".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(670),
            downloads: Some(2400),
            last_updated: Some("2026-05-20".to_string()),
        },
        MarketplaceSkill {
            id: "code-review".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "code-review".to_string(),
            description: "Automated code review and quality assurance skill".to_string(),
            long_description: Some("Performs comprehensive code reviews including style consistency, security vulnerabilities, performance issues, and best practices validation.".to_string()),
            author: Some("DevTools Team".to_string()),
            version: Some("2.3.0".to_string()),
            categories: vec!["Development".to_string(), "AI".to_string()],
            tags: vec!["code".to_string(), "review".to_string(), "quality".to_string()],
            install_command: Some("cp -r skills/code-review ~/.cursor/skills/".to_string()),
            install_path: Some("skills/code-review/".to_string()),
            repository: Some("https://github.com/example/code-review".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(1500),
            downloads: Some(6200),
            last_updated: Some("2026-06-10".to_string()),
        },
        MarketplaceSkill {
            id: "security-audit".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "security-audit".to_string(),
            description: "Security vulnerability scanning and audit skill".to_string(),
            long_description: Some("Conducts thorough security audits including dependency vulnerability scanning, secret detection, authentication testing, and compliance checks.".to_string()),
            author: Some("Security Labs".to_string()),
            version: Some("1.9.0".to_string()),
            categories: vec!["Development".to_string(), "AI".to_string()],
            tags: vec!["security".to_string(), "audit".to_string(), "vulnerability".to_string()],
            install_command: Some("cp -r skills/security-audit ~/.cursor/skills/".to_string()),
            install_path: Some("skills/security-audit/".to_string()),
            repository: Some("https://github.com/example/security-audit".to_string()),
            homepage: None,
            license: Some("Apache-2.0".to_string()),
            stars: Some(980),
            downloads: Some(4100),
            last_updated: Some("2026-06-08".to_string()),
        },
        MarketplaceSkill {
            id: "automation".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "automation".to_string(),
            description: "Workflow automation and CI/CD integration skill".to_string(),
            long_description: Some("Automates repetitive development tasks including CI/CD pipeline setup, deployment workflows, and infrastructure provisioning.".to_string()),
            author: Some("AutoDev Team".to_string()),
            version: Some("4.0.0".to_string()),
            categories: vec!["Tools".to_string(), "AI".to_string()],
            tags: vec!["automation".to_string(), "ci".to_string(), "cd".to_string()],
            install_command: Some("cp -r skills/automation ~/.cursor/skills/".to_string()),
            install_path: Some("skills/automation/".to_string()),
            repository: Some("https://github.com/example/automation".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(1750),
            downloads: Some(5900),
            last_updated: Some("2026-06-12".to_string()),
        },
        MarketplaceSkill {
            id: "data-analysis".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "data-analysis".to_string(),
            description: "Data analytics and visualization skill for business insights".to_string(),
            long_description: Some("Analyzes datasets to extract meaningful insights, generates reports, and creates visualizations for business decision making.".to_string()),
            author: Some("Data Science Co".to_string()),
            version: Some("2.1.0".to_string()),
            categories: vec!["Business".to_string(), "AI".to_string()],
            tags: vec!["data".to_string(), "analytics".to_string(), "visualization".to_string()],
            install_command: Some("cp -r skills/data-analysis ~/.cursor/skills/".to_string()),
            install_path: Some("skills/data-analysis/".to_string()),
            repository: Some("https://github.com/example/data-analysis".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(1100),
            downloads: Some(4500),
            last_updated: Some("2026-06-03".to_string()),
        },
        MarketplaceSkill {
            id: "translate".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "translate".to_string(),
            description: "Professional translation and localization skill".to_string(),
            long_description: Some("Provides high-quality translation services supporting multiple languages with context-aware localization for software and documentation.".to_string()),
            author: Some("Globalize Team".to_string()),
            version: Some("1.6.0".to_string()),
            categories: vec!["Writing".to_string(), "AI".to_string()],
            tags: vec!["translation".to_string(), "localization".to_string(), "i18n".to_string()],
            install_command: Some("cp -r skills/translate ~/.cursor/skills/".to_string()),
            install_path: Some("skills/translate/".to_string()),
            repository: Some("https://github.com/example/translate".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(720),
            downloads: Some(2800),
            last_updated: Some("2026-05-25".to_string()),
        },
        MarketplaceSkill {
            id: "database-design".to_string(),
            source_id: "awesome-skills".to_string(),
            name: "database-design".to_string(),
            description: "Database architecture and schema design skill".to_string(),
            long_description: Some("Assists with database design including schema modeling, normalization, index optimization, and query performance tuning.".to_string()),
            author: Some("DB Experts".to_string()),
            version: Some("3.2.0".to_string()),
            categories: vec!["Development".to_string(), "Tools".to_string()],
            tags: vec!["database".to_string(), "schema".to_string(), "sql".to_string()],
            install_command: Some("cp -r skills/database-design ~/.cursor/skills/".to_string()),
            install_path: Some("skills/database-design/".to_string()),
            repository: Some("https://github.com/example/database-design".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            stars: Some(1350),
            downloads: Some(5100),
            last_updated: Some("2026-06-06".to_string()),
        },
    ]
}

// Install skill to local directory
pub async fn install_skill(
    skill: &MarketplaceSkill,
    local_path: &PathBuf,
) -> Result<String, String> {
    // `local_path` may be a project root or the skills directory
    // itself (e.g. the value returned by `get_default_skills_dir`).
    // `resolve_skills_dir` picks the right one so we never end up
    // nesting `skills/skills/<name>/`.
    let skills_dir = resolve_skills_dir(local_path);

    // Create skills directory if not exists
    fs::create_dir_all(&skills_dir)
        .await
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let skill_dir = skills_dir.join(&skill.name);
    
    // Check if skill already exists
    if skill_dir.exists() {
        return Err(format!("技能 '{}' 已存在，请先删除后再安装", skill.name));
    }
    
    // Create skill directory
    fs::create_dir_all(&skill_dir)
        .await
        .map_err(|e| format!("创建技能目录失败: {}", e))?;
    
    // Create SKILL.md file
    let skill_md = format!(
        r#"# {name}

{description}

## Details

- **Source**: {source_id}
- **Author**: {author}
- **Version**: {version}
- **Categories**: {categories}
- **Tags**: {tags}

{long_description}

## Installation

```bash
{install_command}
```

## Repository

{repository}

---
*Installed via Forge on {date}*
"#,
        name = skill.name,
        description = skill.description,
        source_id = skill.source_id,
        author = skill.author.as_deref().unwrap_or("Unknown"),
        version = skill.version.as_deref().unwrap_or("N/A"),
        categories = skill.categories.join(", "),
        tags = skill.tags.join(", "),
        long_description = skill.long_description
            .as_ref()
            .map(|d| format!("\n## Description\n\n{}\n", d))
            .unwrap_or_default(),
        install_command = skill.install_command.as_deref().unwrap_or("N/A"),
        repository = skill.repository.as_deref().unwrap_or("N/A"),
        date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    
    let skill_md_path = skill_dir.join("SKILL.md");
    fs::write(&skill_md_path, &skill_md)
        .await
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    // Create metadata.json
    let metadata = serde_json::json!({
        "id": skill.id,
        "name": skill.name,
        "description": skill.description,
        "author": skill.author,
        "version": skill.version,
        "categories": skill.categories,
        "tags": skill.tags,
        "source_id": skill.source_id,
        "installed_at": chrono::Local::now().to_rfc3339(),
        "install_path": skill_dir.to_string_lossy(),
    });
    
    let metadata_path = skill_dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .await
        .map_err(|e| format!("写入元数据失败: {}", e))?;
    
    log::info!("Skill '{}' installed to {}", skill.name, skill_dir.display());
    Ok(skill_dir.to_string_lossy().to_string())
}

// Sync skill to target directory
pub async fn sync_skill_to_target(
    skill_name: &str,
    local_skills_path: &PathBuf,
    target: &SyncTarget,
) -> Result<SyncResult, String> {
    let source_dir = local_skills_path.join("skills").join(skill_name);
    
    if !source_dir.exists() {
        return Err(format!("本地技能目录不存在: {}", source_dir.display()));
    }
    
    // Expand target path (handle ~)
    let target_path = expand_path(&target.path);
    
    // Create target directory
    if !target_path.exists() {
        fs::create_dir_all(&target_path)
            .await
            .map_err(|e| format!("创建目标目录失败: {}", e))?;
    }
    
    let target_skill_dir = target_path.join(skill_name);
    
    match target.method.as_str() {
        "symlink" => {
            // Remove existing directory if exists
            if target_skill_dir.exists() {
                if target_skill_dir.is_symlink() {
                    fs::remove_file(&target_skill_dir)
                        .await
                        .map_err(|e| format!("删除旧链接失败: {}", e))?;
                } else {
                    return Err(format!("目标路径已存在且不是符号链接: {}", target_skill_dir.display()));
                }
            }
            
            // Create symbolic link
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source_dir, &target_skill_dir)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
            
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&source_dir, &target_skill_dir)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
            
            log::info!("Created symlink: {} -> {}", target_skill_dir.display(), source_dir.display());
        }
        "copy" | _ => {
            // Copy directory recursively
            copy_dir_recursive(&source_dir, &target_skill_dir).await?;
            log::info!("Copied skill to: {}", target_skill_dir.display());
        }
    }
    
    Ok(SyncResult {
        success: true,
        skill_name: skill_name.to_string(),
        target_path: target_skill_dir.to_string_lossy().to_string(),
        method: target.method.clone(),
        message: format!("成功同步到 {} ({})", target.name, target.method),
        error: None,
    })
}

type CopyResult = Result<(), String>;

fn copy_dir_recursive_sync(src: &PathBuf, dst: &PathBuf) -> CopyResult {
    if !src.is_dir() {
        return Err(format!("源路径不是目录: {}", src.display()));
    }
    
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("创建目标目录失败: {}", e))?;
    
    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("读取源目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive_sync(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    
    Ok(())
}

async fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    // Use blocking IO for recursive directory copy
    let src_clone = src.clone();
    let dst_clone = dst.clone();
    
    tokio::task::spawn_blocking(move || {
        copy_dir_recursive_sync(&src_clone, &dst_clone)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(path.trim_start_matches("~/")))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

// Resolve the actual skills directory from `local_path`. The
// caller may pass either a project root (in which case the skills
// live under `<root>/skills/`) or the skills directory itself
// (which is what `get_default_skills_dir` returns on the Rust
// side). To stay compatible with both, we look for a child that
// looks like a skill (has `metadata.json` or `SKILL.md`):
//   - if such a child exists directly under `local_path`, treat
//     `local_path` as the skills directory itself;
//   - otherwise fall back to `<local_path>/skills/`.
fn resolve_skills_dir(local_path: &PathBuf) -> PathBuf {
    let direct = local_path.clone();
    if let Ok(read) = std::fs::read_dir(&direct) {
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir()
                && (p.join("metadata.json").exists() || p.join("SKILL.md").exists())
            {
                log::debug!(
                    "resolve_skills_dir: {} already looks like a skills dir",
                    direct.display()
                );
                return direct;
            }
        }
    }
    local_path.join("skills")
}

// Get local installed skills
pub async fn get_local_skills(local_path: &PathBuf) -> Result<Vec<MarketplaceSkill>, String> {
    let skills_dir = resolve_skills_dir(local_path);

    if !skills_dir.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();

    for entry in WalkDir::new(&skills_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.path() != skills_dir {
            let metadata_path = entry.path().join("metadata.json");

            if metadata_path.exists() {
                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                        skills.push(MarketplaceSkill {
                            id: metadata["id"].as_str().unwrap_or_default().to_string(),
                            source_id: metadata["source_id"].as_str().unwrap_or("local").to_string(),
                            name: metadata["name"].as_str().unwrap_or_default().to_string(),
                            description: metadata["description"].as_str().unwrap_or_default().to_string(),
                            long_description: None,
                            author: metadata["author"].as_str().map(String::from),
                            version: metadata["version"].as_str().map(String::from),
                            categories: metadata["categories"]
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                            tags: metadata["tags"]
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                            install_command: None,
                            install_path: Some(entry.path().to_string_lossy().to_string()),
                            repository: None,
                            homepage: None,
                            license: None,
                            stars: None,
                            downloads: None,
                            last_updated: metadata["installed_at"].as_str().map(String::from),
                        });
                    }
                }
            } else {
                // Create basic skill from directory name
                skills.push(MarketplaceSkill {
                    id: entry.file_name().to_string_lossy().to_string(),
                    source_id: "local".to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    description: "本地安装的技能".to_string(),
                    long_description: None,
                    author: None,
                    version: None,
                    categories: vec![],
                    tags: vec![],
                    install_command: None,
                    install_path: Some(entry.path().to_string_lossy().to_string()),
                    repository: None,
                    homepage: None,
                    license: None,
                    stars: None,
                    downloads: None,
                    last_updated: None,
                });
            }
        }
    }

    Ok(skills)
}

// Validate sync target path
pub fn validate_sync_target(target: &SyncTarget) -> bool {
    let expanded = expand_path(&target.path);
    expanded.exists() || expanded.to_string_lossy().contains("~")
}

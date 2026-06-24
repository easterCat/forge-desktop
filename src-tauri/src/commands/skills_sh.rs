use serde::{Deserialize, Serialize};

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkill {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub source: String,
    pub installs: u64,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    #[serde(rename = "installUrl")]
    pub install_url: Option<String>,
    pub url: String,
    #[serde(default)]
    pub is_duplicate: Option<bool>,
    #[serde(rename = "installsYesterday", default)]
    pub installs_yesterday: Option<u64>,
    #[serde(default)]
    pub change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShPage {
    pub data: Vec<SkillsShSkill>,
    pub pagination: SkillsShPagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShPagination {
    pub page: u32,
    #[serde(rename = "perPage")]
    pub per_page: u32,
    pub total: u64,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShCuratedOwner {
    pub owner: String,
    #[serde(rename = "totalInstalls")]
    pub total_installs: u64,
    #[serde(rename = "featuredRepo")]
    pub featured_repo: String,
    #[serde(rename = "featuredSkill")]
    pub featured_skill: String,
    pub skills: Vec<SkillsShSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShCuratedResponse {
    pub data: Vec<SkillsShCuratedOwner>,
    #[serde(rename = "totalOwners")]
    pub total_owners: u32,
    #[serde(rename = "totalSkills")]
    pub total_skills: u64,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkillFile {
    pub path: String,
    pub contents: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkillDetail {
    pub id: String,
    pub source: String,
    pub slug: String,
    pub installs: u64,
    pub hash: Option<String>,
    pub files: Option<Vec<SkillsShSkillFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShAuditEntry {
    pub provider: String,
    pub slug: String,
    pub status: String,
    pub summary: String,
    #[serde(rename = "auditedAt")]
    pub audited_at: String,
    #[serde(rename = "riskLevel", default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShAuditResponse {
    pub id: String,
    pub source: String,
    pub slug: String,
    pub audits: Vec<SkillsShAuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub output: Option<String>,
}

// ============================================================================
// HTTP Client
// ============================================================================

fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
}

// ============================================================================
// Tauri Commands
// ============================================================================

const SKILLS_SH_BASE_URL: &str = "https://skills.sh";

#[tauri::command]
pub async fn fetch_skills_sh_leaderboard(
    view: String,
    page: u32,
    per_page: u32,
) -> Result<SkillsShPage, String> {
    log::info!(
        "Fetching skills.sh leaderboard (view={}, page={}, per_page={})",
        view,
        page,
        per_page
    );

    let client = create_http_client();
    let url = format!(
        "{}/api/v1/skills?view={}&page={}&per_page={}",
        SKILLS_SH_BASE_URL, view, page, per_page
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("skills.sh API request failed: {}", e);
            format!("无法连接到 skills.sh: {}", e)
        })?;

    let status = response.status();
    if status.as_u16() == 401 {
        return Err("skills.sh API 需要认证 (401)。请在浏览器中打开 https://www.skills.sh/ 获取更多信息。".to_string());
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log::error!("skills.sh API returned error {}: {}", status, body);
        return Err(format!("skills.sh API 返回错误 ({}): {}", status, body));
    }

    let result: SkillsShPage = response
        .json()
        .await
        .map_err(|e| {
            log::error!("Failed to parse skills.sh response: {}", e);
            format!("解析 skills.sh 响应失败: {}", e)
        })?;

    log::info!(
        "Successfully fetched {} skills from leaderboard",
        result.data.len()
    );
    Ok(result)
}

#[tauri::command]
pub async fn search_skills_sh(
    query: String,
    limit: u32,
) -> Result<Vec<SkillsShSkill>, String> {
    log::info!("Searching skills.sh for: {}", query);

    if query.len() < 2 {
        return Err("搜索关键词至少需要 2 个字符".to_string());
    }

    let client = create_http_client();
    let url = format!(
        "{}/api/v1/skills/search?q={}&limit={}",
        SKILLS_SH_BASE_URL,
        urlencoding::encode(&query),
        limit
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            log::error!("skills.sh search request failed: {}", e);
            format!("搜索请求失败: {}", e)
        })?;

    let status = response.status();
    if status.as_u16() == 401 {
        return Err("skills.sh API 需要认证 (401)。请在浏览器中打开 https://www.skills.sh/ 获取更多信息。".to_string());
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("搜索失败 ({}): {}", status, body));
    }

    let result: Vec<SkillsShSkill> = response
        .json()
        .await
        .map_err(|e| format!("解析搜索结果失败: {}", e))?;

    log::info!("Search returned {} results", result.len());
    Ok(result)
}

#[tauri::command]
pub async fn fetch_skills_sh_curated() -> Result<SkillsShCuratedResponse, String> {
    log::info!("Fetching skills.sh curated list");

    let client = create_http_client();
    let url = format!("{}/api/v1/skills/curated", SKILLS_SH_BASE_URL);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("无法获取 Curated 列表: {}", e))?;

    let status = response.status();
    if status.as_u16() == 401 {
        return Err("skills.sh API 需要认证 (401)。请在浏览器中打开 https://www.skills.sh/ 获取更多信息。".to_string());
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("获取 Curated 列表失败 ({}): {}", status, body));
    }

    let result: SkillsShCuratedResponse = response
        .json()
        .await
        .map_err(|e| format!("解析 Curated 响应失败: {}", e))?;

    log::info!(
        "Curated list: {} owners, {} total skills",
        result.total_owners,
        result.total_skills
    );
    Ok(result)
}

#[tauri::command]
pub async fn fetch_skills_sh_skill_detail(
    source: String,
    slug: String,
) -> Result<SkillsShSkillDetail, String> {
    log::info!("Fetching skill detail: {}/{}", source, slug);

    let client = create_http_client();
    let url = format!("{}/api/v1/skills/{}/{}", SKILLS_SH_BASE_URL, source, slug);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取技能详情失败: {}", e))?;

    let status = response.status();
    if status.as_u16() == 401 {
        return Err("skills.sh API 需要认证 (401)。请在浏览器中打开 https://www.skills.sh/ 获取更多信息。".to_string());
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("获取技能详情失败 ({}): {}", status, body));
    }

    let result: SkillsShSkillDetail = response
        .json()
        .await
        .map_err(|e| format!("解析技能详情失败: {}", e))?;

    Ok(result)
}

#[tauri::command]
pub async fn fetch_skills_sh_audit(
    source: String,
    slug: String,
) -> Result<SkillsShAuditResponse, String> {
    log::info!("Fetching skill audit: {}/{}", source, slug);

    let client = create_http_client();
    let url = format!("{}/api/v1/skills/audit/{}/{}", SKILLS_SH_BASE_URL, source, slug);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取安全审计失败: {}", e))?;

    let status = response.status();
    if status.as_u16() == 401 {
        return Err("skills.sh API 需要认证 (401)。请在浏览器中打开 https://www.skills.sh/ 获取更多信息。".to_string());
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("获取安全审计失败 ({}): {}", status, body));
    }

    let result: SkillsShAuditResponse = response
        .json()
        .await
        .map_err(|e| format!("解析安全审计失败: {}", e))?;

    Ok(result)
}

#[tauri::command]
pub async fn install_skill_via_skills_sh(
    install_url: String,
    slug: String,
    _target_agent: String,
) -> Result<InstallResult, String> {
    log::info!("Installing skill via npx: {}", slug);

    // Build the npx command
    let npx_cmd = format!("npx skills add {} --skill {}", install_url, slug);

    // Use tokio to run the command
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&npx_cmd)
        .output()
        .await
        .map_err(|e| {
            log::error!("Failed to execute npx: {}", e);
            format!("无法执行 npx: {}", e)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        log::info!("Successfully installed skill: {}", slug);
        Ok(InstallResult {
            success: true,
            message: format!("技能 '{}' 安装成功", slug),
            output: Some(stdout),
        })
    } else {
        let error_msg = if stderr.is_empty() {
            format!("安装失败 (exit code: {:?})", output.status.code())
        } else {
            stderr.clone()
        };

        // Check if npx is not found
        if stderr.contains("npx: command not found") || stderr.contains("not found") {
            return Ok(InstallResult {
                success: false,
                message: "未找到 npx/npm。请确保已安装 Node.js 并配置了 PATH 环境变量。".to_string(),
                output: Some(stderr),
            });
        }

        log::error!("Failed to install skill: {}", error_msg);
        Ok(InstallResult {
            success: false,
            message: error_msg,
            output: Some(stdout),
        })
    }
}

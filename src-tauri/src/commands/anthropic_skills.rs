use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use url;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSource {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub subdir_prefix: String,
    pub cache_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub repository: String,
    pub subdirectory: String,
    pub default_branch: String,
    #[serde(default)]
    pub file_count: u32,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub installed: bool,
    pub installed_path: Option<String>,
    pub installed_at: Option<String>,
    #[serde(default)]
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub skill_id: String,
    pub stage: String,
    pub progress: u8,
    pub message: String,
    pub files_downloaded: u32,
    pub files_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallVerification {
    pub success: bool,
    pub files_downloaded: u32,
    pub total_size: u64,
    pub skill_md_present: bool,
    pub sha256_verified: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    size: u64,
    download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    tags: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallationRecord {
    pub skill_id: String,
    pub repository: String,
    pub subdirectory: String,
    pub installed_at: String,
    pub source_id: String,
    pub files: Vec<FileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileRecord {
    pub path: String,
    pub sha256: String,
    pub source_url: String,
    pub size: u64,
}

// ============================================================================
// Constants
// ============================================================================

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_TOTAL_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_CONCURRENT_DOWNLOADS: usize = 4;
// `CACHE_TTL_SECS` is now treated purely as a *stale hint*: it does NOT
// trigger a network refetch. Network fetches are explicit (the user clicking
// the Refresh button in the UI). The TTL is only used to mark cache entries
// as "stale" in the UI so the user can decide whether to refresh.
const CACHE_TTL_SECS: i64 = 3600; // 1 hour

// ============================================================================
// Skill Source Definitions
// ============================================================================

fn get_default_sources() -> Vec<SkillSource> {
    vec![
        SkillSource {
            id: "anthropic-official".to_string(),
            label: "Anthropic Official".to_string(),
            owner: "anthropics".to_string(),
            repo: "skills".to_string(),
            branch: "main".to_string(),
            subdir_prefix: "skills".to_string(),
            cache_file: "anthropic_skills_cache.json".to_string(),
        },
        SkillSource {
            id: "composio-awesome".to_string(),
            label: "Composio Awesome".to_string(),
            owner: "ComposioHQ".to_string(),
            repo: "awesome-claude-skills".to_string(),
            branch: "master".to_string(),
            subdir_prefix: "".to_string(),
            cache_file: "composio_skills_cache.json".to_string(),
        },
    ]
}

fn get_anthropic_official_source() -> SkillSource {
    SkillSource {
        id: "anthropic-official".to_string(),
        label: "Anthropic Official".to_string(),
        owner: "anthropics".to_string(),
        repo: "skills".to_string(),
        branch: "main".to_string(),
        subdir_prefix: "skills".to_string(),
        cache_file: "anthropic_skills_cache.json".to_string(),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_cache_path(source: &SkillSource) -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("forge")
        .join(&source.cache_file)
}

fn ensure_cache_dir() -> std::io::Result<PathBuf> {
    let cache_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("forge")
        .join("anthropic_skills_cache.json");
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(cache_path)
}

fn validate_skill_id(skill_id: &str) -> Result<(), String> {
    let re = regex::Regex::new(r"^[a-z0-9][a-z0-9_-]{0,63}$").unwrap();
    if !re.is_match(skill_id) {
        return Err(format!(
            "Invalid skill_id '{}': must match ^[a-z0-9][a-z0-9_-]{{0,63}}$",
            skill_id
        ));
    }
    Ok(())
}

/// Lenient check for paths that may not exist yet (e.g. skill_dir before creation).
///
/// Strategy:
/// 1. Canonicalize the longest existing ancestor of `path` (the "resolved" part).
/// 2. Canonicalize `target` (must already exist).
/// 3. Require the resolved ancestor to live inside the canonical target.
/// 4. Reject any `..` segments in the *unresolved* remainder, which is the only way
///    a not-yet-existing path could escape the resolved ancestor.
fn is_safe_subpath(target: &Path, path: &Path) -> bool {
    let Ok(target_canonical) = target.canonicalize() else {
        return false;
    };

    // Find the deepest existing ancestor of `path`.
    let mut ancestor: Option<&Path> = None;
    let mut remainder: Option<&Path> = None;
    let mut cursor = path;
    loop {
        if cursor.exists() {
            ancestor = Some(cursor);
            // The remainder is everything between this ancestor and the original path.
            remainder = path.strip_prefix(cursor).ok().and_then(|p| if p.as_os_str().is_empty() { None } else { Some(p) });
            break;
        }
        match cursor.parent() {
            Some(p) if !p.as_os_str().is_empty() => cursor = p,
            _ => break,
        }
    }

    let Some(ancestor) = ancestor else {
        // Nothing along `path` exists. We can still verify that the lexical
        // form stays within target (no `..` segments).
        if has_parent_traversal(path) {
            return false;
        }
        return path.starts_with(target);
    };

    let Ok(ancestor_canonical) = ancestor.canonicalize() else {
        return false;
    };
    if !ancestor_canonical.starts_with(&target_canonical) {
        return false;
    }

    if let Some(rem) = remainder {
        if has_parent_traversal(rem) {
            return false;
        }
    }
    true
}

/// Returns true if `path` contains any `..` component (lexical traversal).
fn has_parent_traversal(path: &Path) -> bool {
    path.components().any(|c| matches!(c, std::path::Component::ParentDir))
}

fn compute_sha256(path: &Path) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(hex::encode(hasher.finalize()))
}

fn parse_frontmatter(content: &str) -> Option<SkillFrontmatter> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end_marker = content[3..].find("---")?;
    let yaml_content = &content[3..3 + end_marker];

    serde_yaml::from_str::<SkillFrontmatter>(yaml_content).ok()
}

fn create_http_client() -> reqwest::Client {
    // GitHub requires a meaningful User-Agent on every request. The default
    // `reqwest` UA (`reqwest/x.y.z`) is not unique enough and historically
    // caused us to be bucketed into a shared rate-limit pool that hit 403
    // long before the documented anonymous quota.
    //
    // If a token is available — from the Settings panel (highest priority)
    // or the `GITHUB_TOKEN` env var (handy for dev / CI) — we attach it as
    // a Bearer token. That raises the per-hour quota from 60 to 5000 and
    // removes the most common cause of "Skill directory not found" errors
    // that are actually 403 rate-limit responses being mis-reported as 404s
    // by the contents-listing check below.
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("forge-desktop/0.1 (+https://github.com/forge-desktop)"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
    );

    if let Some(token) = crate::commands::settings::read_github_token() {
        if let Ok(value) =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
        {
            headers.insert(reqwest::header::AUTHORIZATION, value);
            log::info!("GitHub API requests will use stored token (authenticated quota)");
        }
    }

    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .default_headers(headers)
        .build()
        .unwrap_or_default()
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
pub fn get_remote_skill_sources() -> Vec<SkillSource> {
    log::info!("Getting available remote skill sources");
    get_default_sources()
}

// ============================================================================
// Cache Status (returned to the UI so it can render a "stale / fresh" hint)
// ============================================================================

/// Information about the on-disk cache for a single remote-skill source.
///
/// Returned alongside the cached skills by `list_remote_skills_cached_only`
/// so the UI can show whether the data is fresh, stale, or missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSkillsResult {
    /// The cached skills. Empty when the cache file is missing or unreadable.
    pub skills: Vec<AnthropicSkill>,
    /// `true` when a cache file was found on disk and successfully read.
    pub cache_exists: bool,
    /// `true` when the cache file is older than `CACHE_TTL_SECS`. This is a
    /// pure UI hint — the Rust side will not refetch automatically.
    pub is_stale: bool,
    /// Unix timestamp (seconds) of the last successful write to the cache
    /// file, or `None` when the cache doesn't exist.
    pub cached_at: Option<i64>,
}

#[tauri::command]
pub async fn list_remote_skills(
    source: SkillSource,
    refresh: Option<bool>,
) -> Result<Vec<AnthropicSkill>, String> {
    log::info!(
        "Listing remote skills from {} (refresh: {:?})",
        source.id,
        refresh
    );

    let cache_path = get_cache_path(&source);
    let should_refresh = refresh.unwrap_or(false);

    // Check cache first (unless refresh is requested)
    if !should_refresh && cache_path.exists() {
        if let Ok(cached) = load_cached_skills(&cache_path) {
            if !is_cache_expired_internal(&cache_path) {
                log::info!(
                    "Returning cached skills for {} ({} items)",
                    source.id,
                    cached.len()
                );
                return Ok(mark_installed_skills(cached));
            }
        }
    }

    // Fetch from GitHub API
    log::info!("Fetching skills from GitHub API for source {}", source.id);
    let skills = fetch_skills_from_github(&source).await?;

    // Cache the results
    if let Err(e) = cache_skills(&skills, &cache_path) {
        log::warn!("Failed to cache skills: {}", e);
    }

    Ok(skills)
}

/// Cache-only read. **Never** issues a network request — used by the UI on
/// view mount so that re-entering the page is instant and the only time
/// the user actually pays for a network round-trip is when they click the
/// "Refresh" button (which calls `list_remote_skills` with `refresh=true`).
///
/// Behavior:
///
/// - Cache file present & readable → return cached data, plus a
///   `cache_exists = true` flag and the `cached_at` / `is_stale` metadata.
/// - Cache file missing or unreadable → return empty list, `cache_exists = false`.
#[tauri::command]
pub fn list_remote_skills_cached_only(source: SkillSource) -> CachedSkillsResult {
    let cache_path = get_cache_path(&source);
    let cached_at = cache_modified_secs(&cache_path);
    let cache_exists = cache_path.exists();
    let is_stale = is_cache_expired_internal(&cache_path);

    let skills = if cache_exists {
        match load_cached_skills(&cache_path) {
            Ok(list) => {
                log::info!(
                    "Cache hit for {} ({} items, stale={})",
                    source.id,
                    list.len(),
                    is_stale
                );
                mark_installed_skills(list)
            }
            Err(e) => {
                log::warn!(
                    "Failed to read cache for {}: {} — returning empty list",
                    source.id,
                    e
                );
                Vec::new()
            }
        }
    } else {
        log::info!("No cache for {} — returning empty list", source.id);
        Vec::new()
    };

    CachedSkillsResult {
        skills,
        cache_exists,
        is_stale,
        cached_at,
    }
}

fn cache_modified_secs(cache_path: &Path) -> Option<i64> {
    let metadata = fs::metadata(cache_path).ok()?;
    let modified = metadata.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .ok()
}

async fn fetch_skills_from_github(source: &SkillSource) -> Result<Vec<AnthropicSkill>, String> {
    let client = create_http_client();

    // Build the API URL based on subdir_prefix
    let list_url = if source.subdir_prefix.is_empty() {
        format!(
            "https://api.github.com/repos/{}/{}/contents/",
            source.owner, source.repo
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            source.owner, source.repo, source.subdir_prefix
        )
    };

    // First, get the list of skill directories
    let response = client
        .get(&list_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("GitHub API returned error: {}", status));
    }

    let contents: Vec<GitHubContent> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    // Filter only directories
    let skill_dirs: Vec<_> = contents
        .into_iter()
        .filter(|c| c.content_type == "dir")
        .collect();

    log::info!("Found {} skill directories", skill_dirs.len());

    // Fetch SKILL.md for each directory with throttling (4 concurrent limit)
    let mut handles = Vec::new();

    for dir in skill_dirs {
        let skill_id = dir.name.clone();
        let skill_path = dir.path.clone();
        let client = client.clone();
        let source = source.clone();

        let handle = tokio::spawn(async move {
            fetch_skill_metadata(&client, &source, &skill_id, &skill_path).await
        });

        handles.push(handle);
    }

    // Collect results with throttling (wait for previous batch to complete)
    let mut skills = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        // Throttle to 4 concurrent requests
        if i > 0 && i % MAX_CONCURRENT_DOWNLOADS == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        if let Ok(Ok(Some(skill))) = handle.await {
            skills.push(skill);
        }
    }

    // Sort by name
    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    log::info!("Successfully fetched {} skills", skills.len());
    Ok(skills)
}

async fn fetch_skill_metadata(
    client: &reqwest::Client,
    source: &SkillSource,
    skill_id: &str,
    skill_path: &str,
) -> Result<Option<AnthropicSkill>, String> {
    // Build SKILL.md URL based on subdir_prefix
    let skill_md_url = if source.subdir_prefix.is_empty() {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}/SKILL.md",
            source.owner, source.repo, skill_id
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}/SKILL.md",
            source.owner, source.repo, skill_path
        )
    };

    let response = match client
        .get(&skill_md_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::warn!("Failed to fetch SKILL.md for {}: {}", skill_id, e);
            return Ok(None);
        }
    };

    if !response.status().is_success() {
        log::warn!("SKILL.md not found for {}", skill_id);
        return Ok(None);
    }

    let content: GitHubContent = match response.json().await {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to parse SKILL.md response for {}: {}", skill_id, e);
            return Ok(None);
        }
    };

    let Some(download_url) = content.download_url else {
        return Ok(None);
    };

    // Download the SKILL.md content
    let skill_md_content = match client.get(&download_url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                log::warn!("Failed to download SKILL.md for {}: {}", skill_id, e);
                return Ok(None);
            }
        },
        Err(e) => {
            log::warn!("Failed to download SKILL.md for {}: {}", skill_id, e);
            return Ok(None);
        }
    };

    let frontmatter = parse_frontmatter(&skill_md_content);

    // Count files in the directory
    let file_count = count_skill_files(client, source, skill_path).await.unwrap_or(0);

    // Build subdirectory path based on source
    let subdirectory = if source.subdir_prefix.is_empty() {
        skill_id.to_string()
    } else {
        format!("{}/{}", source.subdir_prefix, skill_id)
    };

    Ok(Some(AnthropicSkill {
        id: skill_id.to_string(),
        name: frontmatter
            .as_ref()
            .and_then(|f| f.name.clone())
            .unwrap_or_else(|| skill_id.to_string()),
        description: frontmatter
            .as_ref()
            .and_then(|f| f.description.clone())
            .unwrap_or_default(),
        version: frontmatter.as_ref().and_then(|f| f.version.clone()),
        author: frontmatter.as_ref().and_then(|f| f.author.clone()),
        tags: frontmatter
            .as_ref()
            .and_then(|f| f.tags.clone())
            .unwrap_or_default(),
        dependencies: frontmatter
            .as_ref()
            .and_then(|f| f.dependencies.clone())
            .unwrap_or_default(),
        repository: format!("https://github.com/{}/{}", source.owner, source.repo),
        subdirectory,
        default_branch: source.branch.clone(),
        file_count,
        size_bytes: 0,
        installed: false,
        installed_path: None,
        installed_at: None,
        source_id: source.id.clone(),
    }))
}

async fn count_skill_files(
    client: &reqwest::Client,
    source: &SkillSource,
    skill_path: &str,
) -> Option<u32> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        source.owner, source.repo, skill_path
    );

    let response = match client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return None,
    };

    if !response.status().is_success() {
        return None;
    }

    let contents: Vec<GitHubContent> = match response.json().await {
        Ok(c) => c,
        Err(_) => return None,
    };

    let count = contents
        .iter()
        .filter(|c| c.content_type == "file")
        .count() as u32;

    Some(count)
}

// ============================================================================
// Legacy Commands (Backward Compatible)
// ============================================================================

#[tauri::command]
pub async fn list_anthropic_skills(refresh: Option<bool>) -> Result<Vec<AnthropicSkill>, String> {
    let source = get_anthropic_official_source();
    list_remote_skills(source, refresh).await
}

#[tauri::command]
pub async fn install_anthropic_skill(
    app: AppHandle,
    skill_id: String,
    target_dir: String,
) -> Result<InstallProgress, String> {
    let source = get_anthropic_official_source();
    install_remote_skill(app, source, skill_id, target_dir).await
}

#[tauri::command]
pub async fn install_remote_skill(
    app: AppHandle,
    source: SkillSource,
    skill_id: String,
    target_dir: String,
) -> Result<InstallProgress, String> {
    log::info!(
        "Installing skill {} from {} to {}",
        skill_id,
        source.id,
        target_dir
    );

    // Validate skill_id
    validate_skill_id(&skill_id)?;

    let target_path = PathBuf::from(&target_dir);
    let skills_dir = target_path.clone();
    let skill_dir = skills_dir.join(&skill_id);

    // Ensure skills directory exists
    fs::create_dir_all(&skills_dir)
        .map_err(|e| format!("Failed to create skills directory: {}", e))?;

    // Verify path is within target_dir (security check).
    if !is_safe_subpath(&target_path, &skill_dir) {
        return Err("Path traversal detected: invalid skill directory".to_string());
    }

    // Determine event name based on source
    let event_name = if source.id == "anthropic-official" {
        "anthropic-skill-install-progress"
    } else {
        "remote-skill-install-progress"
    };

    // Emit initial progress
    let _ = app.emit(
        event_name,
        InstallProgress {
            skill_id: skill_id.clone(),
            stage: "listing".to_string(),
            progress: 0,
            message: "正在获取文件列表...".to_string(),
            files_downloaded: 0,
            files_total: 0,
        },
    );

    // Build API URL for skill directory listing based on source
    let api_url = if source.subdir_prefix.is_empty() {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            source.owner, source.repo, skill_id
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}/{}",
            source.owner, source.repo, source.subdir_prefix, skill_id
        )
    };

    // List files in the skill directory
    let client = create_http_client();

    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to list skill files: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        // Differentiate the most common failure modes so the UI can show
        // a useful message instead of "Skill directory not found" for what
        // is actually a 403 / 429 / 5xx.
        let body = response.text().await.unwrap_or_default();
        match status.as_u16() {
            403 => {
                log::warn!(
                    "GitHub 403 while listing {} (likely rate-limit): {}",
                    api_url,
                    body
                );
                return Err(format!(
                    "GitHub API 拒绝访问 ({})。这通常是匿名请求配额已用完 (60 req/h)。请设置环境变量 GITHUB_TOKEN 以使用认证额度 (5000 req/h) 后重试。\n\n请求: {}",
                    status, api_url
                ));
            }
            404 => {
                return Err(format!(
                    "Skill directory not found: {} (url: {})",
                    skill_id, api_url
                ));
            }
            429 => {
                return Err(format!(
                    "GitHub API 限流 (429)。请稍后重试，或设置 GITHUB_TOKEN 提升配额。\n\n请求: {}",
                    api_url
                ));
            }
            _ => {
                return Err(format!(
                    "GitHub API 返回 {} for skill {}: {}",
                    status, skill_id, body
                ));
            }
        }
    }

    // Collect all files (including subdirectories)
    let mut all_files: Vec<GitHubContent> = Vec::new();
    collect_all_files(&client, &source, &api_url, &mut all_files).await?;

    let files_total = all_files.len() as u32;
    log::info!("Found {} files to download", files_total);

    // Emit listing complete
    let _ = app.emit(
        event_name,
        InstallProgress {
            skill_id: skill_id.clone(),
            stage: "listing".to_string(),
            progress: 10,
            message: format!("找到 {} 个文件", files_total),
            files_downloaded: 0,
            files_total,
        },
    );

    // Create skill directory
    fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    // Download files with concurrency limit using batched approach
    let mut downloaded_files: Vec<FileRecord> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut files_downloaded: u32 = 0;
    let mut total_size: u64 = 0;

    for (i, file) in all_files.iter().enumerate() {
        // Throttle to avoid rate limiting
        if i > 0 && i % MAX_CONCURRENT_DOWNLOADS == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        let Some(download_url) = &file.download_url else {
            continue;
        };

        let file_path = skill_dir.join(&file.name);
        let source_url = download_url.clone();
        let file_size = file.size;

        // Check total size
        if total_size + file_size > MAX_TOTAL_SIZE {
            errors.push(format!(
                "Skill exceeds maximum size limit ({} > {})",
                total_size + file_size,
                MAX_TOTAL_SIZE
            ));
            continue;
        }

        // Check individual file size
        if file_size > MAX_FILE_SIZE {
            errors.push(format!("File {} exceeds 10MB limit", file.name));
            continue;
        }

        // Verify path is within skill_dir.
        if !is_safe_subpath(&skill_dir, &file_path) {
            errors.push(format!("Path traversal detected: {}", file.name));
            continue;
        }

        match download_single_file(&client, &source_url, &file_path).await {
            Ok(sha256) => {
                files_downloaded += 1;
                total_size += file_size;
                downloaded_files.push(FileRecord {
                    path: file.name.clone(),
                    sha256,
                    source_url,
                    size: file_size,
                });

                let progress = 10 + ((files_downloaded as f64 / files_total as f64) * 80.0) as u8;

                let _ = app.emit(
                    event_name,
                    InstallProgress {
                        skill_id: skill_id.clone(),
                        stage: "downloading".to_string(),
                        progress,
                        message: format!("正在下载: {}", file.name),
                        files_downloaded,
                        files_total,
                    },
                );
            }
            Err(e) => {
                errors.push(format!("Download failed for {}: {}", file.name, e));
            }
        }
    }

    // Emit verification stage
    let _ = app.emit(
        event_name,
        InstallProgress {
            skill_id: skill_id.clone(),
            stage: "verifying".to_string(),
            progress: 95,
            message: "正在验证安装...".to_string(),
            files_downloaded,
            files_total,
        },
    );

    // Build subdirectory path based on source
    let subdirectory = if source.subdir_prefix.is_empty() {
        skill_id.clone()
    } else {
        format!("{}/{}", source.subdir_prefix, skill_id)
    };

    // Write installation record with source_id
    let installation_record = InstallationRecord {
        skill_id: skill_id.clone(),
        repository: format!("https://github.com/{}/{}", source.owner, source.repo),
        subdirectory,
        installed_at: chrono::Utc::now().to_rfc3339(),
        source_id: source.id.clone(),
        files: downloaded_files,
    };

    let record_path = skill_dir.join("installation.json");
    let record_json = serde_json::to_string_pretty(&installation_record)
        .map_err(|e| format!("Failed to serialize installation record: {}", e))?;
    fs::write(&record_path, record_json)
        .map_err(|e| format!("Failed to write installation record: {}", e))?;

    // Emit completion
    let success_msg = if errors.is_empty() {
        format!("技能 '{}' 安装成功", skill_id)
    } else {
        format!("技能 '{}' 安装完成 ({} 个警告)", skill_id, errors.len())
    };

    let _ = app.emit(
        event_name,
        InstallProgress {
            skill_id: skill_id.clone(),
            stage: "complete".to_string(),
            progress: 100,
            message: success_msg.clone(),
            files_downloaded,
            files_total,
        },
    );

    Ok(InstallProgress {
        skill_id,
        stage: "complete".to_string(),
        progress: 100,
        message: success_msg,
        files_downloaded,
        files_total,
    })
}

async fn collect_all_files_impl(
    client: &reqwest::Client,
    source: &SkillSource,
    api_url: &str,
    files: &mut Vec<GitHubContent>,
) -> Result<(), String> {
    let response = client
        .get(api_url)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if status.as_u16() == 403 || status.as_u16() == 429 {
            return Err(format!(
                "GitHub API 限流 / 拒绝访问 ({}): {}。请设置 GITHUB_TOKEN 提升配额后重试。",
                status, body
            ));
        }
        return Err(format!("API returned error: {} ({})", status, body));
    }

    let contents: Vec<GitHubContent> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    for item in contents {
        match item.content_type.as_str() {
            "file" => {
                files.push(item);
            }
            "dir" => {
                // Recursively collect files from subdirectory
                let subdir_url = format!(
                    "https://api.github.com/repos/{}/{}/contents/{}",
                    source.owner, source.repo, item.path
                );
                Box::pin(collect_all_files_impl(client, source, &subdir_url, files)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn collect_all_files(
    client: &reqwest::Client,
    source: &SkillSource,
    api_url: &str,
    files: &mut Vec<GitHubContent>,
) -> Result<(), String> {
    collect_all_files_impl(client, source, api_url, files).await
}

// ============================================================================
// URL Security - Download Whitelist
// ============================================================================

const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &[
    "raw.githubusercontent.com",
    "objects.githubusercontent.com",
    "api.github.com",
];

fn is_allowed_download_url(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // Must be HTTPS
    if parsed.scheme() != "https" {
        log::warn!("URL rejected: not HTTPS - {}", url);
        return false;
    }

    // No credentials in URL (prevent user:pass@host attacks)
    if !parsed.username().is_empty() || parsed.password().is_some() {
        log::warn!("URL rejected: contains credentials - {}", url);
        return false;
    }

    // Host must be in whitelist
    let host = match parsed.host_str() {
        Some(h) => h.to_lowercase(),
        None => {
            log::warn!("URL rejected: no host - {}", url);
            return false;
        }
    };

    // Reject IP addresses (prevent DNS rebinding attacks)
    if host.parse::<std::net::IpAddr>().is_ok() {
        log::warn!("URL rejected: IP address not allowed - {}", url);
        return false;
    }

    let allowed = ALLOWED_DOWNLOAD_HOSTS.iter().any(|allowed| host == *allowed);
    if !allowed {
        log::warn!("URL rejected: host not in whitelist - {} (host: {})", url, host);
    }
    allowed
}

// ============================================================================
// File Download
// ============================================================================

async fn download_single_file(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
) -> Result<String, String> {
    // Security: Validate URL is from allowed domain
    if !is_allowed_download_url(url) {
        return Err(format!(
            "Download URL not allowed: {}. Only GitHub URLs are permitted.",
            url
        ));
    }

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    log::info!("Downloading file from: {}", url);

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read content: {}", e))?;

    fs::write(path, &bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[tauri::command]
pub async fn verify_remote_skill_install(
    source: SkillSource,
    skill_id: String,
    target_dir: String,
) -> Result<InstallVerification, String> {
    log::info!(
        "Verifying installation of skill {} from {}",
        skill_id,
        source.id
    );

    validate_skill_id(&skill_id)?;

    let skill_dir = PathBuf::from(&target_dir).join(&skill_id);
    let record_path = skill_dir.join("installation.json");

    let record_content = fs::read_to_string(&record_path)
        .map_err(|e| format!("installation.json not found: {}", e))?;

    let record: InstallationRecord = serde_json::from_str(&record_content)
        .map_err(|e| format!("Failed to parse installation record: {}", e))?;

    // Verify SKILL.md exists
    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_md_present = skill_md_path.exists();

    // Verify file count and SHA256
    let mut errors: Vec<String> = Vec::new();
    let mut sha256_verified = true;
    let mut total_size: u64 = 0;
    let mut files_verified: u32 = 0;

    for file_record in &record.files {
        let file_path = skill_dir.join(&file_record.path);

        if !file_path.exists() {
            errors.push(format!("Missing file: {}", file_record.path));
            sha256_verified = false;
            continue;
        }

        let current_sha256 = compute_sha256(&file_path)
            .unwrap_or_else(|_| "error".to_string());

        if current_sha256 != file_record.sha256 {
            errors.push(format!(
                "SHA256 mismatch for {}: expected {}, got {}",
                file_record.path, file_record.sha256, current_sha256
            ));
            sha256_verified = false;
        }

        total_size += file_record.size;
        files_verified += 1;
    }

    // Check for extra files not in record
    let recorded_paths: std::collections::HashSet<_> = record
        .files
        .iter()
        .map(|f| f.path.clone())
        .collect();

    if let Ok(entries) = fs::read_dir(&skill_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if file_name == "installation.json" {
                    continue;
                }

                // Check if this file is in our record
                if !recorded_paths.contains(file_name) {
                    if let Ok(metadata) = fs::metadata(&path) {
                        total_size += metadata.len();
                    }
                }
            }
        }
    }

    let success = skill_md_present && sha256_verified && errors.is_empty();

    Ok(InstallVerification {
        success,
        files_downloaded: files_verified,
        total_size,
        skill_md_present,
        sha256_verified,
        errors,
    })
}

#[tauri::command]
pub async fn verify_anthropic_skill_install(
    skill_id: String,
    target_dir: String,
) -> Result<InstallVerification, String> {
    let source = get_anthropic_official_source();
    verify_remote_skill_install(source, skill_id, target_dir).await
}

#[tauri::command]
pub async fn uninstall_remote_skill(
    source: SkillSource,
    skill_id: String,
    target_dir: String,
) -> Result<(), String> {
    log::info!(
        "Uninstalling skill {} from {}",
        skill_id,
        source.id
    );

    validate_skill_id(&skill_id)?;

    let target_path = PathBuf::from(&target_dir);
    let skill_dir = target_path.join(&skill_id);

    // Verify path is within target_dir.
    if !is_safe_subpath(&target_path, &skill_dir) {
        return Err("Path traversal detected: invalid skill directory".to_string());
    }

    if !skill_dir.exists() {
        return Err(format!("Skill '{}' is not installed", skill_id));
    }

    // Remove the skill directory
    fs::remove_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to remove skill directory: {}", e))?;

    log::info!("Successfully uninstalled skill: {}", skill_id);
    Ok(())
}

#[tauri::command]
pub async fn uninstall_anthropic_skill(
    skill_id: String,
    target_dir: String,
) -> Result<(), String> {
    let source = get_anthropic_official_source();
    uninstall_remote_skill(source, skill_id, target_dir).await
}

#[tauri::command]
pub async fn get_local_remote_skills(
    source: SkillSource,
    target_dir: String,
) -> Result<Vec<AnthropicSkill>, String> {
    log::info!(
        "Getting local skills from {} in: {}",
        source.id,
        target_dir
    );

    let skills_dir = PathBuf::from(&target_dir);

    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    let entries = fs::read_dir(&skills_dir)
        .map_err(|e| format!("Failed to read skills directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let skill_id = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Validate skill_id
        if validate_skill_id(&skill_id).is_err() {
            continue;
        }

        // Check for SKILL.md
        let skill_md_path = path.join("SKILL.md");
        let Some(skill_md_content) = fs::read_to_string(&skill_md_path).ok() else {
            continue;
        };

        let frontmatter = parse_frontmatter(&skill_md_content);

        // Check for installation.json
        let record_path = path.join("installation.json");
        let (installed_at, repository, subdirectory, source_id) =
            if let Ok(record_content) = fs::read_to_string(&record_path) {
                if let Ok(record) = serde_json::from_str::<InstallationRecord>(&record_content) {
                    (
                        Some(record.installed_at),
                        Some(record.repository),
                        Some(record.subdirectory),
                        Some(record.source_id),
                    )
                } else {
                    (None, None, None, None)
                }
            } else {
                (None, None, None, None)
            };

        // Count files
        let file_count = count_local_files(&path);

        // Build subdirectory based on source
        let subdir = subdirectory.unwrap_or_else(|| {
            if source.subdir_prefix.is_empty() {
                skill_id.clone()
            } else {
                format!("{}/{}", source.subdir_prefix, skill_id)
            }
        });

        skills.push(AnthropicSkill {
            id: skill_id,
            name: frontmatter
                .as_ref()
                .and_then(|f| f.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            description: frontmatter
                .as_ref()
                .and_then(|f| f.description.clone())
                .unwrap_or_default(),
            version: frontmatter.as_ref().and_then(|f| f.version.clone()),
            author: frontmatter.as_ref().and_then(|f| f.author.clone()),
            tags: frontmatter
                .as_ref()
                .and_then(|f| f.tags.clone())
                .unwrap_or_default(),
            dependencies: frontmatter
                .as_ref()
                .and_then(|f| f.dependencies.clone())
                .unwrap_or_default(),
            repository: repository.unwrap_or_else(|| {
                format!("https://github.com/{}/{}", source.owner, source.repo)
            }),
            subdirectory: subdir,
            default_branch: source.branch.clone(),
            file_count,
            size_bytes: 0,
            installed: true,
            installed_path: Some(path.to_string_lossy().to_string()),
            installed_at,
            source_id: source_id.unwrap_or_else(|| source.id.clone()),
        });
    }

    // Sort by name
    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    log::info!("Found {} local skills for source {}", skills.len(), source.id);
    Ok(skills)
}

#[tauri::command]
pub fn get_local_anthropic_skills(target_dir: String) -> Result<Vec<AnthropicSkill>, String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let source = get_anthropic_official_source();
        get_local_remote_skills(source, target_dir).await
    })
}

// ============================================================================
// Cache Functions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedSkills {
    skills: Vec<AnthropicSkill>,
    cached_at: i64,
}

fn is_cache_expired_internal(cache_path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(cache_path) {
        if let Ok(modified) = metadata.modified() {
            let modified_secs = modified
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let now = chrono::Utc::now().timestamp();
            return now - modified_secs > CACHE_TTL_SECS;
        }
    }
    true
}

fn is_cache_expired(_skills: &[AnthropicSkill]) -> bool {
    let cache_path = get_cache_path(&get_anthropic_official_source());
    is_cache_expired_internal(&cache_path)
}

fn load_cached_skills(cache_path: &Path) -> Result<Vec<AnthropicSkill>, String> {
    let content = fs::read_to_string(cache_path)
        .map_err(|e| format!("Failed to read cache: {}", e))?;

    let cached: CachedSkills = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse cache: {}", e))?;

    Ok(cached.skills)
}

fn cache_skills(skills: &[AnthropicSkill], cache_path: &Path) -> Result<(), String> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }

    let cached = CachedSkills {
        skills: skills.to_vec(),
        cached_at: chrono::Utc::now().timestamp(),
    };

    let content = serde_json::to_string_pretty(&cached)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;

    fs::write(cache_path, content)
        .map_err(|e| format!("Failed to write cache: {}", e))?;

    Ok(())
}

fn mark_installed_skills(mut skills: Vec<AnthropicSkill>) -> Vec<AnthropicSkill> {
    for skill in &mut skills {
        skill.installed = false;
        skill.installed_path = None;
        skill.installed_at = None;
    }
    skills
}

fn count_local_files(dir: &Path) -> u32 {
    let mut count = 0u32;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                count += 1;
            } else if entry.path().is_dir() {
                count += count_local_files(&entry.path());
            }
        }
    }

    count
}

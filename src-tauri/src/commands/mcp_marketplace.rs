// MCP Marketplace Commands - Tauri command handlers

use crate::models::{
    MCPServer, MCPSyncResult, MCPSyncTarget, PaginatedMCPServers, MCPSource,
};
use std::path::PathBuf;
use tokio::fs as async_fs;

#[tauri::command]
pub async fn get_mcp_sources() -> Result<Vec<MCPSource>, String> {
    Ok(get_preset_mcp_sources())
}

#[tauri::command]
pub async fn fetch_mcp_servers(
    source_id: String,
    page: u32,
    page_size: u32,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<PaginatedMCPServers, String> {
    // Get all sample servers
    let all_servers = get_sample_mcp_servers();
    
    // Apply filters
    let filtered: Vec<MCPServer> = all_servers
        .into_iter()
        .filter(|server| {
            // Source filter
            if server.source_id != source_id && source_id != "all" {
                return false;
            }
            
            // Category filter
            if let Some(cat) = &category {
                if !server.categories.iter().any(|c| 
                    c.to_lowercase().contains(&cat.to_lowercase())
                ) && !server.tags.iter().any(|t| 
                    t.to_lowercase().contains(&cat.to_lowercase())
                ) {
                    return false;
                }
            }
            
            // Keyword filter
            if let Some(kw) = &keyword {
                let kw_lower = kw.to_lowercase();
                if !server.name.to_lowercase().contains(&kw_lower)
                    && !server.description.to_lowercase().contains(&kw_lower)
                    && !server.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower))
                {
                    return false;
                }
            }
            
            true
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
    
    Ok(PaginatedMCPServers {
        items,
        total,
        page,
        page_size,
        total_pages: total_pages.max(1),
        has_next: page < total_pages.max(1),
        has_prev: page > 1,
    })
}

#[tauri::command]
pub async fn get_local_mcp_servers() -> Result<Vec<MCPServer>, String> {
    // Return empty for now - actual implementation would scan ~/.claude/, ~/.cursor/, etc.
    Ok(vec![])
}

#[tauri::command]
pub async fn install_mcp_server(
    server: MCPServer,
    install_dir: String,
) -> Result<MCPInstallResult, String> {
    let base_dir = if install_dir.is_empty() {
        dirs::home_dir()
            .map(|h| h.join(".mcp"))
            .unwrap_or_else(|| PathBuf::from(".mcp"))
    } else {
        PathBuf::from(&install_dir)
    };
    
    let servers_dir = base_dir.join("servers");
    
    // Create servers directory
    async_fs::create_dir_all(&servers_dir)
        .await
        .map_err(|e| format!("创建目录失败: {}", e))?;
    
    let server_dir = servers_dir.join(&server.name);
    
    // Check if already exists
    if server_dir.exists() {
        return Ok(MCPInstallResult {
            success: false,
            server_name: server.name.clone(),
            path: None,
            message: format!("MCP 服务器 '{}' 已存在", server.name),
            error: Some("Server already installed".to_string()),
        });
    }
    
    // Create server directory
    async_fs::create_dir_all(&server_dir)
        .await
        .map_err(|e| format!("创建服务器目录失败: {}", e))?;
    
    // Create server metadata file
    let metadata = serde_json::json!({
        "id": server.id,
        "name": server.name,
        "description": server.description,
        "author": server.author,
        "version": server.version,
        "categories": server.categories,
        "tags": server.tags,
        "source_id": server.source_id,
        "protocol": server.protocol,
        "npm_package": server.npm_package,
        "repository": server.repository,
        "installed_at": chrono::Local::now().to_rfc3339(),
        "install_path": server_dir.to_string_lossy(),
    });
    
    let metadata_path = server_dir.join("metadata.json");
    async_fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .await
        .map_err(|e| format!("写入元数据失败: {}", e))?;
    
    // Create server config file (clone name for reuse)
    let server_name = server.name.clone();
    let config = serde_json::json!({
        "mcpServers": {
            server_name: {
                "command": server.npm_package.as_ref().map(|_| "npx").unwrap_or("node"),
                "args": server.npm_package.as_ref().map(|p| vec![p.clone()]).unwrap_or_default(),
                "env": serde_json::json!({}),
            }
        }
    });
    
    let config_path = server_dir.join("config.json");
    async_fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .await
        .map_err(|e| format!("写入配置失败: {}", e))?;
    
    log::info!("MCP Server '{}' installed to {}", server.name, server_dir.display());
    
    Ok(MCPInstallResult {
        success: true,
        server_name: server.name.clone(),
        path: Some(server_dir.to_string_lossy().to_string()),
        message: format!("MCP 服务器 '{}' 安装成功", server.name),
        error: None,
    })
}

#[tauri::command]
pub async fn sync_mcp_to_target(
    server_name: String,
    install_dir: String,
    target: MCPSyncTarget,
) -> Result<MCPSyncResult, String> {
    let source_dir = if install_dir.is_empty() {
        dirs::home_dir()
            .map(|h| h.join(".mcp").join("servers").join(&server_name))
            .unwrap_or_else(|| PathBuf::from(".mcp/servers").join(&server_name))
    } else {
        PathBuf::from(&install_dir).join("servers").join(&server_name)
    };
    
    if !source_dir.exists() {
        return Err(format!("本地服务器目录不存在: {}", source_dir.display()));
    }
    
    // Expand target path (~)
    let target_path = expand_path(&target.path);
    
    // Create parent directory if needed
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建目标目录失败: {}", e))?;
        }
    }
    
    let method = target.method.as_str();
    
    match method {
        "symlink" => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source_dir, &target_path)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
            
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&source_dir, &target_path)
                .map_err(|e| format!("创建符号链接失败: {}", e))?;
            
            log::info!("Created symlink: {} -> {}", target_path.display(), source_dir.display());
        }
        _ => {
            // Copy directory
            copy_dir_recursive(&source_dir, &target_path).await?;
            log::info!("Copied server to: {}", target_path.display());
        }
    }
    
    Ok(MCPSyncResult {
        success: true,
        server_name,
        target_path: target_path.to_string_lossy().to_string(),
        method: target.method,
        message: format!("成功同步到 {}", target.name),
        error: None,
    })
}

#[tauri::command]
pub async fn get_mcp_sync_targets() -> Result<Vec<MCPSyncTarget>, String> {
    Ok(get_default_mcp_sync_targets())
}

#[tauri::command]
pub async fn add_mcp_sync_target(target: MCPSyncTarget) -> Result<MCPSyncTarget, String> {
    // Validate path
    let expanded = expand_path(&target.path);
    let exists = expanded.exists();
    
    Ok(MCPSyncTarget {
        id: target.id,
        name: target.name,
        path: target.path,
        method: target.method,
        is_valid: true,
        exists: Some(exists),
        config_file: target.config_file,
    })
}

#[tauri::command]
pub async fn remove_mcp_sync_target(_target_id: String) -> Result<(), String> {
    // In a real implementation, this would remove from persistent storage
    Ok(())
}

// Helper functions

fn get_preset_mcp_sources() -> Vec<MCPSource> {
    vec![
        MCPSource {
            id: "mcpmarket".to_string(),
            name: "MCPMarket.com".to_string(),
            name_zh: None,
            region: "mcp-specific".to_string(),
            url: "https://mcpmarket.com".to_string(),
            api_endpoint: "https://mcpmarket.com/api/v1/servers".to_string(),
            description: "Comprehensive MCP server marketplace with 10000+ servers".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(10000),
            requires_auth: Some(false),
        },
        MCPSource {
            id: "mcpservers-org".to_string(),
            name: "MCPServers.org".to_string(),
            name_zh: None,
            region: "mcp-specific".to_string(),
            url: "https://mcpservers.org".to_string(),
            api_endpoint: "https://api.mcpservers.org/v1/servers".to_string(),
            description: "Community-driven MCP server directory".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(5000),
            requires_auth: Some(false),
        },
        MCPSource {
            id: "mcplug".to_string(),
            name: "MCPlug.store".to_string(),
            name_zh: None,
            region: "mcp-specific".to_string(),
            url: "https://mcplug.store".to_string(),
            api_endpoint: "https://api.mcplug.store/v1/servers".to_string(),
            description: "Premium MCP plugins and integrations".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(2000),
            requires_auth: Some(false),
        },
        MCPSource {
            id: "agenticskills".to_string(),
            name: "AgenticSkills.io".to_string(),
            name_zh: None,
            region: "international".to_string(),
            url: "https://agenticskills.io".to_string(),
            api_endpoint: "https://api.agenticskills.io/mcp/servers".to_string(),
            description: "69,000+ AI agent skills and MCP servers".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(69000),
            requires_auth: Some(false),
        },
        MCPSource {
            id: "openclaw".to_string(),
            name: "OpenClaw / ClawHub".to_string(),
            name_zh: Some("爪哇市场".to_string()),
            region: "china".to_string(),
            url: "https://clawhub.ai".to_string(),
            api_endpoint: "https://clawhub.ai/api/mcp/servers".to_string(),
            description: "OpenClaw 官方 MCP 服务器市场".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(3000),
            requires_auth: Some(false),
        },
        MCPSource {
            id: "awesome-mcp".to_string(),
            name: "Awesome MCP Servers".to_string(),
            name_zh: Some("MCP服务器聚合".to_string()),
            region: "github".to_string(),
            url: "https://github.com/punksecurity/awesome-mcp-servers".to_string(),
            api_endpoint: "https://api.github.com/repos/punksecurity/awesome-mcp-servers/contents".to_string(),
            description: "GitHub aggregated MCP servers collection".to_string(),
            icon: None,
            is_available: true,
            last_checked: None,
            server_count: Some(800),
            requires_auth: Some(false),
        },
    ]
}

fn get_sample_mcp_servers() -> Vec<MCPServer> {
    vec![
        MCPServer {
            id: "filesystem".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "filesystem".to_string(),
            description: "Read, write, and manage files on your local filesystem".to_string(),
            long_description: Some("A comprehensive filesystem MCP server that provides secure file operations including read, write, delete, move, and list operations with full path support.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["development".to_string(), "tools".to_string()],
            tags: vec!["file".to_string(), "filesystem".to_string(), "storage".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-filesystem".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-filesystem".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: None,
            required_permissions: None,
            last_updated: Some("2026-06-01".to_string()),
            stars: Some(2500),
            downloads: Some(50000),
        },
        MCPServer {
            id: "github".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "github".to_string(),
            description: "Interact with GitHub repositories, issues, and pull requests".to_string(),
            long_description: Some("GitHub MCP server providing comprehensive access to GitHub's API including repository management, issue tracking, pull requests, and code search.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.2.0".to_string()),
            categories: vec!["development".to_string(), "tools".to_string()],
            tags: vec!["git".to_string(), "github".to_string(), "repository".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-github".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-github".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                    description: Some("GitHub personal access token with repo scope".to_string()),
                    required: true,
                    default_value: None,
                    example: Some("ghp_xxxxxxxxxxxx".to_string()),
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-06-10".to_string()),
            stars: Some(4200),
            downloads: Some(85000),
        },
        MCPServer {
            id: "brave-search".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "brave-search".to_string(),
            description: "Web search using Brave Search API".to_string(),
            long_description: Some("Brave Search MCP server for privacy-focused web searches with high-quality results.".to_string()),
            author: Some("Brave Software".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["search".to_string(), "productivity".to_string()],
            tags: vec!["search".to_string(), "web".to_string(), "brave".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-brave-search".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-brave-search".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "BRAVE_API_KEY".to_string(),
                    description: Some("Brave Search API key".to_string()),
                    required: true,
                    default_value: None,
                    example: Some("BSA-xxxxxxxxxxxx".to_string()),
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-05-28".to_string()),
            stars: Some(1200),
            downloads: Some(25000),
        },
        MCPServer {
            id: "slack".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "slack".to_string(),
            description: "Send messages and manage Slack channels".to_string(),
            long_description: Some("Slack MCP server for sending messages, managing channels, and integrating with your Slack workspace.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.1.0".to_string()),
            categories: vec!["productivity".to_string(), "business".to_string()],
            tags: vec!["slack".to_string(), "messaging".to_string(), "collaboration".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-slack".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-slack".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "SLACK_BOT_TOKEN".to_string(),
                    description: Some("Slack bot user OAuth token".to_string()),
                    required: true,
                    default_value: None,
                    example: Some("xoxb-xxxxxxxxxxxx".to_string()),
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-06-05".to_string()),
            stars: Some(1800),
            downloads: Some(35000),
        },
        MCPServer {
            id: "postgres".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "postgres".to_string(),
            description: "Interact with PostgreSQL databases".to_string(),
            long_description: Some("PostgreSQL MCP server providing database operations including query execution, schema inspection, and data manipulation.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["development".to_string(), "data".to_string()],
            tags: vec!["database".to_string(), "postgres".to_string(), "sql".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-postgres".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-postgres".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "DATABASE_URL".to_string(),
                    description: Some("PostgreSQL connection string".to_string()),
                    required: true,
                    default_value: None,
                    example: Some("postgresql://user:pass@localhost:5432/db".to_string()),
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-05-20".to_string()),
            stars: Some(2100),
            downloads: Some(42000),
        },
        MCPServer {
            id: "aws-kb-retrieval".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "aws-kb-retrieval".to_string(),
            description: "Search Amazon Bedrock Knowledge Bases".to_string(),
            long_description: Some("AWS Knowledge Bases retrieval server for searching Amazon Bedrock knowledge bases with semantic search capabilities.".to_string()),
            author: Some("AWS".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["data".to_string(), "ai".to_string()],
            tags: vec!["aws".to_string(), "bedrock".to_string(), "knowledge-base".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-aws-kb-retrieval".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-aws-kb-retrieval".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "AWS_ACCESS_KEY_ID".to_string(),
                    description: Some("AWS access key ID".to_string()),
                    required: true,
                    default_value: None,
                    example: None,
                },
                crate::models::EnvVar {
                    name: "AWS_SECRET_ACCESS_KEY".to_string(),
                    description: Some("AWS secret access key".to_string()),
                    required: true,
                    default_value: None,
                    example: None,
                },
                crate::models::EnvVar {
                    name: "AWS_REGION".to_string(),
                    description: Some("AWS region".to_string()),
                    required: false,
                    default_value: Some("us-east-1".to_string()),
                    example: None,
                },
            ]),
            required_permissions: Some(vec!["bedrock:Retrieve".to_string(), "bedrock:RetrieveAndGenerate".to_string()]),
            last_updated: Some("2026-06-08".to_string()),
            stars: Some(950),
            downloads: Some(18000),
        },
        MCPServer {
            id: "sentry".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "sentry".to_string(),
            description: "Monitor and manage Sentry error tracking".to_string(),
            long_description: Some("Sentry MCP server for accessing error reports, performance metrics, and project management features.".to_string()),
            author: Some("Sentry".to_string()),
            version: Some("2.0.0".to_string()),
            categories: vec!["development".to_string(), "security".to_string()],
            tags: vec!["sentry".to_string(), "monitoring".to_string(), "error-tracking".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-sentry".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-sentry".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "SENTRY_AUTH_TOKEN".to_string(),
                    description: Some("Sentry authentication token".to_string()),
                    required: true,
                    default_value: None,
                    example: Some("sntrys_xxxxxxxxxxxx".to_string()),
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-06-12".to_string()),
            stars: Some(1500),
            downloads: Some(30000),
        },
        MCPServer {
            id: "everart".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "everart".to_string(),
            description: "AI-powered image generation and editing".to_string(),
            long_description: Some("EverArt MCP server for generating and editing images using AI models.".to_string()),
            author: Some("EverArt".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["ai".to_string(), "productivity".to_string()],
            tags: vec!["image".to_string(), "ai".to_string(), "generation".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-everart".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-everart".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "EVERART_API_KEY".to_string(),
                    description: Some("EverArt API key".to_string()),
                    required: true,
                    default_value: None,
                    example: None,
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-05-15".to_string()),
            stars: Some(800),
            downloads: Some(15000),
        },
        MCPServer {
            id: "everything".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "everything".to_string(),
            description: "Windows desktop search using Everything".to_string(),
            long_description: Some("Windows-only MCP server that integrates with Everything for ultra-fast desktop file search.".to_string()),
            author: Some("Voidtools & MCP".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["search".to_string(), "productivity".to_string()],
            tags: vec!["windows".to_string(), "search".to_string(), "file".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-everything".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-everything".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: None,
            required_permissions: None,
            last_updated: Some("2026-04-20".to_string()),
            stars: Some(1100),
            downloads: Some(22000),
        },
        MCPServer {
            id: "fetch".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "fetch".to_string(),
            description: "Fetch web pages and content from URLs".to_string(),
            long_description: Some("HTTP fetch MCP server for retrieving web page content, supporting both HTML and text extraction.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["search".to_string(), "tools".to_string()],
            tags: vec!["web".to_string(), "fetch".to_string(), "http".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-fetch".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-fetch".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: None,
            required_permissions: None,
            last_updated: Some("2026-05-10".to_string()),
            stars: Some(2800),
            downloads: Some(55000),
        },
        MCPServer {
            id: "google-maps".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "google-maps".to_string(),
            description: "Location services and route planning".to_string(),
            long_description: Some("Google Maps MCP server for place search, directions, distance matrix, and geocoding services.".to_string()),
            author: Some("Google".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["productivity".to_string(), "business".to_string()],
            tags: vec!["google".to_string(), "maps".to_string(), "location".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-google-maps".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-google-maps".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: Some(vec![
                crate::models::EnvVar {
                    name: "GOOGLE_MAPS_API_KEY".to_string(),
                    description: Some("Google Maps API key".to_string()),
                    required: true,
                    default_value: None,
                    example: None,
                }
            ]),
            required_permissions: None,
            last_updated: Some("2026-06-01".to_string()),
            stars: Some(1600),
            downloads: Some(32000),
        },
        MCPServer {
            id: "memory".to_string(),
            source_id: "mcpmarket".to_string(),
            name: "memory".to_string(),
            description: "Persistent memory for AI agents".to_string(),
            long_description: Some("Memory MCP server that stores and retrieves information across sessions using vector embeddings.".to_string()),
            author: Some("Model Context Protocol".to_string()),
            version: Some("1.0.0".to_string()),
            categories: vec!["ai".to_string(), "tools".to_string()],
            tags: vec!["memory".to_string(), "vector".to_string(), "embeddings".to_string()],
            install_command: Some("npx @modelcontextprotocol/server-memory".to_string()),
            install_path: None,
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            npm_package: Some("@modelcontextprotocol/server-memory".to_string()),
            protocol: "stdio".to_string(),
            required_env_vars: None,
            required_permissions: None,
            last_updated: Some("2026-05-25".to_string()),
            stars: Some(3500),
            downloads: Some(70000),
        },
    ]
}

fn get_default_mcp_sync_targets() -> Vec<MCPSyncTarget> {
    vec![
        MCPSyncTarget {
            id: "claude-desktop".to_string(),
            name: "Claude Desktop".to_string(),
            path: "~/Library/Application Support/Claude/claude_desktop_config.json".to_string(),
            method: "symlink".to_string(),
            is_valid: true,
            exists: Some(dirs::home_dir()
                .map(|h| h.join("Library/Application Support/Claude/claude_desktop_config.json").exists())
                .unwrap_or(false)),
            config_file: Some("claude_desktop_config.json".to_string()),
        },
        MCPSyncTarget {
            id: "cursor-mcp".to_string(),
            name: "Cursor".to_string(),
            path: "~/.cursor/mcp.json".to_string(),
            method: "copy".to_string(),
            is_valid: true,
            exists: Some(dirs::home_dir()
                .map(|h| h.join(".cursor/mcp.json").exists())
                .unwrap_or(false)),
            config_file: Some("mcp.json".to_string()),
        },
        MCPSyncTarget {
            id: "openclaw-mcp".to_string(),
            name: "OpenClaw".to_string(),
            path: "~/.openclaw/mcp.json".to_string(),
            method: "copy".to_string(),
            is_valid: true,
            exists: Some(dirs::home_dir()
                .map(|h| h.join(".openclaw/mcp.json").exists())
                .unwrap_or(false)),
            config_file: Some("mcp.json".to_string()),
        },
        MCPSyncTarget {
            id: "cline-mcp".to_string(),
            name: "Cline".to_string(),
            path: "~/.cline/mcp_servers.json".to_string(),
            method: "copy".to_string(),
            is_valid: true,
            exists: Some(dirs::home_dir()
                .map(|h| h.join(".cline/mcp_servers.json").exists())
                .unwrap_or(false)),
            config_file: Some("mcp_servers.json".to_string()),
        },
        MCPSyncTarget {
            id: "zed-mcp".to_string(),
            name: "Zed".to_string(),
            path: "~/.config/zed/mcp.json".to_string(),
            method: "copy".to_string(),
            is_valid: true,
            exists: Some(dirs::home_dir()
                .map(|h| h.join(".config/zed/mcp.json").exists())
                .unwrap_or(false)),
            config_file: Some("mcp.json".to_string()),
        },
    ]
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

async fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    let src_clone = src.clone();
    let dst_clone = dst.clone();
    
    tokio::task::spawn_blocking(move || {
        copy_dir_recursive_sync(&src_clone, &dst_clone)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn copy_dir_recursive_sync(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
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

// Add missing type alias
use crate::models::MCPInstallResult;

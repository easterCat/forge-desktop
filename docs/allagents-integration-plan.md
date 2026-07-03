# AllAgents 集成方案 — 开发详情与开发计划

> 版本：v1.0 | 日期：2026-06-26 | 作者：Forge Desktop 架构团队

---

## 目录

1. [集成分析报告](#1-集成分析报告)
2. [详细开发方案](#2-详细开发方案)
3. [开发计划](#3-开发计划)

---

## 1. 集成分析报告

### 1.1 现有同步机制 vs AllAgents 核心功能对比

#### 现有机制的现状

当前 Forge Desktop 对 5 种内容类型（plugins、skills、agents、rules、mcps）采用**独立实现、各自同步**的方式：

| 维度 | 现有机制 | AllAgents |
|------|---------|-----------|
| **配置方式** | 代码内硬编码（`PRESET_MARKETPLACE_SOURCES`、`CliToolManager::get_supported_tools()`）+ 运行时 SQLite + 文件系统双存储 | 声明式 `workspace.yaml` 单一配置源 |
| **类型系统** | 4 套 Skill 接口、2 套 Plugin 接口，无统一抽象 | 统一的 Plugin 规范（包含 skills/agents/hooks/commands/.mcp.json） |
| **同步目标** | 仅支持 Claude Code、Cursor、Copilot 三个 CLI 工具，每个单独实现 | 支持 **23 个客户端**，包括 Claude/Copilot/Cursor/Codex/Windsurf/Cline 等 |
| **同步策略** | 5 种不同的同步逻辑（文件复制、JSON 注册、settings.json 操作、符号链接、直接写文件） | 统一的 sync orchestrator + transform engine，非破坏性增量同步 |
| **插件来源** | 3 个硬编码 GitHub 源 + 手动添加 | Marketplace 注册机制，支持 GitHub 仓库、本地路径、owner/repo 格式 |
| **依赖解析** | 无 | 内置冲突解决（同名 skill 重命名、优先级规则）+ 同步状态追踪 |
| **MCP 管理** | 5 个 Rust 命令文件 + 2 个服务文件，支持 health check、audit log | workspace.yaml 声明式配置 + 多客户端格式同步 + HTTP-to-stdio 代理 |
| **错误处理** | 各模块独立实现 | 统一的 git 错误分类、缓存去重、sync state 持久化 |

#### AllAgents 的优势

1. **声明式管理**：通过 `workspace.yaml` 一个文件即可定义所有插件/技能/MCP/客户端配置，替代分散的硬编码配置
2. **广泛客户端支持**：23 个客户端的同步开箱即用，无需为每个 CLI 工具编写适配层
3. **非破坏性同步**：`.allagents/sync-state.json` 追踪已同步文件，首次覆盖但不删除用户文件，后续仅删除 AllAgents 之前同步的文件
4. **Marketplace 生态**：标准化的插件发现和安装机制，GitHub 仓库即可作为 marketplace
5. **MCP 原生支持**：内置 HTTP-to-stdio 代理能力，解决不同客户端的 MCP 协议兼容问题
6. **冲突自动解决**：跨插件同名 skill 的自动重命名和优先级排序

#### AllAgents 的局限

1. **纯 CLI 工具，无 SDK**：所有操作通过命令行暴露，没有可编程 API，需要通过子进程调用集成
2. **无自定义生命周期钩子**：不提供 `init/start/stop` 等生命周期事件，仅有客户端原生 hooks（Claude/Copilot/Codex 支持）
3. **不直接管理 "rules"**：allagents 的核心概念是 skills、agents、hooks、commands、MCP servers，不包含 "rules" 类型
4. **无健康检查/审计日志**：不提供 MCP 服务的 health check 和 audit log 功能（Forge Desktop 已有）
5. **无 UI 集成**：纯命令行工具，需要 Forge Desktop 自行构建管理界面

### 1.2 核心概念映射

| AllAgents 概念 | Forge Desktop 模块 | 映射关系 | 适配策略 |
|---|---|---|---|
| **Plugin** | Plugins + Skills + Agents | 一个 allagents plugin 可包含 skills、agents、hooks、commands | Forge Desktop 的 "plugin" 需要拆分为：marketplace 来源管理（保留）+ 安装/同步（委托 allagents） |
| **Skill** (SKILL.md) | Skills | 直接对应 | 4 套 Skill 接口统一为 allagents 的 Skill 规范 |
| **Agent** (.md 文件) | Agents | 直接对应 | 保留部门分类逻辑，同步委托 allagents |
| **Hooks** | — (新增能力) | Forge Desktop 无对应模块 | 作为新能力引入，allagents 管理 hooks 的发现和分发 |
| **Commands** (斜杠命令) | — (新增能力) | Forge Desktop 无对应模块 | 作为新能力引入 |
| **MCP Servers** | MCP | 功能重叠但 allagents 更轻量 | **混合策略**：Forge Desktop 保留 health check/audit log/grouping 等高级功能，同步层委托 allagents |
| **Marketplace** | Plugins Market / Skill Sources / MCP Sources | 理念一致，实现不同 | 统一为 allagents marketplace 格式 |
| **Clients** | CliToolManager | 目标一致 | Forge Desktop 扩展支持的客户端列表为 allagents 的 23 个客户端 |
| **Rules** | Rules | allagents 无对应概念 | **保留独立实现**，通过 allagents 的 `workspace.files` 机制同步 rule 文件 |

### 1.3 技术可行性评估

| 评估项 | 结论 | 说明 |
|--------|------|------|
| **技术栈兼容** | ✅ 可行 | allagents 是 TypeScript CLI，Forge Desktop 是 Tauri (Rust + Vue)，通过子进程调用即可集成 |
| **平台支持** | ⚠️ 需验证 | allagents 基于 Bun 运行时，Windows 兼容性需测试（Node >=18 fallback 存在） |
| **功能覆盖** | ⚠️ 部分覆盖 | 核心同步功能覆盖好，但 health check、audit log、grouping 等高级功能需 Forge Desktop 保留 |
| **向后兼容** | ⚠️ 渐进式 | 需要渐进式迁移，不能一次性替换所有现有功能 |
| **工作量** | 🟡 中等 | 约 35-50 人天（详见第 3 节） |
| **风险等级** | 🟡 中等 | 主要风险：CLI 调用性能、Windows 兼容性、迁移期间的数据一致性 |

---

## 2. 详细开发方案

### 2.1 架构设计

#### 集成后系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Forge Desktop (Tauri)                      │
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐  │
│  │  PluginsView │  │ SkillsView  │  │   AgentsView/Rules   │  │
│  │  MCPView     │  │  (合并)     │  │      View            │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬───────────┘  │
│         │                │                     │               │
│  ┌──────▼────────────────▼─────────────────────▼───────────┐  │
│  │              Unified Plugin Store (Pinia)                │  │
│  │   统一类型: UnifiedPlugin = Skill | Agent | Rule | MCP  │  │
│  └──────────────────────┬──────────────────────────────────┘  │
│                         │                                     │
│  ┌──────────────────────▼──────────────────────────────────┐  │
│  │           AllAgents Service Layer (Rust)                │  │
│  │                                                         │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │  │
│  │  │ Config Mgr  │  │ CLI Executor │  │ Result Parse │  │  │
│  │  │ (YAML 生成)  │  │ (子进程调用)  │  │ (JSON 解析)  │  │  │
│  │  └─────────────┘  └──────┬───────┘  └──────────────┘  │  │
│  └──────────────────────────┼──────────────────────────────┘  │
│                             │                                  │
│  ┌──────────────────────────▼──────────────────────────────┐  │
│  │           Extended Services (保留)                       │  │
│  │                                                         │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │ Health   │ │ Audit    │ │ Grouping │ │ Rules    │  │  │
│  │  │ Check    │ │ Log      │ │ Manager  │ │ Manager  │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────┬───────────────────────────────┘
                              │ 子进程调用
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    AllAgents CLI                             │
│                                                               │
│  allagents workspace init / update / plugin / skill / mcp    │
│  workspace.yaml → .allagents/sync-state.json → 客户端目录     │
│                                                               │
│  支持的 23 个客户端:                                           │
│  Claude | Copilot | Cursor | Codex | Windsurf | Cline | ... │
└─────────────────────────────────────────────────────────────┘
```

#### 数据流

```
用户操作 (Vue UI)
    │
    ▼
Pinia Store (统一状态管理)
    │
    ▼
Tauri Command (IPC 调用)
    │
    ├──► AllAgentsService
    │       │
    │       ├── 1. 生成/更新 workspace.yaml
    │       ├── 2. 调用 allagents CLI (子进程)
    │       ├── 3. 解析 JSON 输出
    │       └── 4. 更新本地状态
    │
    ├──► HealthService (保留)
    │       └── 定期检查 MCP 服务健康状态
    │
    ├──► AuditService (保留)
    │       └── 记录操作审计日志
    │
    └──► RulesService (保留独立)
            └── rules 文件同步（通过 workspace.files 机制）
```

### 2.2 核心集成步骤

#### 步骤 1：环境准备与 AllAgents 安装 ✅ 已完成

```rust
// src-tauri/src/services/allagents_service.rs

use std::process::Command;
use std::path::PathBuf;

pub struct AllAgentsService {
    /// allagents CLI 可执行文件路径
    cli_path: PathBuf,
    /// 项目工作区路径
    workspace_path: PathBuf,
    /// workspace.yaml 文件路径
    config_path: PathBuf,
}

impl AllAgentsService {
    /// 检查 allagents 是否已安装，未安装则自动安装
    pub fn ensure_installed() -> Result<PathBuf, String> {
        // 1. 检查 PATH 中是否有 allagents
        // 2. 检查 npm 全局安装
        // 3. 自动执行: npm install -g allagents
        // 4. 返回可执行文件路径
    }

    /// 初始化工作区
    pub fn init_workspace(&self, from: Option<&str>) -> Result<(), String> {
        // 调用: allagents init <workspace_path> [--from <source>]
    }
}
```

**验收标准**：
- [x] `allagents --version` 可正常执行
- [x] `allagents init` 可成功创建 `.allagents/` 目录和 `workspace.yaml`

#### 步骤 2：编写 workspace.yaml 生成器 ✅ 已完成

```typescript
// src/services/allagents-config.ts (前端配置生成)

interface WorkspaceConfig {
  workspace?: {
    source?: string;
    files?: Array<string | { source: string; dest?: string }>;
  };
  repositories?: Array<{
    path: string;
    source?: string;
    repo?: string;
    description?: string;
  }>;
  plugins?: Array<string | {
    name: string;
    clients?: string[];
    install?: 'file' | 'native';
    exclude?: string[];
    skills?: string[] | { exclude: string[] };
    pin?: string;
  }>;
  clients?: string[];
  mcpServers?: Record<string, {
    type: 'http' | 'stdio';
    url?: string;
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    headers?: Record<string, string>;
    clients?: string[];
  }>;
}

// 将 Forge Desktop 的插件状态转换为 workspace.yaml
function generateWorkspaceConfig(state: PluginState): WorkspaceConfig {
  return {
    clients: state.syncTargets.map(t => t.clientKey),
    plugins: state.installedPlugins.map(p => ({
      name: p.allagentsSpec, // e.g., "plugin-name@owner/repo"
      clients: p.targetClients,
      skills: p.skillFilter,
    })),
    mcpServers: state.mcpServers.reduce((acc, server) => {
      acc[server.name] = {
        type: server.protocol,
        url: server.protocol === 'http' ? server.endpoint : undefined,
        command: server.protocol === 'stdio' ? server.command : undefined,
        args: server.protocol === 'stdio' ? server.args : undefined,
        env: server.envVars,
      };
      return acc;
    }, {} as Record<string, any>),
  };
}
```

**验收标准**：
- [ ] 能将现有插件配置转换为合法的 workspace.yaml
- [ ] 生成的 YAML 通过 `allagents plugin validate` 验证

#### 步骤 3：开发 Rust CLI 执行器 ✅ 已完成

```rust
// src-tauri/src/commands/allagents.rs

use tauri::command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AllAgentsResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
}

/// 同步所有插件到配置的客户端
#[command]
pub async fn allagents_update(
    workspace_path: String,
    offline: bool,
    dry_run: bool,
    client: Option<String>,
) -> Result<AllAgentsResult, String> {
    let mut args = vec!["update".to_string()];

    if offline { args.push("--offline".to_string()); }
    if dry_run { args.push("--dry-run".to_string()); }
    if let Some(c) = client {
        args.push("--client".to_string());
        args.push(c);
    }
    args.push("--json".to_string()); // 强制 JSON 输出

    let output = Command::new("allagents")
        .args(&args)
        .current_dir(&workspace_path)
        .output()
        .map_err(|e| format!("Failed to execute allagents: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let result: AllAgentsResult = serde_json::from_str(&stdout)
        .unwrap_or(AllAgentsResult {
            success: output.status.success(),
            output: stdout,
            errors: vec![],
        });

    Ok(result)
}

/// 安装插件
#[command]
pub async fn allagents_plugin_install(
    workspace_path: String,
    plugin_spec: String, // e.g., "code-review@claude-plugins-official"
    scope: Option<String>, // "user" or "project"
    skills: Option<Vec<String>>,
) -> Result<AllAgentsResult, String> {
    let mut args = vec![
        "plugin".to_string(),
        "install".to_string(),
        plugin_spec,
        "--json".to_string(),
    ];

    if let Some(s) = scope {
        args.push("--scope".to_string());
        args.push(s);
    }

    // 执行 allagents plugin install ...
}

/// 列出已安装的技能
#[command]
pub async fn allagents_skill_list(
    workspace_path: String,
    scope: Option<String>,
) -> Result<AllAgentsResult, String> {
    let mut args = vec![
        "skill".to_string(),
        "list".to_string(),
        "--json".to_string(),
    ];

    // 执行 allagents skill list ...
}

/// 添加 MCP 服务器
#[command]
pub async fn allagents_mcp_add(
    workspace_path: String,
    name: String,
    command_or_url: String,
    transport: Option<String>, // "http" or "stdio"
    client: Option<String>,
) -> Result<AllAgentsResult, String> {
    let mut args = vec![
        "mcp".to_string(),
        "add".to_string(),
        name,
        command_or_url,
        "--json".to_string(),
    ];

    if let Some(t) = transport {
        args.push("--transport".to_string());
        args.push(t);
    }
    if let Some(c) = client {
        args.push("--client".to_string());
        args.push(c);
    }

    // 执行 allagents mcp add ...
}
```

**验收标准**：
- [x] 所有 allagents 命令可通过 Tauri IPC 调用
- [x] JSON 输出解析正确，错误信息完整
- [x] 异步执行不阻塞 UI 线程

#### 步骤 4：统一类型系统 ✅ 已完成

```typescript
// src/types/unified-plugin.ts

/** 统一插件类型 —— 替代现有的 4 套 Skill 接口和 2 套 Plugin 接口 */
export interface UnifiedPlugin {
  id: string;
  name: string;
  description?: string;
  version?: string;

  // 来源信息
  source: PluginSource;
  scope: 'user' | 'project';

  // 内容分类
  type: 'skill' | 'agent' | 'rule' | 'mcp' | 'hook' | 'command';

  // 安装状态
  installed: boolean;
  enabled: boolean;
  installedAt?: string;
  installedPath?: string;

  // 同步信息
  syncTargets: SyncTarget[];
  syncStatus: 'synced' | 'pending' | 'error' | 'conflict';

  // 元数据
  tags: string[];
  categories: string[];

  // 客户端过滤
  targetClients: string[];

  // allagents 特有
  allagentsSpec?: string; // e.g., "plugin@owner/repo"
}

export interface PluginSource {
  type: 'marketplace' | 'github' | 'local' | 'url';
  marketplace?: string;
  repo?: string;
  path?: string;
  url?: string;
}

export interface SyncTarget {
  client: string; // e.g., "claude", "cursor", "copilot"
  path: string;
  status: 'synced' | 'pending' | 'error';
  lastSyncedAt?: string;
}

/** MCP 服务 —— 扩展 allagents 的 MCP 定义 */
export interface UnifiedMCP {
  name: string;
  type: 'http' | 'stdio';
  url?: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  headers?: Record<string, string>;

  // Forge Desktop 扩展功能
  groupIds: string[];
  tags: string[];
  healthStatus: 'healthy' | 'unhealthy' | 'unknown';
  lastHealthCheck?: string;
  auditLog: AuditEntry[];
}

export interface AuditEntry {
  timestamp: string;
  action: string;
  detail: string;
  userId?: string;
}

/** Rules —— 独立管理，通过 workspace.files 同步 */
export interface Rule {
  id: string;
  name: string;
  type: 'md' | 'mdc';
  content: string;
  filePath?: string;
  isActive: boolean;
  syncTargets: SyncTarget[];
}
```

**验收标准**：
- [ ] 所有现有 Store 迁移到 UnifiedPlugin 类型
- [ ] 旧类型标记为 `@deprecated`，6 个月内移除
- [ ] TypeScript 编译无错误

#### 步骤 5：开发 AllAgents Service Layer ✅ 已完成

```rust
// src-tauri/src/services/allagents_service.rs

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// AllAgents 服务层 —— 封装所有 allagents CLI 交互
pub struct AllAgentsService {
    workspace_path: PathBuf,
}

/// allagents update 的 JSON 输出结构
#[derive(Debug, Deserialize)]
struct SyncReport {
    synced_files: Vec<SyncedFile>,
    errors: Vec<SyncError>,
    skipped: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SyncedFile {
    source: String,
    destination: String,
    client: String,
    action: String, // "created" | "updated" | "unchanged"
}

#[derive(Debug, Deserialize)]
struct SyncError {
    file: String,
    client: String,
    error: String,
}

impl AllAgentsService {
    pub fn new(workspace_path: PathBuf) -> Self {
        Self { workspace_path }
    }

    /// 生成 workspace.yaml 配置文件
    pub fn write_config(&self, config: &WorkspaceConfig) -> Result<(), String> {
        let yaml = serde_yaml::to_string(config)
            .map_err(|e| format!("YAML serialization failed: {}", e))?;

        let config_path = self.workspace_path.join("workspace.yaml");
        std::fs::write(&config_path, yaml)
            .map_err(|e| format!("Failed to write workspace.yaml: {}", e))?;

        Ok(())
    }

    /// 执行 allagents update 并返回同步报告
    pub fn sync(
        &self,
        offline: bool,
        client_filter: Option<&str>,
    ) -> Result<SyncReport, String> {
        let mut cmd = std::process::Command::new("allagents");
        cmd.arg("update")
           .arg("--json")
           .current_dir(&self.workspace_path);

        if offline { cmd.arg("--offline"); }
        if let Some(client) = client_filter {
            cmd.arg("--client").arg(client);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to run allagents: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("allagents update failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse allagents output: {}", e))
    }

    /// 获取 workspace 状态
    pub fn status(&self) -> Result<WorkspaceStatus, String> {
        let output = std::process::Command::new("allagents")
            .args(["workspace", "status", "--json"])
            .current_dir(&self.workspace_path)
            .output()
            .map_err(|e| format!("Failed to get status: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse status: {}", e))
    }
}
```

**验收标准**：
- [x] workspace.yaml 生成正确
- [x] 同步报告解析完整
- [x] 错误处理覆盖所有 allagents CLI 错误场景

#### 步骤 6：迁移现有模块 ✅ 已完成

**Plugins 模块迁移**：
1. 保留 `plugin-marketplace.ts` 的 marketplace 来源管理逻辑
2. 将安装/同步逻辑委托给 AllAgentsService
3. 移除 `plugin_sync.rs` 中的硬编码同步逻辑

**Skills 模块迁移**：
1. 合并 5 个 Store 为 1 个 UnifiedSkillStore
2. 保留 Anthropic Skills 的特殊缓存逻辑（作为 marketplace 来源）
3. 移除 `skill-import.ts` 和 `skills-sh.ts`，改为 allagents marketplace

**Agents 模块迁移**：
1. 保留部门分类逻辑
2. 同步委托给 AllAgentsService
3. 扩展客户端支持（从 3 个 → 23 个）

**MCP 模块迁移**：
1. 保留 health check、audit log、grouping 功能
2. 同步层委托给 AllAgentsService
3. 移除 `mcp_bridge.rs` 和 `mcp_protocol.rs`（allagents 内置代理）

**Rules 模块增强**：
1. 补全 RulesView（替换 mock 数据）
2. 通过 `workspace.files` 机制同步 rule 文件
3. 保留 Rules 独立的 CRUD 管理

**验收标准**：
- [x] 每个模块的现有功能在迁移后正常工作
- [x] 无功能回退（所有现有特性在新架构下可用）
- [x] 单元测试覆盖率达到 80%

### 2.3 接口定义

#### Tauri Commands（前端 → 后端）

```typescript
// 统一的 AllAgents 命令接口

// 工作区管理
invoke('allagents_init', { workspacePath, from?: string })
invoke('allagents_update', { offline?, dryRun?, client?: string })
invoke('allagents_status', {})

// 插件管理
invoke('allagents_plugin_install', { pluginSpec, scope?, skills?: string[] })
invoke('allagents_plugin_uninstall', { pluginSpec })
invoke('allagents_plugin_list', { marketplace?: string })

// 技能管理
invoke('allagents_skill_list', { scope?: string })
invoke('allagents_skill_add', { name, from?, plugin?, scope?: string })
invoke('allagents_skill_remove', { name, plugin?, scope?: string })

// MCP 管理
invoke('allagents_mcp_add', { name, commandOrUrl, transport?, client? })
invoke('allagents_mcp_remove', { name })
invoke('allagents_mcp_list', {})
invoke('allagents_mcp_update', { offline? })

// Marketplace 管理
invoke('allagents_marketplace_add', { source, name?, branch? })
invoke('allagents_marketplace_remove', { name })
invoke('allagents_marketplace_list', {})
```

#### 事件流（后端 → 前端）

```typescript
// Tauri 事件定义
interface AllAgentsEvents {
  // 同步进度
  'allagents:sync-progress': {
    phase: 'cloning' | 'transforming' | 'copying' | 'registering';
    current: number;
    total: number;
    message: string;
  };

  // 同步完成
  'allagents:sync-complete': {
    success: boolean;
    syncedCount: number;
    errorCount: number;
    skippedCount: number;
  };

  // 同步错误
  'allagents:sync-error': {
    file: string;
    client: string;
    error: string;
    recoverable: boolean;
  };

  // 配置变更
  'allagents:config-changed': {
    section: 'plugins' | 'skills' | 'mcp' | 'clients';
    action: 'added' | 'removed' | 'updated';
  };
}
```

#### workspace.yaml 标准模板

```yaml
# Forge Desktop 项目 workspace.yaml 模板
# 由 AllAgentsService 自动生成

workspace:
  source: .forge/config

repositories:
  - path: .
    description: Forge Desktop 主项目
    skills:
      - .forge/skills

plugins:
  # Marketplace 来源的插件
  - code-review@claude-plugins-official
  - name: superpowers@obra/superpowers
    clients: [claude, cursor]
    skills:
      - brainstorming
      - test-driven-development

  # 本地插件
  - name: forge-custom
    source: .forge/plugins/custom

clients:
  - claude
  - copilot
  - cursor
  - codex
  - windsurf
  - cline

mcpServers:
  forge-server:
    type: stdio
    command: node
    args:
      - .forge/mcp-server/index.js
    env:
      FORGE_HOME: ${FORGE_HOME}

# Rules 通过 workspace.files 同步
workspace:
  files:
    - source: .forge/rules/
      dest: CLAUDE.md
    - source: .forge/rules/agents.md
      dest: AGENTS.md
```

---

## 3. 开发计划

### 3.1 里程碑划分

```
M1: 调研准备 (1 周)
│
M2: 核心框架集成 (2 周)
│
M3: 插件逐个迁移适配 (3 周)
│
M4: 集成测试与优化 (1.5 周)
│
M5: 文档编写与发布 (0.5 周)
│
─────────────────────────
总计: ~8 周 (约 35-50 人天)
```

### 3.2 任务分解

#### M1: 调研准备（第 1 周）

| 任务 | 负责人 | 人天 | 交付物 |
|------|--------|------|--------|
| allagents CLI 功能验证测试 | 后端 | 2 | 测试报告 |
| Windows 平台兼容性测试 | 后端 | 1 | 兼容性报告 |
| 现有模块代码审计 | 全栈 | 1 | 审计报告 |
| 迁移方案细化 | 架构师 | 1 | 详细迁移文档 |

**验收标准**：
- [ ] allagents 在 Windows 上正常运行（含 npm install -g）
- [ ] 现有所有功能点清单完成
- [ ] 迁移方案经团队评审通过

#### M2: 核心框架集成（第 2-3 周）

| 任务 | 负责人 | 人天 | 交付物 |
|------|--------|------|--------|
| AllAgentsService 开发 | 后端 | 3 | Rust 服务模块 |
| workspace.yaml 生成器 | 全栈 | 2 | 配置生成逻辑 |
| Tauri Commands 封装 | 后端 | 2 | 10+ 个 IPC 命令 |
| 统一类型系统设计 | 前端 | 2 | UnifiedPlugin 类型定义 |
| 事件流系统 | 全栈 | 1 | 事件定义和监听 |

**验收标准**：
- [ ] `allagents init` → `allagents update` 流程端到端通过
- [ ] workspace.yaml 自动生成正确
- [ ] 所有 Tauri Commands 可调用并返回正确结果

#### M3: 插件逐个迁移适配（第 4-6 周）

| 任务 | 负责人 | 人天 | 交付物 |
|------|--------|------|--------|
| Skills 模块迁移（5 Store → 1） | 前端 | 4 | UnifiedSkillStore |
| Plugins 模块迁移 | 全栈 | 3 | PluginStore 重构 |
| Agents 模块迁移 | 全栈 | 2 | AgentStore 重构 |
| MCP 模块迁移 | 全栈 | 3 | MCPStore 重构（保留高级功能） |
| Rules 模块增强 | 全栈 | 2 | RulesView 补全 + 同步 |
| UI 适配（SkillsView 合并） | 前端 | 3 | 统一的插件管理界面 |
| 客户端支持扩展（23 个） | 后端 | 2 | 客户端选择 UI + 同步逻辑 |

**验收标准**：
- [ ] 每个模块的现有功能在新架构下正常工作
- [ ] 所有 PENDING 注释的功能点已实现
- [ ] 无功能回退

#### M4: 集成测试与优化（第 7-8.5 周）

| 任务 | 负责人 | 人天 | 交付物 |
|------|--------|------|--------|
| 端到端集成测试 | 测试 | 3 | 测试用例 + 报告 |
| 性能测试（同步速度） | 后端 | 1 | 性能基准报告 |
| 错误处理和边界情况 | 全栈 | 2 | 错误处理完善 |
| 向后兼容性验证 | 全栈 | 1 | 兼容性报告 |
| Bug 修复 | 全栈 | 2 | Bug fix commits |

**验收标准**：
- [ ] 所有测试用例通过
- [ ] 同步 100 个插件耗时 < 30 秒
- [ ] 无 P0/P1 级别 Bug

#### M5: 文档编写与发布（第 8.5-9 周）

| 任务 | 负责人 | 人天 | 交付物 |
|------|--------|------|--------|
| 开发文档更新 | 全栈 | 1 | 更新后的 CLAUDE.md |
| 用户使用指南 | 前端 | 1 | 使用文档 |
| 迁移指南（旧 → 新） | 架构师 | 0.5 | 迁移指南 |
| 发布准备 | 全栈 | 0.5 | Release notes |

**验收标准**：
- [ ] 所有文档已更新
- [ ] 用户可按指南完成插件安装和同步

### 3.3 资源与依赖

#### 人力需求

| 角色 | 人数 | 投入比例 | 职责 |
|------|------|---------|------|
| 架构师 | 1 | 20% | 方案设计、评审、技术决策 |
| 后端开发 (Rust) | 1-2 | 80% | AllAgentsService、Tauri Commands |
| 前端开发 (Vue) | 1-2 | 80% | Store 迁移、UI 适配、类型系统 |
| 测试 | 1 | 40% | 集成测试、兼容性测试 |

#### 测试环境

- **开发环境**：Windows 11 + Node.js >= 18 + npm
- **allagents 安装**：`npm install -g allagents`
- **测试客户端**：Claude Code、Cursor、Copilot（至少 3 个）
- **测试项目**：至少 2 个不同规模的项目（小型 < 10 插件、大型 > 50 插件）

#### 第三方依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| allagents | >= 1.12.0 | CLI 工具 |
| js-yaml | ^4.1.0 | YAML 解析（前端配置生成） |
| zod | ^3.22.0 | workspace.yaml 校验 |
| serde_yaml | ^0.9 | Rust YAML 序列化 |

### 3.4 验收标准总览

#### 功能验收

| 功能点 | 验收标准 | 优先级 |
|--------|---------|--------|
| workspace.yaml 自动生成 | 从现有配置正确生成，通过 allagents validate | P0 |
| 插件安装/卸载 | 通过 allagents CLI 安装到指定客户端 | P0 |
| 技能同步 | 技能文件同步到所有配置的客户端目录 | P0 |
| MCP 服务器管理 | 添加/移除/同步 MCP 服务器配置 | P0 |
| 客户端选择 | 支持选择 23 个客户端中的任意子集 | P1 |
| Rules 同步 | Rules 文件通过 workspace.files 机制同步 | P1 |
| 健康检查 | MCP 服务器定期健康检查（保留现有功能） | P1 |
| 审计日志 | 操作审计日志记录（保留现有功能） | P2 |
| Hooks 管理 | 发现和分发客户端原生 hooks | P2 |

#### 非功能验收

| 维度 | 验收标准 |
|------|---------|
| **性能** | 同步 100 个插件 < 30 秒 |
| **可靠性** | allagents CLI 调用失败率 < 1% |
| **兼容性** | Windows 10/11 + macOS + Linux |
| **可维护性** | 新增客户端支持仅需修改配置，无需代码变更 |
| **可扩展性** | 新增插件类型仅需实现 adapter，无需修改核心框架 |
| **用户体验** | 进度条显示同步状态，错误信息清晰可操作 |

---

## 附录

### A. 风险清单与缓解措施

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| allagents Windows 兼容性问题 | 中 | 高 | M1 阶段提前验证；准备 Node.js fallback 方案 |
| CLI 调用性能瓶颈 | 中 | 中 | 批量操作替代逐个调用；缓存机制 |
| 迁移期间数据不一致 | 中 | 高 | 渐进式迁移；保留旧接口作为 fallback |
| allagents 上游变更 | 低 | 中 | 锁定版本；监控 release notes |
| 现有功能回退 | 低 | 高 | 完善的回归测试；功能开关机制 |

### B. 迁移兼容性策略

**Phase 1（M2-M3）**：新旧接口并存
- AllAgentsService 和原有 services 同时存在
- 通过 feature flag 控制使用哪个后端
- 用户可随时切换回旧实现

**Phase 2（M4）**：验证切换
- 默认使用 AllAgentsService
- 旧接口作为 fallback
- 收集用户反馈

**Phase 3（M5+）**：移除旧接口
- 确认稳定后移除旧 services
- 清理遗留代码

### C. 关键技术决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| allagents 集成方式 | 子进程调用 | allagents 是纯 CLI 工具，无 SDK |
| Rules 管理方式 | 独立管理 + workspace.files 同步 | allagents 不直接支持 rules 概念 |
| MCP 高级功能 | 保留 Forge Desktop 实现 | allagents 无 health check/audit log |
| 类型系统 | 统一为 UnifiedPlugin | 消除现有 4 套 Skill 接口的碎片化 |
| 迁移策略 | 渐进式 + feature flag | 降低迁移风险，保证业务连续性 |

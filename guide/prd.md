# Forge — 产品需求文档 (PRD)

**版本**: 2.4.0
**日期**: 2026-06-18
**作者**: Forge Team
**状态**: 草稿

---

## 变更说明（v2.3.0 → v2.4.0）

> **Software Management 二义性澄清**：PRD 中 "Software Management" 涵盖两条平行的产品线——**CLI 工具管理（FEAT-01）**和**软件检测（FEAT-02）**。两者共用 `software` store（`src/stores/software.ts`）和同处"Software Management"功能域，但在命令层、持久化策略和 UX 流程上有本质差异。本版本在文档中显式区分二者，避免后续读者混淆。

| 类型 | 说明 |
|------|------|
| **Software Management 二义性澄清** | 见上方引用块。本版本在文档中显式区分 CLI 工具管理（FEAT-01）和软件检测（FEAT-02）两条产品线。 |
| **CLI 工具管理补全（FEAT-01）** | `CliToolStatus` 字段表补全（`latestVersion`、`needsUpgrade`、`hasConflict`、`conflictInfo`、`installMethod`）；新增 7 阶段操作状态机（`preparing` / `downloading` / `installing` / `verifying` / `completed` / `failed` / `cancelled`，来自 `src/composables/useOperationProgress.ts` L3）；"Checking…" pending 占位（`isPendingStatus` + 脉冲徽章 / 卡片动画）；`pendingRetry` 语义（取消 / 重试复用）；Methods tooltip Teleport + ESC / 点击外部关闭；install options Modal（用户点击 Install 时可选安装方式）；冲突检测（npm 全局包 > 10 → 顶部 `alert-warning`）；CLI 工具列表从 7 个扩展为 9 个，新增 `deepseek-reasonix` / `kiro` / `mimo-code`（`services/cli_tools.rs` L164-337）；二进制检测映射（`claude` / `gemini` / `cursor-agent` / `agent`，`binary_name_candidates` L866-894）。本次无代码变更，仅文档更新。 |
| **软件检测补全（FEAT-02）** | 补全 `selectedPlatform` + `setSelectedPlatform`（持久化到 `localStorage['forge-selected-platform']`，默认 `auto`，`software.ts` L82-89）；`detectSoftware` 后台空闲调度（`runWhenIdle` 包裹，`software.ts` L13-19 / L191-194 / L207-210）；`sync_software` 语义澄清（`commands/software.rs` L44-55：并行扫描本地软件 + 批量 upsert 到 DB，用于与远端配置同步）；`update_software` 触发 `runWhenIdle`（`software.ts` L242）；`install_software` / `uninstall_software` 完成后触发 `runWhenIdle`（`software.ts` L191 / L209）。本次无代码变更，仅文档更新。 |
| **useOperationProgress 组合式函数文档化（v2.4.0 新增）** | 文件位置：`src/composables/useOperationProgress.ts`（148 行）。暴露 API：`operations`（Map ref）/ `activeOperation`（computed）/ `isAnyActive`（computed）/ `startOperation(key)` / `updateProgress(key, stage, progress, message)` / `completeOperation(key, success, message)` / `cancelOperation(key)` / `retryOperation(key)` / `getOperation(key)` / `clearCompleted()`。8 个 `OperationStage`：idle / preparing / downloading / installing / verifying / completed / failed / cancelled。`STAGE_CONFIG` 阶段配置（icon + label）；`canCancel` 仅在 downloading / installing 阶段为 true；`canRetry` 在 failed / cancelled 阶段为 true；取消 / 重试 cross-fade 切换。当前用于 CLI 工具安装 / 升级，后续可被 PluginsView / SoftwareManagementView 复用。 |
| **行号锚点** | `software.ts` L13-19 runWhenIdle / L82-89 selectedPlatform / L191-194 installSoftware idle / L207-210 uninstallSoftware idle / L242 updateSoftware idle；`CliToolsView.vue` L57-62 v-memo + is-pending / L356-364 isPendingStatus / L615-617 pendingRetry / L681-683 handleUpgrade / L694-703 handleRetry；`useOperationProgress.ts` L1-148（8 阶段枚举 L3，API 列表 L16-27，STAGE_CONFIG L138-147）；`commands/software.rs` L44-55 sync_software / L57-68 install_software / L64-68 uninstall_software / L71-100 update_software；`commands/cli_tools.rs` L99-166（5 命令）；`services/cli_tools.rs` L164-337（9 个工具，L866-894 binary_name_candidates）。 |


---

## 变更说明（v2.2.0 → v2.3.0）

|| 类型 | 说明 |
||------|------|
|| **新增 4 个 Vue 组件文档化** | `PluginCard.vue`（插件卡片，含进度条/source badge）、`PluginDetailsDialog.vue`（详情弹窗，含能力探测/Hook执行/MCP探测/验证）、`SourceNoteDialog.vue`（Markdown 备注弹窗）、`AddRepoSourceDialog.vue`（添加来源弹窗，含 repo_type 选择器）。以上组件在 v2.2.0 已有实现，本次补全文档。 |
|| **SourceNote 备注功能** | 完整链路：后端 `services/plugin_marketplace.rs` 新增 `SourceNotesRegistry`/`read_source_notes()`/`write_source_notes()`；`commands/plugin_marketplace.rs` 新增 `get_source_notes`/`save_source_note` 命令；`stores/plugin-marketplace.ts` 新增 `sourceNotes` state + `loadSourceNotes`/`saveSourceNote` action；`src/utils/markdown.ts` 实现轻量 Markdown 渲染器（无外部依赖，12 种语法）。 |
|| **source_notes.json 持久化** | 存储路径 `$FORGE_HOME/plugins/source_notes.json`（`~/.forge/plugins/source_notes.json`），JSON 结构 `{ "version": "1", "notes": { "source_id": "# markdown" } }`。 |
|| **repo_type 切换功能** | 用户来源支持在 `market`（多插件仓库）和 `res`（单插件仓库）之间切换；前端 Sources Tab 以切换按钮组呈现；预置来源不可切换/移除。 |
|| **marketplace 搜索（FEAT-021）** | Marketplace Tab 新增防抖搜索栏（300ms），支持按名称/描述过滤；无结果显示友好空状态；ESC/按钮清空。 |
|| **来源安装路径展示** | Sources Tab 中每个来源卡片显示已安装路径列表，含"安装位置"标题和 formatInstallPath 截断。 |
|| **来源安装进度条** | Sources Tab 安装过程显示进度条和状态消息。 |
|| **来源"全部安装"按钮** | Sources Tab 头部一键安装所有未安装来源。 |
|| **PluginDetailsDialog 能力扩展** | 新增：5 种能力统计卡片（Skills/Hooks/Commands/MCP/LSP）、Hook 执行测试、MCP 连接探测、路径验证报告、6 个 Tab（Overview/Skills/Commands/Hooks/MCP/LSP）。 |
|| **sync_records.json 顶层优化（FEAT-021）** | `copy_dir_inner` 只记录顶层条目，`remove_synced_files` 通过 `is_dir()` 判断目录；目标从 ~322KB 降至 ~5-10KB。 |
|| **新增 3 个 Tauri 命令** | `get_source_notes`、`save_source_note`、`update_source_repo_type`（已在 v2.2.0 提及，本次补全详细文档）。 |
|| **store 数量确认** | 仍为 17 个（`sourceNotes` 等为 state 字段，非独立 store）。 |

---

## 变更说明（v2.1.0 → v2.2.0）

| 类型 | 说明 |
|------|------|
| **Agents 模块已实现** | 第 10 部分从"计划"改为"已实现"：10 个 Tauri 命令（agent.rs）、Pinia store（agent.ts）、11 部门静态定义、3 个 Vue 组件（AgentCard / AgentDetailsDialog / AgentImportDialog）、AgentsView.vue、自动导入逻辑、Sidebar 入口。安装目标仅实现 claude-code / cursor / copilot 三个；opencode / windsurf / codex / openclaw 标注为待实现。 |
| **新增 MCP Manager 模块（FEAT-022）** | 独立命令模块 `commands/mcp_manager.rs`（11 命令）；新增 4 张数据库表（mcp_health_log、mcp_groups、mcp_service_groups、mcp_audit_log）；新增 11 个 Vue 组件（`src/components/mcp/*`）；`mcp_protocol.rs` 实现 stdio/HTTP 协议处理和 DiscoveryCache。 |
| **视图数修正** | 10 → **11**（新增 `AgentsView.vue`） |
| **Store 数修正** | 16 → **17**（新增 `src/stores/agent.ts`） |
| **命令模块文件数修正** | 21 → **22**（新增 `agent.rs` 和 `mcp_manager.rs`；移除重复的 `plugin_sync.rs`） |
| **命令总数修正** | ~137 → **162**（新增约 25 个命令） |
| **数据库表数修正** | 7 → **11**（新增 mcp_health_log、mcp_groups、mcp_service_groups、mcp_audit_log、agents） |
| **表名修正** | `backup_records` → `backups`（实际运行时使用 `connection.rs` 第 194 行的 `backups` 表名；`schema.rs` 中的 `backup_records` 为参考未使用） |
| **plugin_marketplace 命令数** | 17 → **22**（新增 `update_source_repo_type`、`get_source_notes`、`save_source_note`） |
| **file 命令数修正** | 3 → **10**（`read_file`、`write_file`、`list_directory` 等 10 个命令） |
| **skill_marketplace 命令数修正** | 12 → **11** |
| **anthropic_skills 命令数修正** | 13 → **12** |

---

## 第 1 部分：项目元信息

### 1.1 产品定位

Forge 是一款跨平台（macOS / Windows）的桌面应用，用于集中管理 AI 开发工具的配置生态系统。产品覆盖 Cursor IDE 及其他主流 AI Coding 工具，支持插件、技能、MCP 服务器、规则文件和用户配置的安装、更新、编辑、备份与共享。

### 1.2 版本信息

| 字段 | 值 |
|------|-----|
| 应用名称 | Forge |
| 标识符 | `com.forge.desktop` |
| 版本 | `0.1.0`（tauri.conf.json） |
| Tauri | 2.0 |
| Rust Edition | 2021 |

### 1.3 核心价值主张

| 价值维度 | 描述 |
|---------|------|
| **统一管理** | 一站式管理多个 AI 工具的配置，无需在多个目录间切换 |
| **版本控制** | 配置文件版本化，支持回滚和变更追踪 |
| **团队协作** | 配置模板市场，支持导入/导出和团队共享 |
| **隐私优先** | 本地优先架构，所有数据存储在本地，无云端依赖 |
| **跨平台一致** | macOS 与 Windows 提供统一的使用体验 |

### 1.4 目标用户画像

| 用户类型 | 使用场景 | 核心需求 |
|---------|---------|---------|
| **个人开发者** | 日常维护自己的 AI 工具配置 | 快速安装/更新/备份，避免重复配置 |
| **AI 爱好者** | 尝试新的插件和技能 | 发现新资源，一键安装 |
| **技术团队** | 统一团队开发环境 | 共享配置模板，确保环境一致性 |
| **DevOps 工程师** | 管理多台机器的配置 | 批量部署和配置同步 |

---

## 第 2 部分：技术架构

### 2.1 技术栈

| 层级 | 技术选型 | 版本 |
|-----|---------|------|
| **框架** | Tauri | 2.0 |
| **前端** | Vue 3 + TypeScript + Vite | Vue 3.5.13 / TypeScript 5.8.3 / Vite 6.3.5 |
| **状态管理** | Pinia | 3.0.2 |
| **样式** | Tailwind CSS | 3.4.17 |
| **后端** | Rust | (Cargo.toml) |
| **存储** | SQLite (rusqlite) | 0.32 (bundled) |
| **异步运行时** | Tokio | 1.0 (full) |
| **HTTP 客户端** | reqwest | 0.12 |

### 2.2 项目结构

```
forge-desktop/
├── src/                          # Vue 3 前端
│   ├── App.vue                   # 根组件
│   ├── main.ts                   # 入口文件
│   ├── router/index.ts           # 路由配置（11 个路由）
│   ├── views/                    # 11 个页面视图
│   │   ├── DashboardView.vue     # 仪表盘
│   │   ├── CliToolsView.vue      # CLI 工具管理
│   │   ├── SoftwareManagementView.vue  # 软件检测
│   │   ├── PluginsView.vue       # 插件管理
│   │   ├── SkillsView.vue        # 技能管理
│   │   ├── MCPView.vue           # MCP 服务器
│   │   ├── RulesView.vue         # 规则管理
│   │   ├── BackupView.vue        # 备份恢复
│   │   ├── SettingsView.vue      # 设置
│   │   ├── PromptManagerView.vue  # 提示词管理
│   │   └── AgentsView.vue        # Agents 管理（v2.2.0 新增）
│   ├── stores/                   # 17 个 Pinia Store
│   ├── components/               # UI 组件
│   ├── composables/              # 组合式函数
│   └── types/                    # TypeScript 类型定义
├── src-tauri/                   # Rust 后端
│   ├── src/
│   │   ├── lib.rs               # 命令注册中心 + invoke_handler
│   │   ├── main.rs              # 应用入口
│   │   ├── commands/            # 22 个命令模块（按功能分组）
│   │   ├── services/            # 核心业务逻辑服务
│   │   ├── models/              # 数据模型（6 个子模块）
│   │   └── db/                  # SQLite 数据库
│   ├── Cargo.toml               # Rust 依赖
│   └── tauri.conf.json          # Tauri 配置
└── docs/                        # 文档
```

### 2.3 数据存储路径

| 路径 | 说明 |
|------|------|
| `~/.local/share/forge/` | 应用数据目录 |
| `~/.local/share/forge/logs/` | 日志文件 |
| `~/.local/share/forge/forge.db` | SQLite 数据库 |
| `~/.local/share/forge/marketplace/` | 市场插件缓存 |
| `~/.local/share/forge/plugins/cache/` | 插件缓存 |
| `~/.config/Code/User/` | VS Code 配置 |
| `~/.cursor/` | Cursor 配置 |
| `~/.windsurf/` | Windsurf 配置 |
| `~/.claude/` | Claude Desktop 配置 |
| `~/.continue/` | Continue 配置 |
| `~/.cody/` | Cody 配置 |

### 2.4 系统架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                         Forge v0.1.0                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   UI Layer (Vue 3 + Pinia)                 │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐ │  │
│  │  │Dashboard│ │  CLI   │ │Software│ │Plugins │ │Skills│ │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └──────┘ │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐ │  │
│  │  │  MCP   │ │ Rules  │ │Backup  │ │Settings│ │Prompts│ │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └──────┘ │  │
│  │  ┌────────┐                                              │  │
│  │  │ Agents │ (v2.2.0 新增)                               │  │
│  │  └────────┘                                              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Tauri IPC Bridge (invoke/emit)              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   Core Layer (Rust)                        │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │ CLI Tool │ │ Software │ │ Plugin   │ │  Skill   │   │  │
│  │  │ Manager  │ │ Scanner  │ │Marketplace│ │Marketplace│  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │   MCP    │ │  Rules   │ │  Backup  │ │PluginSync│   │  │
│  │  │Marketplace│ │ Manager │ │  Manager │ │         │   │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │  │
│  │  ┌──────────┐ ┌──────────┐                               │  │
│  │  │  Agent   │ │   MCP    │                               │  │
│  │  │ Manager  │ │ Manager  │ (FEAT-022, v2.2.0)           │  │
│  │  └──────────┘ └──────────┘                               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Data Layer (SQLite + File System)           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 第 3 部分：9 步流与功能模块映射

### 3.1 概述

9 步工作流将每个功能模块的生命周期拆解为 9 个标准步骤，每步对应特定的角色和产物。下方按**步骤**组织功能模块，每个模块说明：

- **所属步骤**：主要参与步骤（可能跨越多步）
- **责任角色**：@角色（主责）
- **Tauri Commands**：从 `src-tauri/src/commands/` 验证存在的命令
- **Pinia Store**：对应的 store 文件
- **UI 视图**：对应的视图文件
- **数据模型**：涉及的数据库表
- **验收标准**：可测试的验收点
- **9 步产物**：该功能在 9 步中的文件路径

---

### 步骤 1：立项与需求拆解 → @product-manager

所有功能模块均需此步骤。

| 功能模块 | 产物文件 |
|---------|---------|
| CLI 工具管理 | `docs/features/FEAT-01-cli-tools/task.md` |
| 软件检测 | `docs/features/FEAT-02-software-detection/task.md` |
| 插件管理 | `docs/features/FEAT-03-plugins/task.md` |
| 技能管理 | `docs/features/FEAT-04-skills/task.md` |
| MCP 服务 | `docs/features/FEAT-05-mcp/task.md` |
| 规则管理 | `docs/features/FEAT-06-rules/task.md` |
| 备份恢复 | `docs/features/FEAT-07-backup/task.md` |
| 设置 | `docs/features/FEAT-08-settings/task.md` |
| 仪表盘 | `docs/features/FEAT-09-dashboard/task.md` |
| 提示词管理 | `docs/features/FEAT-10-prompts/task.md` |
| Agents 管理（FEAT-11） | `docs/features/FEAT-11-agents/task.md` |
| MCP 管理器（FEAT-022） | `docs/features/FEAT-12-mcp-manager/task.md` |

---

### 步骤 2：设计与评审 → @design-director + @design-ux + @design-ui

#### 3.2.1 仪表盘（DashboardView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 2（主责）、步骤 8（终审） |
| **责任角色** | @design-director / @design-ux / @design-ui |
| **UI 视图** | `src/views/DashboardView.vue` |
| **Pinia Store** | `src/stores/app.ts`（全局状态） |
| **9 步产物** | `docs/features/FEAT-09-dashboard/design-notes.md` |

**用户故事**：作为开发者，我想要在仪表盘看到系统概览（CLI 工具状态、软件数量、插件数量、备份时间），以便快速了解当前环境健康状况。

**设计要点**：
- 统计卡片：CLI Tools（已安装/总数）、Software（检测数量）、Plugins（活跃数量）、Last Backup
- 数据来源：分别调用 `cliTools`、`detectedSoftware`、`pluginCount`、`lastBackup`
- 布局：网格卡片 + 快捷操作入口

---

#### 3.2.2 设置（SettingsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 2（主责）、步骤 8（终审） |
| **责任角色** | @design-director / @design-ui |
| **UI 视图** | `src/views/SettingsView.vue` |
| **Pinia Store** | `src/stores/settings.ts` |
| **Tauri Commands** | `has_github_token`, `get_github_token_preview`, `set_github_token`, `clear_github_token`（`commands/settings.rs`） |
| **9 步产物** | `docs/features/FEAT-08-settings/design-notes.md` |

**用户故事**：作为用户，我想要在设置页面管理 GitHub Token，以便通过 GitHub API 获取私有仓库的插件和技能。

**设计要点**：
- GitHub Token 输入/显示/清除
- Token 预览（显示前 4 位 + `****`）
- Token 状态指示器

---

#### 3.2.3 插件管理（PluginsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 2（主责）、步骤 8（终审） |
| **责任角色** | @design-director / @design-ux / @design-ui |
| **UI 视图** | `src/views/PluginsView.vue` |
| **Pinia Store** | `src/stores/plugin.ts`, `src/stores/plugin-marketplace.ts` |
| **9 步产物** | `docs/features/FEAT-03-plugins/design-notes.md` |

**用户故事**：作为用户，我想要管理本地插件和市场插件，以便安装、卸载、启用/禁用插件。

**设计要点**：
- 本地插件列表 + 市场插件列表 + 数据源管理（3 Tab 切换）
- **Installed Tab**：插件卡片网格，含来源 Badge、安装路径展示（截断 + Copy 按钮）、Sync 按钮组（Skeleton 动画）、卸载/View 按钮
- **Marketplace Tab**：来源 Tab 栏 + 防抖搜索栏（300ms）+ 虚拟滚动卡片网格（VirtualGrid）+ 无结果空状态
- **Sources Tab**：数据源安装状态头部 + 卡片网格（安装状态徽章 + 安装路径列表 + 进度条 + 备注入口 + repo_type 切换 + 全部安装按钮）
- 插件卡片（PluginCard）：名称/描述/Categories Tags + 安装进度条（stages: pending/downloading/installing/success/failed）+ 操作按钮（Details/Uninstall/Install/Update/Installed）
- 插件详情弹窗（PluginDetailsDialog）：5 种能力统计卡片（Skills/Hooks/Commands/MCP/LSP）+ 6 个 Tab（Overview/Skills/Commands/Hooks/MCP/LSP）+ Hook 执行测试 + MCP 连接探测 + 路径验证报告
- 来源备注弹窗（SourceNoteDialog）：Markdown 编辑/预览双 Tab，轻量渲染器（`src/utils/markdown.ts`）
- 添加来源弹窗（AddRepoSourceDialog）：repo_type 选择器（market/res 单选卡片）+ URL 输入

**FEAT 标签**：
- FEAT-016：来源安装状态管理
- FEAT-018：添加仓库源
- FEAT-019：来源 repo_type 切换
- FEAT-020：来源 Markdown 备注（source_notes.json）
- FEAT-021：marketplace 搜索 + sync_records.json 顶层优化

---

#### 3.2.4 技能管理（SkillsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 2（主责）、步骤 8（终审） |
| **责任角色** | @design-director / @design-ux / @design-ui |
| **UI 视图** | `src/views/SkillsView.vue` + 子组件 |
| **Pinia Store** | `src/stores/skill.ts`, `src/stores/skill-marketplace.ts`, `src/stores/skills-sh.ts`, `src/stores/anthropic-skills.ts` |
| **9 步产物** | `docs/features/FEAT-04-skills/design-notes.md` |

**用户故事**：作为用户，我想要发现、安装和管理各类技能（本地/Anthropic/Remote/Marketplace/skills.sh），以便扩展 AI 能力。

**设计要点**：
- 多来源技能聚合（5 个来源）
- 同步设置（Repository 管理）
- 导入对话框（ZIP / 本地 / 推荐站点）

---

### 步骤 3：技术方案制定 → @frontend-director + @backend-director

#### 3.3.1 数据模型设计

| 数据库表 | 主要字段 | 说明 |
|---------|---------|------|
| `software` | id, name, key, version, install_path, config_path, is_installed, last_checked, latest_version, is_upgradable, status | 已检测的软件（新增 `latest_version`, `is_upgradable`, `status` 字段） |
| `plugins` | id, software_id, name, version, author, description, installed_path, enabled, installed_at, last_updated | 已安装的插件 |
| `skills` | id, software_id, name, type, config, file_path, installed_at | 技能配置 |
| `mcp_services` | id, software_id, name, endpoint, auth_type, config, is_healthy, last_checked | MCP 服务连接 |
| `rules` | id, software_id, name, type, file_path, content, is_active, created_at, updated_at | 规则文件 |
| `backups` | id, name, path, size, file_count, created_at, includes | 备份记录（注：实际运行时表名 `backups`；`schema.rs` 中 `backup_records` 为参考未使用） |
| `config_templates` | id, name, description, category, author, version, files, created_at, updated_at | 配置模板 |
| `agents` | id, name, description, emoji, color, department, content, source, tags, installed_targets, is_custom, created_at, updated_at | Agent 定义（v2.2.0 新增） |
| `mcp_health_log` | id, service_id, status, latency_ms, error_message, checked_at | MCP 健康历史（FEAT-022） |
| `mcp_groups` | id, name, color, is_visible, created_at | MCP 分组（FEAT-022） |
| `mcp_service_groups` | service_id, group_id（联合主键） | MCP 服务-分组关联（FEAT-022） |
| `mcp_audit_log` | id, actor, action, service_id, service_name, details, status, created_at | MCP 操作审计日志（FEAT-022） |

**索引**：

| 索引名 | 表 | 字段 | 说明 |
|--------|-----|------|------|
| `idx_agents_department` | agents | department | Agent 按部门筛选 |
| `idx_mcp_health_log_service_checked` | mcp_health_log | service_id, checked_at | 健康历史查询 |
| `idx_mcp_audit_log_created` | mcp_audit_log | created_at | 审计日志时间查询 |
| `idx_mcp_audit_log_service` | mcp_audit_log | service_id | 按服务查审计日志 |
| `idx_mcp_service_groups_group` | mcp_service_groups | group_id | 按分组查服务 |

**关键架构决策**：
- SQLite 作为单一事实来源（`forge.db`）
- Tauri IPC 通过 `invoke` 调用 Rust 后端
- Pinia stores 与 Tauri Commands 一一对应
- 日志文件：`app_{timestamp}.log`，存储在 `~/.local/share/forge/logs/`

#### 3.3.2 CLI 工具检测架构

- **并行检测**：`rayon` 并行迭代，使用 60s 全局超时
- **异步执行**：`tokio::process::Command` + `tokio::select!` 超时控制
- **冲突检测**：npm 全局包数量 > 10 时标记冲突
- **升级超时**：300s 命令执行超时

#### 3.3.3 插件同步架构

- 同步记录存储在 `~/.local/share/forge/marketplace/sync_records.json`（v2.3.0 优化为只记录顶层条目，详见下方）
- 支持将市场插件缓存同步到各 CLI 工具的 `plugins/` 目录
- 增量同步（记录已同步文件，精准删除）

**sync_records.json 顶层条目优化（FEAT-021）**：
- **问题**：`synced_files` 记录了每个文件的完整相对路径，ECC 插件记录 3162 个文件，导致 JSON 膨胀到 ~322KB
- **优化方案**（`plugin_sync.rs`）：
  - `copy_dir_inner` 只记录顶层条目（目录名带 `/` 后缀，文件名不带路径前缀），而非递归记录所有文件路径
  - `remove_synced_files` 通过 `is_dir()` 判断条目类型（目录用 `remove_dir_all`，文件用 `remove_file`）
- **预期效果**：ECC 插件从 ~3162 条降至 ~20-30 条；JSON 体积从 ~322KB 降至 ~5-10KB
- **兼容性**：旧格式（完整路径）仍然能被新的 `remove_synced_files` 正常处理
- **参考文档**：`docs/superpowers/plans/2026-06-17-sync-records-top-level-only.md`

---

### 步骤 4：开发与实现 → @frontend-engineer + @backend-engineer

#### 3.4.1 CLI 工具管理（CliToolsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责）、步骤 7（性能重点） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/CliToolsView.vue` |
| **Pinia Store** | `src/stores/software.ts`（CLI 工具状态复用 software store） |
| **Tauri Commands** | `get_cli_tools`, `check_cli_tool_status`, `check_all_cli_tools_status`, `check_all_cli_tools_status_parallel`, `upgrade_cli_tool`（`commands/cli_tools.rs`） |
| **Service** | `services/cli_tools.rs` |
| **FEAT 标签** | FEAT-01 |

**功能细节（v2.4.0 补全）**：

**操作状态机**（`useOperationProgress.ts`）：

| 阶段 | 进度范围 | 说明 | canCancel | canRetry |
|------|---------|------|-----------|---------|
| `preparing` | 0–5% | 准备阶段，初始化操作 | ✓ | ✗ |
| `downloading` | 5–55% | 下载阶段 | ✓ | ✗ |
| `installing` | 55–85% | 安装阶段 | ✓ | ✗ |
| `verifying` | 85–100% | 验证阶段 | ✗ | ✗ |
| `completed` | 100% | 成功完成 | ✗ | ✗ |
| `failed` | — | 操作失败 | ✗ | ✓ |
| `cancelled` | — | 用户取消 | ✗ | ✓ |

**`CliToolStatus` 字段表**（来自 `services/cli_tools.rs` L74-84）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `toolKey` | `string` | 工具唯一标识键 |
| `isInstalled` | `boolean` | 是否已安装 |
| `installedVersion` | `string \| null` | 已安装版本号 |
| `installMethod` | `string \| null` | 安装方式（npm / curl-bash / npm-curl-fallback / brew） |
| `installPath` | `string \| null` | 可执行文件路径 |
| `hasConflict` | `boolean` | 是否有冲突 |
| `conflictInfo` | `string \| null` | 冲突描述（npm 全局包 > 10） |
| `latestVersion` | `string \| null` | 最新可用版本（从 npm view 获取） |
| `needsUpgrade` | `boolean` | 是否需要升级（installedVersion ≠ latestVersion） |

**操作行为**：

| 操作 | 触发条件 | 行为 |
|------|---------|------|
| Install | 未安装且无操作中 | 点击执行安装，调用 `upgrade_cli_tool` 命令 |
| Update | 已安装且 `needsUpgrade === true` | 调用 `upgrade_cli_tool` 命令 |
| Check | 任意状态 | 调用 `checkCliToolStatus(toolKey)` |
| Cancel | 操作在 downloading/installing 阶段 | 调用 `cancelOperation`，标记 `cancelled` |
| Retry | 操作在 failed/cancelled 阶段 | 复用 `pendingRetry` 中保存的 method，重新执行 |
| Show Install Options | 未安装时点击 Install | 打开 Modal，显示所有 `installMethods`，用户选择后执行 |
| Methods Tooltip | 点击 "Methods" 链接 | Teleport 到 body，展开 tooltip，ESC/点击外部关闭 |

**状态展示矩阵**：

| 状态 | Badge 文案 | 按钮 |
|------|-----------|------|
| pending | **Checking…**（脉冲动画） | disabled 脉冲按钮 |
| operating | `STAGE_CONFIG[stage].label`（preparing/downloading/installing/verifying） | Cancel（downloading/installing）/ disabled（其他） |
| completed | **Installed** | Check |
| failed | **Failed** | Cancel + Retry |
| cancelled | **Cancelled** | Cancel + Retry |
| upgrade | **Update available** | Update |
| installed | **Installed** | Check |

**CLI 工具列表**（从 `services/cli_tools.rs` 验证，共 9 个）：

| 工具 | Key | 安装方式 | npm 包名 | plugin_dir |
|------|-----|---------|---------|-----------|
| Claude Code | `claude-code` | curl-bash（优先）/ npm（fallback） | `@anthropic-ai/claude-code` | `~/.claude/plugins` |
| Codex | `codex` | npm | `@openai/codex` | `~/.codex/plugins` |
| Gemini CLI | `gemini-cli` | npm | `@google/gemini-cli` | `~/.gemini/plugins` |
| OpenCode | `opencode` | curl-bash（优先）/ npm（fallback） | `opencode-ai` | `~/.opencode/plugins` |
| OpenClaw | `openclaw` | npm | `openclaw` | `~/.openclaw/plugins` |
| Hermes | `hermes` | curl-bash | — | `~/.hermes/plugins` |
| Cursor CLI | `cursor` | curl-bash | — | `~/.cursor/plugins` |
| DeepSeek-Reasonix | `deepseek-reasonix` | npm（优先）/ brew（fallback） | `reasonix` | `~/.reasonix/plugins` |
| Kiro CLI | `kiro` | curl-bash | — | `~/.kiro/plugins` |
| MiMo Code | `mimo-code` | curl-bash（优先）/ npm（fallback） | `@mimo-ai/cli` | `~/.mimo/plugins` |

**二进制检测映射**（`services/cli_tools.rs` 验证）：

| 工具 | 检测的二进制名 |
|------|--------------|
| Claude Code | `claude` |
| Gemini CLI | `gemini` |
| Cursor CLI | `cursor-agent`, `agent` |
| 其他 | 工具 key 本身 |

**验收标准**：
- [ ] 9 个 CLI 工具均能通过 `get_cli_tools` 返回
- [ ] 并行检测在 60s 内完成
- [ ] 冲突检测正确识别 npm 全局包 > 10
- [ ] 升级命令正确执行，支持 300s 超时
- [ ] 操作进行中显示 5 阶段进度条
- [ ] 取消操作立即停止后续阶段并标记 cancelled
- [ ] 重试操作复用 `pendingRetry` 中保存的 method
- [ ] 全局检测完成前未返回状态的卡片显示 "Checking…" 占位（不误显示 Install）
- [ ] 工具的 `installMethods` 数组长度 ≥ 1 时，"Methods" 链接可点击展开 tooltip（Teleport 到 body）
- [ ] 未安装时点击 Install 触发 install options Modal
- [ ] npm 全局包 > 10 时顶部显示 alert-warning 冲突提示

---

#### 3.4.2 软件检测（SoftwareManagementView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责）、步骤 7（性能重点） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/SoftwareManagementView.vue` |
| **Pinia Store** | `src/stores/software.ts` |
| **Tauri Commands** | `get_software_list`, `detect_software`, `get_software_by_id`, `get_software_by_key`, `sync_software`, `install_software`, `uninstall_software`（`commands/software.rs`） |
| **Service** | `services/software_scanner.rs`, `services/software_installer.rs` |
| **FEAT 标签** | FEAT-02 |
| **数据表** | `software` |
| **9 步产物** | `docs/features/FEAT-02-software-detection/dev-done.md` |

**平台过滤与空闲调度（v2.4.0 补全）**：

- **`selectedPlatform`**：`software.ts` L82-84，类型 `string`，默认值 `'auto'`，从 `localStorage.getItem('forge-selected-platform')` 恢复，不存在则取 `'auto'`
- **`setSelectedPlatform(platform: string)`**：`software.ts` L86-89，写入 `localStorage['forge-selected-platform']` + 更新 `selectedPlatform.value`
- 平台选项：`auto` / `macos` / `windows` / `linux`；视图层按平台筛选软件列表
- **`detectSoftware` 后台空闲调度**（`software.ts` L13-19/L191-194/L207-210）：`install_software`/`uninstall_software`/`update_software` 成功后，以 `runWhenIdle(detectSoftware)` 回调触发后台重新扫描；`runWhenIdle` 使用 `requestIdleCallback`（100ms timeout fallback），避免阻塞 UI

**`sync_software` 语义**（`commands/software.rs` L44-55）：

- 并行扫描本地所有软件（`scanner.detect_software_parallel()`）
- 对每个检测到的软件执行 `state.db.upsert_software(software)`，批量写入/更新 `software` 表
- 与云/远端配置同步的入口（当前为本地全量扫描 + DB 同步）

**软件分级清单**（从 `services/software_scanner.rs` 验证，共 6 Tier）：

**Tier 0: AI Tools**

| 软件 | Key | 配置文件路径 | 版本检测方式 |
|------|-----|-------------|-------------|
| Cursor | `cursor` | `~/.cursor/` | `settings.json` / `product.json` |
| Windsurf | `windsurf` | `~/.windsurf/` | `settings.json` |
| Claude Desktop | `claude-desktop` | `~/.claude/` | `localStorage.json` |
| Continue | `continue` | `~/.continue/` | `config.json` |
| Cody | `cody` | `~/.cody/` | `settings.json` |

**Tier 1: Foundation Environment**

| 软件 | Key | 平台 | 配置文件路径 |
|------|-----|------|-------------|
| Homebrew | `homebrew` | macOS | `~/.brew/` |
| Chocolatey | `chocolatey` | Windows | `C:/ProgramData/chocolatey` |
| Scoop | `scoop` | Windows | `~/.local/share/scoop` |
| Git | `git` | Cross-platform | `~/.gitconfig` |
| SSH Config | `ssh` | Cross-platform | `~/.ssh/` |
| Windows Terminal | `windows-terminal` | Windows | `AppData/Local/Packages/Microsoft.WindowsTerminal` |
| iTerm2 | `iterm2` | macOS | `Library/Application Support/iTerm2` |
| Oh My Posh | `oh-my-posh` | Cross-platform | `~/.oh-my-posh.omp.json` |
| VS Code | `vscode` | Cross-platform | `~/.config/Code/User/` |

**Tier 2: Language Version Managers**

| 软件 | Key | 配置文件路径 |
|------|-----|-------------|
| nvm (Node) | `nvm` | `~/.nvm/` |
| pyenv (Python) | `pyenv` | `~/.pyenv/` |
| goenv (Go) | `goenv` | `~/.goenv/` |
| jenv (Java) | `jenv` | `~/.jenv/` |
| asdf | `asdf` | `~/.asdf/` |

**Tier 3: Runtime & Containers**

| 软件 | Key | 版本检测方式 |
|------|-----|-------------|
| Docker | `docker` | `docker --version` |
| Docker Compose | `docker-compose` | `docker compose version` / `docker-compose --version` |
| FFmpeg | `ffmpeg` | `ffmpeg -version` |

**Tier 4: Debug & Collaboration**

| 软件 | Key | 配置文件路径 |
|------|-----|-------------|
| Apifox | `apifox` | `Library/Application Support/Apifox` |
| Postman | `postman` | `Library/Application Support/Postman` |
| Charles Proxy | `charles` | `Library/Application Support/Charles/` |
| Cyberduck (SFTP) | `cyberduck` | `Library/Application Support/Cyberduck/` |
| FileZilla (SFTP) | `filezilla` | `~/.config/filezilla/` |

**Tier 5: Productivity Tools**

| 软件 | Key | 配置文件路径 |
|------|-----|-------------|
| Snipaste | `snipaste` | `Library/Application Support/Snipaste` |
| Obsidian | `obsidian` | `Library/Application Support/obsidian` |
| Excalidraw | `excalidraw` | `~/.excalidraw/` |

**验收标准**：
- [ ] 检测所有 6 Tier 软件的安装状态
- [ ] 版本检测正确识别各软件版本格式
- [ ] 软件列表可按 Tier 分组显示
- [ ] `selectedPlatform` 切换后立即持久化到 `localStorage['forge-selected-platform']`
- [ ] 跨次会话后 `selectedPlatform` 状态保留（从 localStorage 恢复）

---

#### 3.4.3 插件管理（PluginsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/PluginsView.vue` |
| **Pinia Store** | `src/stores/plugin.ts`, `src/stores/plugin-marketplace.ts` |
| **Tauri Commands** | 34 个命令（见下方详细清单） |
| **Service** | `services/plugin_marketplace.rs`, `services/plugin_capabilities.rs`, `services/plugin_repo_sync.rs` |
| **数据表** | `plugins` |
| **9 步产物** | `docs/features/FEAT-03-plugins/dev-done.md` |

**Tauri Commands 清单**（全部已在 `lib.rs` 注册）：

| Command | 函数文件 | 说明 |
|---------|---------|------|
| `get_plugins` | `commands/plugin.rs` | 获取本地插件列表 |
| `install_plugin` | `commands/plugin.rs` | 安装本地插件 |
| `uninstall_plugin` | `commands/plugin.rs` | 卸载插件 |
| `update_plugin` | `commands/plugin.rs` | 更新插件 |
| `toggle_plugin` | `commands/plugin.rs` | 启用/禁用插件 |
| `get_marketplace_sources` | `commands/plugin_marketplace.rs` | 获取市场来源 |
| `fetch_marketplace_plugins` | `commands/plugin_marketplace.rs` | 搜索市场插件 |
| `get_marketplace_plugins` | `commands/plugin_marketplace.rs` | 获取本地市场插件 |
| `get_marketplace_manifest` | `commands/plugin_marketplace.rs` | 获取完整市场清单 |
| `install_marketplace_plugin` | `commands/plugin_marketplace.rs` | 安装市场插件 |
| `uninstall_marketplace_plugin` | `commands/plugin_marketplace.rs` | 卸载市场插件 |
| `update_marketplace_plugin` | `commands/plugin_marketplace.rs` | 更新市场插件 |
| `add_marketplace_source` | `commands/plugin_marketplace.rs` | 添加市场来源 |
| `is_plugin_installed` | `commands/plugin_marketplace.rs` | 检查插件是否已安装 |
| `set_plugin_disabled` | `commands/plugin_marketplace.rs` | 设置禁用状态 |
| `get_marketplace_source_status` | `commands/plugin_marketplace.rs` | 获取来源状态 |
| `install_marketplace_source` | `commands/plugin_marketplace.rs` | 安装市场来源 |
| `install_all_marketplace_sources` | `commands/plugin_marketplace.rs` | 批量安装来源 |
| `get_user_marketplace_sources` | `commands/plugin_marketplace.rs` | 获取用户来源 |
| `add_user_marketplace_source` | `commands/plugin_marketplace.rs` | 添加用户来源 |
| `remove_user_marketplace_source` | `commands/plugin_marketplace.rs` | 移除用户来源 |
| `update_source_repo_type` | `commands/plugin_marketplace.rs` | 切换来源仓库类型（market/res） |
| `get_source_notes` | `commands/plugin_marketplace.rs` | 获取所有来源备注 |
| `save_source_note` | `commands/plugin_marketplace.rs` | 保存/删除来源备注 |
| `get_plugin_capabilities` | `commands/plugin_capabilities.rs` | 获取插件能力 |
| `execute_plugin_hook` | `commands/plugin_capabilities.rs` | 执行插件钩子 |
| `validate_plugin_path` | `commands/plugin_capabilities.rs` | 验证插件路径 |
| `probe_plugin_mcp` | `commands/mcp_bridge.rs` | 探测 MCP 支持 |
| `get_installed_registry` | `commands/installed_registry.rs` | 获取已安装注册表 |
| `update_plugin_inuse_cmd` | `commands/installed_registry.rs` | 更新 in-use 命令 |
| `sweep_inuse_cmd` | `commands/installed_registry.rs` | 扫描 in-use 命令 |
| `sync_plugin_to_cli_tool` | `commands/plugin_sync.rs` | 同步插件到 CLI 工具 |
| `unsync_plugin_from_cli_tool` | `commands/plugin_sync.rs` | 取消同步 |
| `get_plugin_sync_status` | `commands/plugin_sync.rs` | 获取同步状态 |

**Vue 组件清单**（`src/components/plugins/`）：

| 组件 | 说明 | FEAT |
|------|------|------|
| `PluginCard.vue` | 插件卡片：名称/描述/Categories Tags + 安装进度条 + source badge + 操作按钮 | — |
| `PluginDetailsDialog.vue` | 插件详情弹窗：5 能力统计卡片 + 6 Tab + Hook 执行 + MCP 探测 + 路径验证 | — |
| `SourceNoteDialog.vue` | Markdown 备注弹窗：编辑/预览双 Tab | FEAT-020 |
| `AddRepoSourceDialog.vue` | 添加仓库源弹窗：repo_type 选择器（market/res）+ URL 输入 | FEAT-018 |

**前端工具**：

| 文件 | 说明 |
|------|------|
| `src/utils/markdown.ts` | 轻量 Markdown 渲染器（无外部依赖，支持 12 种语法：标题/粗体/斜体/行内代码/代码块/列表/链接/引用/分割线） |

**SourceNote 持久化机制**：

| 维度 | 内容 |
|------|------|
| 存储文件 | `$FORGE_HOME/plugins/source_notes.json`（`~/.forge/plugins/source_notes.json`） |
| JSON 结构 | `{ "version": "1", "notes": { "source_id": "# markdown content" } }` |
| Rust 服务 | `SourceNotesRegistry` 结构体 + `read_source_notes()` + `write_source_notes()` |
| Store 状态 | `sourceNotes: Record<string, string>` + `loadSourceNotes()` + `saveSourceNote()` |

**验收标准**：
- [ ] 本地插件 CRUD 操作正常
- [ ] 市场插件搜索和安装正常
- [ ] 插件能力探测返回正确的能力清单
- [ ] 插件同步到 CLI 工具成功
- [ ] 来源备注添加/编辑/保存正常
- [ ] repo_type 切换正常

#### 3.4.4 技能管理（SkillsView）#### 3.4.4 技能管理（SkillsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/SkillsView.vue` + 多个子组件 |
| **Pinia Store** | `src/stores/skill.ts`, `src/stores/skill-import.ts`, `src/stores/skill-marketplace.ts`, `src/stores/skills-sh.ts`, `src/stores/anthropic-skills.ts`, `src/stores/recommended-sites.ts` |
| **Tauri Commands** | 54 个命令（5 个来源） |
| **Service** | `services/skill_marketplace.rs` |
| **数据表** | `skills` |
| **9 步产物** | `docs/features/FEAT-04-skills/dev-done.md` |

**技能来源与命令**：

| 来源 | Commands 文件 | 主要命令 |
|------|-------------|---------|
| 本地技能 | `commands/skill.rs` | `get_skills`, `create_skill`, `update_skill`, `delete_skill` |
| 技能导入 | `commands/skill_import.rs` | `unzip_skill_package`, `scan_local_skills`, `import_local_skill`, `detect_cli_skills_paths`, `get_default_skills_dir` |
| 技能仓库 | `commands/skill_repository.rs` | `get_repositories`, `add_repository`, `remove_repository`, `validate_repository`, `sync_repository`, `sync_all_repositories`, `download_skill_from_repository` |
| Anthropic 技能 | `commands/anthropic_skills.rs` | `list_anthropic_skills`, `install_anthropic_skill`, `verify_anthropic_skill_install`, `uninstall_anthropic_skill`, `get_local_anthropic_skills`, `get_remote_skill_sources`, `list_remote_skills`, `install_remote_skill` |
| 技能市场 | `commands/skill_marketplace.rs` | `get_skill_sources`, `fetch_marketplace_skills`, `get_skill_categories`, `install_marketplace_skill`, `sync_skill_to_target`, `get_sync_targets`, `add_sync_target`, `is_skill_installed` |
| Skills.sh | `commands/skills_sh.rs` | `fetch_skills_sh_leaderboard`, `search_skills_sh`, `fetch_skills_sh_curated`, `fetch_skills_sh_skill_detail`, `install_skill_via_skills_sh` |

**验收标准**：
- [ ] 5 个来源的技能均可列出和安装
- [ ] 仓库同步功能正常
- [ ] 技能导入（ZIP/本地/推荐站点）正常
- [ ] 技能同步到目标目录成功

---

#### 3.4.5 MCP 服务（MCPView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/MCPView.vue` + 子组件 |
| **Pinia Store** | `src/stores/mcp.ts`, `src/stores/mcp-marketplace.ts` |
| **Tauri Commands** | 12 个命令 |
| **Service** | `services/mcp_bridge.rs` |
| **数据表** | `mcp_services` |
| **9 步产物** | `docs/features/FEAT-05-mcp/dev-done.md` |

**Tauri Commands**：

| Command | 函数文件 | 说明 |
|---------|---------|------|
| `get_mcp_services` | `commands/mcp.rs` | 获取 MCP 服务列表 |
| `add_mcp_service` | `commands/mcp.rs` | 添加 MCP 服务 |
| `update_mcp_service` | `commands/mcp.rs` | 更新 MCP 服务 |
| `delete_mcp_service` | `commands/mcp.rs` | 删除 MCP 服务 |
| `check_mcp_service_health` | `commands/mcp.rs` | 健康检查 |
| `get_mcp_sources` | `commands/mcp_marketplace.rs` | 获取 MCP 来源 |
| `fetch_mcp_servers` | `commands/mcp_marketplace.rs` | 获取 MCP 服务器 |
| `get_local_mcp_servers` | `commands/mcp_marketplace.rs` | 获取本地服务器 |
| `install_mcp_server` | `commands/mcp_marketplace.rs` | 安装 MCP 服务器 |
| `sync_mcp_to_target` | `commands/mcp_marketplace.rs` | 同步到目标 |
| `get_mcp_sync_targets` | `commands/mcp_marketplace.rs` | 获取同步目标 |
| `add_mcp_sync_target` | `commands/mcp_marketplace.rs` | 添加同步目标 |
| `remove_mcp_sync_target` | `commands/mcp_marketplace.rs` | 移除同步目标 |

**验收标准**：
- [ ] MCP 服务 CRUD 操作正常
- [ ] 市场 MCP 服务器安装正常
- [ ] 同步到目标目录成功
- [ ] 健康检查正确反映服务状态

---

#### 3.4.6 规则管理（RulesView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/RulesView.vue` |
| **Pinia Store** | `src/stores/rule.ts` |
| **Tauri Commands** | `get_rules`, `create_rule`, `update_rule`, `delete_rule`, `toggle_rule`（`commands/rule.rs`） |
| **数据表** | `rules` |
| **9 步产物** | `docs/features/FEAT-06-rules/dev-done.md` |

**验收标准**：
- [ ] 规则 CRUD 操作正常
- [ ] 启用/禁用切换正常
- [ ] 规则内容编辑和保存正常

---

#### 3.4.7 备份与恢复（BackupView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/BackupView.vue` |
| **Pinia Store** | `src/stores/backup.ts` |
| **Tauri Commands** | `create_backup`, `get_backups`, `restore_backup`, `delete_backup`, `get_backup_contents`（`commands/backup.rs`） |
| **数据表** | `backups` |
| **9 步产物** | `docs/features/FEAT-07-backup/dev-done.md` |

**验收标准**：
- [ ] 一键备份所有配置成功
- [ ] 历史备份列表正确显示
- [ ] 从备份恢复配置成功
- [ ] 备份内容预览正常

---

#### 3.4.8 提示词管理（PromptManagerView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/PromptManagerView.vue` |
| **Pinia Store** | 无专属 store（复用 app.ts 或独立） |
| **Tauri Commands** | 10 个命令（`commands/file.rs`） |
| **数据表** | 无 |
| **9 步产物** | `docs/features/FEAT-10-prompts/dev-done.md` |

**Tauri Commands**（`commands/file.rs`）：

| Command | 说明 |
|---------|------|
| `read_file` | 读取文件内容 |
| `write_file` | 写入文件内容 |
| `list_directory` | 列出目录内容 |
| `file_exists` | 检查文件是否存在 |
| `create_directory` | 创建目录 |
| `delete_file` | 删除文件 |
| `copy_file` | 复制文件 |
| `move_file` | 移动文件 |
| `get_file_info` | 获取文件信息 |
| `read_directory_tree` | 读取目录树 |

**验收标准**：
- [ ] 提示词模板可创建、编辑、删除
- [ ] 提示词分类和标签管理正常
- [ ] 导入/导出功能正常

---

#### 3.4.9 Agents 管理（AgentsView）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | `src/views/AgentsView.vue` |
| **Pinia Store** | `src/stores/agent.ts` |
| **Tauri Commands** | 10 个命令（`commands/agent.rs`） |
| **Service** | agents marketplace 路径解析 |
| **数据表** | `agents` |
| **9 步产物** | `docs/features/FEAT-11-agents/dev-done.md` |

**Tauri Commands**（`commands/agent.rs`，10 个命令）：

| Command | 说明 |
|---------|------|
| `get_agents` | 获取 Agent 列表 |
| `search_agents` | 搜索 Agent |
| `create_agent` | 创建自定义 Agent |
| `update_agent` | 更新 Agent |
| `delete_agent` | 删除 Agent |
| `import_agents_from_repo` | 从本地仓库导入 Agent |
| `install_agent_to_target` | 安装 Agent 到指定工具 |
| `uninstall_agent_from_target` | 从目标工具卸载 Agent |
| `get_agents_marketplace_path` | 获取 Agent 市场路径 |
| `has_agents_marketplace` | 检查 Agent 市场是否存在 |

**已实现的安装目标**：

| 工具 | 安装路径 | 格式转换 |
|------|---------|---------|
| Claude Code | `~/.claude/agents/{slug}.md` | 直接使用 |
| Cursor | `<project>/.cursor/rules/{slug}.mdc` | `.md` → `.mdc`（frontmatter 包装） |
| Copilot | `~/.github/agents/{slug}.md` | 直接使用 |

**待实现目标**（PRD 中描述但代码未实现）：

| 工具 | 计划路径 | 状态 |
|------|---------|------|
| OpenCode | `<project>/.opencode/agents/{slug}.md` | 待实现 |
| Windsurf | `<project>/.windsurfrules` | 待实现 |
| Codex | `~/.codex/agents/{slug}.md` | 待实现 |
| OpenClaw | `~/.openclaw/agents/{slug}.md` | 待实现 |

**前端结构**：

| 文件 | 说明 |
|------|------|
| `src/views/AgentsView.vue` | 主视图（11 部门侧边栏、Agent 卡片网格、详情/导入对话框） |
| `src/stores/agent.ts` | Pinia store |
| `src/types/agent.ts` | TypeScript 类型 + 18 个部门静态定义 |
| `src/components/agents/AgentCard.vue` | Agent 卡片组件 |
| `src/components/agents/AgentDetailsDialog.vue` | Agent 详情对话框 |
| `src/components/agents/AgentImportDialog.vue` | Agent 导入对话框 |
| `src/router/index.ts` | 新增 `/agents` 路由 |
| `src/components/layout/Sidebar.vue` | 新增 Agents 入口（含数量徽章） |

**自动导入逻辑**：`AgentsView.vue` 的 `onMounted` 检测到 agents 表为空时，自动从 `~/.forge/agents/marketplace/agency-agents-zh` 目录导入 Agent。

**验收标准**：
- [ ] Agent 列表按部门分类展示
- [ ] 搜索功能正常
- [ ] 从 agency-agents-zh 导入正常
- [ ] 安装到 claude-code / cursor / copilot 三个目标成功
- [ ] 自定义 Agent 创建/编辑/删除正常

---

#### 3.4.10 MCP 管理器（MCPManager，FEAT-022）

| 属性 | 内容 |
|------|------|
| **所属步骤** | 步骤 4（主责） |
| **责任角色** | @frontend-engineer / @backend-engineer |
| **UI 视图** | 复用 `src/views/MCPView.vue` |
| **Pinia Store** | 复用 `src/stores/mcp.ts` 和 `src/stores/mcp-marketplace.ts`（无独立 store） |
| **Tauri Commands** | 11 个命令（`commands/mcp_manager.rs`） |
| **Service** | `services/mcp_protocol.rs`（stdio/HTTP 协议处理器 + DiscoveryCache） |
| **数据表** | `mcp_health_log`, `mcp_groups`, `mcp_service_groups`, `mcp_audit_log` |
| **9 步产物** | `docs/features/FEAT-12-mcp-manager/dev-done.md` |

**Tauri Commands**（`commands/mcp_manager.rs`，11 个命令）：

| Command | 说明 |
|---------|------|
| `get_mcp_service_detail` | 获取 MCP 服务详情 |
| `invoke_mcp_tool` | 调用 MCP 工具 |
| `discover_mcp_service` | 发现 MCP 服务 |
| `export_mcp_services` | 导出 MCP 服务配置（JSON/YAML） |
| `import_mcp_services` | 导入 MCP 服务配置（JSON/YAML，支持 skip/overwrite 模式） |
| `get_mcp_health_history` | 获取 MCP 健康历史 |
| `get_mcp_groups` | 获取 MCP 分组列表 |
| `create_mcp_group` | 创建 MCP 分组 |
| `update_mcp_group` | 更新 MCP 分组 |
| `delete_mcp_group` | 删除 MCP 分组 |
| `get_mcp_audit_log` | 获取 MCP 审计日志 |

**协议层**（`services/mcp_protocol.rs`）：

- **stdio 协议**：通过 `tokio::process::Command` 执行 MCP 服务器进程，stdin/stdout 通信
- **HTTP 协议**：通过 `reqwest` 发送 HTTP 请求到 MCP 服务器端点
- **DiscoveryCache**：缓存服务发现结果，TTL 5 分钟，容量上限 100 条（LRU 淘汰）

**导入/导出**：
- 支持 JSON 和 YAML 两种格式
- skip 模式：跳过已存在的服务
- overwrite 模式：覆盖已存在的服务
- 详细的错误报告（每条记录的成功/失败状态）

**审计日志字段**：actor（操作者）/ action（操作类型）/ service_id / service_name / details / status / created_at。

**前端组件**（`src/components/mcp/*`，11 个）：

| 组件 | 说明 |
|------|------|
| `MCPServerCard.vue` | MCP 服务卡片 |
| `MCPDetailsDialog.vue` | 服务详情（37K，包含工具调用能力） |
| `MCPExportDialog.vue` | 导出对话框 |
| `MCPImportDialog.vue` | 导入对话框（skip/overwrite 模式） |
| `MCPInstallDialog.vue` | 安装对话框 |
| `MCPInvocationDialog.vue` | 工具调用对话框 |
| `MCPServerFormDialog.vue` | 服务表单（创建/编辑） |
| `MCPSyncSettingsDialog.vue` | 同步设置对话框 |
| `MCPGroupsPanel.vue` | 分组管理面板 |
| `MCPAuditLogTable.vue` | 审计日志表格 |
| `MCPHealthBadge.vue` | 健康状态徽章 |

**验收标准**：
- [ ] MCP 服务详情展示正确
- [ ] 工具调用功能正常
- [ ] JSON/YAML 导入/导出正常
- [ ] 健康历史记录正确
- [ ] 分组管理（CRUD）正常
- [ ] 审计日志记录正确

---

### 步骤 5：代码审查与质量把关 → @review-expert + 3 总监 soft sign-off

所有功能模块均需此步骤。

| 功能模块 | 产物文件 |
|---------|---------|
| 所有功能 | `docs/features/FEAT-{nn}-{name}/review.md` |

**审查要点**：
- Rust 后端：`lib.rs` 中所有 invoke_handler 注册命令均已验证存在
- Vue 前端：Pinia stores 与 Tauri commands 一一对应
- 数据库：表结构与 `connection.rs` 一致
- 性能：CLI 并行检测、SQLite 索引

---

### 步骤 6：测试与验证 → @qa-director + @qa-engineer

所有功能模块均需此步骤。

| 功能模块 | 产物文件 |
|---------|---------|
| 所有功能 | `docs/features/FEAT-{nn}-{name}/test-cases.md` |

**测试范围**：
- 单元测试：`src/composables/__tests__/`（已存在 `useVirtualGrid.spec.ts`）
- Tauri 命令集成测试
- 跨平台测试（macOS / Windows）

---

### 步骤 7：性能分析与优化 → @perf-director + @perf-engineer

#### 3.7.1 性能重点模块

| 模块 | 性能关注点 | 产物文件 |
|------|-----------|---------|
| CLI 工具并行检测 | `check_all_installations_parallel()` 使用 `rayon` 并行 + 60s 超时 | `docs/features/FEAT-01-cli-tools/perf-report.md` |
| SQLite 查询优化 | 10 个索引（`idx_*_software_id`, `idx_backup_records_created_at`, `idx_agents_department`, MCP Manager 5 个索引） | `docs/features/FEAT-02-software-detection/perf-report.md` |
| 软件扫描 | 分 Tier 检测，避免阻塞 UI | `docs/features/FEAT-02-software-detection/perf-report.md` |
| MCP Manager 协议层 | DiscoveryCache（HashMap 预加载避免 N+1，100 容量 LRU 淘汰） | `docs/features/FEAT-12-mcp-manager/perf-report.md` |

**性能基线**：

| 指标 | 要求 |
|-----|------|
| 启动时间 | < 3 秒（冷启动） |
| 内存占用 | 空闲状态 < 150MB |
| UI 操作响应 | < 100ms |
| CLI 检测 | 60 秒超时，并行检测 |
| SQLite 查询 | 单次查询 < 50ms |

---

### 步骤 8：评审与交付 → 5 总监联合 soft sign-off

所有功能模块均需此步骤。

| 功能模块 | 产物文件 |
|---------|---------|
| 所有功能 | `docs/features/FEAT-{nn}-{name}/final-review.md` |

**签收矩阵**：

| 角色 | 签收职责 |
|------|---------|
| @design-director | 视觉风格、设计规范 |
| @frontend-director | 前端架构、Vue 3 + Pinia |
| @backend-director | Rust 后端、Tauri IPC、SQLite |
| @qa-director | 测试覆盖、缺陷率 |
| @perf-director | 性能指标达成 |

---

### 步骤 9：部署上线与监控 → @pm + @frontend-director + @backend-director + @perf-director

| 功能模块 | 产物文件 |
|---------|---------|
| 所有功能 | `docs/features/FEAT-{nn}-{name}/deploy-report.md` |

**部署内容**：
- Tauri 打包配置（`tauri.conf.json`）
- macOS 签名配置（证书、entitlements）
- Windows 签名配置
- CI/CD 配置（如 GitHub Actions）
- 监控告警配置

---

## 第 4 部分：Pinia Stores 清单

| Store | 文件 | 说明 |
|-------|------|------|
| `anthropic-skills` | `anthropic-skills.ts` | Anthropic 官方技能状态 |
| `app` | `app.ts` | 应用全局状态 |
| `backup` | `backup.ts` | 备份状态 |
| `mcp` | `mcp.ts` | MCP 服务状态 |
| `mcp-marketplace` | `mcp-marketplace.ts` | MCP 市场状态 |
| `plugin` | `plugin.ts` | 插件状态 |
| `plugin-marketplace` | `plugin-marketplace.ts` | 插件市场状态 |
| `recommended-sites` | `recommended-sites.ts` | 推荐站点 |
| `rule` | `rule.ts` | 规则状态 |
| `settings` | `settings.ts` | 设置状态 |
| `skill` | `skill.ts` | 技能状态 |
| `skill-import` | `skill-import.ts` | 技能导入状态 |
| `skill-marketplace` | `skill-marketplace.ts` | 技能市场状态 |
| `skills-sh` | `skills-sh.ts` | Skills.sh 状态 |
| `software` | `software.ts` | 软件状态 |
| `agent` | `agent.ts` | Agent 状态（v2.2.0 新增） |
| **总计** | **16 个 Store**（不含 `index.ts`） | |

---

## 第 5 部分：Tauri Commands 完整清单（22 个模块文件）

| 模块文件 | 命令数量 | 说明 |
|---------|---------|------|
| `commands/software.rs` | 7 | 软件检测 CRUD |
| `commands/cli_tools.rs` | 5 | CLI 工具管理 |
| `commands/plugin.rs` | 5 | 插件 CRUD |
| `commands/plugin_marketplace.rs` | 19 | 插件市场（含 source_note 命令，v2.2.0） |
| `commands/plugin_capabilities.rs` | 3 | 插件能力 |
| `commands/plugin_sync.rs` | 3 | 插件同步 |
| `commands/mcp_bridge.rs` | 1 | MCP 桥接 |
| `commands/installed_registry.rs` | 3 | 已安装注册表 |
| `commands/skill.rs` | 4 | 技能 CRUD |
| `commands/skill_import.rs` | 5 | 技能导入 |
| `commands/skill_repository.rs` | 9 | 技能仓库 |
| `commands/anthropic_skills.rs` | 12 | Anthropic 技能 |
| `commands/skill_marketplace.rs` | 11 | 技能市场 |
| `commands/skills_sh.rs` | 6 | Skills.sh |
| `commands/mcp.rs` | 5 | MCP 服务 CRUD |
| `commands/mcp_marketplace.rs` | 8 | MCP 市场 |
| `commands/agent.rs` | 10 | Agent 管理（v2.2.0 新增） |
| `commands/mcp_manager.rs` | 11 | MCP 管理器（FEAT-022，v2.2.0 新增） |
| `commands/rule.rs` | 5 | 规则 CRUD |
| `commands/backup.rs` | 5 | 备份 CRUD |
| `commands/file.rs` | 10 | 文件操作 |
| `commands/settings.rs` | 4 | 设置管理 |
| **总计** | **22 个模块文件** | **162 个命令** |

---

## 第 6 部分：非功能性需求

### 6.1 性能需求

| 指标 | 要求 |
|-----|------|
| 启动时间 | < 3 秒（冷启动） |
| 内存占用 | 空闲状态 < 150MB |
| UI 操作响应 | < 100ms |
| CLI 检测 | 60 秒超时，并行检测 |
| SQLite 查询 | 单次查询 < 50ms |

### 6.2 兼容性需求

| 平台 | 最低版本 |
|-----|---------|
| macOS | macOS 12 Monterey 及以上 |
| Windows | Windows 10 (1903) 及以上 |

### 6.3 安全性需求

| 需求 | 描述 |
|-----|------|
| 数据存储 | 所有数据本地存储，不上传用户数据 |
| GitHub Token | 加密存储在本地 |
| 文件操作 | 通过 Tauri IPC 沙箱执行 |

---

## 第 7 部分：v1.4.0 → v2.0.0 修正清单

以下为 v1.4.0 中与实际代码不符的陈述，已在 v2.0 中修正：

| # | v1.4.0 错误描述 | 实际代码 | 修正说明 |
|---|----------------|---------|---------|
| 1 | Cursor CLI 二进制检测包含 `cursor-agent`, `agent` | 仅 `curl-bash` 安装方式，无 npm；`cursor-agent`/`agent` 是**检测的二进制名**，不是安装方式 | 修正安装方式和二进制检测描述 |
| 2 | AI Tools 软件分类标注为 "Tier 0: AI Tools" 但文档表格无 Tier 0 标题 | 代码中 `tier: 0`，注释为 "Legacy: AI Tools (from original code)" | 修正文档表格结构，增加 Tier 0 标题 |
| 3 | `backups` 表名 | 实际表名为 `backup_records`（`schema.rs` 第 73 行） | 修正表名 |
| 4 | v1.4.0 缺少 `plugin_sync` 命令模块 | `commands/plugin_sync.rs` 存在，包含 3 个命令 | 新增完整文档 |
| 5 | 数据结构中 `Rule.type` 字段名 | Rust serde 使用 `#[serde(rename = "type")]` 即 JSON 中为 `type`，非 `rule_type` | 修正字段说明 |

---

## 第 8 部分：版本历史

| 版本 | 日期 | 变更说明 |
|-----|------|---------|
| 1.0.0 | 2026-06-12 | 初始版本 |
| 1.1.0 | 2026-06-12 | 新增 CLI 工具管理模块 |
| 1.2.0 | 2026-06-12 | 新增分开安装按钮、独立更新按钮 |
| 1.3.0 | 2026-06-13 | 新增技能导入与发现模块 |
| 1.4.0 | 2026-06-16 | 完整重构，基于代码事实更新所有功能描述 |
| **2.0.0** | **2026-06-16** | **按 14 角色 · 9 步线性工作流格式重写；修正 5 处 v1.4.0 错误；新增 plugin_sync 模块；按步骤映射所有功能模块** |
| **2.1.0** | **2026-06-17** | **新增第 10 部分 Agents 模块草案（基于 agency-agents-zh 调研）；更新变更说明格式；新增表名修正记录；修正多处数据统计** |
| **2.2.0** | **2026-06-17** | **Agents 模块已实现（10 命令 + view/store/3 组件）；新增 MCP Manager 模块 FEAT-022（11 命令 + 4 张表 + 11 组件）；视图数 10→11；Store 数 16→17；命令模块 21→22；命令总数 ~137→162；数据库表 7→11；表名 `backup_records`→`backups`；plugin_marketplace 命令 17→19；file 命令 3→10；skill_marketplace 命令 12→11；anthropic_skills 命令 13→12** |
| **2.3.0** | **2026-06-17** | **plugins 模块文档化补全**：新增 4 个 Vue 组件（PluginCard/PluginDetailsDialog/SourceNoteDialog/AddRepoSourceDialog）；source_notes.json 持久化（FEAT-020）；repo_type 切换 UI（FEAT-019）；marketplace 搜索（FEAT-021）；来源安装路径展示/进度条；sync_records.json 顶层优化（FEAT-021）。本次无代码变更，仅文档更新。 |
| **2.4.0** | **2026-06-18** | **Software Management 模块文档化补全**："Software Management"二义性澄清（CLI 工具管理 FEAT-01 + 软件检测 FEAT-02 共用 software store）；CLI 工具管理补全 CliToolStatus 字段（latestVersion/needsUpgrade/hasConflict/conflictInfo/installMethod）、5 阶段操作状态机、取消/重试语义、Checking… pending 占位、Methods tooltip Teleport、install options Modal、冲突检测、工具列表从 7 增至 9（新增 deepseek-reasonix/kiro/mimo-code）；软件检测补全 selectedPlatform + localStorage 持久化、runWhenIdle 空闲调度、sync_software 语义；新增 useOperationProgress 组合式函数文档（8 阶段模型 + 阶段化 UI + 复用范围）。本次无代码变更，仅文档更新。 |

---

## 第 9 部分：附录

### 9.1 术语表

| 术语 | 定义 |
|-----|------|
| Agent | AI 代理/智能体 |
| Skill | 技能扩展，增强 AI 能力 |
| MCP | Model Context Protocol，模型上下文协议 |
| Rules | 规则文件，指导 AI 行为 |
| Marketplace | 市场，聚合插件/技能等资源的平台 |
| Skills.sh | 社区 Skills 市场 |
| Anthropic Skills | Anthropic 官方提供的技能 |
| Forge | 应用主标识符 |
| plugin_sync | 插件缓存到 CLI 工具 plugins 目录的同步机制 |
| FEAT-020 | 来源 Markdown 备注功能（SourceNoteDialog + source_notes.json） |
| FEAT-021 | marketplace 搜索 + sync_records.json 顶层优化 |
| FEAT-022 | MCP Manager 模块的内部功能编号 |
| source_notes.json | 来源备注持久化文件，存储于 `$FORGE_HOME/plugins/source_notes.json` |
| repo_type | 来源仓库类型：`market`（多插件仓库）或 `res`（单插件仓库） |
| sync_records.json | 插件同步记录文件，存储于 `~/.local/share/forge/marketplace/sync_records.json`（v2.3.0 优化为只记录顶层条目） |

### 9.2 参考资料

- [Tauri 官方文档](https://tauri.app/)
- [Tauri 2.0 迁移指南](https://v2.tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [Pinia 文档](https://pinia.vuejs.org/)
- [Rust Tokio 异步运行时](https://tokio.rs/)
- [SQLite rusqlite](https://github.com/rusqlite/rusqlite)

### 9.3 扫描验证记录

本次 v2.3.0 更新验证了以下 plugins 模块目录和文件：

| 目录/文件 | 验证内容 |
|----------|---------|
| `src/components/plugins/*.vue` | 4 个插件组件（PluginCard.vue、PluginDetailsDialog.vue、SourceNoteDialog.vue、AddRepoSourceDialog.vue） |
| `src/stores/plugin-marketplace.ts` | sourceNotes state + loadSourceNotes/saveSourceNote/addSource/removeSource/switchSourceType action |
| `src/stores/plugin.ts` | 基础插件 store（CRUD operations） |
| `src/utils/markdown.ts` | 轻量 Markdown 渲染器（12 种语法，无外部依赖） |
| `src/views/PluginsView.vue` | 完整 plugins 视图（含 3 Tab、搜索、来源管理等） |
| `src-tauri/src/commands/plugin_marketplace.rs` | 22 个命令（含 get_source_notes、save_source_note、update_source_repo_type） |
| `src-tauri/src/services/plugin_marketplace.rs` | SourceNotesRegistry、source_notes_path、read/write_source_notes |
| `src-tauri/src/commands/plugin_sync.rs` | copy_dir_inner（顶层条目优化）、remove_synced_files（目录兼容） |
| `docs/superpowers/plans/2026-06-17-source-note.md` | FEAT-020 实现计划 |
| `docs/superpowers/specs/2026-06-17-source-note-design.md` | FEAT-020 设计文档 |
| `docs/superpowers/plans/2026-06-17-sync-records-top-level-only.md` | FEAT-021 sync_records 优化计划 |
| `docs/superpowers/specs/2026-06-17-sync-records-top-level-only-design.md` | FEAT-021 sync_records 优化设计 |

本次 v2.2.0 更新验证了以下目录和文件：

| 目录/文件 | 验证内容 |
|----------|---------|
| `src/views/*.vue` | 11 个视图文件（含新增 `AgentsView.vue`） |
| `src/stores/*.ts` | 17 个 store 文件（含新增 `agent.ts`，不含 `index.ts`） |
| `src-tauri/src/commands/*.rs` | 22 个命令模块文件（新增 `agent.rs`、`mcp_manager.rs`） |
| `src-tauri/src/commands/agent.rs` | 10 个 Tauri 命令 |
| `src-tauri/src/commands/mcp_manager.rs` | 11 个 Tauri 命令 |
| `src-tauri/src/commands/file.rs` | 10 个 Tauri 命令（修正） |
| `src-tauri/src/commands/plugin_marketplace.rs` | 22 个 Tauri 命令（含 source_note、repo_type 切换命令） |
| `src-tauri/src/commands/anthropic_skills.rs` | 12 个 Tauri 命令 |
| `src-tauri/src/commands/skill_marketplace.rs` | 11 个 Tauri 命令 |
| `src-tauri/src/db/connection.rs` | 数据库表初始化（agents 表、`backups` 表） |
| `src-tauri/src/db/schema.rs` | 参考 schema（`backup_records` 表为参考未使用） |
| `src-tauri/src/db/mcp_tables.rs` | MCP Manager 4 张表的 DB 方法 |
| `src-tauri/src/services/mcp_protocol.rs` | stdio/HTTP 协议处理器、DiscoveryCache |
| `src/components/agents/*` | 3 个 Agent 组件 |
| `src/components/mcp/*` | 11 个 MCP 组件 |
| `src/types/agent.ts` | TypeScript 类型 + 18 个部门定义 |
| `src/router/index.ts` | 11 个路由（含新增 `/agents`） |
| `src/components/layout/Sidebar.vue` | Agents 入口（含数量徽章） |

### 9.4 v2.4.0 扫描验证记录

本次 v2.4.0 更新验证了以下 Software Management 模块目录和文件：

| 目录/文件 | 验证内容 |
|-----------|---------|
| `src/composables/useOperationProgress.ts` | L1-148：8 个 `OperationStage`（idle/preparing/downloading/installing/verifying/completed/failed/cancelled，L3）；`UseOperationProgressReturn` API 列表（L16-27）；`STAGE_CONFIG` 阶段配置（L138-147）；`canCancel`/`canRetry` 语义（L67/L79/L84/L95） |
| `src/stores/software.ts` | L13-19 `runWhenIdle`；L82-89 `selectedPlatform`/`setSelectedPlatform`/`localStorage`；L191-194 `installSoftware` idle 回调；L207-210 `uninstallSoftware` idle 回调；L242 `updateSoftware` idle 回调；`CliToolStatus` 接口（L32-42） |
| `src/views/CliToolsView.vue` | L57-62 v-memo/is-pending 类；L59 `getOperation`/`toolStatusMap`；L222-229 Checking 脉冲按钮；L270-296 install options Modal；L328-331 `pendingRetry`/`expandedMethods`；L356-364 `isPendingStatus`；L435-441 状态文案；L615-617 `pendingRetry` 赋值；L681-683 `handleUpgrade`；L694-703 `handleRetry`；L830-834 pending-glow 动画 |
| `src-tauri/src/commands/software.rs` | L44-55 `sync_software`（并行扫描 + 批量 upsert）；L57-62 `install_software`；L64-68 `uninstall_software`；L71-100 `update_software`；共 7 个命令 |
| `src-tauri/src/services/software_scanner.rs` | L72-447 `get_supported_software`（6 Tier + Tier 0 AI Tools）；`detect_software_parallel`（L450-458，rayon 并行）；Tier 清单与 v2.3.0 一致 |
| `src-tauri/src/services/cli_tools.rs` | L164-337 `get_supported_tools`（9 个工具）；L74-84 `CliToolStatus` 结构体；L866-894 `binary_name_candidates`（claude/gemini/cursor-agent/agent 等） |
| `src-tauri/src/commands/cli_tools.rs` | L99-106 `get_cli_tools`；L108-116 `check_cli_tool_status`；L120-134 `check_all_cli_tools_status_parallel`；L136-150 `check_all_cli_tools_status`；L152-165 `upgrade_cli_tool`；共 5 个命令 |

---

## 第 10 部分：Agents 模块（v2.2.0 — 已实现）

> 本节从 v2.1.0 的"计划"状态更新为 v2.2.0 的"已实现 + 实施记录"状态。实施周期：2026-06-17，4 天完成（估算 5-8 天）。

### 10.1 模块定位

在 Forge 现有 Skills 模块基础上，新增 **Agents 模块**——管理 AI Agent 角色定义文件（Markdown），支持浏览、搜索、安装到各 AI 工具、自定义编辑。

| 维度 | Skills | Agents |
|------|--------|--------|
| **抽象层级** | 工具/能力扩展 | 角色/人格定义 |
| **消费者** | AI 模型调用 | 用户通过切换角色定制 AI 行为 |
| **数据来源** | 各类插件/技能市场 | agency-agents-zh 翻译、社区贡献、用户自建、Git 仓库 |
| **核心操作** | 安装/启用/更新 | 浏览/搜索/安装/编辑/导出 |

### 10.2 数据模型

**数据库表**（`src-tauri/src/db/connection.rs` 第 206-224 行）：

```sql
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    emoji TEXT,
    color TEXT,
    department TEXT NOT NULL,              -- engineering / design / product / ...
    content TEXT NOT NULL,                -- 完整 Markdown 正文
    source TEXT NOT NULL DEFAULT 'builtin', -- builtin/community/custom/repository
    tags TEXT,                            -- JSON array
    installed_targets TEXT,                -- JSON array: ["claude-code","cursor",...]
    is_custom INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_agents_department ON agents(department);
```

**TypeScript 类型**（`src/types/agent.ts`）：包含 18 个部门静态定义。

### 10.3 安装到各工具的路径映射

**已实现**：

| 工具 | 安装路径 | 格式转换 |
|------|---------|---------|
| Claude Code | `~/.claude/agents/{slug}.md` | 直接使用 |
| Cursor | `<project>/.cursor/rules/{slug}.mdc` | `.md` → `.mdc`（frontmatter 包装） |
| Copilot | `~/.github/agents/{slug}.md` | 直接使用 |

**待实现**（代码中未实现，PRD 中曾描述）：

| 工具 | 计划路径 | 状态 |
|------|---------|------|
| OpenCode | `<project>/.opencode/agents/{slug}.md` | 待实现 |
| Windsurf | `<project>/.windsurfrules` | 待实现 |
| Codex | `~/.codex/agents/{slug}.md` | 待实现 |
| OpenClaw | `~/.openclaw/agents/{slug}.md` | 待实现 |

### 10.4 UI 结构

- **主页面**：`src/views/AgentsView.vue`
  - 顶部：搜索框 + "导入 agency-agents-zh" 按钮 + "+ 自建 Agent" 按钮
  - 左侧：部门分类面板（Department 侧边栏，18 个部门）
  - 右侧：Agent 卡片网格（AgentCard）
- **弹窗**：AgentDetailsDialog（详情）、AgentImportDialog（导入）
- **导航**：Sidebar 新增 Agents 入口（显示徽章数量）
- **自动导入**：`AgentsView.vue` 的 `onMounted` 检测到 agents 表为空时，自动从 `~/.forge/agents/marketplace/agency-agents-zh` 目录导入 Agent

### 10.5 与现有模块的复用

| 已有模块 | 可复用部分 |
|----------|-----------|
| **Skills** | 整体架构模式（store/commands/views），安装到工具的文件操作逻辑 |
| **MCP Marketplace** | GitHub API 调用、分页加载、在线安装流程 |
| **Plugins** | 卡片式 UI、详情弹窗、启用/禁用状态管理 |
| **Rules** | Markdown 内容编辑、文件同步逻辑 |
| **Backup** | 批量导入/导出的文件操作模式 |

### 10.6 实施记录

**实施周期**：2026-06-17（4 天完成，估算 5-8 天）

| 阶段 | 内容 | 状态 |
|------|------|------|
| **Phase 1：基础框架** | 数据层（model/schema/CRUD commands）+ 前端骨架（types/store/view/router/sidebar） | 完成 |
| **Phase 2：导入与浏览** | import_from_agency_agents 服务 + AgentCard / DepartmentPanel + 搜索/过滤 | 完成 |
| **Phase 3：安装与编辑** | 文件路径映射 + 格式转换 + AgentDetailsDialog + AgentImportDialog | 完成 |
| **Phase 4：高级（可选）** | GitHub 在线导入 / Agent 编排（DAG）/ 分享与导出 | **未实现** |

### 10.7 关键注意事项

1. **文件格式兼容**：agency-agents-zh 使用标准 YAML frontmatter，解析时需处理中文字符
2. **路径差异**：不同工具的 agent 目录不同，macOS/Linux/Windows 路径也不同
3. **幂等安装**：同一 Agent 重复安装应覆盖而非重复创建
4. **大文件处理**：部分 Agent 内容较长（含代码示例），SQLite TEXT 字段无长度限制但前端渲染需考虑
5. **编码问题**：Markdown 文件中的中文、代码块、特殊字符需确保 UTF-8 编码一致
6. **已实现的 3 个安装目标**：仅 claude-code / cursor / copilot 已实现；opencode / windsurf / codex / openclaw 待后续实现
7. **自动导入逻辑**：`onMounted` 检测 agents 表为空时自动导入，无需手动触发

### 10.8 实施时再产出的文档

完整的技术 schema、Tauri commands、TypeScript 类型、Pinia store、Vue 模板等代码级细节在 9 步流的步骤 3、4 中产出（`docs/features/FEAT-11-agents/{tech-spec.md,dev-done.md}` 等），**不在 PRD 中照搬代码**。本节为产品/项目管理层概要。

---

## 第 11 部分：MCP 管理器（FEAT-022，v2.2.0）

> 本节记录 MCP Manager 模块（内部编号 FEAT-022）的完整规格。

### 11.1 模块定位

MCP Manager 是对现有 MCP 模块的功能增强，提供更丰富的 MCP 服务管理能力：

| 维度 | 现有 MCP 模块（`mcp.rs`/`mcp_marketplace.rs`） | MCP Manager（FEAT-022） |
|------|--------------------------------------------|------------------------|
| **核心能力** | CRUD、健康检查、市场安装 | 详情查看、工具调用、服务发现、导入/导出、分组、审计 |
| **数据持久化** | `mcp_services` 表 | `mcp_health_log`、`mcp_groups`、`mcp_service_groups`、`mcp_audit_log` |
| **协议层** | 无（仅 CLI 配置管理） | `mcp_protocol.rs`（stdio/HTTP） |

### 11.2 数据模型

**新增 4 张表**（`src-tauri/src/db/schema.rs` 第 103-152 行）：

| 表名 | 用途 | 主要字段 |
|------|------|---------|
| `mcp_health_log` | MCP 健康历史 | id, service_id, status, latency_ms, error_message, checked_at |
| `mcp_groups` | MCP 分组 | id, name, color, is_visible, created_at |
| `mcp_service_groups` | 服务-分组关联 | service_id, group_id（联合主键） |
| `mcp_audit_log` | 操作审计日志 | id, actor, action, service_id, service_name, details, status, created_at |

**新增索引**：

| 索引 | 表 | 字段 |
|------|-----|------|
| `idx_mcp_health_log_service_checked` | mcp_health_log | service_id, checked_at DESC |
| `idx_mcp_audit_log_created` | mcp_audit_log | created_at DESC |
| `idx_mcp_audit_log_service` | mcp_audit_log | service_id |
| `idx_mcp_service_groups_group` | mcp_service_groups | group_id |

### 11.3 协议层

`src-tauri/src/services/mcp_protocol.rs`（约 23K）实现两种 MCP 协议：

**stdio 协议**：
- 通过 `tokio::process::Command` 启动 MCP 服务器进程
- 通过 stdin/stdout 进行 JSON-RPC 通信
- 超时控制（`tokio::select!`）

**HTTP 协议**：
- 通过 `reqwest` 发送 HTTP POST 请求到 MCP 服务器端点
- 支持认证头传递

**DiscoveryCache**：
- 缓存服务发现结果（避免每次调用重新发现）
- TTL：5 分钟
- 容量上限：100 条记录
- 淘汰策略：LRU（Least Recently Used）

### 11.4 命令清单

| Command | 说明 |
|---------|------|
| `get_mcp_service_detail` | 获取 MCP 服务详情（含可用工具列表） |
| `invoke_mcp_tool` | 调用 MCP 服务提供的工具（JSON-RPC） |
| `discover_mcp_service` | 发现 MCP 服务的工具能力（带缓存） |
| `export_mcp_services` | 导出 MCP 服务配置，支持 JSON/YAML 格式 |
| `import_mcp_services` | 导入 MCP 服务配置（JSON/YAML，支持 skip/overwrite 冲突处理） |
| `get_mcp_health_history` | 获取 MCP 服务的健康历史记录 |
| `get_mcp_groups` | 获取所有 MCP 分组 |
| `create_mcp_group` | 创建 MCP 分组 |
| `update_mcp_group` | 更新 MCP 分组（名称、颜色、可见性） |
| `delete_mcp_group` | 删除 MCP 分组 |
| `get_mcp_audit_log` | 获取 MCP 操作审计日志（按时间/服务筛选） |

### 11.5 UI 组件

| 组件 | 说明 |
|------|------|
| `MCPServerCard.vue` | MCP 服务卡片（显示状态、健康、快捷操作） |
| `MCPDetailsDialog.vue` | 服务详情（37K，含工具调用表单） |
| `MCPExportDialog.vue` | 导出配置对话框 |
| `MCPImportDialog.vue` | 导入配置对话框（skip/overwrite 模式选择） |
| `MCPInstallDialog.vue` | 安装对话框 |
| `MCPInvocationDialog.vue` | 工具调用对话框（参数输入 + 结果展示） |
| `MCPServerFormDialog.vue` | 服务创建/编辑表单 |
| `MCPSyncSettingsDialog.vue` | 同步设置对话框 |
| `MCPGroupsPanel.vue` | 分组管理面板（CRUD） |
| `MCPAuditLogTable.vue` | 审计日志表格（时间/服务/操作者筛选） |
| `MCPHealthBadge.vue` | 健康状态徽章组件 |

### 11.6 导入/导出模式

- **格式**：JSON 和 YAML 两种格式
- **skip 模式**：当服务已存在时跳过（保留现有配置）
- **overwrite 模式**：当服务已存在时覆盖（使用导入的版本）
- **错误报告**：逐条记录每条导入的成功/失败状态及原因

### 11.7 审计日志

审计日志记录所有 MCP 操作的详细信息：

| 字段 | 说明 |
|------|------|
| actor | 操作者（`user` / `system`） |
| action | 操作类型（create / update / delete / invoke / import / export / ...） |
| service_id | 关联的服务 ID |
| service_name | 关联的服务名称 |
| details | 操作详情（JSON 字符串） |
| status | 操作结果（`success` / `failure`） |
| created_at | 操作时间 |

### 11.8 与现有 MCP 模块的关系

```
mcp_marketplace.rs   MCP 市场（来源管理、安装、卸载）
        ↓
    mcp.rs           MCP 服务 CRUD（添加、更新、删除、健康检查）
        ↓
mcp_manager.rs       MCP 管理器（详情、调用、发现、导入/导出、分组、审计）
```

三个模块协同工作：
- `mcp_marketplace.rs` 负责从市场获取和安装 MCP 服务器
- `mcp.rs` 负责已安装 MCP 服务的基础管理
- `mcp_manager.rs` 负责增强管理（协议层调用、审计、分组）

### 11.9 性能要点

- **DiscoveryCache**：使用 `std::collections::HashMap` 缓存服务发现结果，避免每次调用重新启动进程（减少 N+1 开销）
- **容量上限 + LRU 淘汰**：防止缓存无限增长
- **异步协议处理**：`tokio::process::Command` 异步执行，不阻塞 UI 线程
- **批量审计写入**：审计日志异步写入，不影响主流程性能

---

*本 PRD 文档基于代码实现逐项验证生成，所有功能描述均可在 `src/` 和 `src-tauri/src/` 目录中找到对应实现。*

# Forge Desktop

> 跨平台 AI 工具配置管理桌面应用 · Tauri 2.0 + Vue 3 + TypeScript + Rust

**AI Agent 速成指南**：用最少阅读获取项目心智模型，知道关键文件在哪、规则是什么。

---

## 1. 项目定位

**Forge Desktop = 桌面端 AI 工具编排器**：用统一 UI 管理 CLI 工具的安装升级、AI 客户端的插件/技能/规则/MCP 服务器注册、跨 23 个 AI 客户端的多端同步，并基于 `allagents` CLI 引擎完成 `workspace.yaml` 驱动的声明式分发。

**关键决策**：本地 SQLite 是 UI 状态的唯一真相，`allagents` 仅作为**同步执行器**（不直接驱动 UI 渲染）。

### 核心能力

| 能力域 | 关键内容 | 入口视图 |
|--------|----------|----------|
| **CLI 工具管理** | 12 个工具的检测/版本/升级 | `CliToolsView` |
| **软件检测** | 25+ 软件路径与版本扫描 | `SoftwareManagementView` |
| **插件管理** | 11 类市场、能力探测、跨 CLI 同步 | `PluginsView` |
| **技能管理** | 6 类源、ZIP/仓库/本地导入 | `SkillsView` |
| **MCP 服务** | marketplace 安装、HTTP/stdio 双传输、健康检查 | `MCPView` |
| **规则 / Agent / 备份 / 导入导出 / 设置 / 提示词** | 见 `src/views/` 12 个视图 | — |

> 详见 `src-tauri/src/lib.rs::tauri::generate_handler!`（注册 95 个命令）和 `src/router/index.ts`（12 条路由）。

### AllAgents 集成

[`allagents`](https://www.npmjs.com/package/allagents) ^1.12.0 是**统一的客户端分发引擎**：
- **声明式**：`workspace.yaml` 描述所有插件/技能/MCP/客户端 → `allagents update` 跨 23 客户端同步
- **非破坏性**：`.allagents/sync-state.json` 追踪已同步文件
- **客户端矩阵**：通用（copilot/codex/opencode/gemini/ampcode/vscode/replit/kimi/universal）+ 专属（claude/cursor/factory/windsurf/cline/continue/roo/kilo/trae/augment/zencoder/junie/openhands/kiro）

> 📚 深入：`docs/allagents-integration-plan.md` · `docs/DEVELOPMENT-PLAN.md` · `docs/MIGRATION-GUIDE.md`

---

## 2. 技术栈与数据流

### 技术栈

| 层 | 技术 | 版本 |
|----|------|------|
| 桌面框架 | Tauri | 2.0 |
| 前端 | Vue 3 (Composition) + TS + Vite + Pinia | 3.5 / 5.8 / 6.3 / 3.0 |
| 样式 | Tailwind（实际以 CSS 变量为主） | 3.4 |
| 后端 | Rust (edition 2021) + SQLite (rusqlite) | 1.75+ / 0.32 |
| 异步 | Tokio (full) + Rayon | 1.0 / 1.10 |
| 同步引擎 | `allagents` CLI | ^1.12.0 |
| 测试 | Vitest（前端）+ `#[cfg(test)] mod tests`（后端） | 4.1 |

### 数据流

```
Vue 3 (views ↔ Pinia stores)
  ↓ invoke<CommandResult>
Rust commands (95 个注册于 lib.rs)
  ↓ AllAgentsService (auto-install + execute + emit events)
allagents CLI 子进程 → forge.db (SQLite) + ~/.local/share/forge/
```

**关键点**：
- 所有 Tauri IPC 统一返回 `CommandResult { success, data?, error? }`（Rust 端 `Result<T, String>`）
- 事件总线：`emit('allagents:event')` 等 5 类事件，详见 §4.4

### 仓库布局（精简）

```
src/                    Vue 3 前端
  views/                12 个页面
  stores/               27 个 Pinia（业务 + unified-* 适配器）
  composables/          useAllAgents / useErrorHandler / useClientSync / useVirtualGrid
  services/             allagents-config.ts（workspace.yaml 生成器）
  types/unified-plugin.ts  核心类型定义
  assets/               主题 CSS + 静态资源
src-tauri/src/          Rust 后端
  commands/             25 个模块（含 allagents_commands.rs 的 17 个 IPC）
  services/             12 个服务（allagents_service / cli_tools 等）
  db/                   connection.rs / kv_store.rs / mcp_tables.rs / schema.rs
  lib.rs                Tauri Builder + 95 个命令注册
docs/                   设计/计划/审查/测试文档
guide/                  PRD + 设计规范
```

---

## 3. AllAgents 集成层（核心架构）

### 3.1 类型系统 · `src/types/unified-plugin.ts`

```typescript
interface UnifiedPlugin {
  id: string;
  type: 'skill' | 'agent' | 'rule' | 'mcp' | 'hook' | 'command';
  source: PluginSource;                    // marketplace | github | local | url
  scope: 'user' | 'project';
  installed: boolean; enabled: boolean;
  syncStatus: 'synced' | 'pending' | 'partial' | 'error' | 'conflict' | 'unknown';
  targetClients: string[];                 // 空数组 = 全部
  allagentsSpec?: string;                  // "name@owner/repo"
}

const SUPPORTED_CLIENTS = [...23 个客户端...] as const;
```

### 3.2 服务层 · `src-tauri/src/services/allagents_service.rs`

| 结构 | 作用 |
|------|------|
| `AllAgentsService` | 主服务，含 `EventEmitter` 推送 Tauri 事件 |
| `AllAgentsConfig` | 配置（workspace_path / cli_path / auto_install） |
| `AllAgentsEvent` | 4 类事件：`sync-progress` / `sync-complete` / `sync-error` / `config-changed` |
| `WorkspaceConfig` | YAML 数据模型（与前端镜像） |
| `SyncReport` | 同步结果（synced_files / errors / skipped） |

**关键方法**：
- `ensure_installed()` → `which allagents` → `npm install -g` 自动安装
- `init_workspace(from)` / `update(offline, dry_run, client_filter)` → `SyncReport`
- `write_config(&WorkspaceConfig)` / `read_config()` / `update_config()` — workspace.yaml 读写

### 3.3 Tauri IPC 命令 · `src-tauri/src/commands/allagents_commands.rs`（17 个）

| 类别 | 命令 |
|------|------|
| 工作区 | `allagents_init` · `allagents_update` · `allagents_status` |
| 插件 | `allagents_plugin_install` · `_uninstall` · `_list` |
| 技能 | `allagents_skill_list` · `_add` · `_remove` |
| MCP | `allagents_mcp_add` · `_remove` · `_list` · `_update` |
| Marketplace | `allagents_marketplace_add` · `_remove` · `_list` |
| 配置 | `allagents_generate_config` |

所有命令统一返回 `CommandResult`，前端通过 `useAllAgents` 组合式函数封装（提供 `installAndSync` / `uninstallAndSync` / `addMcpAndSync` / `removeMcpAndSync` / `sync` / `generateWorkspaceYaml` 等快捷方法）。

### 3.4 前端封装

| 文件 | 作用 |
|------|------|
| `src/composables/useAllAgents.ts` | 响应式封装 + 快捷方法 |
| `src/services/allagents-config.ts` | workspace.yaml 生成器 |
| `src/stores/unified-plugin.ts` | 统一 Plugin Store（参考实现） |
| `src/stores/unified-skill.ts` | 统一 Skill Store |
| `src/stores/unified-{agent,rule,mcp,plugin-adapter}.ts` | 适配器 |

事件监听：
```typescript
import { listen } from '@tauri-apps/api/event';
await listen('allagents:event', (e) => {
  const { type, ...data } = e.payload;
  // type: 'sync-progress' | 'sync-complete' | 'sync-error' | 'config-changed'
});
```

---

## 4. 其他核心模块

### 4.1 Store 迁移矩阵（适配器模式）

| 旧 Store | 新 Store/Adapter | 状态 |
|----------|------------------|------|
| `useSkillStore` / `useSkillMarketplaceStore` / `useSkillImportStore` / `useAnthropicSkillsStore` / `useSkillsShStore` | `useUnifiedSkillStore` | ✅ 已合并 |
| `usePluginMarketplaceStore` | `useUnifiedPluginAdapterStore` | ✅ 委托 allagents |
| `useAgentStore` / `useRuleStore` | `useUnifiedAgentStore` / `useUnifiedRuleStore` | ✅ 已迁移 |
| `useMcpStore` / `useMcpMarketplaceStore` | `useUnifiedMcpStore` | ✅ 保留 health/audit/grouping |
| `useSoftwareStore` / `useSettingsStore` | 保留原状（独立域） | ✅ 不动 |

> ⚠️ 新代码**必须**使用统一 Store；旧 API 适配器兼容但不再演进。

### 4.2 SQLite 数据层 · `src-tauri/src/db/`

- **单例**：`Database { conn: Mutex<Connection> }` + 全局 `GLOBAL_DB: OnceLock<Database>`，启动期注入
- **13 张表**：`software / plugins / skills / mcp_services / mcp_health_log / mcp_groups / mcp_service_groups / mcp_audit_log / rules / backups / agents / config_templates / kv_store`
- **统一 K/V 存储** (`kv_store.rs`)：替代散落 JSON 文件，启动期 `migrate_json_to_kv` 自动迁移，原文件改为 `.json.bak`
- **Schema 迁移**：`CREATE TABLE IF NOT EXISTS` + 启动期 `ALTER TABLE ADD COLUMN`（已存在则忽略）

### 4.3 CLI 工具管理 · `src-tauri/src/services/cli_tools.rs` (1246 行)

支持的 12 个工具：`claude-code / codex / gemini-cli / opencode / openclaw / hermes / cursor / deepseek-reasonix / mimo-code / qwen-code / copilot`

每个 `CliToolConfig` 含 `key / name / icon / install_methods / npm_package / plugin_dir / install_timeout_secs`（Hermes 为 600s，其余 300s）。

- **检测**：`npm list -g --depth=0 <pkg>` → 备选 `binary_name_candidates()` → 失败返回 `(false, ...)`
- **升级**：解析命令（`| / && / || / >` 走 shell，否则 `shell_words` 分词）→ Windows 加 `CREATE_NO_WINDOW (0x08000000)` → `tokio::select!` 超时 → 成功后 `--version` 确认

### 4.4 关键事件总线

| 事件名 | Payload |
|--------|---------|
| `allagents:event` | `{ type, ... }`（4 子类型见 §3.2） |
| `skill-install-progress` | `{ stage, progress, message, error }` |
| `plugin-install-progress` | `{ pluginId, stage, progress }` |
| `source-install-progress` | `{ sourceId, stage, progress }` |
| `plugin-sync-progress` | `{ pluginId, cliToolKey, stage, progress }` |

### 4.5 主题系统 · `src/assets/theme.css`

5 套主题：`dark / warm / cool / soft / amber`，Token 分级（z-index 7 级 · 圆角 5 级 · 按钮高 4 级 · 断点 4 级）。`useThemeStore().initTheme()` 在 `main.ts` 挂载时从 localStorage 恢复。

### 4.6 GitHub Token 存储 · `src-tauri/src/commands/settings.rs`

- 路径：`<app_data_dir>/github_token`（**不**入库，避免 VACUUM INTO 泄漏）
- Unix 权限 `0o600` + 原子写入（先 `.tmp` 后 `fs::rename`）
- 优先级：settings 文件 > `GITHUB_TOKEN` 环境变量
- UI 只暴露 `mask_token()` 预览（前缀+后 4 字符）
- 命令：`has_github_token` / `get_github_token_preview` / `set_github_token` / `clear_github_token`

---

## 5. 开发指南

### 5.1 环境要求

| 工具 | 版本 |
|------|------|
| Node.js / pnpm | 20+ / 推荐 |
| Rust / Cargo | 1.75+ / 随 Rust |
| Tauri CLI | 2.0+（`@tauri-apps/cli`） |
| **allagents** | **^1.12.0**（运行时必须，自动 `npm install -g`） |

### 5.2 常用命令

```bash
# 开发
pnpm dev               # Tauri 开发模式（前端 + 后端）
pnpm dev:web           # 仅前端（Vite dev server :1420）
pnpm tauri:dev:win     # Windows（含 vcvarsall）
pnpm tauri:dev:unix    # macOS/Linux
pnpm tauri build       # 完整打包

# 测试与质量
pnpm test / test:watch / test:coverage
pnpm lint              # ESLint
pnpm typecheck         # vue-tsc --noEmit
pnpm plugins           # 插件管理 CLI（scripts/plugins/）
```

### 5.3 添加功能的标准路径

**以"新增 AI 客户端 X"为例**：

1. **类型** — `src/types/unified-plugin.ts`：`SUPPORTED_CLIENTS` 加 `'x'`，`CLIENT_DISPLAY_NAMES` 加映射，`/icons/x.svg`
2. **Tauri 命令**（如需）— `src-tauri/src/commands/*.rs`：`#[command] async fn xxx` + `commands/mod.rs` 注册模块
3. **`lib.rs`** — `tauri::generate_handler![...]` 宏注册
4. **Capabilities** — `src-tauri/capabilities/*.json` 加权限
5. **Store** — `src/stores/*.ts` 或直接调用 `allagents_xxx`（推荐）
6. **UI** — `src/views/PluginsView.vue` 等视图

### 5.4 添加 Store 的标准路径

1. `src/stores/<name>.ts` — 使用 `defineStore('<name>', () => { ... })` 优先 Setup Store
2. 在 `src/stores/index.ts` 聚合导出
3. 单测位于 `src/stores/__tests__/<name>.test.ts`

### 5.5 单测约定

- 前端：`vitest` + `@vue/test-utils` + `jsdom`，目录：`src/{composables,services,stores,types,utils,components}/__tests__/`
- 后端：`#[cfg(test)] mod tests`（文件内）或 `src-tauri/tests/`（集成测试）

---

## 6. 代码规范（强制）

### 6.1 强制规则

1. **所有回复使用中文**
2. **使用统一类型系统** —— 新代码必须用 `UnifiedPlugin` / `UnifiedMCP`
3. **Store 迁移采用适配器模式** —— 旧 API 兼容，新代码用新 Store
4. **统一错误处理** —— 使用 `useErrorHandler`（带重试、用户友好消息）

### 6.2 TypeScript / Rust 约定

| TypeScript | Rust |
|------------|------|
| 严格模式（`@vue/tsconfig`） | Edition 2021 |
| `<script setup lang="ts">` + Composition API | `thiserror::Error` 派生错误（如 `CliToolError`） |
| Setup Store 优先 | `tokio::select!` + 自定义超时（避免 `tokio::time::timeout`，Windows 兼容） |
| Tauri IPC 包装返回 `CommandResult` | 命令统一 `Result<T, String>` |
| 大 Map/Set 变更要**整体替换**引用（避免响应式失效） | Windows 加 `creation_flags(0x08000000)` 防控制台弹窗 |
| 避免 `any`（历史遗留 `data?: any` 应改 `CommandResult`） | `Mutex<Connection>` 全局单例，所有 CRUD 走 `Database` 方法 |

### 6.3 命名约定

| 类型 | 风格 | 示例 |
|------|------|------|
| Rust 模块/函数 | snake_case | `allagents_service.rs` · `ensure_installed` |
| Rust 类型 | PascalCase | `AllAgentsService` |
| Tauri 命令 | snake_case | `allagents_plugin_install` |
| TS 文件 | camelCase | `useAllAgents.ts` |
| TS 类/接口 | PascalCase | `UnifiedPlugin` |
| TS 函数/变量 | camelCase | `installAndSync` |
| Pinia Store | `useXxxStore` | `useUnifiedPluginStore` |
| CSS 变量 | kebab-case | `--btn-height-sm` |

### 6.4 提交前自检

- [ ] `pnpm lint` / `typecheck` / `test` 全过
- [ ] 新增/修改命令已在 `lib.rs::generate_handler!` 注册
- [ ] 新增权限已在 `capabilities/*.json` 注册
- [ ] 新增数据库表已加 `IF NOT EXISTS` + 索引
- [ ] 涉及 file I/O 用 `path.join` / `Path::join`（跨平台）
- [ ] 涉及子进程调用已加超时与 `CREATE_NO_WINDOW`（Windows）

---

## 7. 常见问题

### 7.1 allagents 未安装 / 版本不符

`AllAgentsService::ensure_installed()` 首次调用会自动 `npm install -g allagents@^1.12.0`。手动修复：`npm install -g allagents@^1.12.0`，确认 `allagents --version >= 1.12.0`。Windows 优先走 Bun，未装时 fallback Node.js。

### 7.2 数据库锁死

```bash
pkill -f forge-desktop                              # 关闭所有实例
sqlite3 ~/.local/share/forge/forge.db "PRAGMA integrity_check;"
mv ~/.local/share/forge/forge.db{,.bak}              # 备份后删除，自动重建
```

> 数据库调试：macOS 路径 `~/Library/Application Support/com.forge.desktop/`

### 7.3 Vue 响应式失效（Map / Set）

`Map.set()` 不触发响应式——必须整体替换引用：

```typescript
// ❌
resolvedVersions.value.set(key, version);
// ✅
const next = new Map(resolvedVersions.value);
next.set(key, version);
resolvedVersions.value = next;
```

### 7.4 Tauri invoke 字段命名

Tauri 2.x 默认：`snake_case`（Rust）↔ `camelCase`（TS）自动转换。如 `{ workspacePath: '...' }` 自动匹配 Rust 的 `workspace_path`。如需关闭：`tauri.conf.json` 设置 `app.withGlobalTauri: false`。

### 7.5 跨平台路径

- **永远**用 `path.join` / `Path::join`，**不**用字符串拼接
- 用户目录：`dirs::data_local_dir()` / `dirs::config_dir()`
- Tauri 路径：`app.path().app_data_dir()` / `app_config_dir()`
- Windows 不要硬编码 `\\`

### 7.6 调试流程

1. **前端**：`console.log` + Tauri devtools
2. **后端**：命令首行 `log::info!("xxx called: {:?}", args)`
3. **日志**：`tail -f ~/.local/share/forge/logs/app_*.log | grep -i <keyword>`
4. **DB**：`sqlite3 ~/.local/share/forge/forge.db`
5. **allagents**：在命令前加 `allagents --json <subcmd> --debug` 或 `RUST_LOG=debug`

### 7.7 常见报错

| 错误 | 修复 |
|------|------|
| `allagents: command not found` | §7.1 |
| `Failed to initialize database` | 检查 app_data_dir 磁盘权限 |
| `Lock file exists` | 删除 `<app_data_dir>/.lock` |
| `Plugin not found: <name>` | `npm run plugins:init` 重建 marketplace |
| `workspace.yaml parse error` | 删除重建：`allagents init .` |
| `Permission denied (os error 13)` | `chmod 600` 对应 secret 文件 |
| `Cannot find module 'X'` | `rm -rf node_modules && pnpm install` |
| `EACCES: permission denied`（npm） | `sudo chown -R $USER /usr/local/lib/node_modules` |

---

## 8. 速成路径与扩展阅读

### 8.1 30 分钟阅读顺序

| 分钟 | 内容 | 目标 |
|------|------|------|
| 0-3 | 本文件 §1-2 | 知道项目是什么、技术栈 |
| 3-6 | `docs/MIGRATION-GUIDE.md` | 理解新旧 Store 关系 |
| 6-10 | `src/types/unified-plugin.ts` + `src/composables/useAllAgents.ts` | 掌握核心 API |
| 10-14 | `src-tauri/src/services/allagents_service.rs` | 理解后端同步机制 |
| 14-18 | `src/stores/unified-plugin.ts` | 一个完整 Store 范例 |
| 18-22 | `src/views/PluginsView.vue` | UI 消费 Store 的模式 |
| 22-25 | `src-tauri/src/lib.rs` | 95 个命令注册全景 |
| 25-30 | `docs/DEVELOPMENT-PLAN.md` §3 | 当前进度与待办 |

**完成目标**：能够解释 4 套 Skill 接口如何被 `UnifiedPlugin` 统一；描述 `allagents update` 端到端同步路径；新增 Tauri 命令并完成前端 Store 封装；区分 DB（真相）与 allagents（执行器）边界。

### 8.2 扩展阅读

| 主题 | 文档 |
|------|------|
| 集成方案细节 | `docs/allagents-integration-plan.md` |
| 9 周开发计划 | `docs/DEVELOPMENT-PLAN.md` |
| Store 迁移细节 | `docs/MIGRATION-GUIDE.md` |
| UI 整改交付 | `docs/delivery-report.md` |
| 产品需求 / 设计规范 | `guide/prd.md` · `guide/design.md` |
| 审查 / 测试 / 规格 | `docs/review/` · `docs/test/` · `docs/tech-spec/` |
| 功能特性 | `docs/features/FEAT-*/` |
| Cursor 工作流 | `.cursor/rules/workflow.mdc` |
| 简版指南 | `AGENTS.md` |

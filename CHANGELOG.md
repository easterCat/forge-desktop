# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> 项目尚未发布（`0.1.0` 为预发布前的内部版本）。本文档记录从首次 commit
> (`init: forge desktop app`) 到当前 `main` 分支之间的累计变更。
> 一旦发布第一个稳定版本，将按 `## [Unreleased]` / `## [1.0.0]` 的结构
> 划分，并在每个版本下分 *Added / Changed / Fixed / Security / Performance*
> 五个子段落。

---

## [Unreleased]

### Added

- **AllAgents 集成架构** — 完整插件、技能、Agent、规则、MCP 统一管理。
  新增 17 个 Tauri IPC 命令（`allagents_init` / `allagents_update` 等），
  同步事件系统，支持 23 个 AI 客户端（Copilot、Cursor、Codex、Windsurf
  等）。详细说明见 `docs/allagents-integration-plan.md`。
- **CliSyncChip 客户端同步** — 单组件 + 对话框 + composable + 集成测试。
  `useClientSync` 提供 `initClients` / `toggleSync` / `syncAll` 三个核心
  操作，错误全部通过项目自有 toast 系统上报（不再依赖
  `vue-toastification`）。
- **License** — 仓库根添加 `MIT` License 文件（与 README 中声明一致）。
- **CI** — 添加 GitHub Actions (`.github/workflows/ci.yml`)：
  前端 `pnpm install --frozen-lockfile` + typecheck + lint (advisory) +
  format:check (advisory) + vitest；后端 `cargo fmt` + clippy (advisory) +
  cargo build + cargo test。Concurrency group 防止同分支重复触发。

### Changed

- `CliSyncChip` 默认值：`showSyncCount` 由 `true` 改为 `false`，避免在
  初次进入页面时显示误导性的同步计数。
- **RepositoryDialog.vue 模板结构修复** — 删除 L60-L71 错误的
  `</Teleport>` + `</template>` 提前关闭，恢复 Vue 编译器期望的
  `Teleport > div.dialog-overlay > div.dialog > ...` 嵌套结构。修复前
  Vue 编译器在 `</template>` 处截断导致 L72 后的 dialog-content、repo
  list、actions 等内容**未被渲染**；同时清理未使用的 `computed` import。
- **`.eslintrc.json` ignore 路径** — `design/**`、`guide/**`、`docs/**`
  加入 `ignorePatterns`，原型代码 149 个 lint 错误不再阻塞 CI。
- **`scripts/plugins/*.mjs` 静态分析清理** — `no-useless-escape` 简化为
  `[^/]+`、`no-fallthrough` 加 `break`、`no-empty` 加注释意图说明。
- **`src/stores/anthropic-skills.ts` switch case 块作用域** — 给含
  `const` 声明的 case 加 `{ ... }` 包围，满足 `no-case-declarations`。
- **`src/components/mcp/MCPView.vue` & `src/views/CliToolsView.vue`**
  `inject<Function>` 改为精确的 `(message, type?) => void` 签名，满足
  `no-unsafe-function-type`。
- **`src/components/common/*.vue` 18 个组件** — 移除 template 自动解构
  时多余的 `const props = withDefaults(defineProps<...>(), ...)` 绑定，
  保留宏调用副作用；保留 `const props` 的两处（`PluginDetailsDialog`、
  `MCPInvocationDialog`，因 script 中实际使用 `props.xxx`）加 inline
  disable。

### Security

- **`PathGuard`** — 新增 `src-tauri/src/utils/path_guard.rs`（273 行 +
  6 个单元测试），覆盖路径穿越（`..` 段、绝对路径、NUL 字节）、符号链接
  逃逸、保留名（Windows `CON`/`PRN` 等）、可疑字符。所有文件型 Tauri
  命令（`file::*`）注入路径校验。
- **版本字符串校验** — `version_manager.rs` 中 `validate_version_string`
  白名单 `[0-9a-zA-Z.+_-]` + 长度上限，拒绝 shell 元字符注入；`nvm_args`
  处二次防御。
- **Tauri 锁恢复** — `mcp_manager.rs` 中所有直接调用 `connection.lock()`
  的 `unwrap()` 替换为 `read_or_recover_discovery_cache` /
  `write_or_recover_discovery_cache` helper，毒化锁不再 panic。
- **Markdown 渲染** — `utils/markdown.ts` 对输入做 256 KB 长度上限 + 占位
  符协议 + URL 净化（`javascript:` 协议过滤）。`SourceNoteDialog.vue` 同
  步应用长度限制并向用户展示警告。

### Performance

- **N+1 查询消除** — `db/mcp_tables.rs` 新增
  `get_service_groups_batch(service_ids)`，单条 `WHERE service_id IN (...)`
  查询替代 `export_mcp_services` 中逐服务调用 `get_service_groups` 的循环。
  导出的并发量不再线性放大 SQLite 互斥锁的争用开销。
- **事务包装** — `mcp_tables::set_service_groups` 与 `kv_store::import_all`
  包裹在 `BEGIN ... COMMIT`，失败时 `ROLLBACK` 还原。避免部分导入或
  部分分组变更时留下中间状态。

---

## [0.1.0] - 2026-07-04

Initial internal pre-release. Never tagged for public distribution.

### Added

- 项目初始化：Tauri 2.0 + Vue 3 + TypeScript + Pinia + SQLite 基础脚手架。
- 核心域模型：Software、MCPService、Skill、Plugin、Rule、Agent、Backup。
- 17 视图（侧栏 + 设置 + 各域详情）。
- `kv_store` 后端表 + `mcp_*` 系列表与索引。
- 单元测试基础设施（Vitest + jsdom）。

### Notes

- 95 个 TypeScript 类型错误在 `main` 上保留（与本次改动无关，源于前后端
  数据模型扩展未对齐：`DashboardView` 期望的 `color/needsUpdate/pkg/desc/
  current/latest` 字段未在 `Software` 类型中声明）。已记录到 backlog，
  CI typecheck 设为 advisory。

[Unreleased]: https://github.com/your-org/forge-desktop/compare/main...HEAD
[0.1.0]: https://github.com/your-org/forge-desktop/commit/416c520c
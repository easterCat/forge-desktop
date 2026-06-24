# FEAT-023 · plugins 模块 PRD 同步（v2.2.0 → v2.3.0）

> **类型**：文档化补全
> **日期**：2026-06-17
> **状态**：✅ 已完成

## 1. 任务目标

对 `env-manager/guide/prd.md` 中 plugins 模块相关的描述进行完善/更新，反映当前 v2.2.0 实际代码中已实现但 PRD 未充分文档化的功能点。版本从 v2.2.0 递增到 **v2.3.0**。

## 2. 背景

v2.2.0 开发期间，plugins 模块实际实现了多项功能，但 PRD（v2.2.0）存在以下文档缺口：

1. **SourceNoteDialog.vue**（备注功能 UI）— 未在 PRD 中记录
2. **SourceNote 数据持久化**（`source_notes.json`）— 未在 PRD 中记录
3. **AddRepoSourceDialog.vue** — 仅在变更说明中一笔带过，未详细文档化
4. **marketplace 搜索**（FEAT-021）— 变更说明中有，但设计细节未展开
5. **repo_type 切换**（market/res 切换）— `update_source_repo_type` 命令在变更说明中提到，但 UI 交互细节缺失
6. **sync_records.json 顶层优化**（FEAT-021）— 计划文档已存在但 PRD 未引用

## 3. 代码核查结果

### 3.1 新增/未文档化的 Vue 组件

| 组件文件 | 功能 | 状态 |
|---------|------|------|
| `src/components/plugins/SourceNoteDialog.vue` | Markdown 备注弹窗（编辑/预览 Tab） | 已实现（FEAT-020） |
| `src/components/plugins/AddRepoSourceDialog.vue` | 添加仓库源弹窗（含 repo_type 选择器） | 已实现（FEAT-018） |
| `src/components/plugins/PluginCard.vue` | 插件卡片（含安装进度条、source badge） | 已实现 |
| `src/components/plugins/PluginDetailsDialog.vue` | 插件详情弹窗（含能力探测/Hook执行/MCP探测/验证） | 已实现 |

### 3.2 新增的 Tauri 命令

| 命令 | 文件 | 功能 |
|------|------|------|
| `get_source_notes` | `commands/plugin_marketplace.rs` | 读取所有来源备注 |
| `save_source_note` | `commands/plugin_marketplace.rs` | 保存/删除来源备注 |
| `update_source_repo_type` | `commands/plugin_marketplace.rs` | 切换来源仓库类型（market ↔ res） |

### 3.3 新增的 Rust 服务函数

| 函数 | 文件 | 功能 |
|------|------|------|
| `SourceNotesRegistry` 结构体 | `services/plugin_marketplace.rs` | 备注数据模型 |
| `source_notes_path()` | `services/plugin_marketplace.rs` | 获取 `$FORGE_HOME/plugins/source_notes.json` 路径 |
| `read_source_notes()` | `services/plugin_marketplace.rs` | 从磁盘读取备注 |
| `write_source_notes()` | `services/plugin_marketplace.rs` | 写入备注到磁盘 |

### 3.4 新增的前端工具

| 文件 | 功能 |
|------|------|
| `src/utils/markdown.ts` | 轻量级 Markdown 渲染器（无外部依赖，支持 12 种语法） |

### 3.5 新增的 Pinia Store 状态

| 状态/Action | 位置 | 功能 |
|------------|------|------|
| `sourceNotes` state | `src/stores/plugin-marketplace.ts` | 来源备注映射表 |
| `loadSourceNotes()` | `src/stores/plugin-marketplace.ts` | 加载所有备注 |
| `saveSourceNote()` | `src/stores/plugin-marketplace.ts` | 保存单条备注 |
| `addSource()` | `src/stores/plugin-marketplace.ts` | 添加用户来源 |
| `removeSource()` | `src/stores/plugin-marketplace.ts` | 移除用户来源 |
| `switchSourceType()` | `src/stores/plugin-marketplace.ts` | 切换 repo_type |

### 3.6 PluginsView.vue 新增功能点

| 功能 | 标签 | 说明 |
|------|------|------|
| Marketplace 搜索栏 | FEAT-021 | 带防抖（300ms）的插件搜索框 |
| 搜索无结果空状态 | FEAT-021 | 友好提示 + 清空按钮 |
| 来源备注入口 | FEAT-020 | 来源卡片内的铅笔图标按钮 |
| 来源备注弹窗集成 | FEAT-020 | SourceNoteDialog 挂载 |
| 来源 repo_type 切换 | FEAT-019 | 已安装用户来源的 market/res 切换按钮 |
| 来源安装路径展示 | FEAT-016 | 每个来源显示已安装的路径列表 |
| 来源安装进度条 | FEAT-016 | 安装过程中的进度条和信息 |
| 预置来源保护 | — | 预置来源（anthropics/ccplugins/ananddtyagi/addyoshani）不可移除/切换类型 |
| 全部安装按钮 | FEAT-016 | 一键安装所有未安装的来源 |
| 自动导入插件 | — | Marketplace Tab 自动加载已安装来源的插件 |
| Sync Skeleton 动画 | — | 同步按钮加载骨架屏动画 |
| Copy 按钮状态 | — | 安装路径复制按钮的"Copied"状态反馈 |

## 4. 差异分析总结

### 4.1 已实现但 PRD 完全缺失的功能

1. **SourceNoteDialog.vue + source_notes.json**：完整的 Markdown 备注功能，从后端服务到前端组件已全链路实现
2. **AddRepoSourceDialog.vue**：来源添加弹窗（含 market/res 类型选择）
3. **repo_type 切换交互**：前端 UI + Tauri 命令完整实现
4. **marketplace 搜索**：带防抖的搜索栏 + 无结果空状态
5. **来源安装路径展示**：Sources Tab 中每个来源显示已安装路径列表
6. **来源安装进度条**：安装过程的状态反馈

### 4.2 PluginDetailsDialog.vue 扩展功能

原有 PRD 中提到 PluginDetailsDialog 但未记录其完整能力：
- 5 种能力统计卡片（Skills/Hooks/Commands/MCP/LSP）
- Hook 执行测试（带结果弹窗）
- MCP 连接探测（带结果弹窗）
- 路径验证报告（带详情弹窗）
- Overview/Skills/Commands/Hooks/MCP/LSP 六个 Tab

### 4.3 sync_records.json 顶层优化（FEAT-021）

- 计划文档：`docs/superpowers/plans/2026-06-17-sync-records-top-level-only.md`
- 设计文档：`docs/superpowers/specs/2026-06-17-sync-records-top-level-only-design.md`
- 目标：sync_records.json 从 ~322KB 降至 ~5-10KB
- 实现：copy_dir_inner 只记录顶层条目，remove_synced_files 通过 is_dir() 判断目录

## 5. PRD 更新方案

### 5.1 版本号更新

- 当前版本：v2.2.0
- 更新版本：**v2.3.0**
- 更新日期：2026-06-17

### 5.2 变更说明（v2.2.0 → v2.3.0）

在 `env-manager/guide/prd.md` 顶部新增变更说明，涵盖：
- 新增 4 个 Vue 组件文档化
- 新增 source_notes.json 持久化机制
- 新增 marketplace 搜索功能（FEAT-021）
- 新增来源 repo_type 切换功能
- 新增来源安装路径展示
- 新增来源安装进度条
- PluginDetailsDialog 能力扩展
- sync_records.json 顶层优化（FEAT-021）
- 新增 3 个 Tauri 命令

### 5.3 章节更新计划

| 章节 | 更新内容 |
|------|---------|
| 变更说明 | 新增 v2.3.0 变更块 |
| 3.2.3 插件管理设计 | 补充 4 个组件、备注功能、repo_type 切换、marketplace 搜索 |
| 3.3.3 插件同步架构 | 补充 sync_records.json 顶层优化方案 |
| 3.4.3 插件管理开发 | 补充新命令清单、SourceNoteDialog 组件、source_notes 持久化 |
| 第 5 部分 Tauri Commands | 补充 3 个新命令（get_source_notes/save_source_note/update_source_repo_type） |
| 第 4 部分 Pinia Stores | 确认 store 数量仍为 17（无新增 store） |
| 第 8 部分 版本历史 | 新增 v2.3.0 行 |

## 6. 后续建议

由于本次是文档化补全工作（无实际代码变更），建议：

- **步骤 2-9**：无需重复执行——所有功能已在 v2.2.0 实现
- **设计文档**：无需更新——本次仅更新产品 PRD
- **下一步**：PM 确认 PRD v2.3.0 后，文档化工作完成

## 7. 产出文件

- `env-manager/guide/prd.md` — v2.3.0 更新
- `docs/features/FEAT-023-plugins-prd-sync/task.md` — 本文档
- `docs/features/FEAT-023-plugins-prd-sync/README.md` — 状态矩阵

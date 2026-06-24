# FEAT-024 · 跨平台玻璃态原型 100% 还原

> **任务级别**：L3（大型，多视图、多角色、多阶段）
> **优先级**：P0
> **负责人**：@pm
> **创建日期**：2026-06-18
> **状态**：📝 步骤 1 立项中

---

## 1. 任务背景

`design/forge-cross-platform-glass.html`（1809 行高保真原型）定义了 Forge v3 的视觉与交互基线，但与当前 `src/` 前端实现在玻璃体系、响应式断点、主题切换、tint 氛围层动画等方面存在显著差距。本任务将该原型按 9 个主路由进行**像素级**还原。

**参考基线**：
- 视觉/交互：`design/forge-cross-platform-glass.html`（HTML 像素级，**绝对权威**）
- 设计意图/边界：`env-manager/guide/design.md` v3.0（高层级原则，**不固化数值**）
- 还原率评估：以 HTML 原型为唯一对照，design.md 仅在原型未明确时提供意图参考

**关键约束**：
- 仅还原前端，后端 API 沿用现有实现（`src-tauri/src/commands/` + `services/`）；后端缺口列入 PENDING，**不修改后端**
- 主题实现为「切换机制 + 2 套占位主题」（warm 默认 + midnight 暗色基线），**不**预置 20 套
- 现有 4 个未提交修改（`theme.css` / `Sidebar.vue` / `useGlassTheme.ts` / `SkillsView.vue`）**暂存 stash**，避免污染还原基线
- 9 个主路由全部覆盖，对接现有 Tauri API（无 API 用 mock fallback）
- design/tokens/ 骨架在本次同步建立

---

## 2. 9 视图范围

| # | 路由 | 视图文件 | 子模块 | 视图后端命令 / mock fallback |
|---|------|----------|--------|-----------------------------|
| 1 | `/` | `DashboardView.vue` | Environment Overview + Quick Actions | mock fallback |
| 2 | `/cli-tools` | `CliToolsView.vue` | Quick Sync | `cli_tools::*`（已有 5 命令） |
| 3 | `/software` | `SoftwareManagementView.vue` | — | `software::*`（已有 4 命令） |
| 4 | `/plugins` | `PluginsView.vue` | Installed / Marketplace / Sources | `plugin_marketplace::*`（已有 22 命令） |
| 5 | `/skills` | `SkillsView.vue` | All / Local / Anthropic / Marketplace / Skills.sh | `skill_marketplace::*` + `skills_sh::*`（已有） |
| 6 | `/agents` | `AgentsView.vue` | — | `agent::*`（已有） |
| 7 | `/mcp` | `MCPView.vue` | Groups / Health / Audit Log | `mcp_manager::*`（已有 11 命令） |
| 8 | `/rules` | `RulesView.vue` | — | `rule::*`（已有） |
| 9 | `/backup` | `BackupView.vue` | — | `backup::*`（已有） |
| 10 | `/settings` | `SettingsView.vue` | Theme Picker + Token + Application | `settings::*`（已有） |

> 注：原型无 `Prompts` 视图、`Import/Export` 独立视图。`/prompts` 路由保留但本次不还原；`Import/Export` 按原型合并到 SettingsView。

---

## 3. 组件与交互矩阵

### 3.1 全局 Shell（FEAT-024-A，frontend-engineer）

| 组件 | 状态 | 原型 CSS 类 | 当前实现状态 |
|------|------|-------------|--------------|
| App Frame | window frame | `.window-frame` | ❌ 未实现 |
| Sidebar | default/hover/active | `.sidebar` `.nav-item` | ⚠️ 现有，需重写 |
| Topbar | default | `.topbar` | ⚠️ 现有，需重写 |
| Mobile Tabbar | default | `.mobile-tabbar` (≤768px) | ❌ 未实现 |
| Theme Picker | 卡片网格 | `.theme-grid` `.theme-card` | ❌ 未实现 |

### 3.2 通用组件（FEAT-024-B，frontend-engineer）

| 组件 | 状态 | 原型 CSS 类 | 当前实现状态 |
|------|------|-------------|--------------|
| Button | default/hover/active/focus/disabled | `.btn` × 4 变体 | ⚠️ 现有 |
| Tab Bar | default/active | `.tab-bar` `.source-tab` | ❌ 未实现 |
| CLI Sync Chip | unsynced/syncing/synced | `.cli-sync-chip` | ⚠️ PluginCard 部分 |
| Search Input | default/focus/filled | `.search-input` | ⚠️ 现有 |
| Filter Bar | default | `.filter-bar` `.filter-select` | ⚠️ 现有 |
| Modal | enter/exit | `.modal` | ⚠️ 现有 |
| Toast | success/error/info | `.toast` × 4 类型 | ⚠️ 现有 |
| Skeleton | shimmer | `.skeleton` | ❌ 未实现 |
| Badge | 6 种类型 | `.badge` | ⚠️ 现有 |
| Stat Card | 4 种 tint | `.stat-card.tint-*` | ❌ 未实现 |
| Marketplace Card | default/hover/installed/update | `.marketplace-card` | ⚠️ PluginCard |
| Progress Badge | pulse | `.badge.progress` | ❌ 未实现 |
| Avatar | default | `.avatar` | ❌ 未实现 |
| Operation Stage | 7 阶段 | `.op-stage` | ❌ 未实现 |

### 3.3 玻璃体系（FEAT-024-C，design-ui）

| 玻璃层级 | 变量 | 浅色 | 暗色 |
|----------|------|------|------|
| Window | `--glass-window` | `rgba(255,255,255,0.25)` | `rgba(255,255,255,0.04)` |
| Sidebar | `--glass-sidebar` | `rgba(255,255,255,0.35)` | `rgba(255,255,255,0.06)` |
| Topbar | `--glass-topbar` | `rgba(255,255,255,0.38)` | `rgba(255,255,255,0.06)` |
| Card BG | `--glass-bg` | `rgba(255,255,255,0.45)` | `rgba(255,255,255,0.08)` |
| Input | `--glass-input` | `rgba(255,255,255,0.40)` | `rgba(255,255,255,0.10)` |
| Input Focus | `--glass-input-focus` | `rgba(255,255,255,0.60)` | `rgba(255,255,255,0.16)` |

### 3.4 Tint 氛围层（FEAT-024-C）

| Tint | CSS 类 | 周期 | 错峰延迟 |
|------|--------|------|----------|
| warm | `.tint-warm` | 8s (drift) + 4.5s (sweep) | 0s |
| cool | `.tint-cool` | 8s + 4.5s | -2s |
| soft | `.tint-soft` | 8s + 4.5s | -4s |
| amber | `.tint-amber` | 8s + 4.5s | -6s |

### 3.5 响应式断点（FEAT-024-C）

| 断点 | 宽度 | 关键变化 |
|------|------|----------|
| 宽视口 | ≥ 1024px | 完整布局 |
| 平板 | 768~1023px | stat 4→2 栏；settings 双列→单列；agents 网格收紧 |
| 手机 | 480~767px | 侧栏隐藏，底部 tab bar 出现 |
| 窄视口 | < 480px | stat 单列；card padding 收紧；按钮 padding/字号缩小 |

---

## 4. 子 FEAT 拆分

| 子 FEAT | 范围 | 主执行 | 状态 |
|---------|------|--------|------|
| **FEAT-024-A** | 全局 Shell：AppFrame + Sidebar + Topbar + MobileTabbar | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-B** | 通用组件库：Button/Chip/Tab/Search/Modal/Toast/Skeleton/Badge/StatCard/MarketplaceCard | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-C** | 玻璃体系 + Tint 氛围 + 响应式断点 + design/tokens/ 骨架 | `@design-ui` + `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-D** | Dashboard 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-E** | CLI Tools 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-F** | Software 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-G** | Plugins 视图还原（3 Tab） | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-H** | Skills 视图还原（5 Source Tab） | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-I** | Agents 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-J** | MCP 视图还原（3 区块） | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-K** | Rules 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-L** | Backup 视图还原 | `@frontend-engineer` | ⏳ 待执行 |
| **FEAT-024-M** | Settings 视图还原（含 Theme Picker） | `@frontend-engineer` | ⏳ 待执行 |

**执行顺序**：
- A、B、C 必须先完成（D、C 是视觉地基；B 是组件库；A 是应用骨架）
- D ~ M 可**并行**执行（互不依赖，仅依赖 A/B/C 产出的全局组件）

---

## 5. 后端 PENDING 缺口（不修改，仅记录）

| 缺口 | 视图 | mock fallback 策略 |
|------|------|---------------------|
| Dashboard 概览数据无独立 API | Dashboard | 用现有 cli_tools::get_all 数量 + 静态 mock |
| Settings Theme 应用范围配置 | Settings | 仅 UI，不持久化（stash 范围内） |
| `/prompts` 路由还原 | Prompts | 本次不还原，仅路由保留 |

---

## 6. 风险与未决事项

| 风险 | 缓解 |
|------|------|
| HTML 原型内联 CSS 移植到 Vue 组件时存在 specificity 冲突 | 优先迁移 CSS 变量到 :root，组件 scoped 仅写结构 |
| 未提交修改 4 个文件可能与本次还原冲突 | **先 stash 再开工**（用户已确认） |
| 后端 17 个 Rust 警告可能影响数据返回 | 9 步骤 5 由 review-expert 单列 PENDING，**不**让 perf 关注 |
| 9 子 FEAT 并行易出现 theme.css 冲突 | 由 design-ui 锁 theme.css 唯一编辑者；其他子 FEAT 不得改 theme.css |
| 原型未覆盖的视图（/prompts）| 路由保留，UI 标 PENDING |

---

## 7. 验收标准

- [ ] 9 路由（`/`、`/cli-tools`、`/software`、`/plugins`、`/skills`、`/agents`、`/mcp`、`/rules`、`/backup`、`/settings`）逐路由视觉与 HTML 原型**像素级一致**（含断点）
- [ ] 全局 Shell 组件（Sidebar / Topbar / MobileTabbar / AppFrame）100% 还原
- [ ] 通用组件库 13 个组件状态完整（见 §3.2）
- [ ] 玻璃体系 6 变量在浅/暗两基线均生效
- [ ] Tint 氛围层 4 种动画错峰启动，`prefers-reduced-motion` 可关闭
- [ ] 响应式 4 档断点行为与原型一致
- [ ] 主题切换机制可工作，默认 warm，可切换至 midnight 暗色基线
- [ ] `design/tokens/` 骨架建立，至少含 `colors.css` `glass.css` `motion.css` `themes/` 目录
- [ ] 9 视图所有数据通过现有 Tauri API 获取；无 API 处使用 mock fallback 并标注 PENDING
- [ ] `/prompts` 路由不还原，UI 标 PENDING

---

## 8. PM 决断（advisory 5 项）

> 解决步骤 2 / 步骤 3 中两位总监的 advisory veto 与冲突质询。

| 编号 | 决断 | 依据 |
|------|------|------|
| **PM-D1** | `useGlassTheme.ts` **硬编码玻璃值全部废弃**；改为从 `theme.css` 读 CSS 变量的薄包装层（仅暴露 `isDarkMode` / `currentGlassBg` 等 getter）。 | design-notes §1.3 冲突 B + tech-spec §3.2 双方一致建议；避免双写风险 |
| **PM-D2** | `[data-theme="warm"]` 在 FEAT-024-C 第一步**整体替换**为 HTML 原型 `:root` 的完整玻璃/色值变量；保留 `theme.css` 中 `:root` 基础变量块（`--bg` / `--fg` / `--font-*` / `--radius` 等）不动。 | design-notes §1.3 冲突 A（P0） + tech-spec §2.2 保留项；还原率 100% 必要条件 |
| **PM-D3** | `AppFrame.vue` 新增 `.window-frame` 包装层；macOS 上 Tauri 窗口圆角与 `border-radius: 16px` 双重生效（OS 圆角外层 + CSS 圆角内层，不冲突）。 | design-notes §1.3 冲突 A + tech-spec R-04 合并结论 |
| **PM-D4** | `localStorage` 键名采用 `forge-theme`（tech-spec §3.1），覆盖 design-notes §5.4 建议的 `aem-theme`。 | 项目命名空间一致性 |
| **PM-D5** | `useThemeStore` 提供 `setGlassVariant()` API（tech-spec §3.1）但本次**不暴露给 UI**——仅保留 API 形态，Theme Picker 只显示 2 套占位主题（warm / midnight），不显示变体选择器。 | 范围约束：仅机制 + 2 套占位；为 FEAT-025 留口 |

## 9. 不在本次范围

- 后端 API 扩展或修复（所有缺口 PENDING 处理）
- 20 套主题预置（仅机制 + 2 套占位）
- `/prompts` 视图还原
- 现有 4 个未提交修改的清理（stash 暂存，PM 在收尾时处理）
- 性能优化（步骤 7 不进行专项 perf）

---

**PM 软签收**：✅ 步骤 1 完成
**下一步**：并行调度 `@design-director`（步骤 2）+ `@frontend-director`（步骤 3），同时将 4 个未提交修改 stash。

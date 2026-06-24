# FEAT-024 · 跨平台玻璃态原型 100% 还原 — 前端技术方案

> **文档级别**：步骤 3 · 技术方案（前端总监签发）
> **版本**：v1.0
> **编写角色**：`@frontend-director`
> **日期**：2026-06-18
> **状态**：✅ 初稿完成，待 `@frontend-engineer` 评审后实施

---

## 1. 工程目录改造

### 1.1 新增 / 修改文件清单

| 操作 | 路径 | 说明 |
|------|------|------|
| **新增** | `design/tokens/colors.css` | 核心色彩角色变量（从 HTML 原型 `:root` 提取） |
| **新增** | `design/tokens/glass.css` | 玻璃层级变量（6 层 + 暗色变体） |
| **新增** | `design/tokens/motion.css` | 动画关键帧（`tint-drift` / `tint-sweep` / `shimmer` / `pulse`） |
| **新增** | `design/tokens/themes/warm.css` | warm 主题完整变量集 |
| **新增** | `design/tokens/themes/midnight.css` | midnight 暗色主题变量集（占位基线） |
| **新增** | `design/tokens/index.ts` | Token 统一导出与 TS 类型 |
| **新增** | `src/stores/theme.ts` | Pinia store：主题切换 + 玻璃变体注入 |
| **新增** | `src/composables/useGlassTheme.ts` → **覆盖** | 合并现有 `useGlassTheme.ts`，统一为 token-driven 架构 |
| **修改** | `src/assets/theme.css` | 保留 `:root` 基础变量 + 暗色 / warm 主题块；其余 4 个主题块（`glass`/`yellow`）标记 `@deprecated`，后续废弃 |
| **修改** | `tailwind.config.js` | 扩展 `screens`（1024/768/480）与 `colors` 引用 CSS 变量 |
| **修改** | `src/App.vue` | 接入 Pinia theme store，移除 `data-theme` 手动设置 |
| **新增** | `src/components/layout/AppFrame.vue` | 原型 `.window-frame` 容器组件 |
| **新增** | `src/components/layout/MobileTabbar.vue` | 原型 `.mobile-tabbar`（≤768px 可见） |
| **新增** | `src/components/common/CliSyncChip.vue` | `.cli-sync-chip` 三态组件 |
| **新增** | `src/components/common/OpStageBadge.vue` | `.op-stage` 7 阶段操作徽章 |
| **新增** | `src/components/common/ProgressBadge.vue` | `.badge.progress` 脉冲徽章 |
| **新增** | `src/components/common/SkeletonCard.vue` | `.skeleton` 骨架屏 |
| **新增** | `src/components/common/ThemeGrid.vue` | `.theme-grid` 主题选择器 |
| **新增** | `src/components/common/SourceTabs.vue` | `.source-tabs` 来源 Tab 切换 |
| **新增** | `src/components/common/GroupChips.vue` | `.group-chips` 群组筛选芯片 |
| **新增** | `src/components/common/HealthTimeline.vue` | MCP 健康历史时间线 |
| **新增** | `src/components/common/PlatformPills.vue` | 平台徽章（mac/win/linux/cross） |
| **新增** | `src/components/common/TargetChip.vue` | Agent 安装目标芯片 |
| **新增** | `src/components/common/ConfirmDialog.vue` | 确认对话框 |
| **新增** | `src/components/common/ErrorState.vue` | 错误状态展示 |
| **新增** | `src/components/common/EmptyState.vue` | 空状态展示 |
| **新增** | `src/components/common/FilterBar.vue` | 通用过滤器栏 |
| **重写** | `src/components/layout/Sidebar.vue` | 完全重写，精确匹配原型 `.sidebar` 布局 |
| **重写** | `src/components/layout/Topbar.vue` | 完全重写，精确匹配原型 `.topbar` 布局 |
| **重写** | `src/views/DashboardView.vue` | 还原 Dashboard 视图 |
| **重写** | `src/views/CliToolsView.vue` | 还原 CLI Tools 视图（已有内容，校验对齐） |
| **重写** | `src/views/SoftwareManagementView.vue` | 还原 Software 视图 |
| **重写** | `src/views/PluginsView.vue` | 还原 Plugins 视图（3 Tab） |
| **重写** | `src/views/SkillsView.vue` | 还原 Skills 视图（5 Source Tab） |
| **重写** | `src/views/AgentsView.vue` | 还原 Agents 视图 |
| **重写** | `src/views/MCPView.vue` | 还原 MCP 视图（3 区块） |
| **重写** | `src/views/RulesView.vue` | 还原 Rules 视图 |
| **重写** | `src/views/BackupView.vue` | 还原 Backup 视图 |
| **重写** | `src/views/SettingsView.vue` | 还原 Settings 视图（含 Theme Picker） |

### 1.2 `design/tokens/` 骨架结构

```
design/tokens/
├── colors.css          # --bg / --fg / --accent / --success / --error / --info / --warn
├── glass.css           # --glass-window / --glass-sidebar / --glass-topbar / --glass-bg / --glass-input / --glass-input-focus
├── motion.css          # @keyframes tint-drift / tint-sweep / shimmer / pulse / sync-spin / toastIn / toastOut
├── themes/
│   ├── warm.css        # warm 浅色主题（原型基准）
│   └── midnight.css    # midnight 暗色主题（占位基线）
└── index.ts            # export type GlassToken / export const THEME_TOKENS / TS interfaces
```

### 1.3 组件归属规则

| 归属目录 | 组件类型 | 示例 |
|----------|----------|------|
| `src/components/layout/` | Shell 级：AppFrame / Sidebar / Topbar / MobileTabbar / Titlebar | 4 个 |
| `src/components/common/` | 通用 UI：Badge / Button / Tab / Modal / Toast / Skeleton / Chip / FilterBar 等 | 12+ 个 |
| `src/components/<domain>/` | 领域组件：PluginCard / AgentCard / SkillCard / MCPServerCard 等（已有，按需增强） | — |
| `src/views/<View>.vue` | 页面视图：9 个主路由对应视图 | 10 个文件 |

### 1.4 与 `src/components/icons/` 的复用关系

- 现有 `BaseIcon.vue` + `nav/` / `action/` / `file/` / `status/` 分类图标**直接复用**，无需新建
- HTML 原型中内联 SVG（brand、agent-dept 等）提取为 `BaseIcon` 调用或内联 SVG，不新增图标文件
- 品牌图标（CLI 工具 Logo）保留 HTML 原型中的文字缩写方案（`.tool-icon` 内含 `CC` / `GM` 等），不依赖外部图标库

---

## 2. CSS 体系合并策略

### 2.1 HTML 原型 CSS 提取与存放位置

| 原型 CSS 类 | 提取后存放位置 |
|-------------|----------------|
| `:root` 全局变量（第 13~62 行） | `design/tokens/colors.css` + `glass.css` |
| `body` / `body::before` / `body::after` 背景纹理 | `design/tokens/themes/warm.css`（warm 专属） |
| `.window-frame` | `src/components/layout/AppFrame.vue`（`<style scoped>`） |
| `.sidebar` / `.nav-*` / `.sidebar-*` | `src/components/layout/Sidebar.vue`（`<style scoped>`） |
| `.topbar` / `.topbar-*` | `src/components/layout/Topbar.vue`（`<style scoped>`） |
| `.mobile-tabbar` | `src/components/layout/MobileTabbar.vue`（`<style scoped>`） |
| `.card` / `.tool-*` / `.stat-*` | `design/tokens/` 全局类（无 scoped，注入到 `theme.css`） |
| `.btn` / `.badge` / `.modal` / `.toast` 等 | `design/tokens/` 全局类（无 scoped，注入到 `theme.css`） |
| `@media` 断点规则 | 分散至各组件的 `<style scoped>` + `theme.css` 全局响应式 |

### 2.2 与现有 `src/assets/theme.css` 的合并规则

**保留（迁移至 `design/tokens/`）**：
- `:root` 核心变量（`--bg` / `--fg` / `--accent` / `--font-*` / `--radius` / `--ease` / `--t-fast/base/slow` / `--shadow-*` / `--z-*`）
- `[data-theme="warm"]` 完整块（306~812 行）— 作为 warm 主题基准
- 4 个 `@keyframes`（`tint-drift` / `tint-sweep` / `shimmer` / `toastIn/Out`）
- 全局基础样式（`html/body` / `button` / `input` / `scrollbar` / `.shell` / `.main` / `.content`）

**保留（暂不动）**：
- `[data-theme="dark"]` 块（159~195 行）— 与 midnight 主题合并时再处理
- `[data-theme="glass"]` / `[data-theme="yellow"]` 块 — **标记 `@deprecated`，视觉上由 warm 主题统一承载**

**废弃（待删除）**：
- `[data-theme="glass"]` 块（817~1222 行）— 视觉统一到 warm 主题
- `[data-theme="yellow"]` 块（1227~3532 行）— 视觉统一到 warm 主题
- 旧 light/dark/warm 主题中所有过时的 CSS（如旧 `:root` 亮色块，需合并）

**变量命名约定**：
- CSS 变量：`--<category>-<variant>`（如 `--glass-sidebar`）
- Tailwind 配置中引用：`theme('colors.glass-sidebar')` 或直接引用 CSS 变量
- 新增变量必须在 `design/tokens/` 有对应记录

### 2.3 Tailwind 配置最小改动

```javascript
// tailwind.config.js 最小改动
module.exports = {
  // 扩展 screens（1024/768/480 为新增，480px 以下为窄视口）
  screens: {
    'sm': '480px',   // 新增：窄视口
    'md': '768px',   // 已有
    'lg': '1024px',  // 已有
    'xl': '1280px',  // 已有
  },
  // 扩展 colors 引用 CSS 变量（玻璃层专用）
  theme: {
    extend: {
      colors: {
        // 引用 CSS 变量，供 Tailwind 工具类使用（如 bg-glass-sidebar）
        'glass-window': 'var(--glass-window)',
        'glass-sidebar': 'var(--glass-sidebar)',
        'glass-topbar': 'var(--glass-topbar)',
        'glass-bg': 'var(--glass-bg)',
        'glass-input': 'var(--glass-input)',
        // 语义色
        'accent': 'var(--accent)',
        'accent-hover': 'var(--accent-hover)',
      },
    },
  },
}
```

> **注意**：Tailwind 配置**仅添加变量引用**，不写死色值。所有色值来源于 CSS 变量，由 `design/tokens/` 单一维护。

### 2.4 避免 Specificity 冲突：采用 Scoped CSS + 轻全局

| CSS 策略 | 适用场景 | 理由 |
|----------|----------|------|
| **Vue Scoped (`<style scoped>`)** | 所有 `.vue` 组件内部样式 | 隔离，无冲突 |
| **全局 CSS 类（`theme.css`）** | 布局结构（`.shell` / `.sidebar` / `.topbar` / `.content`）+ 通用组件（`.card` / `.btn` / `.badge` / `.modal`） | 这些类跨组件复用，不适合 scoped |
| **CSS 自定义属性（CSS Variables）** | 所有设计 token（颜色 / 玻璃 / 阴影 / 动画） | 通过级联继承，无需 specificity 竞争 |

**理由**：选择 **scoped CSS + 轻全局类** 而非 CSS Modules / BEM，原因：
1. Vue 3 + Vite 项目已有 scoped 实践，团队熟悉
2. `theme.css` 全局类（如 `.card` / `.btn`）是设计系统基础，需要跨组件共享
3. CSS Modules 引入会增加构建复杂度，收益不高
4. CSS Variables 通过级联解决 specificity，组件内只需覆盖变量而非重写选择器

---

## 3. 状态管理

### 3.1 Pinia Theme Store 最小设计

```typescript
// src/stores/theme.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeId = 'warm' | 'midnight'  // 步骤 3 仅支持 2 套
export type GlassVariant = 'default' | 'cool-mist' | 'sage' | 'lavender' | 'ember' | 'slate' | 'arctic' | 'rose-gold'

interface ThemeState {
  activeThemeId: ThemeId
  activeGlassVariant: GlassVariant | null
  prefersReducedMotion: boolean
}

export const useThemeStore = defineStore('theme', () => {
  // --- State ---
  const activeThemeId = ref<ThemeId>('warm')
  const activeGlassVariant = ref<GlassVariant | null>(null)
  const prefersReducedMotion = ref(false)

  // --- Getters ---
  const isDarkMode = computed(() => activeThemeId.value === 'midnight')
  const shouldAnimate = computed(() => !prefersReducedMotion.value)

  // --- Actions ---
  function setTheme(themeId: ThemeId) {
    activeThemeId.value = themeId
    document.documentElement.setAttribute('data-theme', themeId)
    localStorage.setItem('forge-theme', themeId)
    // 注入 / 清除 CSS 变量
    injectThemeVariables(themeId, activeGlassVariant.value)
  }

  function setGlassVariant(variant: GlassVariant | null) {
    activeGlassVariant.value = variant
    localStorage.setItem('forge-glass-variant', variant ?? 'default')
    injectThemeVariables(activeThemeId.value, variant)
  }

  function initTheme() {
    // 读取 localStorage 恢复
    const savedTheme = localStorage.getItem('forge-theme') as ThemeId | null
    const savedVariant = localStorage.getItem('forge-glass-variant') as GlassVariant | null
    if (savedTheme) setTheme(savedTheme)
    else setTheme('warm') // 默认 warm

    // 监听 prefers-reduced-motion
    prefersReducedMotion.value = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  }

  return { activeThemeId, activeGlassVariant, prefersReducedMotion, isDarkMode, shouldAnimate, setTheme, setGlassVariant, initTheme }
})
```

### 3.2 现有 `useTheme.ts` 与 `useGlassTheme.ts` 去重与统一

| 操作 | 文件 | 说明 |
|------|------|------|
| **废弃** | `src/composables/useTheme.ts` | 原有 `Theme` 类型包含 `light`/`dark`/`warm`/`glass`/`yellow`，本次步骤 3 后仅保留 `warm`/`midnight`，其余废弃 |
| **覆盖** | `src/composables/useGlassTheme.ts` | 重写为调用 `useThemeStore()`，提供 `selectGlassVariant()` 等高层 API |
| **新增** | `src/stores/theme.ts` | 作为单一事实来源（Single Source of Truth） |

统一后的 composable 包装：

```typescript
// src/composables/useGlassTheme.ts（重写）
import { computed } from 'vue'
import { useThemeStore } from '@/stores/theme'

export function useGlassTheme() {
  const store = useThemeStore()
  const isGlassTheme = computed(() => store.activeThemeId === 'warm')
  const isMidnightMode = computed(() => store.activeThemeId === 'midnight')

  function selectGlassVariant(variant: GlassVariant | null) {
    store.setGlassVariant(variant)
  }

  return {
    isGlassTheme,
    isMidnightMode,
    selectGlassVariant,
    variants: GLASS_VARIANTS,  // 从 design/tokens/ 导入
  }
}
```

### 3.3 玻璃变量注入点

| 注入时机 | 调用位置 | 说明 |
|----------|----------|------|
| **应用初始化** | `src/main.ts` → `app.mount()` 前调用 `themeStore.initTheme()` | 恢复用户上次主题 |
| **主题切换** | `useThemeStore.setTheme()` 内部 | 调用 `injectThemeVariables()` |
| **玻璃变体切换** | `useThemeStore.setGlassVariant()` 内部 | 调用 `injectThemeVariables()` |
| **prefers-reduced-motion 变化** | `window.matchMedia` 监听回调 | 更新 `prefersReducedMotion`，各组件通过 `shouldAnimate` getter 控制动画 |

```typescript
// 玻璃变量注入函数（内部实现）
function injectThemeVariables(themeId: ThemeId, variant: GlassVariant | null) {
  const r = document.documentElement.style
  const isDark = themeId === 'midnight'
  const base = isDark ? DARK_BASE : WARM_BASE  // WARM_BASE / DARK_BASE 从 design/tokens/ 导入

  // 注入 6 层玻璃变量
  Object.entries(base.glass).forEach(([k, v]) => r.setProperty(`--${k}`, v))

  // 注入 tint 氛围色
  Object.entries(base.tint).forEach(([k, v]) => r.setProperty(`--${k}`, v))

  // 注入 accent / semantic
  Object.entries(base.semantic).forEach(([k, v]) => r.setProperty(`--${k}`, v))
}
```

---

## 4. 9 视图实施顺序与依赖

### 4.1 执行顺序总览

| 阶段 | 子 FEAT | 内容 | 依赖关系 |
|------|---------|------|----------|
| **Phase 0（前置）** | — | Stash 现有修改 + 建立 `design/tokens/` 骨架 | 无 |
| **Phase 1（串行）** | FEAT-024-C | 建立 `design/tokens/` 骨架 + CSS 变量 + Tailwind 配置 | Phase 0 |
| **Phase 1（串行）** | FEAT-024-A | 全局 Shell：AppFrame + Sidebar + Topbar + MobileTabbar | FEAT-024-C 完成后 |
| **Phase 1（串行）** | FEAT-024-B | 通用组件库：13 个通用组件实现 | FEAT-024-C 完成后 |
| **Phase 2（并行）** | FEAT-024-D ~ M | 9 个视图还原 | FEAT-024-A + FEAT-024-B 完成后 |
| **Phase 3（收尾）** | — | 全局主题切换机制 + mock 数据清理 + 评审 | Phase 2 全部完成 |

### 4.2 13 个子 FEAT（A~M）并行性矩阵

```
FEAT-024-C (tokens)   [================================] (串行)
FEAT-024-A (shell)    [================================] (串行，依赖 C)
FEAT-024-B (common)   [================================] (串行，依赖 C)
──────────────────────────────────────────────────────────────
FEAT-024-D (Dashboard)    [====================]  ──┐
FEAT-024-E (CLI Tools)    [====================]  ──┤
FEAT-024-F (Software)     [====================]  ──┤ 并行
FEAT-024-G (Plugins)      [====================]  ──┤ 互不依赖
FEAT-024-H (Skills)       [====================]  ──┤
FEAT-024-I (Agents)       [====================]  ──┤
FEAT-024-J (MCP)          [====================]  ──┤
FEAT-024-K (Rules)        [====================]  ──┤
FEAT-024-L (Backup)       [====================]  ──┤
FEAT-024-M (Settings)     [====================]  ──┘
```

**可并行执行的子 FEAT**：`D / E / F / G / H / I / J / K / L / M`（10 个视图）  
**必须串行的子 FEAT**：`C → A / B`（tokens 必须先于 shell 和 common；shell 必须先于 views）

### 4.3 `theme.css`「单点编辑」规则（防并发冲突）

> `design/tokens/` CSS 文件（`colors.css` / `glass.css` / `motion.css` / `themes/warm.css` / `themes/midnight.css`）仅 FEAT-024-C 工程师有编辑权限。

| 角色 | 可编辑文件 | 不可编辑文件 |
|------|-----------|-------------|
| FEAT-024-C 工程师 | `design/tokens/*.css` | `src/assets/theme.css` |
| FEAT-024-A 工程师 | `src/components/layout/*.vue` | `design/tokens/*.css` / `theme.css` |
| FEAT-024-B 工程师 | `src/components/common/*.vue` | `design/tokens/*.css` / `theme.css` |
| FEAT-024-D~M 工程师 | `src/views/<View>.vue` | `design/tokens/*.css` / `theme.css` |

> 若 FEAT-024-D~M 工程师需要新变量，向 `@frontend-director` 申请，通过 FEAT-024-C 工程师注入，不自行添加。

---

## 5. 现有实现复用决策

### 5.1 复用（无需改动）

| 模块 | 路径 | 复用策略 |
|------|------|----------|
| 路由配置 | `src/router/index.ts` | 直接复用，视图文件路径更新即可 |
| Pinia 集成 | `src/main.ts` 中的 `createPinia()` | 直接复用 |
| Tauri invoke 封装 | `src/services/` 或 `src-tauri/` | 直接复用，不改后端 |
| `BaseIcon.vue` | `src/components/icons/BaseIcon.vue` | 直接复用 |
| 所有 `nav/` / `action/` / `file/` / `status/` 图标组件 | `src/components/icons/` | 直接复用 |
| 领域组件（PluginCard / AgentCard / SkillCard / MCPServerCard 等） | `src/components/<domain>/*.vue` | 增强样式以匹配原型，核心逻辑复用 |

### 5.2 重写（完全替换）

| 文件 | 重写理由 |
|------|----------|
| `src/components/layout/Sidebar.vue` | 原型 `.sidebar` / `.nav-item` / `.sidebar-brand` / `.sidebar-footer` 结构与现有实现差异大，需精确还原 |
| `src/components/layout/Topbar.vue` | 原型 `.topbar` / `.topbar-btn` / `.topbar-search` / `.topbar-title` 布局与现有实现不同 |
| `src/composables/useGlassTheme.ts` | 需从「固定色值数组」改为「token-driven + 动态 CSS 变量注入」架构 |
| `src/assets/theme.css` 关键段 | `[data-theme="warm"]` 块从 HTML 原型提取对齐；废弃 `[data-theme="glass"]` / `[data-theme="yellow"]` |

### 5.3 删除 / 废弃

| 状态 | 内容 | 说明 |
|------|------|------|
| **废弃** | `src/composables/useTheme.ts` 的 `light` / `dark` / `glass` / `yellow` 主题枚举值 | 步骤 3 后仅 `warm` / `midnight` 有效 |
| **废弃** | `[data-theme="glass"]` CSS 块 | 视觉统一到 warm 主题 |
| **废弃** | `[data-theme="yellow"]` CSS 块 | 视觉统一到 warm 主题 |
| **废弃** | 旧 `Theme` 类型（`light`/`dark`/`warm`/`glass`/`yellow`） | 替换为新的 `ThemeId` 类型 |
| **保留** | `src/composables/useTheme.ts` 核心导出（`setTheme` / `cycleTheme`） | 重构为调用 `useThemeStore` |

### 5.4 未提交修改处理

根据 task.md 描述，4 个未提交文件（`theme.css` / `Sidebar.vue` / `useGlassTheme.ts` / `SkillsView.vue`）已 stash，工作树干净。本次技术方案基于**干净基线**制定，无需额外 stash 操作。

---

## 6. 风险与缓解

### 6.1 9 子 FEAT 并行的文件锁方案

| 风险 | 缓解方案 |
|------|----------|
| 多个工程师同时修改 `theme.css` | 严格按 §4.3 单点编辑规则：`design/tokens/` 仅 C 工程师可写；`src/assets/theme.css` 仅 A 工程师可写；views 仅对应工程师可写 |
| 多个工程师同时修改通用组件 | FEAT-024-B 完成后，其他工程师只读 common 目录，不修改 |
| 视图工程师需要新增 CSS 变量 | 通过 `@frontend-director` 向 C 工程师申请，不自行添加 |
| Git 分支策略 | FEAT-024-C 合并到 `main` 后，A 和 B 再创建分支；D~M 在 A+B 合并后再并行创建分支 |

### 6.2 9 视图数据接入的真实 API 列表 + mock fallback

| 视图 | 真实 API（Tauri invoke） | mock fallback 触发条件 |
|------|-------------------------|------------------------|
| Dashboard | —（无独立 API） | `cli_tools::get_all` + `plugin_marketplace::list` 计数，无数据时显示静态 mock |
| CLI Tools | `cli_tools::get_all` / `cli_tools::install` / `cli_tools::uninstall` | 捕获错误，显示 error toast |
| Software | `software::detect_all` / `software::get_details` | API 返回空数组时显示空状态 |
| Plugins | `plugin_marketplace::list` / `plugin_marketplace::install` / `plugin_marketplace::uninstall` | API 返回错误时显示 error state 组件 |
| Skills | `skill_marketplace::list` / `skills_sh::list` / `skills_sh::sync` | API 无返回时显示 skeleton → 3s timeout → 空状态 |
| Agents | `agent::list` / `agent::create` / `agent::import_from_source` | API 无返回时显示 empty state |
| MCP | `mcp_manager::list_servers` / `mcp_manager::health_check` / `mcp_manager::audit_log` | `health_check` 超时 → warning badge |
| Rules | `rule::list` / `rule::create` / `rule::delete` | API 无返回时显示 empty state |
| Backup | `backup::list` / `backup::create` / `backup::restore` | API 无返回时显示 empty state |
| Settings | `settings::get` / `settings::set` | Theme 切换仅 UI，暂不持久化（见 PENDING） |

**Mock 数据原则**：所有 mock fallback 均需在视图文件内以 `// PENDING: <描述>` 标注，说明期望的真实 API 行为。

### 6.3 原型未覆盖的边界场景

| 边界场景 | 处理策略 |
|----------|----------|
| **空数据** | 每个视图实现 `<EmptyState>` 组件，文字根据视图上下文定制（禁止"暂无数据"通用文案） |
| **超长文本** | `.source-card-notes` / `.marketplace-card-desc` 等使用 `text-overflow: ellipsis` + `-webkit-line-clamp` 截断；工具提示展示完整内容 |
| **网络异常** | `<ErrorState>` 组件，显示具体错误信息 + 重试按钮；不显示通用错误弹窗 |
| **加载中** | `<SkeletonCard>` 骨架屏（而非 spinner）；各视图对应 skeleton 布局与原型 card 一致 |
| **操作失败** | Toast notification（error 类型），显示后端返回的具体错误信息（禁止"操作失败"通用文案） |
| **/prompts 视图** | 路由保留，视图 UI 显示 `<EmptyState>` 标注 "PENDING — not in scope" |

### 6.4 像素级一致验证方案

采用 **目视评审 + DOM class 对照** 而非自动化截图对比，理由：

1. **原型是 HTML 静态原型**，无真实数据，截图对比会产生大量误报（数据填充后的渲染差异）
2. **玻璃态效果依赖 `backdrop-filter`**，不同浏览器渲染存在差异，自动化像素对比不稳定
3. **Vue 动态渲染**（数据驱动）与静态 HTML 原型结构不完全一致

**验证流程**：
1. **步骤 5 评审时**：`@frontend-engineer` 提交 PR → `@frontend-director` 对照 HTML 原型逐视图目视检查
2. **检查项**（按 DOM class 对照）：
   - `.stats-row` 布局（4 栏 / 2 栏 / 1 栏）
   - `.stat-card` 玻璃透明度（warm 浅色基准）
   - `.tint-*` 动画错峰延迟（`-2s` / `-4s` / `-6s`）
   - `.mobile-tabbar` 在 768px 断点显示 / 隐藏
   - `.filter-bar` 移动端垂直堆叠
   - `.modal` 圆角（`var(--radius-xl)` = 28px）vs 旧实现（20px）
   - `.window-frame` 圆角（16px）
3. **步骤 8 终审**：在 Tauri 应用内实际运行，人工对比原型截图
4. **已知差异容忍项**：
   - 原型字体加载（Google Fonts）vs 应用内字体（本地 Inter / JetBrains Mono）
   - 原型 `backdrop-filter` 在低端 GPU 上可能有渲染差异

---

## 附录：设计决策记录

### Q：为什么选择「全局 CSS 类 + Scoped」而非 Tailwind 原子化？

**决策**：在 `theme.css` 中保留 `.card` / `.btn` / `.badge` 等全局设计系统类，视图组件通过**组合**这些类实现 UI，而非全量 Tailwind 化。

**理由**：
- 原型使用传统 CSS 类（如 `.card` / `.stat-card`），迁移成本最低
- 玻璃态效果（`backdrop-filter` / 多层 `box-shadow`）在 Tailwind 中需要大量 `arbitrary values`，可维护性差
- 现有组件（PluginCard / AgentCard 等）已使用 `.card` 类，统一类系统减少重复
- Tailwind 仅用于：断点控制（`md:` / `lg:`）、间距（`p-4` / `gap-2`）

---

## 失败 / 打回

> 记录前端架构评审中发现的问题与质询。

| 编号 | 问题 | 级别 | 状态 |
|------|------|------|------|
| R-01 | 现有 `useGlassTheme.ts` 的动态 CSS 变量注入与 `theme.css` 静态变量存在双写风险，可能导致主题切换时变量覆盖不一致 | P1 | **待步骤 4 frontend-engineer 确认注入点** |
| R-02 | `tailwind.config.js` 中 `screens` 扩展需验证与现有 Tailwind 断点不冲突（现有项目是否已有 `sm` 键？） | P2 | **待步骤 4 frontend-engineer 实施时确认** |
| R-03 | midnight 暗色主题仅有占位变量（`midnight.css`），其 `--bg` / `--fg` / `--accent` 等未与 `@design-ui` 确认色值规范 | P1 | **待步骤 4 frontend-engineer 实施时向 @design-ui 确认** |
| R-04 | 原型 `.window-frame` 在 macOS 上有 `border-radius: 16px`（应用窗口圆角），但 Tauri 窗口圆角由系统控制，需确认是否需要双重圆角 | P2 | **待步骤 4 frontend-engineer 确认 Tauri 窗口配置** |
| R-05 | 13 个通用组件（D~M 视图工程师均需使用）的 API 接口需在 FEAT-024-B 实施前锁定，避免返工 | P1 | **待步骤 4 frontend-engineer 实施前锁定接口** |

---

**前端总监软签收**：✅ 步骤 3 技术方案评审通过

| 角色 | 签字 | 意见 |
|------|------|------|
| `@frontend-director` | ✅ | 技术方案整体可行，4 项 P1/P2 质询已记录至"失败/打回"小节，待步骤 4 实施时逐一确认 |
| **veto 状态** | — | 无一票否决。方案整体架构合理，轻全局类 + scoped 方案在当前团队规模下可维护性良好 |

**下一步**：将本方案移交 `@frontend-engineer` 作为步骤 4 实施指南。

# FEAT-024 · 设计审核意见：跨平台玻璃态原型 100% 还原

> **审核角色**：`@design-director`（设计总监，advisory veto）
> **审核日期**：2026-06-18
> **参考基线**：`design/forge-cross-platform-glass.html`（绝对权威）
> **设计规范**：`env-manager/guide/design.md` v3.0

---

## 1. 视觉还原判定基线

### 1.1 像素级还原容差规则

| 维度 | 容差 | 说明 |
|------|------|------|
| 圆角 | ±2px | 18px → 16~20px 可接受 |
| 阴影 blur | ±4px | 20px → 16~24px 可接受 |
| 阴影 spread | ±2px | 4px → 2~6px 可接受 |
| backdrop-filter blur | ±8px | 24px → 16~32px 可接受 |
| 透明度（rgba） | ±0.05 | 0.45 → 0.40~0.50 可接受 |
| 字重/字号 | 0 容忍 | 必须完全匹配 |
| 间距（padding/margin） | ±4px | 24px → 20~28px 可接受 |
| 玻璃面之间隙 | ≥8px | 必须遵守 design.md §5.3 禁止模式 |

**容差触发条件**：仅当原型与设计系统 Token 有歧义时使用；全局 Token（`--radius`、`--t-base` 等）以原型 CSS 变量值为准。

### 1.2 还原优先级排序

执行顺序遵循「地基 → 骨架 → 组件 → 视图」的依赖链：

| 优先级 | 范围 | 理由 |
|--------|------|------|
| **P1** | FEAT-024-C（玻璃体系 + tokens/ 骨架） | 全局玻璃 CSS 变量、Tint 动画、响应式断点，是所有视图的视觉地基 |
| **P2** | FEAT-024-A（Shell：AppFrame + Sidebar + Topbar + MobileTabbar） | 应用骨架，9 个视图全部依赖 |
| **P3** | FEAT-024-B（通用组件库） | 组件库先就位，FEAT-024-M（Settings，含 ThemePicker）和 FEAT-024-D（Dashboard）可同步进行 |
| **P4** | FEAT-024-M（SettingsView）含 ThemePicker | ThemePicker 组件依赖 A+B，但其他视图不依赖它；Dashboard 依赖 A+B，可同步 |
| **P5** | D → L（9 视图，按复杂度分批） | 可完全并行，仅依赖 A+B+C 产出 |

**关键视图先行策略**：Dashboard（FEAT-024-D）应排 P5 第一位，因其集成了所有全局组件，是验收「视觉一致」的首要判据。

### 1.3 HTML 原型 → CSS 系统合并策略

**冲突发现（严重）**：

`src/assets/theme.css` 的 `[data-theme="warm"]` 与 HTML 原型基准存在系统性偏差：

| CSS 变量 | HTML 原型值 | theme.css 现有值 | 偏差 |
|----------|-------------|-------------------|------|
| `--glass-bg` | `rgba(255,255,255,0.45)` | `rgba(255,255,255,0.28)` | -0.17 ❌ |
| `--glass-sidebar` | `rgba(255,255,255,0.35)` | `rgba(255,255,255,0.18)` | -0.17 ❌ |
| `--glass-topbar` | `rgba(255,255,255,0.38)` | `rgba(255,255,255,0.20)` | -0.18 ❌ |
| `--glass-input` | `rgba(255,255,255,0.40)` | `rgba(255,255,255,0.25)` | -0.15 ❌ |
| `--glass-input-focus` | `rgba(255,255,255,0.60)` | `rgba(255,255,255,0.45)` | -0.15 ❌ |
| `--topbar-h` | `64px` | `56px` | -8px ❌ |
| backdrop-filter | `blur(24px) saturate(1.2)` | `blur(12px)`（部分） | 不一致 ❌ |

**合并决策**：

1. **`[data-theme="warm"]` 整体替换**：将 HTML 原型 `:root` 的全部玻璃变量、色值、圆角、阴影值完整迁移到 `theme.css` 的 `[data-theme="warm"]` 选择器。**不等同于"新建"，而是"对齐"——warm 是原型默认主题。**

2. **useGlassTheme.ts 冲突（严重）**：`src/composables/useGlassTheme.ts` 硬编码了与 `theme.css` 不一致的玻璃透明度值（见上表），且使用 `localStorage` 键 `aem-glass-variant`。**需将其改为读 CSS 变量，或完全废弃，仅保留 `theme.css` 单一真相源。**

3. **theme.css 保留项**：`[data-theme="glass"]`、`[data-theme="yellow"]`、`[data-theme="dark"]` 保留——本次不还原这些主题，且不影响 FEAT-024 还原率。

4. **design/tokens/ 骨架优先**：`design/tokens/glass.css` 应作为单一真相源，`theme.css` 的 `[data-theme="warm"]` 从中 `@import` 引用（由 frontend-engineer 在 FEAT-024-C 步骤中实现）。

5. **`.window-frame` 新增**：HTML 原型有 `.window-frame`（`backdrop-filter: blur(40px) saturate(1.3)`，内含 shell + tabbar），当前 Vue 实现中无此层。**FEAT-024-A 应补入此 wrapper div。**

> ⚠️ **[P0 决策，待 frontend-director 确认]**：useGlassTheme.ts 是否废弃（theme.css 单一真相源）？还是保留但改为读取 CSS 变量？两种方案各有利弊。

---

## 2. 组件状态矩阵

### 2.1 全局 Shell（FEAT-024-A）

| 组件 | 原型 CSS 类 | 状态 | 还原要求 |
|------|-------------|------|----------|
| **AppFrame** | `.window-frame` | window-frame | 新增 wrapper：`position:fixed;inset:0;background:rgba(255,255,255,0.32);backdrop-filter:blur(40px) saturate(1.3);border-radius:16px` |
| **Sidebar** | `.sidebar` | default / hover / active | default: `background:rgba(255,255,255,0.35);border-right:1px solid rgba(255,255,255,0.22)`；hover: 背景提升至 `rgba(255,255,255,0.30)`；active: 背景 `rgba(255,255,255,0.30)` + 左侧 3px accent 色边框 |
| **Topbar** | `.topbar` | default / mobile-hidden | `height:64px`（非 56px）；搜索框在 ≤768px 隐藏（`display:none`） |
| **MobileTabbar** | `.mobile-tabbar` | default / active / scrolled | `display:none` 在 ≥769px；滚动后无特殊阴影（原型无此状态） |
| **ThemePicker** | `.theme-grid` / `.theme-card` | default / hover / active | active 有 accent 边框 + 右上角勾选图标 |

### 2.2 通用组件（FEAT-024-B）

| 组件 | 原型 CSS 类 | 状态 | 还原要求 |
|------|-------------|------|----------|
| **Button** | `.btn-primary` / `.btn-secondary` / `.btn-ghost` / `.btn-icon` | default / hover / active / focus / **disabled（推断）** | focus: 原型无显式定义，fallback 到 `box-shadow:0 0 0 2px var(--accent-glow)`；disabled: `opacity:0.5;cursor:not-allowed;pointer-events:none`（design.md §5.1） |
| **Tab Bar** | `.tab-bar` / `.tab-item` | default / hover / active | active: `border-bottom-color:var(--accent);color:var(--accent)` |
| **CLI Sync Chip** | `.cli-sync-chip` | unsynced / syncing / synced | unsynced: `border-color:rgba(184,148,74,0.20);background:rgba(255,255,255,0.40)`；syncing: `pointer-events:none`；synced: 绿色系；**hover 状态**：仅触发背景色微调 |
| **Search Input** | `.search-input` / `.sidebar-search` | default / focus / filled | focus: `background:rgba(255,255,255,0.40);border-color:rgba(255,255,255,0.40)`；filled: 原型通过 `:not(:placeholder-shown)` 触发清除按钮显示 |
| **Filter Bar** | `.filter-bar` / `.filter-select` | default / hover / focus | select: 原生 `<select>` + CSS arrow；hover: 无变化（原型无 hover 效果） |
| **Modal** | `.modal` / `.modal-overlay` | enter / exit | enter: 原型无动画（仅 display:block）；**exit: 原型无定义**，fallback 到 design.md §7.1 的 `t-slow (300ms)` 淡出 |
| **Toast** | `.toast` | success / error / info / warn | 4 种类型各不同 border-color；原型有 `toastIn` / `toastOut` 关键帧动画 |
| **Skeleton** | `.skeleton` | shimmer | **原型未定义 skeleton 样式**；fallback: 骨架屏替代 loading，全屏 spinner 禁用（design.md §5.3） |
| **Badge** | `.badge` | success / warn / error / info / outline / progress | **原型未定义 warn/error/info 的 hover/focus/disabled**；全部 fallback 到 design.md §5.1 五态 |
| **StatCard** | `.stat-card.tint-*` | default / hover | hover: 背景 +0.14 不透明度，`::before` / `::after` 的 `filter:brightness(1.6)` + `opacity:0.95`（见 §3.3） |
| **MarketplaceCard** | `.marketplace-card` | default / hover / installed | installed: 左上角绿色圆点（`.installed-dot`）；hover: 同 `.card` 抬升效果 |
| **FilterBar** | `.filter-bar` | default / mobile-stacked | ≤768px: `flex-direction:column;align-items:stretch`（垂直堆叠） |
| **Avatar** | `.avatar` | default | **原型未显式定义样式**，根据 sidebar-footer 上下文推断：`28×28px;border-radius:var(--radius-sm);backdrop-filter:blur(12px)` |
| **OpStage** | `.op-stage` | preparing / downloading / installing / verifying / completed / failed / cancelled | 7 阶段各自 bg + color；**原型未定义 hover / disabled**；fallback: opacity 降低 |

### 2.3 未定义状态决策汇总

| 组件 | 未定义状态 | 决策 |
|------|-----------|------|
| Button | disabled / focus | disabled: `opacity:0.5;cursor:not-allowed`；focus: `box-shadow:0 0 0 2px var(--accent-glow)` |
| Modal | exit 动画 | 300ms fade-out via `toastOut` 关键帧 |
| Badge | 所有状态的 hover / disabled | hover: 无变化；disabled: opacity 0.6 |
| Skeleton | — | 原型无 skeleton；用骨架屏替代 loading spinner |
| Avatar | hover / active / error | 原型仅有 default；hover/active fallback 到与 `.nav-item` hover 相同 |
| OpStage | hover / disabled | hover: 背景加深 10%；disabled: opacity 0.5 |
| Toast | hover | 原型无 hover；默认静态 |

---

## 3. 玻璃 + Tint 体系迁移方案

### 3.1 六玻璃面变量迁移表（绝对权威值）

来源：`design/forge-cross-platform-glass.html` `:root`（lines 13–62）

| 变量 | 浅色基线（warm 默认） | 暗色基线（midnight 占位） | 说明 |
|------|---------------------|-------------------------|------|
| `--glass-window` | `rgba(255,255,255,0.25)` | `rgba(255,255,255,0.03)` | 最外层，blur 40px |
| `--glass-sidebar` | `rgba(255,255,255,0.35)` | `rgba(255,255,255,0.04)` | 侧栏，blur 24px |
| `--glass-topbar` | `rgba(255,255,255,0.38)` | `rgba(255,255,255,0.04)` | 顶栏，blur 24px |
| `--glass-bg` | `rgba(255,255,255,0.45)` | `rgba(255,255,255,0.06)` | 卡片背景，blur 20px |
| `--glass-input` | `rgba(255,255,255,0.40)` | `rgba(255,255,255,0.08)` | 输入框默认，blur 16px |
| `--glass-input-focus` | `rgba(255,255,255,0.60)` | `rgba(255,255,255,0.12)` | 输入框聚焦，blur 16px |

> 注：暗色基线值从原型 `selectTheme('midnight')` 的 JS 逻辑中提取（lines 1309–1314）。这些是**目标值**，实际暗色主题（`midnight`）的完整色板在 §5 主题机制中定义。

### 3.2 Tint 氛围色色相与饱和度决策

| Tint | CSS 类 | 色相来源 | 透明度（浅色） | 透明度（暗色） | 用途 |
|------|--------|----------|----------------|----------------|------|
| warm | `.tint-warm` | 原型固定值：`rgba(195,178,155,X)` | `0.12`（drift）/ `0.08`（sweep） | `0.06`（drift） | 卡片氛围层 |
| cool | `.tint-cool` | 原型固定值：`rgba(195,178,155,X)`（同 warm，动画错峰） | `0.10` / `0.06` | `0.05` / `0.03` | 卡片氛围层 |
| soft | `.tint-soft` | 原型固定值：`rgba(195,178,155,X)` | `0.14` / `0.08` | `0.07` / `0.04` | 卡片氛围层 |
| amber | `.tint-amber` | 原型固定值：`rgba(195,178,155,X)` | `0.08` / `0.05` | `0.04` / `0.02` | 卡片氛围层 |

**色相决策**：原型使用统一的 `rgba(195,178,155)` 作为所有 4 种 tint 的色相，通过**透明度差异 + 动画错峰**实现视觉区分。**不允许改变色相**（design.md §3.2：Tint 氛围色不得作为强调色使用）。

### 3.3 tint-drift + tint-sweep 动画关键帧

**tint-drift（`@keyframes tint-drift`，8s 周期）**：

```css
@keyframes tint-drift {
  0%   { transform: translate(0, 0) rotate(0deg) }
  25%  { transform: translate(10%, 6%) rotate(1deg) }
  50%  { transform: translate(-4%, 10%) rotate(-0.5deg) }
  75%  { transform: translate(6%, -5%) rotate(0.5deg) }
  100% { transform: translate(0, 0) rotate(0deg) }
}
```

- 作用于：`.stat-card.tint-*::before`
- 运动方式：径向渐变在卡面内缓慢位移 + 微旋转，模拟光晕漂浮
- hover 增强：原型第 194 行，`filter:brightness(1.6);opacity:0.95`

**tint-sweep（`@keyframes tint-sweep`，4.5s 周期）**：

```css
@keyframes tint-sweep {
  0%   { transform: translateX(0) }
  50%  { transform: translateX(33%) }
  100% { transform: translateX(0) }
}
```

- 作用于：`.stat-card.tint-*::after`
- 运动方式：高光带（`height:2px`）从左向右横扫
- 渐变：`linear-gradient(90deg, transparent, rgba(255,255,255,0.30), rgba(255,255,255,0.70), rgba(255,255,255,0.30), transparent)`
- hover 增强：原型第 195 行，`filter:brightness(1.5);opacity:0.85`

### 3.4 错峰延迟表（4 种 tint 交叉错峰）

| 动画 | warm | cool | soft | amber |
|------|------|------|------|-------|
| `tint-drift` delay | `0s` | `-2s` | `-4s` | `-6s` |
| `tint-sweep` delay | `0s` | `-1.5s` | `-3s` | `-4.5s` |

> 设计意图：warm 视角下用户看到的光晕永远不会整齐同步地移动，通过错峰制造自然的呼吸感。

### 3.5 `prefers-reduced-motion` 关闭方案

```css
@media (prefers-reduced-motion: reduce) {
  .stat-card.tint-warm::before,
  .stat-card.tint-cool::before,
  .stat-card.tint-soft::before,
  .stat-card.tint-amber::before {
    animation: none;
  }
  .stat-card.tint-warm::after,
  .stat-card.tint-cool::after,
  .stat-card.tint-soft::after,
  .stat-card.tint-amber::after {
    animation: none;
  }
  .cli-sync-chip.syncing .chip-status {
    animation: none;
  }
  .badge.progress::before {
    animation: none;
  }
}
```

- 关闭方案：**直接移除 `animation` 属性**（不设置为 `animation: none` 再设置时长——那样仍会触发重绘）
- `animation: none` 声明置于 `@media` 查询内，不影响启用了 motion 的用户
- 骨架屏 shimmer 动画（`.skeleton`）：原型定义在 lines 506-507，同样纳入 `@media (prefers-reduced-motion: reduce)` 关闭

---

## 4. 响应式断点细化

### 4.1 四档断点组件级行为

| 断点 | 宽度 | Shell | 内容区 | 按钮 | 卡片 | 表单 |
|------|------|-------|--------|------|------|------|
| **宽视口** | ≥1024px | 侧栏 240px + 顶栏 64px 全显 | 4 栏 stat / 3 栏 marketplace / 过滤器并排 | 正常 padding | 正常 padding | 正常宽度 |
| **平板** | 768–1023px | 侧栏保留（宽度压缩至 200px） | stat 4→2 栏；settings 2→1 列；agents 网格收紧 | padding 缩减 | padding 缩减 | 搜索框宽度 160px→140px |
| **手机** | 480–767px | 侧栏隐藏（`display:none`） | stat 2→1 栏；过滤器垂直堆叠 | 全宽按钮 | — | 搜索框全宽 |
| **窄视口** | <480px | 同手机 | stat 单列 | padding:6px 10px;font-size:10px | padding 收紧 | 全宽 |

### 4.2 移动端底部 Tab Bar 入口选择

**推荐方案（5 入口，精确对应原型 HTML lines 907-912）**：

| 入口 | 路由 | 图标（SVG） | 理由 |
|------|------|-------------|------|
| Home | `/` | `<rect>` 4 格 | Dashboard 概览 |
| CLI | `/cli-tools` | `<polyline>` CLI 箭头 | CLI Tools 视图 |
| Plugins | `/plugins` | `<path>` 3 层叠放 | Plugins 视图 |
| Skills | `/skills` | `<polygon>` 六角星 | Skills 视图 |
| Settings | `/settings` | `<circle>` 齿轮 | Settings 视图 |

**排除入口**：Agents / MCP / Rules / Backup — 这些通过侧栏抽屉（原生 `<nav>` 折叠或独立页面路由）访问，不占用底部 5 个位置。原型 lines 908-912 确认此 5 入口方案。

### 4.3 断点切换状态保留清单

| 状态 | 保留？ | 实现方式 |
|------|--------|----------|
| 当前路由（视图） | ✅ 必须 | URL path 不变，Vue Router 保持 |
| 滚动位置 | ✅ 推荐 | `keep-alive` 或手动保存 scrollTop |
| 激活的 Tab/Source Tab | ✅ 必须 | 与路由同级的状态（如 `activePluginTab`） |
| 过滤器值（Search + Select） | ✅ 必须 | URL query params 或 Pinia store |
| Modal 打开状态 | ✅ 推荐 | 断点切换时关闭 modal（避免布局冲突） |
| 选中的 Theme | ✅ 必须 | localStorage 持久化 |
| Toast 队列 | ✅ 推荐 | 跨断点保留 |
| 操作进度（如 syncing 状态） | ✅ 推荐 | Pinia store 持久化 |

---

## 5. 主题切换机制

### 5.1 主题注册 API 最小接口设计

```typescript
// 最小接口（由 frontend-engineer 在步骤 4 实现）
interface ThemeDefinition {
  id: string           // 唯一标识，如 'warm'、'midnight'
  name: string         // 显示名
  colors: {            // 6 色色板：[bg, surface, border, fg, accent, warn]
    bg: string
    surface: string
    border: string
    fg: string
    accent: string
    warn: string
  }
  baseline: 'light' | 'dark'  // 基线类别
}

function registerTheme(theme: ThemeDefinition): void
function unregisterTheme(id: string): void

// useTheme() 扩展
function useTheme() {
  const current = ref<string>()       // 当前主题 ID
  const themes = computed(() => [...]) // 已注册主题列表
  function setTheme(id: string): void
  return { current, themes, setTheme }
}
```

### 5.2 warm 与 midnight 色板最小差异

| Token | warm（浅色默认） | midnight（暗色基线） |
|-------|------------------|---------------------|
| `--bg` | `#E8E4DE` | `#1A1D24` |
| `--bg-card` | `rgba(255,255,255,0.45)` | `rgba(255,255,255,0.06)` |
| `--fg` | `#1A1A1A` | `#E0E4EC` |
| `--accent` | `#2D2D2D` | `#5A9A7A` |
| `--success` | `#5A8A64` | `#5A9A7A` |
| `--warn` | `#B8944A` | `#7A8AAA` |
| `--border` | `rgba(255,255,255,0.22)` | `rgba(255,255,255,0.10)` |
| `--glass-bg` | `rgba(255,255,255,0.45)` | `rgba(255,255,255,0.06)` |
| 基线 | light | dark |

> midnight 的 `--accent` 从原型 JS `selectTheme('midnight')` 第 1301 行提取：`#5A9A7A`（青绿色）；`--warn` 为 `#7A8AAA`（蓝灰色）。

### 5.3 主题切换过渡策略

| 策略 | 实现 | 决策 |
|------|------|------|
| 瞬切（无动画） | 直接设置 `data-theme` | ❌ 不采用——用户体验断层 |
| 淡入 200ms | CSS transition on `[data-theme]` | ✅ **推荐**：200ms 与 `t-base` 一致；过渡 target 为 `background`、`color`、`box-shadow` |
| 仅内容过渡 | Vue `<Transition>` | ❌ 不采用——背景和组件样式不同步 |

**实现方案**：

```css
/* 主题切换过渡（加入 [data-theme] 选择器以避免与组件状态冲突） */
[data-theme="warm"],
[data-theme="midnight"] {
  transition:
    background-color 200ms cubic-bezier(0.4, 0, 0.2, 1),
    color 200ms cubic-bezier(0.4, 0, 0.2, 1),
    box-shadow 200ms cubic-bezier(0.4, 0, 0.2, 1),
    border-color 200ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

> 注：玻璃透明度（`backdrop-filter`）无法 CSS 过渡——通过先移除旧主题 DOM，再添加新主题 DOM 的方式绕过（Vue 层实现）。

### 5.4 localStorage 键名

**建议键名**：`aem-theme`

| 键值示例 | 说明 |
|----------|------|
| `"warm"` | 当前激活的主题 ID |
| `"midnight"` | 用户切换后的暗色主题 |

- 与原型保持一致（`useGlassTheme.ts` 已有 `aem-theme` 使用先例）
- 值：主题 ID 字符串
- 读取时机：`App.vue` `onMounted`，早于组件渲染
- 写入时机：`setTheme(id)` 调用后同步写入

---

## 附录：发现的设计冲突

### 冲突 A（需 PM + frontend-director 决策）

`theme.css` 的 `[data-theme="warm"]`（lines 207-341）与 HTML 原型 `:root` 存在系统性玻璃透明度偏差（见 §1.3）。当前 theme.css 中**没有** `--glass-window`、`--border-window`、`--border-outer-glow` 等原型定义的关键变量。

**影响**：若不替换 warm 主题，则 FEAT-024 的还原率将永远无法达到 100%。

**建议**：frontend-engineer 在 FEAT-024-C 第一步即更新 `[data-theme="warm"]` 的全部值。

### 冲突 B（需 frontend-director 决策）

`useGlassTheme.ts` 与 `theme.css` 两套系统对玻璃透明度值不一致（`useGlassTheme.ts` 的值 ≠ `theme.css` warm 的值）。

**建议**：废弃 `useGlassTheme.ts` 中硬编码的玻璃值，改为从 `theme.css` 读取 CSS 变量，或将其降级为「主题选择器 UI」的纯展示逻辑。

---

**设计总监软签收**

✅ 步骤 2 完成，设计审核意见已输出。

**Veto 状态**：

- ⚠️ **[advisory] useGlassTheme.ts 与 theme.css 的玻璃值冲突**：当前两套系统对同一玻璃透明度给出不同数值（P1）。frontend-director 需在步骤 3 决策废弃或统一。
- ⚠️ **[advisory] theme.css [data-theme="warm"] 整体替换**：warm 主题当前值与原型偏差超过容差（P0）。frontend-engineer 在 FEAT-024-C 必须执行替换，不得跳过。
- ⚠️ **[advisory] .window-frame 缺失**：应用最外层缺少此 wrapper，会导致 AppFrame 的 `backdrop-filter:blur(40px)` 无法正确作用于整个窗口（P1）。FEAT-024-A 必须包含此 wrapper。

**无阻塞性硬否决**——以上均为 advisory veto，已记录在案，不阻塞 PM 推进工作流。

---

*设计总监（@design-director）审核完成 — 2026-06-18*

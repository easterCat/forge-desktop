# Forge Glass UI — 全量还原设计规范

## Overview

将 Forge Desktop 的所有 11 个页面、~75 个组件全面还原为 `design/forge-cross-platform-glass.html` 原型设计中的 Cross-Platform Glass 风格。基于 100 条 UI 还原清单，采用审计优先、差距驱动的实施策略。

**核心原则：** 原型设计 (`design/forge-cross-platform-glass.html`) 优先于 `design.md` 规范。当两者冲突时，以原型为准。

**硬性约束：** 严禁删除或禁用任何现有功能模块。所有既有功能须完整保留并可正常运行。

**主题策略：** Glass 为当前唯一实现主题，代码结构预留多主题扩展性。

---

## 1. 差距审计发现

### 1.1 系统性问题（6 个）

| # | 问题 | 影响范围 | 严重度 |
|---|------|---------|--------|
| 1 | `[data-theme="glass"]` 缺少所有 `--glass-*` 令牌 | 全局 | 高 |
| 2 | Vue scoped CSS 高优先级抑制全局 glass 样式 | 所有组件 | 高 |
| 3 | CSS 变量命名不一致（`--sidebar-bg` vs `--bg-sidebar`） | Sidebar、Topbar | 中 |
| 4 | 大量硬编码 `rgba()` 值，未使用主题变量 | 所有组件 | 高 |
| 5 | Tint 类名不匹配（warm 用 `tint-warm`，glass 用 `tint-green`） | Dashboard | 中 |
| 6 | PluginCard 绕过 `.card` 基类，无法继承全局 glass 样式 | Plugins | 中 |

### 1.2 主题系统差距

**完全缺失的 CSS 变量（从未定义）：**
- `--shadow-btn`、`--shadow-window`
- `--ease`、`--t-fast`、`--t-base`、`--t-slow`（现有等效：`--transition-fast/base/slow`）

**仅在 `[data-theme="warm"]` 中定义的变量（16 个）：**
- `--glass-bg`、`--glass-bg-hover`、`--glass-sidebar`、`--glass-topbar`、`--glass-input`、`--glass-input-focus`、`--glass-window`、`--glass-inner-glow`、`--glass-highlight`
- `--border-window`、`--border-outer-glow`、`--shadow-inner`
- `--tint-warm`、`--tint-cool`、`--tint-soft`、`--tint-amber`

**`[data-theme="glass"]` 的问题：** 无任何 `--glass-*` 变量，glass 效果通过硬编码 `rgba()` 实现到组件选择器上。

### 1.3 组件层面差距（抽样 5 个组件）

| 组件 | 主要差距 |
|------|---------|
| `Sidebar.vue` | scoped CSS 中 `var(--sidebar-bg)` 未定义；nav-item hover 硬编码 `rgba(255,255,255,0.18)`；无 `backdrop-filter` |
| `Titlebar.vue` | glass 主题无 titlebar 样式；`--titlebar-h` 仅 warm 主题定义 |
| `Topbar.vue` | scoped `.topbar-btn.primary` 的 `rgba(45,45,45,0.85)` 抑制全局 glass 按钮样式；无 `backdrop-filter` |
| `DashboardView.vue` | tint 类名不匹配（模板用 `tint-warm`，glass 主题定义 `tint-green`） |
| `PluginCard.vue` | 使用 `.plugin-card` 而非 `.card`；背景用 `var(--bg-secondary)`（实色）而非 glass 透明度 |

---

## 2. 主题系统架构

### 2.1 Glass 令牌统一方案

在 `:root` 中定义所有 glass 令牌的默认值（使用原型 warm glass 值），确保任何主题下变量均有值。各主题通过 `[data-theme]` 选择器覆盖差异值。

**统一后的变量体系：**

```css
:root {
  /* Glass 令牌 — 默认值（原型 warm glass） */
  --glass-bg: rgba(255,255,255,0.28);
  --glass-bg-hover: rgba(255,255,255,0.42);
  --glass-sidebar: rgba(255,255,255,0.18);
  --glass-topbar: rgba(255,255,255,0.20);
  --glass-input: rgba(255,255,255,0.25);
  --glass-input-focus: rgba(255,255,255,0.45);
  --glass-window: rgba(255,255,255,0.12);
  --glass-inner-glow: rgba(255,255,255,0.70);
  --glass-highlight: rgba(255,255,255,0.85);
  --border-window: rgba(255,255,255,0.12);
  --border-outer-glow: rgba(255,255,255,0.10);
  --shadow-btn: 0 1px 4px rgba(0,0,0,0.06);
  --shadow-window: 0 16px 48px rgba(0,0,0,0.10), 0 4px 16px rgba(0,0,0,0.04);
  --shadow-inner: inset 0 1px 2px rgba(0,0,0,0.03);
  --tint-warm: rgba(200,190,175,0.15);
  --tint-cool: rgba(180,185,195,0.12);
  --tint-soft: rgba(220,215,208,0.18);
  --tint-amber: rgba(184,148,74,0.12);
  --titlebar-h: 38px;
  --sidebar-w: 240px;
  --topbar-h: 64px;

  /* 时间令牌（别名，指向现有变量） */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --t-fast: var(--transition-fast);
  --t-base: var(--transition-base);
  --t-slow: var(--transition-slow);
}
```

### 2.2 CSS 变量映射表

| 原型变量 | 现有变量 | 统一后 |
|---------|---------|--------|
| `--glass-bg` | 仅 warm 主题 | `:root` 默认值 + 各主题覆盖 |
| `--glass-sidebar` | 仅 warm 主题 | 同上 |
| `--shadow-btn` | 不存在 | 新增到 `:root` |
| `--shadow-window` | 不存在 | 新增到 `:root` |
| `--ease` | 不存在 | 新增（复用 `cubic-bezier(0.4,0,0.2,1)`） |
| `--t-fast/base/slow` | 不存在 | 映射到现有 `--transition-fast/base/slow` |

### 2.3 useGlassTheme.ts 扩展

`selectGlassVariant()` 需扩展动态设置以下变量（当前仅设置基础色）：

```typescript
// 新增动态设置的变量
'--glass-bg', '--glass-bg-hover', '--glass-sidebar', '--glass-topbar',
'--glass-input', '--glass-input-focus', '--glass-window',
'--border-window', '--shadow-inner',
'--tint-warm', '--tint-cool', '--tint-soft', '--tint-amber'
```

### 2.4 Tailwind 配置修复

`tailwind.config.js` 中 `colors.primary` 当前为 sky blue 色阶，与所有主题的 amber accent 不匹配。需更新为 amber 色阶或移除对 primary 的依赖。

---

## 3. 组件级实施策略：三层分离

### 3.1 核心原则

将每个组件的样式分为三层，解决 scoped CSS 抑制全局 glass 样式的问题：

| 层级 | 位置 | 内容 | 示例 |
|------|------|------|------|
| **结构层** | scoped CSS | 布局尺寸、flex、grid、margin/padding | `width: 240px; display: flex; flex-direction: column` |
| **主题层** | theme.css 全局 | 颜色、背景、边框、阴影、backdrop-filter | `background: var(--glass-sidebar); backdrop-filter: blur(24px)` |
| **交互层** | theme.css 全局 | hover/active/focus 状态 | `.nav-item:hover { background: var(--glass-bg-hover) }` |

### 3.2 实施规则

**规则 1：scoped CSS 中删除所有颜色/背景/边框/阴影声明**

```scss
// ❌ 之前（scoped）
.sidebar {
  background: var(--sidebar-bg);           // 颜色 → 移到全局
  border-right: 1px solid var(--sidebar-border);  // 边框 → 移到全局
  width: var(--sidebar-w);                 // 尺寸 → 保留
  display: flex;                           // 布局 → 保留
}

// ✅ 之后（scoped）
.sidebar {
  width: var(--sidebar-w);
  display: flex;
  flex-direction: column;
}
```

**规则 2：theme.css 中为每个主题写完整样式**

```css
[data-theme="warm"] .sidebar {
  background: var(--glass-sidebar);
  backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid rgba(255,255,255,0.22);
}
```

**规则 3：hover/active/focus 状态全部在全局定义**

```css
[data-theme="warm"] .nav-item:hover { background: var(--glass-bg-hover); }
[data-theme="warm"] .nav-item.active { background: rgba(255,255,255,0.18); border-left-color: var(--accent); }
[data-theme="warm"] .nav-item:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
```

### 3.3 组件修改分类

**A 类：布局组件（3 个）— 移除 scoped 颜色声明，补全全局 glass 样式**
- `Sidebar.vue` — 删除 scoped 中的 background/border，全局已部分覆盖
- `Titlebar.vue` — 删除 scoped 中的 [data-theme="warm"] 覆盖，补 glass 主题
- `Topbar.vue` — 删除 scoped 中的按钮颜色，全局已部分覆盖

**B 类：通用组件（~10 个）— 统一使用 glass 令牌**
- `PluginCard.vue` — 添加 `.card` 基类或手动补全 glass 样式
- `SkillCard.vue`、`MCPServerCard.vue`、`AgentCard.vue` — 同上
- 各种 Dialog 组件 — 验证 modal 样式

**C 类：页面视图（11 个）— 验证 grid 布局 + 组件组合**
- `DashboardView.vue` — 修复 tint 类名（`tint-warm/cool/soft/amber` 统一）
- 其他视图 — 逐一验证布局和样式

### 3.4 PluginCard 特殊处理

PluginCard 使用 `.plugin-card` 而非 `.card`，无法继承全局 glass 样式。

**方案：** 模板中添加 `.card` 基类（`class="card plugin-card"`），让全局 `.card` 的 glass 样式（backdrop-filter、gradient ::before、glass hover）生效。PluginCard 特有样式（sync chip、cli-row）保留为 scoped。

---

## 4. 实施阶段划分

### Phase 0：Glass 令牌基础设施

**目标：** 统一 CSS 变量体系，消除命名冲突。

**具体工作：**
- 在 `:root` 中定义完整的 glass 令牌集（16 个 `--glass-*` + `--shadow-btn` + `--shadow-window` + `--tint-*`）
- 统一 `--ease` / `--t-fast` / `--t-base` / `--t-slow` 别名
- 扩展 `useGlassTheme.ts` 的 `selectGlassVariant()` 动态设置所有 glass 令牌
- 修复 `tailwind.config.js` 中的 primary 色值

**对应清单条目：** #56-65（字体/色彩/圆角/阴影/边框体系）、#80（背景纹理）

**验收标准：** DevTools 中切换任意主题，所有 `--glass-*` 变量均有有效值。

### Phase 1：布局骨架还原

**目标：** 页面结构与原型一致。

**具体工作：**
- Window Frame：`position:fixed; inset:0` + `backdrop-filter:blur(40px)` + 圆角 16px（#1）
- Titlebar：修复 `--titlebar-h` 全局定义，验证 macOS/Windows 双模式（#2）
- Shell + Sidebar：修复变量命名，统一为 `--glass-sidebar`（#3-5）
- Main + Topbar + Content：修复 `--topbar-h`，验证 flex 布局（#6-8）
- 各类 Grid 布局：stats-row(4列)、tool-grid(单列)、marketplace-grid(自适应)等（#9-14）
- Filter Bar + Tab Bar（#15-16）
- Modal + Confirm + Toast：z-index 层级 100/110/200（#17-19）
- 响应式断点：1024px / 768px / 640px（#23-25）

**对应清单条目：** #1-25

**验收标准：** 1440px 宽度下与原型截图结构一致，768px 时侧边栏隐藏。

### Phase 2：视觉样式还原

**目标：** 颜色、圆角、阴影、字体、动画与原型一致。

**具体工作：**
- 字体体系：Inter + JetBrains Mono，标题层级字号（#56-58）
- 色彩体系：主色调、状态色、Tint 色——修复 tint 类名不匹配（#59-61）
- 玻璃拟态层级 + 圆角 + 阴影 + 边框：将硬编码 rgba() 替换为 CSS 变量（#62-65）
- Badge + Tag + Platform Pill 五种变体（#66-68）
- 统计卡片数值 + 进度条 + 操作阶段标签（#69-71）
- 骨架屏 shimmer + 空状态 + 错误状态（#72-74）
- 滚动条 + 文字选区（#75-76）
- Tint 动画（drift + sweep）+ Syncing 旋转 + Health Timeline（#77-79）

**对应清单条目：** #56-80

**验收标准：** 逐条对比原型 computed style，差异 < 5%。

### Phase 3：交互行为还原

**目标：** 所有可交互元素的行为与原型一致。

**具体工作：**
- 侧边栏导航 switchView()（#26）
- 按钮四态：Primary/Secondary/Ghost/Icon（#27-30）
- 卡片悬停上浮 + 统计卡片亮度增强（#31-32）
- 输入框聚焦发光 + Toggle 开关（#33-34）
- Tab 切换 + Source Tab（#35-36）
- 搜索框清空 + Marketplace 防抖（#37-38）
- CLI Sync Chip 三态流转（#39）
- Target Chip 多选 + Group Chip 单选（#40-41）
- Modal/Confirm/Toast 打开/关闭/自动消失（#42-44）
- CLI 安装流程 + 取消 + 进度条（#45-47）
- Dropdown + Marketplace 安装（#48-49）
- Plugin 详情 + Detail Tab（#50-51）
- Badge 计数 + 状态保持 + Checkbox + Traffic Light（#52-55）

**对应清单条目：** #26-55

**验收标准：** 每个交互行为可手动触发并观察到预期效果。

### Phase 4：数据与状态还原

**目标：** 数据模型、空状态、加载状态、筛选逻辑与原型一致。

**具体工作：**
- CLI 工具数据模型 + 操作状态机（#81-82）
- Software/Plugin/Marketplace/Skills/Agents 数据模型验证（#83-89）
- MCP Server + 审计日志（#90-91）
- Rules/Backups/Sources（#92-94）
- Theme 系统 20 个预设（#95）
- 各模块空状态处理（#96）
- Dashboard 聚合统计（#97）
- 搜索过滤联动（#98）
- Plugin Detail Modal 数据填充（#99）
- renderAll() 全局渲染管线（#100）

**对应清单条目：** #81-100

**验收标准：** 每个模块在有数据/无数据/筛选无结果三种状态下均正确显示。

### 阶段依赖关系

```
Phase 0 (令牌基础)
    ↓
Phase 1 (布局骨架)
    ↓
Phase 2 (视觉样式) ← 依赖 Phase 0 的变量 + Phase 1 的结构
    ↓
Phase 3 (交互行为) ← 依赖 Phase 2 的视觉基础
    ↓
Phase 4 (数据状态) ← 依赖前三个阶段的组件完整性
```

---

## 5. 文件变更清单

### 5.1 修改文件

| 文件 | 改动类型 | Phase |
|------|---------|-------|
| `src/assets/theme.css` | 新增 glass 令牌 + 全局 glass 组件样式 | 0 |
| `src/assets/main.css` | 新增背景纹理 + body 伪元素 | 0 |
| `src/composables/useGlassTheme.ts` | 扩展 selectGlassVariant() | 0 |
| `tailwind.config.js` | 修复 primary 色值 | 0 |
| `src/App.vue` | 验证 window-frame 结构 | 1 |
| `src/components/layout/Sidebar.vue` | 移除 scoped 颜色声明，依赖全局 | 1 |
| `src/components/layout/Titlebar.vue` | 移除 scoped 颜色声明，补 glass 主题 | 1 |
| `src/components/layout/Topbar.vue` | 移除 scoped 按钮颜色，依赖全局 | 1 |
| `src/views/DashboardView.vue` | 修复 tint 类名，验证 stat-card | 2 |
| `src/views/CliToolsView.vue` | 验证 tool-grid + 进度条 | 2-3 |
| `src/views/PluginsView.vue` | PluginCard 添加 .card 基类，验证 marketplace | 2-3 |
| `src/views/SkillsView.vue` | 验证 source-tabs + filter-bar | 2-3 |
| `src/views/AgentsView.vue` | 验证 target-chip 交互 | 3 |
| `src/views/MCPView.vue` | 验证 audit-table + groups | 2-3 |
| `src/views/RulesView.vue` | 验证 filter-bar + 列表 | 2-3 |
| `src/views/BackupView.vue` | 验证 stats-row(3列) | 2 |
| `src/views/SettingsView.vue` | 验证 theme-grid + toggle | 2-3 |
| `src/views/PromptManagerView.vue` | 样式适配 glass 风格 | 2 |
| `src/components/plugins/PluginCard.vue` | 添加 .card 基类，移除 scoped 颜色 | 2 |
| `src/components/plugins/PluginDetailsDialog.vue` | 样式更新 | 2 |
| `src/components/agents/AgentCard.vue` | 验证 target-chip | 3 |
| `src/components/mcp/MCPServerCard.vue` | 样式更新 | 2 |
| `src/components/skills/SkillCard.vue` | 样式更新 | 2 |
| 其他 Dialog 组件（~10 个） | 样式更新 | 2 |

### 5.2 不变文件

- 所有 Pinia store（`src/stores/*.ts`）
- 所有 TypeScript 类型定义（`src/types/*.ts`）
- 路由配置（`src/router/index.ts`）
- Tauri 后端（`src-tauri/src/**`）
- 图标组件（`src/components/icons/**`）

---

## 6. 验证方案

### 6.1 布局结构（#1-25）— 视觉对比

- Chrome DevTools Device Mode 固定宽度（1440/1024/768/640px）
- 并排对比原型 HTML 与 Vue 应用截图
- 检查：sidebar 宽度 240px、stats-row 4 列、modal 居中等

### 6.2 视觉样式（#56-80）— Computed Style 检查

- DevTools Elements > Computed 面板逐条验证
- 示例：`--glass-bg` = `rgba(255,255,255,0.28)` ✓/✗、`.card border-radius` = `18px` ✓/✗

### 6.3 交互行为（#26-55）— 手动触发

- 每个交互行为手动执行并观察
- 示例：点击 nav-item → active 切换 ✓/✗、hover Primary → translateY(-1px) ✓/✗

### 6.4 数据与状态（#81-100）— 三态覆盖

| 场景 | 验证 |
|------|------|
| 有数据 | 正常渲染 |
| 空数据 | 空状态 SVG + 文案 |
| 筛选无结果 | 空状态 + "Try adjusting filters" |

### 6.5 回归防护

- 每个 Phase 完成后在 4 个断点下全页面扫描
- 确保前序 Phase 不被后续 Phase 破坏

---

## 7. 风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| 重度 `backdrop-filter` 性能问题 | 提供 `prefers-reduced-motion` 降级 |
| 其他主题（light/dark/glass/yellow）因结构变更损坏 | 每完成一个组件在所有主题下验证 |
| scoped CSS 移除后样式丢失 | 确保全局 theme.css 已覆盖后再移除 scoped |
| tint 类名不匹配导致 Dashboard 异常 | 统一为 `tint-warm/cool/soft/amber` |

---

## 8. 成功标准

1. Warm 主题下视觉效果与原型 HTML 渲染一致度 > 90%
2. 所有 11 个页面功能正常运行，无 JS 错误
3. 100 条清单中每条均有明确的 ✓/✗ 验收状态
4. 其他 4 个主题（Light、Dark、Glass、Yellow）不受影响
5. `pnpm build` 编译通过，无 TypeScript 错误

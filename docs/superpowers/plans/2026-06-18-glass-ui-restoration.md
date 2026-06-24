# Glass UI 全量还原实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Forge Desktop 的所有 11 个页面、~75 个组件全面还原为 `design/forge-cross-platform-glass.html` 原型设计中的 Cross-Platform Glass 风格，覆盖 100 条 UI 还原清单。

**Architecture:** 采用三层分离策略——scoped CSS 仅保留结构样式，主题层和交互层移至 theme.css 全局规则。分 5 个 Phase 实施：令牌基础 → 布局骨架 → 视觉样式 → 交互行为 → 数据状态。

**Tech Stack:** Vue 3 + TypeScript + Tailwind CSS + Tauri 2.0

**设计规范:** `docs/superpowers/specs/2026-06-18-forge-glass-ui-redesign-design.md`

---

## 文件结构

### Phase 0 修改文件
- `src/assets/theme.css` — 新增 glass 令牌到 `:root`，扩展 `[data-theme="warm"]`
- `src/composables/useGlassTheme.ts` — 扩展 `selectGlassVariant()` 设置 glass 令牌
- `tailwind.config.js` — 修复 primary 色值

### Phase 1 修改文件
- `src/components/layout/Sidebar.vue` — 移除 scoped 颜色声明
- `src/components/layout/Titlebar.vue` — 移除 scoped 颜色声明
- `src/components/layout/Topbar.vue` — 移除 scoped 颜色声明
- `src/App.vue` — 验证 window-frame 结构

### Phase 2 修改文件
- `src/assets/theme.css` — 新增全局 glass 组件样式
- `src/views/*.vue` — 验证 grid 布局
- `src/components/plugins/PluginCard.vue` — 添加 `.card` 基类
- `src/components/skills/SkillCard.vue` — 添加 `.card` 基类
- `src/components/agents/AgentCard.vue` — 添加 `.card` 基类
- `src/components/mcp/MCPServerCard.vue` — 添加 `.card` 基类

---

## Phase 0: Glass 令牌基础设施

### Task 0.1: 在 `:root` 中新增 Glass 令牌

**Files:**
- Modify: `src/assets/theme.css:18-100`

在 `:root` 选择器的末尾（`--card-padding: 20px;` 之后）新增以下变量：

```css
  /* Glass Tokens — Cross-Platform Glass Prototype */
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

  /* Timing aliases */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --t-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --t-base: 200ms cubic-bezier(0.4, 0, 0.2, 1);
  --t-slow: 300ms cubic-bezier(0.4, 0, 0.2, 1);
```

- [ ] **Step 1:** 打开 `src/assets/theme.css`，找到 `:root` 选择器中 `--card-padding: 20px;` 这一行
- [ ] **Step 2:** 在该行之后、`}` 之前插入上述 CSS 变量块
- [ ] **Step 3:** 保存文件，运行 `pnpm dev` 确认无编译错误
- [ ] **Step 4:** 在浏览器 DevTools 中检查 `:root` 的 computed style，确认所有 `--glass-*` 变量存在
- [ ] **Step 5:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add glass token defaults to :root"
```

---

### Task 0.2: 扩展 `[data-theme="warm"]` 覆盖 Glass 令牌

**Files:**
- Modify: `src/assets/theme.css` (warm 主题块)

在 `[data-theme="warm"]` 选择器中，确保以下变量被覆盖（如果已存在则更新值，不存在则新增）：

```css
[data-theme="warm"] {
  /* 基础色 */
  --bg: #E8E4DE;
  --bg-card: rgba(255,255,255,0.42);
  --bg-card-hover: rgba(255,255,255,0.58);
  --bg-input: rgba(255,255,255,0.25);
  --bg-input-focus: rgba(255,255,255,0.45);
  --bg-sidebar: rgba(255,255,255,0.15);
  --bg-topbar: rgba(255,255,255,0.15);
  --bg-primary: var(--bg-card);
  --bg-secondary: rgba(255,255,255,0.18);
  --bg-tertiary: rgba(255,255,255,0.12);

  /* 边框 */
  --border: rgba(255,255,255,0.15);
  --border-hover: rgba(255,255,255,0.28);

  /* 文字 */
  --fg: #1A1A1A;
  --fg-title: #111111;
  --fg-muted: #5C5C5C;
  --fg-ghost: #9A9A9A;
  --fg-white: #1A1A1A;

  /* 强调色 */
  --accent: #2D2D2D;
  --accent-hover: #1A1A1A;
  --accent-press: #0D0D0D;
  --accent-bg: rgba(45,45,45,0.12);

  /* 语义色 */
  --success: #5A8A64;
  --error: #B85A42;
  --info: #5A6B7A;
  --warn: #B8944A;

  /* 圆角 */
  --radius: 18px;
  --radius-sm: 12px;
  --radius-lg: 24px;
  --radius-xl: 28px;

  /* 布局 */
  --sidebar-w: 240px;
  --topbar-h: 64px;
  --titlebar-h: 38px;

  /* 阴影 */
  --shadow: 0 2px 16px rgba(0,0,0,0.04);
  --shadow-hover: 0 8px 32px rgba(0,0,0,0.07);
  --shadow-md: 0 4px 20px rgba(0,0,0,0.05);
  --shadow-lg: 0 8px 32px rgba(0,0,0,0.07);

  /* Glass 令牌 */
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
}
```

- [ ] **Step 1:** 找到 `[data-theme="warm"]` 选择器块
- [ ] **Step 2:** 确保上述所有变量都在该块中定义
- [ ] **Step 3:** 保存，切换到 warm 主题验证变量生效
- [ ] **Step 4:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): update warm theme with complete glass tokens"
```

---

### Task 0.3: 扩展 `useGlassTheme.ts` 的 `selectGlassVariant()`

**Files:**
- Modify: `src/composables/useGlassTheme.ts:41-89`

更新 `selectGlassVariant()` 函数，使其在设置基础色的同时也设置 glass 令牌：

```typescript
function selectGlassVariant(variantId: GlassVariant) {
  if (variantId === 'warm') {
    setTheme('warm' as Theme)
    // Clear all dynamic overrides
    const r = document.documentElement.style
    const varsToRemove = [
      '--bg', '--bg-primary', '--bg-secondary',
      '--fg', '--fg-title', '--fg-muted', '--fg-ghost',
      '--accent', '--accent-hover', '--accent-press', '--accent-bg',
      '--warn', '--fg-white',
      '--glass-bg', '--glass-bg-hover', '--glass-sidebar', '--glass-topbar',
      '--glass-input', '--glass-input-focus', '--glass-window',
      '--border-window', '--shadow-inner',
      '--tint-warm', '--tint-cool', '--tint-soft', '--tint-amber',
    ]
    varsToRemove.forEach(v => r.removeProperty(v))
    document.body.style.background = ''
    localStorage.removeItem('aem-glass-variant')
    return
  }

  const variant = GLASS_VARIANTS.find(v => v.id === variantId)
  if (!variant) return

  setTheme('warm' as Theme)

  const [bg, surface, border, fg, accent, warn] = variant.colors
  const r = document.documentElement.style

  // 基础色
  r.setProperty('--bg', bg)
  r.setProperty('--bg-primary', bg)
  r.setProperty('--bg-secondary', surface)
  r.setProperty('--fg', fg)
  r.setProperty('--fg-title', fg)
  r.setProperty('--fg-white', fg)
  r.setProperty('--fg-muted', fg === '#1A1A1A' ? '#5C5C5C' : 'rgba(255,255,255,0.6)')
  r.setProperty('--fg-ghost', fg === '#1A1A1A' ? '#9A9A9A' : 'rgba(255,255,255,0.35)')
  r.setProperty('--accent', accent)
  r.setProperty('--accent-hover', accent + 'dd')
  r.setProperty('--accent-press', accent + 'bb')
  r.setProperty('--accent-bg', accent + '22')
  r.setProperty('--warn', warn)

  // Glass 令牌 — 根据亮/暗模式调整透明度
  const isDark = parseInt(fg.slice(1, 3), 16) > parseInt(bg.slice(1, 3), 16)
  if (isDark) {
    r.setProperty('--glass-bg', 'rgba(255,255,255,0.06)')
    r.setProperty('--glass-bg-hover', 'rgba(255,255,255,0.10)')
    r.setProperty('--glass-sidebar', 'rgba(255,255,255,0.04)')
    r.setProperty('--glass-topbar', 'rgba(255,255,255,0.04)')
    r.setProperty('--glass-input', 'rgba(255,255,255,0.08)')
    r.setProperty('--glass-input-focus', 'rgba(255,255,255,0.12)')
    r.setProperty('--glass-window', 'rgba(255,255,255,0.03)')
    r.setProperty('--border-window', 'rgba(255,255,255,0.08)')
    r.setProperty('--shadow-inner', 'inset 0 1px 2px rgba(0,0,0,0.20)')
    r.setProperty('--tint-warm', 'rgba(200,190,175,0.06)')
    r.setProperty('--tint-cool', 'rgba(180,185,195,0.05)')
    r.setProperty('--tint-soft', 'rgba(220,215,208,0.06)')
    r.setProperty('--tint-amber', 'rgba(184,148,74,0.05)')
  } else {
    r.setProperty('--glass-bg', 'rgba(255,255,255,0.28)')
    r.setProperty('--glass-bg-hover', 'rgba(255,255,255,0.42)')
    r.setProperty('--glass-sidebar', 'rgba(255,255,255,0.18)')
    r.setProperty('--glass-topbar', 'rgba(255,255,255,0.20)')
    r.setProperty('--glass-input', 'rgba(255,255,255,0.25)')
    r.setProperty('--glass-input-focus', 'rgba(255,255,255,0.45)')
    r.setProperty('--glass-window', 'rgba(255,255,255,0.12)')
    r.setProperty('--border-window', 'rgba(255,255,255,0.12)')
    r.setProperty('--shadow-inner', 'inset 0 1px 2px rgba(0,0,0,0.03)')
    r.setProperty('--tint-warm', 'rgba(200,190,175,0.15)')
    r.setProperty('--tint-cool', 'rgba(180,185,195,0.12)')
    r.setProperty('--tint-soft', 'rgba(220,215,208,0.18)')
    r.setProperty('--tint-amber', 'rgba(184,148,74,0.12)')
  }

  // 背景渐变
  document.body.style.background = isDark
    ? `linear-gradient(180deg, ${bg} 0%, ${bg} 50%, ${bg} 100%)`
    : `linear-gradient(160deg, ${surface} 0%, ${bg} 25%, ${bg}ee 50%, ${surface} 75%, ${bg} 100%)`

  localStorage.setItem('aem-glass-variant', variantId)
}
```

- [ ] **Step 1:** 替换 `src/composables/useGlassTheme.ts` 中 `selectGlassVariant()` 函数
- [ ] **Step 2:** 保存，运行 `pnpm dev` 确认无 TypeScript 错误
- [ ] **Step 3:** 在 Settings 页面切换主题变体，验证 glass 令牌在 DevTools 中正确设置
- [ ] **Step 4:** Commit

```bash
git add src/composables/useGlassTheme.ts
git commit -m "feat(theme): extend selectGlassVariant to set glass tokens"
```

---

### Task 0.4: 修复 Tailwind primary 色值

**Files:**
- Modify: `tailwind.config.js:10-22`

将 sky blue 色阶替换为 amber 色阶，与主题系统一致：

```javascript
colors: {
  primary: {
    50: '#FFFBEB',
    100: '#FEF3C7',
    200: '#FDE68A',
    300: '#FCD34D',
    400: '#FBBF24',
    500: '#F59E0B',
    600: '#D97706',
    700: '#B45309',
    800: '#92400E',
    900: '#78350F',
    950: '#451A03',
  },
},
```

- [ ] **Step 1:** 替换 `tailwind.config.js` 中的 `colors.primary` 对象
- [ ] **Step 2:** 运行 `pnpm dev` 确认无编译错误
- [ ] **Step 3:** Commit

```bash
git add tailwind.config.js
git commit -m "fix(tailwind): update primary color scale to amber"
```

---

## Phase 1: 布局骨架验证与修复

### Task 1.1: 验证 Sidebar 变量命名

**Files:**
- Read: `src/components/layout/Sidebar.vue` (scoped style section)

检查 scoped CSS 中的变量引用：
- `var(--sidebar-bg)` → 应为 `var(--glass-sidebar)` 或删除（由全局覆盖）
- `var(--sidebar-border)` → 应删除（由全局覆盖）
- `var(--search-bg)` → 应为 `var(--glass-input)` 或删除
- `var(--avatar-bg)` → 保留 fallback 值即可

- [ ] **Step 1:** 读取 Sidebar.vue 的 scoped style 部分
- [ ] **Step 2:** 列出所有颜色/背景/边框声明
- [ ] **Step 3:** 标记哪些需要删除（移至全局）、哪些需要更新变量名
- [ ] **Step 4:** 记录发现，暂不修改（Phase 2 统一处理）

---

### Task 1.2: 验证响应式断点

**Files:**
- Read: `src/assets/theme.css` (media query sections)

检查三个断点的规则：
- `@media(max-width:1024px)` — stats-row 变 2 列，settings-grid 变单列
- `@media(max-width:768px)` — sidebar 隐藏，content padding 缩小
- `@media(max-width:640px)` — stats-row 变 1 列

- [ ] **Step 1:** 搜索 theme.css 中的 `@media` 规则
- [ ] **Step 2:** 与原型的断点规则对比
- [ ] **Step 3:** 记录缺失的规则，Phase 2 补充

---

## Phase 2: 视觉样式全局化

### Task 2.1: 为 Sidebar 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css` (在 warm 主题块末尾新增)

```css
/* === Sidebar Glass Styles === */
[data-theme="warm"] .sidebar {
  background: var(--glass-sidebar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid rgba(255,255,255,0.22);
}

[data-theme="warm"] .sidebar-brand {
  border-bottom: 1px solid rgba(255,255,255,0.18);
}

[data-theme="warm"] .sidebar-search input {
  background: var(--glass-input);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
  border: 1px solid rgba(255,255,255,0.12);
}

[data-theme="warm"] .sidebar-search input:focus {
  background: var(--glass-input-focus);
  border-color: rgba(255,255,255,0.25);
}

[data-theme="warm"] .nav-item:hover {
  background: var(--glass-bg-hover);
  color: var(--fg);
}

[data-theme="warm"] .nav-item.active {
  background: rgba(255,255,255,0.18);
  color: var(--fg-title);
  border-left-color: var(--accent);
}

[data-theme="warm"] .nav-item.active svg {
  opacity: 1;
}

[data-theme="warm"] .sidebar-footer {
  border-top: 1px solid rgba(255,255,255,0.18);
}

[data-theme="warm"] .sidebar-footer .avatar {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.12);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}
```

- [ ] **Step 1:** 在 theme.css 的 `[data-theme="warm"]` 块末尾添加上述规则
- [ ] **Step 2:** 保存，切换到 warm 主题验证 sidebar 样式
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add sidebar glass styles to warm theme"
```

---

### Task 2.2: 为 Topbar 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Topbar Glass Styles === */
[data-theme="warm"] .topbar {
  background: var(--glass-topbar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-bottom: 1px solid rgba(255,255,255,0.22);
}

[data-theme="warm"] .topbar-btn.primary {
  background: rgba(45,45,45,0.85);
  color: #fff;
  box-shadow: 0 1px 4px rgba(0,0,0,0.08);
  border: 1px solid rgba(255,255,255,0.08);
}

[data-theme="warm"] .topbar-btn.primary:hover {
  background: rgba(26,26,26,0.90);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.12);
}

[data-theme="warm"] .topbar-btn.primary:active {
  background: rgba(13,13,13,0.95);
  transform: translateY(0);
}

[data-theme="warm"] .topbar-btn.secondary {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.15);
  color: var(--fg-muted);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .topbar-btn.secondary:hover {
  border-color: rgba(255,255,255,0.30);
  color: var(--accent);
  background: rgba(255,255,255,0.25);
}

[data-theme="warm"] .topbar-search input {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.12);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
}
```

- [ ] **Step 1:** 添加上述规则到 theme.css
- [ ] **Step 2:** 保存，验证 topbar 样式
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add topbar glass styles to warm theme"
```

---

### Task 2.3: 为 Card 组件添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Card Glass Styles === */
[data-theme="warm"] .card {
  background: var(--glass-bg);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255,255,255,0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0,0,0,0.05);
  transition: all var(--t-base);
}

[data-theme="warm"] .card:hover {
  background: var(--glass-bg-hover);
  border-color: rgba(255,255,255,0.50);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

/* === Stat Card Glass Styles === */
[data-theme="warm"] .stat-card {
  background: rgba(255,255,255,0.62);
  backdrop-filter: blur(24px) saturate(1.3);
  -webkit-backdrop-filter: blur(24px) saturate(1.3);
  border: 1px solid rgba(255,255,255,0.55);
  border-radius: var(--radius);
  box-shadow: 0 1px 3px rgba(0,0,0,0.04), 0 4px 16px rgba(0,0,0,0.03), inset 0 1px 0 rgba(255,255,255,0.60);
  position: relative;
  overflow: hidden;
  transition: all var(--t-base);
}

[data-theme="warm"] .stat-card:hover {
  background: rgba(255,255,255,0.78);
  border-color: rgba(255,255,255,0.70);
  box-shadow: 0 2px 8px rgba(0,0,0,0.05), 0 8px 24px rgba(0,0,0,0.05), inset 0 1px 0 rgba(255,255,255,0.70);
  transform: translateY(-1px);
}

[data-theme="warm"] .stat-card.tint-warm::before,
[data-theme="warm"] .stat-card.tint-cool::before,
[data-theme="warm"] .stat-card.tint-soft::before,
[data-theme="warm"] .stat-card.tint-amber::before {
  content: '';
  position: absolute;
  top: -80%;
  left: -50%;
  width: 200%;
  height: 200%;
  pointer-events: none;
  animation: tint-drift 8s ease-in-out infinite;
}

[data-theme="warm"] .stat-card.tint-warm::before {
  background: radial-gradient(ellipse at center, rgba(210,190,160,0.12) 0%, rgba(210,190,160,0.04) 30%, transparent 60%);
}

[data-theme="warm"] .stat-card.tint-cool::before {
  background: radial-gradient(ellipse at center, rgba(155,170,210,0.12) 0%, rgba(155,170,210,0.04) 30%, transparent 60%);
  animation-delay: -3s;
  animation-duration: 10s;
}

[data-theme="warm"] .stat-card.tint-soft::before {
  background: radial-gradient(ellipse at center, rgba(195,185,170,0.12) 0%, rgba(195,185,170,0.04) 30%, transparent 60%);
  animation-delay: -5s;
  animation-duration: 9s;
}

[data-theme="warm"] .stat-card.tint-amber::before {
  background: radial-gradient(ellipse at center, rgba(210,175,75,0.10) 0%, rgba(210,175,75,0.03) 30%, transparent 60%);
  animation-delay: -2s;
  animation-duration: 7s;
}

[data-theme="warm"] .stat-card.tint-warm::after,
[data-theme="warm"] .stat-card.tint-cool::after,
[data-theme="warm"] .stat-card.tint-soft::after,
[data-theme="warm"] .stat-card.tint-amber::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 300%;
  height: 2px;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0) 10%, rgba(255,255,255,0.40) 30%, rgba(255,255,255,0.60) 50%, rgba(255,255,255,0.40) 70%, rgba(255,255,255,0) 90%, transparent 100%);
  pointer-events: none;
  animation: tint-sweep 5s ease-in-out infinite;
}

[data-theme="warm"] .stat-card.tint-cool::after { animation-duration: 6s; animation-delay: -2s; }
[data-theme="warm"] .stat-card.tint-soft::after { animation-duration: 7s; animation-delay: -4s; }
[data-theme="warm"] .stat-card.tint-amber::after { animation-duration: 4.5s; animation-delay: -1s; }

@keyframes tint-drift {
  0% { transform: translate(0, 0) }
  33% { transform: translate(8%, 5%) }
  66% { transform: translate(-5%, 8%) }
  100% { transform: translate(0, 0) }
}

@keyframes tint-sweep {
  0% { transform: translateX(0) }
  50% { transform: translateX(33%) }
  100% { transform: translateX(0) }
}
```

- [ ] **Step 1:** 添加上述规则到 theme.css
- [ ] **Step 2:** 验证 Dashboard 统计卡片的 glass 效果和 tint 动画
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add card and stat-card glass styles with tint animations"
```

---

### Task 2.4: 为 Modal/Toast 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Modal Glass Styles === */
[data-theme="warm"] .modal-overlay {
  background: rgba(0,0,0,0.22);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .modal {
  background: rgba(255,255,255,0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255,255,255,0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0,0,0,0.12), 0 4px 20px rgba(0,0,0,0.05), inset 0 1px 0 rgba(255,255,255,0.50);
}

/* === Toast Glass Styles === */
[data-theme="warm"] .toast {
  background: rgba(255,255,255,0.42);
  backdrop-filter: blur(30px) saturate(1.3);
  -webkit-backdrop-filter: blur(30px) saturate(1.3);
  border: 1px solid rgba(255,255,255,0.30);
  border-radius: var(--radius);
  box-shadow: 0 8px 32px rgba(0,0,0,0.10), inset 0 1px 0 rgba(255,255,255,0.40);
}

[data-theme="warm"] .toast.success { border-color: rgba(90,138,100,0.40); }
[data-theme="warm"] .toast.error { border-color: rgba(184,90,66,0.40); }
[data-theme="warm"] .toast.info { border-color: rgba(90,107,122,0.40); }
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证 Modal 和 Toast 的 glass 效果
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add modal and toast glass styles"
```

---

### Task 2.5: 为 Badge/Tag 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Badge Glass Styles === */
[data-theme="warm"] .badge {
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

[data-theme="warm"] .badge.success {
  background: rgba(90,138,100,0.15);
  border: 1px solid rgba(90,138,100,0.20);
  color: var(--success);
}

[data-theme="warm"] .badge.warn {
  background: rgba(184,148,74,0.15);
  border: 1px solid rgba(184,148,74,0.20);
  color: var(--warn);
}

[data-theme="warm"] .badge.error {
  background: rgba(184,90,66,0.15);
  border: 1px solid rgba(184,90,66,0.20);
  color: var(--error);
}

[data-theme="warm"] .badge.info {
  background: rgba(90,107,122,0.15);
  border: 1px solid rgba(90,107,122,0.20);
  color: var(--info);
}

[data-theme="warm"] .badge.outline {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.15);
  color: var(--fg-muted);
}

/* === Tag Glass Styles === */
[data-theme="warm"] .tag {
  background: rgba(255,255,255,0.20);
  border: 1px solid rgba(255,255,255,0.12);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证 Badge 和 Tag 的 glass 效果
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add badge and tag glass styles"
```

---

### Task 2.6: 为 Button 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Button Glass Styles === */
[data-theme="warm"] .btn-primary {
  background: rgba(45,45,45,0.85);
  color: #fff;
  box-shadow: var(--shadow-btn);
  border: 1px solid rgba(255,255,255,0.08);
}

[data-theme="warm"] .btn-primary:hover {
  background: rgba(26,26,26,0.90);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.12);
}

[data-theme="warm"] .btn-primary:active {
  background: rgba(13,13,13,0.95);
  transform: translateY(0);
}

[data-theme="warm"] .btn-secondary {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.15);
  color: var(--fg-muted);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .btn-secondary:hover {
  border-color: rgba(255,255,255,0.30);
  color: var(--accent);
  background: rgba(255,255,255,0.25);
}

[data-theme="warm"] .btn-ghost {
  color: var(--fg-ghost);
}

[data-theme="warm"] .btn-ghost:hover {
  background: rgba(255,255,255,0.15);
  color: var(--fg);
}

[data-theme="warm"] .btn-icon {
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.15);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .btn-icon:hover {
  border-color: rgba(255,255,255,0.30);
  color: var(--accent);
  background: rgba(255,255,255,0.25);
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证按钮的 glass 效果和三态
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add button glass styles with hover/active states"
```

---

### Task 2.7: 为 Input/Toggle 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Input Glass Styles === */
[data-theme="warm"] input,
[data-theme="warm"] textarea,
[data-theme="warm"] select {
  background: var(--glass-input);
  border: 1px solid rgba(255,255,255,0.15);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
  transition: border-color var(--t-fast), background var(--t-fast), box-shadow var(--t-fast);
}

[data-theme="warm"] input:focus,
[data-theme="warm"] textarea:focus,
[data-theme="warm"] select:focus {
  background: var(--glass-input-focus);
  border-color: rgba(255,255,255,0.30);
  box-shadow: 0 0 0 2px var(--accent-bg);
}

[data-theme="warm"] input::placeholder,
[data-theme="warm"] textarea::placeholder {
  color: var(--fg-ghost);
}

/* === Toggle Glass Styles === */
[data-theme="warm"] .toggle {
  background: rgba(255,255,255,0.22);
  border: 1px solid rgba(255,255,255,0.12);
}

[data-theme="warm"] .toggle.on {
  background: var(--accent);
}

[data-theme="warm"] .toggle::after {
  background: white;
  box-shadow: 0 1px 4px rgba(0,0,0,0.12);
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证输入框聚焦效果和 Toggle 开关
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add input and toggle glass styles"
```

---

### Task 2.8: 为 Scrollbar/Selection 添加全局样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Scrollbar Glass Styles === */
[data-theme="warm"] ::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

[data-theme="warm"] ::-webkit-scrollbar-track {
  background: transparent;
}

[data-theme="warm"] ::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.10);
  border-radius: 3px;
}

[data-theme="warm"] ::-webkit-scrollbar-thumb:hover {
  background: rgba(0,0,0,0.18);
}

/* === Selection Styles === */
[data-theme="warm"] ::selection {
  background: rgba(45,45,45,0.10);
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证滚动条和文字选区样式
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add scrollbar and selection styles"
```

---

### Task 2.9: 为 Background 纹理添加全局样式

**Files:**
- Modify: `src/assets/theme.css` (在 `[data-theme="warm"]` 块中或单独的 body 规则)

```css
/* === Background Texture === */
[data-theme="warm"] body {
  background-attachment: fixed;
  background-image:
    linear-gradient(160deg, #DDD8D0 0%, #E5E1DA 25%, #EBE7E0 50%, #E2DED6 75%, #D9D4CC 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(190,175,155,0.20) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(160,170,190,0.18) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(200,195,185,0.12) 0%, transparent 45%),
    linear-gradient(rgba(0,0,0,0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0,0,0,0.025) 1px, transparent 1px),
    linear-gradient(rgba(0,0,0,0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0,0,0,0.015) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
}

[data-theme="warm"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(185,170,150,0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(155,165,185,0.12) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(200,195,185,0.08) 0%, transparent 45%);
}

[data-theme="warm"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.30;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.75' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.06'/%3E%3C/svg%3E");
  background-size: 180px 180px;
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证背景纹理效果（细网格 + 噪点）
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add background texture with grid and noise"
```

---

### Task 2.10: 为 Dropdown/Tab/Filter 添加全局 Glass 样式

**Files:**
- Modify: `src/assets/theme.css`

```css
/* === Dropdown Glass Styles === */
[data-theme="warm"] .dropdown-menu {
  background: rgba(255,255,255,0.45);
  backdrop-filter: blur(30px) saturate(1.3);
  -webkit-backdrop-filter: blur(30px) saturate(1.3);
  border: 1px solid rgba(255,255,255,0.30);
  border-radius: var(--radius-sm);
  box-shadow: 0 12px 40px rgba(0,0,0,0.10), inset 0 1px 0 rgba(255,255,255,0.40);
}

[data-theme="warm"] .dropdown-item:hover {
  background: rgba(255,255,255,0.15);
  color: var(--fg);
}

[data-theme="warm"] .dropdown-item.danger {
  color: var(--error);
}

[data-theme="warm"] .dropdown-item.danger:hover {
  background: rgba(184,90,66,0.12);
}

/* === Tab Bar Glass Styles === */
[data-theme="warm"] .tab-bar {
  border-bottom: 1px solid var(--border);
}

[data-theme="warm"] .tab-item:hover {
  color: var(--fg);
}

[data-theme="warm"] .tab-item.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* === Filter Bar Glass Styles === */
[data-theme="warm"] .filter-select {
  background: rgba(255,255,255,0.20);
  border: 1px solid rgba(255,255,255,0.18);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
}

[data-theme="warm"] .filter-select:focus {
  border-color: rgba(255,255,255,0.30);
}
```

- [ ] **Step 1:** 添加上述规则
- [ ] **Step 2:** 验证 Dropdown、Tab、Filter 的 glass 效果
- [ ] **Step 3:** Commit

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add dropdown, tab, and filter glass styles"
```

---

### Task 2.11: 为 PluginCard 添加 `.card` 基类

**Files:**
- Modify: `src/components/plugins/PluginCard.vue` (template 部分)

找到模板中卡片的根元素，将 `class="plugin-card"` 改为 `class="card plugin-card"`：

```vue
<!-- 之前 -->
<div class="plugin-card" ...>

<!-- 之后 -->
<div class="card plugin-card" ...>
```

- [ ] **Step 1:** 修改 PluginCard.vue 模板
- [ ] **Step 2:** 验证卡片获得全局 `.card` 的 glass 样式
- [ ] **Step 3:** Commit

```bash
git add src/components/plugins/PluginCard.vue
git commit -m "feat(plugins): add .card base class to PluginCard"
```

---

### Task 2.12: 验证所有视图的 Grid 布局

**Files:**
- Read: `src/views/DashboardView.vue` — 验证 stats-row 4 列
- Read: `src/views/BackupView.vue` — 验证 stats-row 3 列
- Read: `src/views/SettingsView.vue` — 验证 settings-grid 2 列
- Read: `src/views/PluginsView.vue` — 验证 marketplace-grid 自适应

- [ ] **Step 1:** 逐个检查上述视图的 grid 布局类名
- [ ] **Step 2:** 与原型对比列数和间距
- [ ] **Step 3:** 记录需要修复的差异

---

## Phase 3: 交互行为修复

### Task 3.1: 移除 Sidebar scoped 颜色声明

**Files:**
- Modify: `src/components/layout/Sidebar.vue` (scoped style section)

从 scoped CSS 中删除以下类型的声明（已由全局 theme.css 覆盖）：
- `background` 属性（使用 `var(--sidebar-bg)` 等的声明）
- `border-right` 属性
- `backdrop-filter` 属性
- nav-item 的 `background` hover/active 声明

保留的声明：
- `width`、`height`、`display`、`flex-direction` 等布局属性
- `padding`、`margin`、`gap` 等间距属性
- `font-size`、`font-weight` 等排版属性

- [ ] **Step 1:** 读取 Sidebar.vue 的 scoped style
- [ ] **Step 2:** 逐行标记需要删除的颜色/背景/边框声明
- [ ] **Step 3:** 删除标记的行
- [ ] **Step 4:** 验证 sidebar 在 warm 主题下仍正确显示（依赖全局样式）
- [ ] **Step 5:** 验证 sidebar 在 light 主题下不丢失基本样式
- [ ] **Step 6:** Commit

```bash
git add src/components/layout/Sidebar.vue
git commit -m "refactor(sidebar): remove scoped color declarations, rely on global theme"
```

---

### Task 3.2: 移除 Topbar scoped 颜色声明

**Files:**
- Modify: `src/components/layout/Topbar.vue` (scoped style section)

同 Task 3.1，从 scoped CSS 中删除颜色/背景/边框声明。

- [ ] **Step 1:** 读取 Topbar.vue 的 scoped style
- [ ] **Step 2:** 删除颜色/背景/边框声明
- [ ] **Step 3:** 验证 topbar 在 warm 主题下正确显示
- [ ] **Step 4:** Commit

```bash
git add src/components/layout/Topbar.vue
git commit -m "refactor(topbar): remove scoped color declarations, rely on global theme"
```

---

### Task 3.3: 移除 Titlebar scoped 颜色声明

**Files:**
- Modify: `src/components/layout/Titlebar.vue` (scoped style section)

同上，删除 scoped 中的颜色声明，依赖全局样式。

- [ ] **Step 1:** 读取 Titlebar.vue 的 scoped style
- [ ] **Step 2:** 删除颜色/背景/边框声明
- [ ] **Step 3:** 验证 titlebar 在 warm 主题下正确显示
- [ ] **Step 4:** Commit

```bash
git add src/components/layout/Titlebar.vue
git commit -m "refactor(titlebar): remove scoped color declarations, rely on global theme"
```

---

### Task 3.4: 验证所有交互状态

**Files:**
- 验证: 所有已修改组件

逐项验证以下交互行为：

- [ ] **Step 1:** 侧边栏导航 — 点击切换 active 状态
- [ ] **Step 2:** Primary 按钮 — hover 上浮 + active 下沉
- [ ] **Step 3:** Secondary 按钮 — hover 边框变亮
- [ ] **Step 4:** Ghost 按钮 — hover 添加背景
- [ ] **Step 5:** Icon 按钮 — hover 边框变亮
- [ ] **Step 6:** 卡片 — hover 上浮 + 背景变亮
- [ ] **Step 7:** 统计卡片 — hover 上浮 + tint 亮度增强
- [ ] **Step 8:** 输入框 — focus 边框变亮 + 外发光
- [ ] **Step 9:** Toggle — 点击切换开/关
- [ ] **Step 10:** Tab — 点击切换 active
- [ ] **Step 11:** Toast — 触发后自动消失
- [ ] **Step 12:** Modal — 打开/关闭/Esc 关闭

---

## Phase 4: 数据与状态验证

### Task 4.1: 验证空状态

**Files:**
- 验证: 各视图的空状态渲染

- [ ] **Step 1:** 在搜索框输入不存在的关键词，验证空状态显示
- [ ] **Step 2:** 检查空状态的 SVG 图标 + 标题 + 描述文案
- [ ] **Step 3:** 验证 "Try adjusting filters" 文案

---

### Task 4.2: 验证筛选联动

**Files:**
- 验证: 各视图的 filter-bar

- [ ] **Step 1:** 验证搜索框 oninput 实时过滤
- [ ] **Step 2:** 验证 select 下拉筛选
- [ ] **Step 3:** 验证多维筛选组合
- [ ] **Step 4:** 验证计数 Badge 更新

---

### Task 4.3: 验证响应式断点

**Files:**
- 验证: 所有视图在不同宽度下

- [ ] **Step 1:** 1440px — 全功能展示
- [ ] **Step 2:** 1024px — stats-row 变 2 列
- [ ] **Step 3:** 768px — sidebar 隐藏
- [ ] **Step 4:** 640px — stats-row 变 1 列

---

## 最终验证

### Task 5.1: 全页面扫描

- [ ] **Step 1:** 逐页访问所有 11 个路由
- [ ] **Step 2:** 对比原型截图，标记差异
- [ ] **Step 3:** 修复剩余差异

### Task 5.2: 构建验证

- [ ] **Step 1:** 运行 `pnpm build` 确认无 TypeScript 错误
- [ ] **Step 2:** 运行 `pnpm test` 确认无测试失败
- [ ] **Step 3:** 最终 Commit

```bash
git add -A
git commit -m "feat: complete glass UI restoration - 100 items verified"
```

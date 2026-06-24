# UI & Interaction Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the Vue implementation's glass design system to the reference prototype (`forge-cross-platform-glass.html`)

**Architecture:** CSS-only changes — fix token duplication in `theme.css`, replace hardcoded rgba with glass tokens, add glass layering to sidebar/topbar, extend theme system. No new components, no logic changes.

**Tech Stack:** Vue 3, CSS custom properties, Pinia theme store

---

## File Map

| File | Role | Tasks |
|------|------|-------|
| `src/assets/theme.css` | Master theme — imports + supplementary styles | T1, T2, T3, T4, T5 |
| `design/tokens/colors.css` | Source of truth: color/glass/radius tokens | Read only |
| `design/tokens/glass.css` | Source of truth: glass blur/saturation tokens | Read only |
| `design/tokens/motion.css` | Source of truth: animation keyframes | Read only |
| `design/tokens/themes/warm.css` | Warm theme overrides | Read only |
| `design/tokens/themes/midnight.css` | Midnight theme overrides | Read only |
| `design/tokens/index.ts` | ThemeId type | T7 |
| `src/stores/theme.ts` | Theme state management | T7 |
| `src/components/layout/AppFrame.vue` | Window frame glass | T3 |
| `src/components/layout/Sidebar.vue` | Sidebar glass layer | T3, T4 |
| `src/components/layout/Topbar.vue` | Topbar glass layer | T2, T3, T4 |
| `src/components/common/StatCard.vue` | Stat card glass + animations | T3 |
| `src/components/common/Toast.vue` | Toast glass | T3 |
| `src/components/common/Modal.vue` | Modal glass | T3 |
| `src/views/SettingsView.vue` | Theme picker UI | T7 |
| `design/tokens/themes/sage.css` | New theme (create) | T7 |
| `design/tokens/themes/lavender.css` | New theme (create) | T7 |
| `design/tokens/themes/arctic.css` | New theme (create) | T7 |

---

### Task 1: Token System Dedup — Remove Conflicts from theme.css

**Files:**
- Modify: `src/assets/theme.css:33-171` (`:root` block)

The `:root` block in `theme.css` defines ~80+ variables. Many overlap with values already defined by the imported `design/tokens/*.css` files. The imported files are the source of truth — `theme.css` `:root` should only define variables NOT present in the token files.

**Variables to REMOVE from theme.css `:root` (already defined in token files):**

| Variable | theme.css value | Token file | Token value |
|----------|----------------|------------|-------------|
| `--bg` | `#FAFAFA` | warm.css:13 | `#F0EDE8` |
| `--border` | `#E4E4E7` | colors.css:46 | `rgba(255,255,255,0.20)` |
| `--border-hover` | `#D4D4D8` | colors.css:47 | `rgba(255,255,255,0.35)` |
| `--fg` | `#18181B` | colors.css:52 | `#1A1A1A` |
| `--fg-muted` | `#52525B` | colors.css:54 | `#5C5C5C` |
| `--fg-ghost` | `#A1A1AA` | colors.css:55 | `#9A9A9A` |
| `--fg-white` | `#09090B` | colors.css:56 | `#1A1A1A` |
| `--accent` | `#D97706` | colors.css:61 | `#2D2D2D` |
| `--accent-hover` | `#B45309` | colors.css:62 | `#1A1A1A` |
| `--accent-press` | `#92400E` | colors.css:63 | `#0D0D0D` |
| `--success` | `#059669` | colors.css:69 | `#5A8A64` |
| `--error` | `#DC2626` | colors.css:70 | `#B85A42` |
| `--info` | `#0891B2` | colors.css:71 | `#5A6B7A` |
| `--warn` | `#D97706` | colors.css:72 | `#B8944A` |
| `--font-display` | Inter... | colors.css:86 | Inter... |
| `--font-body` | Inter... | colors.css:87 | Inter... |
| `--font-mono` | JetBrains... | colors.css:88 | JetBrains... |
| `--sidebar-w` | `240px` | colors.css:93 | `240px` |
| `--topbar-h` | `56px` ❌ | colors.css:94 | `64px` |
| `--radius` | `8px` | colors.css:99 | `18px` |
| `--radius-sm` | `8px` | colors.css:100 | `12px` |
| `--radius-lg` | `16px` | colors.css:101 | `24px` |
| `--radius-xl` | `20px` | colors.css:102 | `28px` |
| `--glass-bg` | `rgba(...,0.28)` | colors.css:32 | `rgba(...,0.45)` |
| `--glass-bg-hover` | `rgba(...,0.42)` | colors.css:33 | `rgba(...,0.58)` |
| `--glass-sidebar` | `rgba(...,0.18)` | colors.css:26 | `rgba(...,0.35)` |
| `--glass-topbar` | `rgba(...,0.20)` | colors.css:29 | `rgba(...,0.38)` |
| `--glass-input` | `rgba(...,0.25)` | colors.css:36 | `rgba(...,0.40)` |
| `--glass-input-focus` | `rgba(...,0.45)` | colors.css:37 | `rgba(...,0.60)` |
| `--glass-window` | `rgba(...,0.12)` | colors.css:21 | `rgba(...,0.25)` |
| `--glass-inner-glow` | `rgba(...,0.70)` | colors.css:40 | same |
| `--glass-highlight` | `rgba(...,0.85)` | colors.css:41 | same |
| `--border-window` | `rgba(...,0.12)` ❌ | colors.css:22 | `rgba(...,0.18)` |
| `--border-outer-glow` | `rgba(...,0.10)` | colors.css:23 | `rgba(...,0.40)` |
| `--shadow-btn` | `0 1px 4px...` | colors.css:109 | same |
| `--shadow-window` | `0 16px 48px...` | colors.css:110 | same |
| `--shadow-inner` | `inset 0 1px 2px...` | colors.css:111 | same |
| `--tint-warm` | `rgba(200,190,175,0.15)` | colors.css:78 | `rgba(195,178,155,0.12)` |
| `--tint-cool` | `rgba(180,185,195,0.12)` | colors.css:79 | `rgba(195,178,155,0.10)` |
| `--tint-soft` | `rgba(220,215,208,0.18)` | colors.css:80 | `rgba(195,178,155,0.14)` |
| `--tint-amber` | `rgba(184,148,74,0.12)` | colors.css:81 | `rgba(195,178,155,0.08)` |
| `--ease` | `cubic-bezier(...)` | colors.css:116 | same |
| `--t-fast` | `150ms ...` | colors.css:117 | same |
| `--t-base` | `200ms ...` | colors.css:118 | same |
| `--t-slow` | `300ms ...` | colors.css:119 | same |
| `--z-sidebar` through `--z-tooltip` | — | colors.css:124-130 | same |

**Variables to KEEP in theme.css `:root` (not in token files):**

- `--bg-card`, `--bg-input`, `--bg-primary`, `--bg-secondary`, `--bg-tertiary` (component aliases)
- `--accent-bg` (not in colors.css)
- `--success-bg`, `--error-bg`, `--info-bg`, `--warn-bg` (semantic bg variants)
- `--card-padding-sm/md/lg`, `--card-radius` (component tokens)
- `--section-header-*` tokens
- `--btn-height-*` tokens
- `--bp-*` responsive breakpoints
- `--content-padding`, `--card-padding`
- `--titlebar-h`
- `--shadow-sm/md/lg` (distinct from `--shadow`/`--shadow-hover`)
- `--focus-ring`
- `--transition-fast/base/slow` (aliases for `--t-fast/base/slow`)
- `--btn-disabled-*` tokens
- `--btn-transition`

- [ ] **Step 1: Remove duplicate variables from `:root` in theme.css**

Open `src/assets/theme.css`. In the `:root` block (lines 33-171), remove all variables listed in the "Variables to REMOVE" table above. Keep only the "Variables to KEEP" list.

The resulting `:root` block should look like:

```css
:root {
  /* Component-level aliases (not in token files) */
  --bg-card: #FFFFFF;
  --bg-input: #F4F4F5;
  --bg-primary: var(--bg-card);
  --bg-secondary: var(--bg-input);
  --bg-tertiary: var(--bg-input);

  --accent-bg: #FEF3C7;

  --success-bg: rgba(5, 150, 105, 0.10);
  --error-bg: rgba(220, 38, 38, 0.10);
  --info-bg: rgba(8, 145, 178, 0.10);
  --warn-bg: rgba(217, 119, 6, 0.10);

  --card-padding-sm: 12px;
  --card-padding-md: 16px;
  --card-padding-lg: 24px;
  --card-radius: 12px;

  --section-header-padding: 16px 0;
  --section-header-margin-bottom: 24px;
  --section-header-gap: 12px;
  --section-header-title-size: 18px;
  --section-header-title-weight: 600;

  --btn-height-sm: 32px;
  --btn-height-md: 40px;
  --btn-height-lg: 48px;
  --btn-height-icon: 34px;

  --bp-sm: 640px;
  --bp-md: 768px;
  --bp-lg: 1024px;
  --bp-xl: 1280px;

  --content-padding: 24px 32px;
  --card-padding: 20px;

  --titlebar-h: 38px;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.08);

  --focus-ring: 0 0 0 2px rgba(217, 119, 6, 0.12);

  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-base: 200ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 300ms cubic-bezier(0.4, 0, 0.2, 1);

  --btn-disabled-opacity: 0.5;
  --btn-disabled-cursor: not-allowed;
  --btn-transition: all 0.15s ease;
}
```

- [ ] **Step 2: Remove duplicate `[data-theme="dark"]` block**

The `[data-theme="dark"]` block (lines 174-210) duplicates midnight.css. Remove the entire block EXCEPT for `--bg-card`, `--bg-input`, `--bg-primary/secondary/tertiary`, `--accent-bg`, `--success-bg/error-bg/info-bg/warn-bg`, `--shadow-sm/md/lg`, and `--focus-ring` — these are component-level overrides not in the token files.

- [ ] **Step 3: Remove duplicate `[data-theme="warm"]` block**

The `[data-theme="warm"]` block (lines 226-268) duplicates warm.css. Remove the entire block. The warm.css token file already defines all these variables.

- [ ] **Step 4: Verify build compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npm run build`
Expected: Build succeeds with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/assets/theme.css
git commit -m "refactor(theme): remove duplicate token definitions, defer to design/tokens/"
```

---

### Task 2: Fix Dimension Mismatches and Remove Component Fallbacks

**Files:**
- Modify: `src/components/layout/Topbar.vue:43-45`
- Modify: `src/components/layout/Sidebar.vue:237-238`
- Modify: `src/assets/theme.css:310-322` (warm window-frame)

- [ ] **Step 1: Fix Topbar.vue fallback values**

In `src/components/layout/Topbar.vue`, change:

```css
/* Before */
.topbar {
  height: var(--topbar-h, 64px);
  min-height: var(--topbar-h, 64px);

/* After */
.topbar {
  height: var(--topbar-h);
  min-height: var(--topbar-h);
```

- [ ] **Step 2: Fix Sidebar.vue fallback values**

In `src/components/layout/Sidebar.vue`, change:

```css
/* Before */
.sidebar {
  width: var(--sidebar-w, 240px);
  min-width: var(--sidebar-w, 240px);

/* After */
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
```

Also fix the brand height:

```css
/* Before */
.sidebar-brand {
  height: var(--topbar-h, 64px);

/* After */
.sidebar-brand {
  height: var(--topbar-h);
```

Also fix the title color fallback:

```css
/* Before */
.sidebar-brand {
  color: var(--fg-title, var(--fg));
.nav-item.active {
  color: var(--fg-title, var(--fg));
.topbar-title {
  color: var(--fg-title, var(--fg));

/* After — fg-title is now defined in colors.css */
.sidebar-brand {
  color: var(--fg-title);
.nav-item.active {
  color: var(--fg-title);
.topbar-title {
  color: var(--fg-title);
```

- [ ] **Step 3: Fix warm theme window-frame hardcoded values**

In `theme.css`, the warm theme window-frame (line 310-322) uses hardcoded `rgba(255,255,255,0.12)` instead of `var(--glass-window)`. Update:

```css
/* Before (line 313) */
[data-theme="warm"] .window-frame {
  background: rgba(255, 255, 255, 0.12);
  ...
  border: 1px solid rgba(255, 255, 255, 0.18);

/* After */
[data-theme="warm"] .window-frame {
  background: var(--glass-window);
  ...
  border: 1px solid var(--border-window);
```

- [ ] **Step 4: Verify build compiles**

Run: `npm run build`

- [ ] **Step 5: Commit**

```bash
git add src/components/layout/Topbar.vue src/components/layout/Sidebar.vue src/assets/theme.css
git commit -m "fix(tokens): remove component fallback values, use token definitions directly"
```

---

### Task 3: Replace Hardcoded rgba with Glass Tokens in Components

**Files:**
- Modify: `src/components/layout/AppFrame.vue:136`
- Modify: `src/components/layout/Sidebar.vue:319,324`
- Modify: `src/components/layout/Topbar.vue:110`
- Modify: `src/components/common/StatCard.vue:36-39,51-53`
- Modify: `src/components/common/Toast.vue:105,108,149,159`
- Modify: `src/components/common/Modal.vue:121,124,162,172,197-198,209-210`
- Modify: `src/views/SettingsView.vue:241,337`

- [ ] **Step 1: Fix AppFrame.vue**

```css
/* Before (line 136) */
.window-frame {
  background: rgba(255, 255, 255, 0.32);

/* After */
.window-frame {
  background: var(--glass-window);
```

- [ ] **Step 2: Fix Sidebar.vue nav hover/active**

```css
/* Before (lines 319, 324) */
.nav-item:hover {
  background: rgba(255, 255, 255, 0.30);
.nav-item.active {
  background: rgba(255, 255, 255, 0.30);

/* After */
.nav-item:hover {
  background: var(--glass-bg-hover);
.nav-item.active {
  background: var(--glass-bg-hover);
```

- [ ] **Step 3: Fix Topbar.vue secondary button hover**

```css
/* Before (line 110) */
.topbar-btn.secondary:hover {
  background: rgba(255, 255, 255, 0.20);

/* After */
.topbar-btn.secondary:hover {
  background: var(--glass-bg-hover);
```

- [ ] **Step 4: Fix StatCard.vue hardcoded values**

```css
/* Before (line 36) */
.stat-card {
  background: rgba(255, 255, 255, 0.58);
  ...
  border: 1px solid rgba(255, 255, 255, 0.45);

/* After */
.stat-card {
  background: var(--glass-bg-hover);
  ...
  border: 1px solid var(--border-hover);
```

```css
/* Before (line 52) */
.stat-card:hover {
  background: rgba(255, 255, 255, 0.72);
  border-color: rgba(255, 255, 255, 0.65);

/* After */
.stat-card:hover {
  background: var(--glass-highlight);
  border-color: var(--border-hover);
```

- [ ] **Step 5: Fix Toast.vue hardcoded values**

```css
/* Before (line 105) */
.toast {
  background: rgba(255, 255, 255, 0.42);
  ...
  border: 1px solid rgba(255, 255, 255, 0.30);

/* After */
.toast {
  background: var(--glass-bg);
  ...
  border: 1px solid var(--border);
```

```css
/* Before (line 149) */
.toast-dismiss {
  background: rgba(255, 255, 255, 0.20);

/* After */
.toast-dismiss {
  background: var(--glass-input);
```

```css
/* Before (line 159) */
.toast-dismiss:hover {
  background: rgba(255, 255, 255, 0.35);

/* After */
.toast-dismiss:hover {
  background: var(--glass-bg-hover);
```

- [ ] **Step 6: Fix Modal.vue hardcoded values**

```css
/* Before (line 121) */
.modal {
  background: rgba(255, 255, 255, 0.48);
  ...
  border: 1px solid rgba(255, 255, 255, 0.35);

/* After */
.modal {
  background: var(--glass-bg);
  ...
  border: 1px solid var(--border);
```

```css
/* Before (line 162) */
.modal-close {
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid rgba(255, 255, 255, 0.32);

/* After */
.modal-close {
  background: var(--glass-input);
  border: 1px solid var(--border);
```

```css
/* Before (line 172) */
.modal-close:hover {
  background: rgba(255, 255, 255, 0.40);

/* After */
.modal-close:hover {
  background: var(--glass-bg-hover);
```

```css
/* Before (line 197) */
.modal-body input, ... {
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);

/* After */
.modal-body input, ... {
  background: var(--glass-input);
  border: 1px solid var(--border);
```

```css
/* Before (line 209) */
.modal-body input:focus, ... {
  border-color: rgba(255, 255, 255, 0.40);
  background: rgba(255, 255, 255, 0.40);

/* After */
.modal-body input:focus, ... {
  border-color: var(--border-hover);
  background: var(--glass-input-focus);
```

- [ ] **Step 7: Fix SettingsView.vue hardcoded values**

```css
/* Before (line 241) */
.setting-row {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);

/* After */
.setting-row {
  border-bottom: 1px solid var(--border);
```

```css
/* Before (line 337) */
.about-row {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);

/* After */
.about-row {
  border-bottom: 1px solid var(--border);
```

- [ ] **Step 8: Verify build compiles**

Run: `npm run build`

- [ ] **Step 9: Commit**

```bash
git add src/components/layout/AppFrame.vue src/components/layout/Sidebar.vue src/components/layout/Topbar.vue src/components/common/StatCard.vue src/components/common/Toast.vue src/components/common/Modal.vue src/views/SettingsView.vue
git commit -m "refactor(glass): replace hardcoded rgba with glass tokens in layout and common components"
```

---

### Task 4: Glass Effect Layering — Sidebar and Topbar

**Files:**
- Modify: `src/components/layout/Sidebar.vue:235-242`
- Modify: `src/components/layout/Topbar.vue:42-50`
- Modify: `src/assets/theme.css` (remove warm theme sidebar/topbar overrides that conflict)

The warm theme in `theme.css` already applies glass to `.sidebar` and `.topbar` (lines 342-358) via `[data-theme="warm"]` selectors. But the scoped styles in the components don't define these properties, relying on the global theme CSS. This is fragile — the components should own their glass styles, with the theme providing token values.

- [ ] **Step 1: Add glass properties to Sidebar.vue scoped styles**

```css
/* Add to .sidebar rule (after line 242) */
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--glass-sidebar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid var(--border-window);
}
```

- [ ] **Step 2: Add glass properties to Topbar.vue scoped styles**

```css
/* Update .topbar rule (line 43) */
.topbar {
  height: var(--topbar-h);
  min-height: var(--topbar-h);
  display: flex;
  align-items: center;
  padding: 0 24px;
  gap: 16px;
  background: var(--glass-topbar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-bottom: 1px solid var(--border-window);
}
```

- [ ] **Step 3: Remove redundant warm theme overrides from theme.css**

In `theme.css`, remove lines 342-358 (the `[data-theme="warm"] .sidebar` and `[data-theme="warm"] .topbar` blocks) since the components now own their glass styles.

- [ ] **Step 4: Verify build compiles**

Run: `npm run build`

- [ ] **Step 5: Verify visual appearance**

The sidebar and topbar should now have their own glass surfaces, visually distinct from the window frame background.

- [ ] **Step 6: Commit**

```bash
git add src/components/layout/Sidebar.vue src/components/layout/Topbar.vue src/assets/theme.css
git commit -m "feat(glass): add independent glass layering to sidebar and topbar"
```

---

### Task 5: Midnight Theme Glass Verification

**Files:**
- Modify: `src/assets/theme.css` (clean up midnight overrides if needed)

After Tasks 3-4, all glass values come from tokens. The midnight theme in `colors.css` and `glass.css` already defines dark glass values. This task verifies no hardcoded light-mode values remain.

- [ ] **Step 1: Grep for remaining hardcoded rgba(255,255,255) in Vue files**

Run:
```bash
grep -rn "rgba(255, 255, 255\|rgba(255,255,255" /Users/rhino/Desktop/AI/env-manager/src/ --include="*.vue" --include="*.css" | grep -v "node_modules" | grep -v ".css.map"
```

Expected: Only `theme.css` warm-theme-specific overrides should remain (body background, window-frame box-shadow). All component `.vue` files should be clean.

- [ ] **Step 2: Fix any remaining hardcoded values found**

If any `.vue` file still has `rgba(255,255,255,...)`, replace with the appropriate `--glass-*` token.

- [ ] **Step 3: Verify build compiles**

Run: `npm run build`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "fix(glass): ensure all glass values use tokens for midnight theme compatibility"
```

---

### Task 6: Background Texture for Midnight Theme

**Files:**
- Modify: `design/tokens/themes/midnight.css:70-104`

The warm theme already has the full 8-layer background texture. The midnight theme has a simpler version — it's missing the grid layers (fine 24px + coarse 96px).

- [ ] **Step 1: Add grid layers to midnight body background**

In `design/tokens/themes/midnight.css`, update the body background:

```css
/* Before (line 76-81) */
:root[data-theme="midnight"] body,
:root[data-theme="midnight"] html {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #1A1D24 0%, #1E2228 50%, #1A1D24 100%),
    radial-gradient(ellipse 100% 80% at 0% 0%, rgba(100, 120, 140, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(90, 100, 120, 0.10) 0%, transparent 50%);
  background-size: 100% 100%, 100% 100%, 100% 100%;
  background-attachment: fixed;
}

/* After */
:root[data-theme="midnight"] body,
:root[data-theme="midnight"] html {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #1A1D24 0%, #1E2228 50%, #1A1D24 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(100, 120, 140, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(90, 100, 120, 0.10) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(80, 90, 110, 0.08) 0%, transparent 45%),
    linear-gradient(rgba(255, 255, 255, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.015) 1px, transparent 1px),
    linear-gradient(rgba(255, 255, 255, 0.008) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.008) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
  background-attachment: fixed;
}
```

- [ ] **Step 2: Add third radial glow to midnight `body::before`**

```css
/* Before (line 84-93) */
:root[data-theme="midnight"] body::before {
  ...
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(90, 154, 122, 0.08) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(90, 100, 120, 0.06) 0%, transparent 50%);
}

/* After */
:root[data-theme="midnight"] body::before {
  ...
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(90, 154, 122, 0.08) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(90, 100, 120, 0.06) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(80, 90, 110, 0.04) 0%, transparent 45%);
}
```

- [ ] **Step 3: Verify build compiles**

Run: `npm run build`

- [ ] **Step 4: Commit**

```bash
git add design/tokens/themes/midnight.css
git commit -m "feat(theme): add grid texture layers to midnight background"
```

---

### Task 7: Theme Expansion — Sage, Lavender, Arctic

**Files:**
- Create: `design/tokens/themes/sage.css`
- Create: `design/tokens/themes/lavender.css`
- Create: `design/tokens/themes/arctic.css`
- Modify: `src/assets/theme.css` (add @imports)
- Modify: `design/tokens/index.ts:10` (extend ThemeId)
- Modify: `src/stores/theme.ts:96` (extend validation)
- Modify: `src/views/SettingsView.vue:93-106` (add theme cards)

- [ ] **Step 1: Create sage.css**

Create `design/tokens/themes/sage.css`:

```css
/**
 * Forge Design Tokens — Sage Theme
 * Muted green glass
 */

:root[data-theme="sage"] {
  /* Background */
  --bg: #EFF3EE;
  --bg-gradient: linear-gradient(160deg, #E8EDE6 0%, #EEF3EC 25%, #F2F5F0 50%, #EFF3EE 75%, #E6EBE4 100%);

  /* Border Colors */
  --border: rgba(255, 255, 255, 0.20);
  --border-hover: rgba(255, 255, 255, 0.35);
  --border-window: rgba(255, 255, 255, 0.18);
  --border-outer-glow: rgba(255, 255, 255, 0.40);

  /* Text Colors */
  --fg: #1A2A1A;
  --fg-title: #111A11;
  --fg-muted: #5C6C5C;
  --fg-ghost: #9AAA9A;
  --fg-white: #1A2A1A;

  /* Accent */
  --accent: #5A8A5A;
  --accent-hover: #4A7A4A;
  --accent-press: #3A6A3A;
  --accent-glow: rgba(90, 138, 90, 0.12);

  /* Semantic */
  --success: #5A8A64;
  --error: #B85A42;
  --info: #5A7A6B;
  --warn: #8AAA6A;

  /* Tint Atmosphere */
  --tint-warm: rgba(165, 185, 155, 0.12);
  --tint-cool: rgba(155, 175, 165, 0.10);
  --tint-soft: rgba(175, 195, 165, 0.14);
  --tint-amber: rgba(155, 175, 145, 0.08);

  /* Layout */
  --sidebar-w: 240px;
  --topbar-h: 64px;

  /* Border Radius */
  --radius: 18px;
  --radius-sm: 12px;
  --radius-lg: 24px;
  --radius-xl: 28px;

  /* Shadows */
  --shadow: 0 2px 16px rgba(0, 0, 0, 0.04);
  --shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.07);
  --shadow-btn: 0 1px 4px rgba(0, 0, 0, 0.06);
  --shadow-window: 0 16px 48px rgba(0, 0, 0, 0.10), 0 4px 16px rgba(0, 0, 0, 0.04);
  --shadow-inner: inset 0 1px 2px rgba(0, 0, 0, 0.03);

  /* Easing */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --t-fast: 150ms var(--ease);
  --t-base: 200ms var(--ease);
  --t-slow: 300ms var(--ease);
}

/* Sage Body Background */
:root[data-theme="sage"] body,
:root[data-theme="sage"] html {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #E8EDE6 0%, #EEF3EC 25%, #F2F5F0 50%, #EFF3EE 75%, #E6EBE4 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(145, 165, 135, 0.20) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(135, 155, 145, 0.18) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(155, 175, 145, 0.12) 0%, transparent 45%),
    linear-gradient(rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(rgba(0, 0, 0, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.015) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
  background-attachment: fixed;
}

:root[data-theme="sage"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(135, 155, 125, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(125, 145, 135, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(155, 175, 145, 0.08) 0%, transparent 45%);
}

:root[data-theme="sage"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.18;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.05'/%3E%3C/svg%3E");
  background-size: 200px 200px;
  will-change: auto;
}
```

- [ ] **Step 2: Create lavender.css**

Create `design/tokens/themes/lavender.css`:

```css
/**
 * Forge Design Tokens — Lavender Theme
 * Purple frosted glass
 */

:root[data-theme="lavender"] {
  /* Background */
  --bg: #F2EFF8;
  --bg-gradient: linear-gradient(160deg, #EFECF6 0%, #F1EEF7 25%, #F4F2F9 50%, #F2EFF8 75%, #EDEAF4 100%);

  /* Border Colors */
  --border: rgba(255, 255, 255, 0.20);
  --border-hover: rgba(255, 255, 255, 0.35);
  --border-window: rgba(255, 255, 255, 0.18);
  --border-outer-glow: rgba(255, 255, 255, 0.40);

  /* Text Colors */
  --fg: #2A2038;
  --fg-title: #1A1028;
  --fg-muted: #6C5C7C;
  --fg-ghost: #A898B8;
  --fg-white: #2A2038;

  /* Accent */
  --accent: #7A5AAA;
  --accent-hover: #6A4A9A;
  --accent-press: #5A3A8A;
  --accent-glow: rgba(122, 90, 170, 0.12);

  /* Semantic */
  --success: #6A8A6A;
  --error: #B85A70;
  --info: #7A6B8A;
  --warn: #A070C0;

  /* Tint Atmosphere */
  --tint-warm: rgba(185, 165, 205, 0.12);
  --tint-cool: rgba(175, 165, 195, 0.10);
  --tint-soft: rgba(195, 175, 215, 0.14);
  --tint-amber: rgba(175, 155, 195, 0.08);

  /* Layout */
  --sidebar-w: 240px;
  --topbar-h: 64px;

  /* Border Radius */
  --radius: 18px;
  --radius-sm: 12px;
  --radius-lg: 24px;
  --radius-xl: 28px;

  /* Shadows */
  --shadow: 0 2px 16px rgba(0, 0, 0, 0.04);
  --shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.07);
  --shadow-btn: 0 1px 4px rgba(0, 0, 0, 0.06);
  --shadow-window: 0 16px 48px rgba(0, 0, 0, 0.10), 0 4px 16px rgba(0, 0, 0, 0.04);
  --shadow-inner: inset 0 1px 2px rgba(0, 0, 0, 0.03);

  /* Easing */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --t-fast: 150ms var(--ease);
  --t-base: 200ms var(--ease);
  --t-slow: 300ms var(--ease);
}

/* Lavender Body Background */
:root[data-theme="lavender"] body,
:root[data-theme="lavender"] html {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #EFECF6 0%, #F1EEF7 25%, #F4F2F9 50%, #F2EFF8 75%, #EDEAF4 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(175, 155, 205, 0.20) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(155, 145, 185, 0.18) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(185, 175, 205, 0.12) 0%, transparent 45%),
    linear-gradient(rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(rgba(0, 0, 0, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.015) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
  background-attachment: fixed;
}

:root[data-theme="lavender"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(165, 145, 195, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(145, 135, 175, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(175, 165, 195, 0.08) 0%, transparent 45%);
}

:root[data-theme="lavender"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.18;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.05'/%3E%3C/svg%3E");
  background-size: 200px 200px;
  will-change: auto;
}
```

- [ ] **Step 3: Create arctic.css**

Create `design/tokens/themes/arctic.css`:

```css
/**
 * Forge Design Tokens — Arctic Theme
 * Cool blue-white glass
 */

:root[data-theme="arctic"] {
  /* Background */
  --bg: #F4F8FA;
  --bg-gradient: linear-gradient(160deg, #F0F5F8 0%, #F3F6F9 25%, #F6F8FA 50%, #F4F8FA 75%, #EEF2F5 100%);

  /* Border Colors */
  --border: rgba(255, 255, 255, 0.20);
  --border-hover: rgba(255, 255, 255, 0.35);
  --border-window: rgba(255, 255, 255, 0.18);
  --border-outer-glow: rgba(255, 255, 255, 0.40);

  /* Text Colors */
  --fg: #142430;
  --fg-title: #0A1A28;
  --fg-muted: #5A6B7A;
  --fg-ghost: #9AAABA;
  --fg-white: #142430;

  /* Accent */
  --accent: #3A7AAA;
  --accent-hover: #2A6A9A;
  --accent-press: #1A5A8A;
  --accent-glow: rgba(58, 122, 170, 0.12);

  /* Semantic */
  --success: #5A9A7A;
  --error: #C06050;
  --info: #5A9ACC;
  --warn: #5A9ACC;

  /* Tint Atmosphere */
  --tint-warm: rgba(155, 175, 205, 0.12);
  --tint-cool: rgba(145, 165, 195, 0.10);
  --tint-soft: rgba(165, 185, 215, 0.14);
  --tint-amber: rgba(145, 165, 185, 0.08);

  /* Layout */
  --sidebar-w: 240px;
  --topbar-h: 64px;

  /* Border Radius */
  --radius: 18px;
  --radius-sm: 12px;
  --radius-lg: 24px;
  --radius-xl: 28px;

  /* Shadows */
  --shadow: 0 2px 16px rgba(0, 0, 0, 0.04);
  --shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.07);
  --shadow-btn: 0 1px 4px rgba(0, 0, 0, 0.06);
  --shadow-window: 0 16px 48px rgba(0, 0, 0, 0.10), 0 4px 16px rgba(0, 0, 0, 0.04);
  --shadow-inner: inset 0 1px 2px rgba(0, 0, 0, 0.03);

  /* Easing */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --t-fast: 150ms var(--ease);
  --t-base: 200ms var(--ease);
  --t-slow: 300ms var(--ease);
}

/* Arctic Body Background */
:root[data-theme="arctic"] body,
:root[data-theme="arctic"] html {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #F0F5F8 0%, #F3F6F9 25%, #F6F8FA 50%, #F4F8FA 75%, #EEF2F5 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(135, 165, 205, 0.20) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(125, 145, 185, 0.18) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(145, 165, 195, 0.12) 0%, transparent 45%),
    linear-gradient(rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(rgba(0, 0, 0, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.015) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
  background-attachment: fixed;
}

:root[data-theme="arctic"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(125, 155, 195, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(115, 135, 175, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(135, 155, 185, 0.08) 0%, transparent 45%);
}

:root[data-theme="arctic"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.18;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.05'/%3E%3C/svg%3E");
  background-size: 200px 200px;
  will-change: auto;
}
```

- [ ] **Step 4: Add @imports to theme.css**

In `src/assets/theme.css`, add after the existing theme imports (line 26):

```css
@import '../../design/tokens/themes/sage.css';
@import '../../design/tokens/themes/lavender.css';
@import '../../design/tokens/themes/arctic.css';
```

- [ ] **Step 5: Extend ThemeId type**

In `design/tokens/index.ts`, change line 10:

```typescript
/* Before */
export type ThemeId = 'warm' | 'midnight'

/* After */
export type ThemeId = 'warm' | 'midnight' | 'sage' | 'lavender' | 'arctic'
```

- [ ] **Step 6: Update theme store validation**

In `src/stores/theme.ts`, change line 96:

```typescript
/* Before */
if (savedTheme && (savedTheme === 'warm' || savedTheme === 'midnight')) {

/* After */
if (savedTheme && (savedTheme === 'warm' || savedTheme === 'midnight' || savedTheme === 'sage' || savedTheme === 'lavender' || savedTheme === 'arctic')) {
```

Also update the default theme fallback (line 99) — keep it as `'warm'`, no change needed.

- [ ] **Step 7: Add theme cards to SettingsView.vue**

In `src/views/SettingsView.vue`, extend the `availableThemes` array (line 93):

```typescript
const availableThemes = [
  {
    id: 'warm' as ThemeId,
    name: 'Warm',
    previewBg: 'linear-gradient(135deg, #E8E4DE 0%, #F5F3EF 100%)',
    previewColors: ['#E8E4DE', '#F5F3EF', '#DDD8D0', '#C8C0B8'],
  },
  {
    id: 'midnight' as ThemeId,
    name: 'Midnight',
    previewBg: 'linear-gradient(135deg, #1A1D24 0%, #2D3340 100%)',
    previewColors: ['#1A1D24', '#2D3340', '#3A4250', '#4A5568'],
  },
  {
    id: 'sage' as ThemeId,
    name: 'Sage',
    previewBg: 'linear-gradient(135deg, #EFF3EE 0%, #D8E4D0 100%)',
    previewColors: ['#EFF3EE', '#D8E4D0', '#B8CCB4', '#5A8A5A'],
  },
  {
    id: 'lavender' as ThemeId,
    name: 'Lavender',
    previewBg: 'linear-gradient(135deg, #F2EFF8 0%, #E0DAF0 100%)',
    previewColors: ['#F2EFF8', '#E0DAF0', '#C4B8E0', '#7A5AAA'],
  },
  {
    id: 'arctic' as ThemeId,
    name: 'Arctic',
    previewBg: 'linear-gradient(135deg, #F4F8FA 0%, #C8DAE8 100%)',
    previewColors: ['#F4F8FA', '#C8DAE8', '#A8C4E0', '#3A7AAA'],
  },
]
```

- [ ] **Step 8: Update theme grid layout for 5 themes**

```css
/* Before */
.theme-grid {
  grid-template-columns: repeat(2, 1fr);
}

/* After */
.theme-grid {
  grid-template-columns: repeat(3, 1fr);
}

/* Responsive */
@media (max-width: 768px) {
  .theme-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
```

- [ ] **Step 9: Verify build compiles**

Run: `npm run build`

- [ ] **Step 10: Commit**

```bash
git add design/tokens/themes/sage.css design/tokens/themes/lavender.css design/tokens/themes/arctic.css src/assets/theme.css design/tokens/index.ts src/stores/theme.ts src/views/SettingsView.vue
git commit -m "feat(theme): add sage, lavender, arctic themes with full glass backgrounds"
```

---

### Task 8: Hover Animation Alignment

**Files:**
- Modify: `src/assets/theme.css` (warm card hover)
- Modify: `src/components/layout/Topbar.vue` (btn hover)

The spec requires card hover `translateY(-3px)` and button hover `translateY(-2px)`. Current values are `-1px`.

- [ ] **Step 1: Fix warm theme card hover**

In `src/assets/theme.css`, update the warm card hover (around line 335-340):

```css
/* Before */
[data-theme="warm"] .card:hover {
  ...
  transform: translateY(-1px);

/* After */
[data-theme="warm"] .card:hover {
  ...
  transform: translateY(-3px);
```

- [ ] **Step 2: Fix Topbar primary button hover**

In `src/components/layout/Topbar.vue`, update:

```css
/* Before */
.topbar-btn.primary:hover {
  transform: translateY(-1px);

/* After */
.topbar-btn.primary:hover {
  transform: translateY(-2px);
```

- [ ] **Step 3: Verify build compiles**

Run: `npm run build`

- [ ] **Step 4: Commit**

```bash
git add src/assets/theme.css src/components/layout/Topbar.vue
git commit -m "fix(anim): align card and button hover distances to spec (-3px / -2px)"
```

---

## Verification Checklist

After all tasks complete, verify:

1. `npm run build` succeeds
2. Warm theme renders correctly with glass layering
3. Midnight theme renders correctly with dark glass
4. Sage/Lavender/Arctic themes switch correctly from Settings
5. Sidebar and topbar have independent glass surfaces
6. Stat cards show tint-drift and tint-sweep animations
7. Card hover lifts work with shadow transitions
8. No `rgba(255,255,255,...)` remains in any `.vue` file (only in `theme.css` warm-specific overrides)
9. Background texture (grids + noise) visible in all themes

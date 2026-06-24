# Responsive Width Adaptation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify all responsive breakpoints and layout widths across 11 views and 5 layout components into a single token-driven system with four breakpoints (768px, 1024px, 1280px, 1441px).

**Architecture:** New `tokens/responsive.css` defines layout CSS custom properties per breakpoint. All views and components reference these tokens instead of hardcoding pixel values. Global grid utilities and filter-bar rules in `theme.css` eliminate duplicate `@media` blocks across views.

**Tech Stack:** Vue 3 SFC scoped styles, CSS custom properties, Vite

---

## File Structure

| Action | File | Responsibility |
|--------|------|---------------|
| Create | `src/assets/tokens/responsive.css` | Breakpoint token definitions |
| Modify | `src/assets/main.css` | Import responsive.css |
| Modify | `src/assets/theme.css` | Global grid utilities, responsive block overhaul, filter-bar consolidation, remove max-width cap |
| Modify | `src/components/layout/AppFrame.vue` | Content padding → token references |
| Modify | `src/components/layout/Sidebar.vue` | Width → token reference, remove duplicate responsive rule |
| Modify | `src/views/DashboardView.vue` | Migrate 3 @media blocks to token system |
| Modify | `src/views/CliToolsView.vue` | Add responsive breakpoints (currently none) |
| Modify | `src/views/SoftwareManagementView.vue` | Migrate 4 @media blocks, unify breakpoints |
| Modify | `src/views/PluginsView.vue` | Migrate 1 @media block, consolidate filter-bar |
| Modify | `src/views/SkillsView.vue` | Migrate 3 @media blocks to token system |
| Modify | `src/views/AgentsView.vue` | Migrate 3 @media blocks to token system |
| Modify | `src/views/MCPView.vue` | Migrate 1 @media block, consolidate filter-bar |
| Modify | `src/views/RulesView.vue` | Migrate 1 @media block, consolidate filter-bar |
| Modify | `src/views/BackupView.vue` | Migrate 2 @media blocks, remove max-width cap |
| Modify | `src/views/SettingsView.vue` | Migrate 1 @media block, use auto-grid |
| Modify | `src/views/PromptManagerView.vue` | Add responsive breakpoints (currently none) |

---

## Task 1: Create responsive.css token file

**Files:**
- Create: `src/assets/tokens/responsive.css`

- [ ] **Step 1: Create the responsive token file**

Create `src/assets/tokens/responsive.css` with the following content:

```css
/* ============================================================
   RESPONSIVE TOKENS
   Unified breakpoint system for desktop window scaling.
   All layout dimensions reference these variables.
   ============================================================ */

/* === Default values (large screen ≥1441px) === */
:root {
  --content-padding-x: 32px;
  --content-padding-y: 28px;
  --content-padding-bottom: 72px;
  --sidebar-width: 240px;
  --grid-gap: 16px;
  --stats-columns: 4;
  --card-min-width: 300px;
}

/* === Desktop 1025px – 1440px === */
@media (max-width: 1440px) {
  :root {
    --content-padding-x: 28px;
    --content-padding-y: 24px;
  }
}

/* === Tablet 769px – 1024px === */
@media (max-width: 1024px) {
  :root {
    --content-padding-x: 20px;
    --content-padding-y: 20px;
    --sidebar-width: 200px;
    --grid-gap: 12px;
    --stats-columns: 2;
    --card-min-width: 260px;
  }
}

/* === Small window ≤768px === */
@media (max-width: 768px) {
  :root {
    --content-padding-x: 16px;
    --content-padding-y: 16px;
    --sidebar-width: 0px;
    --grid-gap: 10px;
    --stats-columns: 2;
    --card-min-width: 100%;
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/assets/tokens/responsive.css
git commit -m "feat(tokens): add responsive breakpoint token definitions"
```

---

## Task 2: Import responsive.css in main.css

**Files:**
- Modify: `src/assets/main.css:5` (after existing `@import './theme.css'`)

- [ ] **Step 1: Add import**

In `src/assets/main.css`, add the import **before** the theme.css import so tokens are available when theme.css processes:

Replace line 4 (`@import './theme.css';`) with:

```css
@import './tokens/responsive.css';
@import './theme.css';
```

- [ ] **Step 2: Commit**

```bash
git add src/assets/main.css
git commit -m "feat: import responsive tokens before theme.css"
```

---

## Task 3: Update theme.css — global grid utilities and responsive overhaul

**Files:**
- Modify: `src/assets/theme.css`

This task has several sub-steps across different sections of theme.css.

- [ ] **Step 3a: Update responsive breakpoint block (lines 1944-2071)**

Replace the entire responsive adjustments block (lines 1944-2071) with token-driven rules. The new block removes hardcoded padding/width values and uses token variables. It also removes the `max-width: 1400px` cap on `.content`.

Replace lines 1944-2071 with:

```css
/* ============================================================
   RESPONSIVE ADJUSTMENTS — token-driven
   Breakpoint values and layout variables are defined in
   tokens/responsive.css. These rules reference them.
   ============================================================ */

/* === Tablet ≤1024px === */
@media (max-width: 1024px) {
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
  }
  .content {
    padding: var(--content-padding-y) var(--content-padding-x);
  }
  .stats-row {
    grid-template-columns: repeat(var(--stats-columns), 1fr);
  }
  .settings-grid,
  .backup-grid {
    grid-template-columns: 1fr;
  }
}

/* === Small window ≤768px === */
@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
  .main {
    width: 100%;
  }
  .topbar {
    padding: 0 var(--content-padding-x);
  }
  .content {
    padding: var(--content-padding-y) var(--content-padding-x);
  }
  .stats-row {
    grid-template-columns: repeat(var(--stats-columns), 1fr);
  }
  .settings-grid,
  .backup-grid {
    grid-template-columns: 1fr;
  }
  .tool-card {
    grid-template-columns: 1fr;
  }
  .tool-card-right {
    flex-direction: row;
    justify-content: flex-start;
  }
}
```

Note: The `@media (max-width: 640px)` block (lines 2011-2056), the `@media (min-width: 1024px) and (max-width: 1280px)` block (lines 2059-2063), and the `@media (min-width: 1280px)` block (lines 2066-2071) are **removed**. The 640px block is not part of the unified four-breakpoint system. The 1024-1280px stats-row rule is now handled by token-driven `--stats-columns`. The 1280px max-width cap is removed per design decision (no content width limit).

- [ ] **Step 3b: Add global grid utility classes**

Add the following after the `.content` base rule (after line 743 area, or wherever fits logically in the non-responsive section):

```css
/* === Responsive grid utilities === */
.stats-row {
  display: grid;
  grid-template-columns: repeat(var(--stats-columns), 1fr);
  gap: var(--grid-gap);
}

.auto-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
}
```

Also add horizontal overflow prevention to the existing `.shell` rule:

```css
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
  overflow-x: hidden;
}
```

And ensure grid containers don't overflow:

```css
.stats-row,
.auto-grid,
.tool-grid,
.marketplace-grid,
.sources-grid,
.agents-grid {
  min-width: 0;
}
```

- [ ] **Step 3c: Update base `.content` rule**

Find the base `.content` rule (around line 735) and update it to use tokens:

Replace:
```css
.content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px;
  padding-bottom: 48px;
}
```

With:
```css
.content {
  flex: 1;
  overflow-y: auto;
  padding: var(--content-padding-y) var(--content-padding-x);
  padding-bottom: var(--content-padding-bottom);
}
```

- [ ] **Step 3d: Update base `.sidebar` rule**

Find the base `.sidebar` rule (around line 726) and ensure it uses `var(--sidebar-width)`:

```css
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  /* ... rest of existing styles unchanged ... */
}
```

- [ ] **Step 3e: Consolidate filter-bar responsive rules**

The filter-bar `@media (max-width: 768px)` block (lines 2636-2663) is already in theme.css. Add a tablet breakpoint rule before it. Replace lines 2636-2663 with:

```css
/* Filter bar — tablet */
@media (max-width: 1024px) {
  .filter-bar {
    gap: 8px 10px;
  }
  .filter-bar .search-input {
    flex: 1 1 160px;
  }
  .filter-bar .filter-select {
    max-width: 140px;
    font-size: 12px;
    padding: 7px 28px 7px 10px;
  }
  .filter-bar .btn {
    padding: 7px 12px;
    font-size: 11px;
  }
}

/* Filter bar — small window */
@media (max-width: 768px) {
  .filter-bar {
    gap: 8px;
    flex-direction: column;
    align-items: stretch;
  }
  .filter-bar .search-input {
    flex: 1 1 auto;
    min-width: 0;
  }
  .filter-bar .filter-select {
    max-width: none;
    width: 100%;
  }
  .filter-bar .btn-group {
    margin-left: 0;
    justify-content: flex-end;
    width: 100%;
  }
  .filter-bar > button:not(.filter-select) {
    width: 100%;
    justify-content: center;
  }
  .section-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
}
```

- [ ] **Step 3f: Commit**

```bash
git add src/assets/theme.css
git commit -m "refactor(theme): token-driven responsive rules, grid utilities, filter-bar consolidation"
```

---

## Task 4: Refactor AppFrame.vue

**Files:**
- Modify: `src/components/layout/AppFrame.vue`

- [ ] **Step 4a: Replace content padding with token references**

In the scoped `<style>` section, find the `.content` rule (around line 160) and replace:

```css
/* BEFORE */
.content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px;
  padding-bottom: 72px;
}
```

With:

```css
/* AFTER */
.content {
  flex: 1;
  overflow-y: auto;
  padding: var(--content-padding-y) var(--content-padding-x);
  padding-bottom: var(--content-padding-bottom);
}
```

- [ ] **Step 4b: Remove all @media rules from AppFrame.vue**

Delete the tablet `@media (max-width: 1024px)` block (around lines 189-194) and the mobile `@media (max-width: 768px)` block (around lines 197-202) entirely. These are now handled by the token variables changing at each breakpoint.

The file should have **zero** `@media` rules after this step.

- [ ] **Step 4c: Commit**

```bash
git add src/components/layout/AppFrame.vue
git commit -m "refactor(AppFrame): content padding driven by responsive tokens"
```

---

## Task 5: Refactor Sidebar.vue

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 5a: Verify width uses token**

The sidebar already uses `var(--sidebar-w)` at line 220. Update it to use the new token name `var(--sidebar-width)`:

Replace:
```css
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
```

With:
```css
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
```

- [ ] **Step 5b: Keep the @media rule**

The `@media (max-width: 768px) { .sidebar { display: none; } }` rule (lines 382-387) stays. This is the scoped override that hides the sidebar on small windows. The token `--sidebar-width: 0px` collapses the space, and `display: none` removes it from the DOM.

- [ ] **Step 5c: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "refactor(Sidebar): use --sidebar-width token"
```

---

## Task 6: Migrate DashboardView.vue

**Files:**
- Modify: `src/views/DashboardView.vue`

- [ ] **Step 6a: Replace stats-row grid with token-driven rule**

Find `.stats-row` base rule (line 232) and replace:
```css
.stats-row {
  grid-template-columns: repeat(4, 1fr);
  gap: 18px;
  margin-bottom: 32px;
}
```

With:
```css
.stats-row {
  grid-template-columns: repeat(var(--stats-columns), 1fr);
  gap: var(--grid-gap);
  margin-bottom: 32px;
}
```

- [ ] **Step 6b: Replace quick-actions-grid with auto-grid**

Find `.quick-actions-grid` base rule (line 267) and replace:
```css
.quick-actions-grid {
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}
```

With:
```css
.quick-actions-grid {
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
  margin-bottom: 32px;
}
```

- [ ] **Step 6c: Delete all 3 @media blocks**

Delete lines 385-421 entirely (the tablet 1024px, mobile 768px, and narrow 480px blocks). The token system now handles all breakpoint transitions.

- [ ] **Step 6d: Commit**

```bash
git add src/views/DashboardView.vue
git commit -m "refactor(DashboardView): migrate responsive rules to token system"
```

---

## Task 7: Migrate CliToolsView.vue

**Files:**
- Modify: `src/views/CliToolsView.vue`

- [ ] **Step 7a: Add responsive @media blocks**

This view currently has **zero** `@media` rules. Add the following at the end of the `<style scoped>` section, before the closing `</style>` tag:

```css
/* === Responsive: token-driven === */
@media (max-width: 1024px) {
  .cli-tool-item {
    gap: 12px;
  }
}

@media (max-width: 768px) {
  .cli-tool-item {
    grid-template-columns: 1fr;
  }
  .cli-tool-item-right {
    flex-direction: row;
    justify-content: flex-start;
  }
}
```

- [ ] **Step 7b: Remove duplicate filter-bar rules**

If the file has its own `.filter-bar` responsive rules (check if any `@media` rules for `.filter-bar` exist in the scoped style), remove them since they are now global in theme.css. The base `.filter-bar` styles (non-media) can stay if they have view-specific overrides.

- [ ] **Step 7c: Commit**

```bash
git add src/views/CliToolsView.vue
git commit -m "refactor(CliToolsView): add responsive breakpoints"
```

---

## Task 8: Migrate SoftwareManagementView.vue

**Files:**
- Modify: `src/views/SoftwareManagementView.vue`

- [ ] **Step 8a: Replace software-grid base rule**

Find `.software-grid` (line 342) and replace:
```css
.software-grid {
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
```

With:
```css
.software-grid {
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
}
```

- [ ] **Step 8b: Delete all 4 @media blocks**

Delete lines 346-368 entirely (the 1280px, 1023px, 767px, and 479px blocks). The `auto-fill` with `minmax(var(--card-min-width), 1fr)` naturally reflows through all breakpoints without media queries.

- [ ] **Step 8c: Commit**

```bash
git add src/views/SoftwareManagementView.vue
git commit -m "refactor(SoftwareManagementView): unify responsive to auto-fill grid"
```

---

## Task 9: Migrate PluginsView.vue

**Files:**
- Modify: `src/views/PluginsView.vue`

- [ ] **Step 9a: Update installed-grid and marketplace-grid**

Find `.installed-grid` (line 865) and replace:
```css
.installed-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}
```

With:
```css
.installed-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
}
```

Find `.marketplace-grid` (line 878) and replace:
```css
.marketplace-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}
```

With:
```css
.marketplace-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: var(--grid-gap);
}
```

Note: marketplace-grid keeps `340px` as its min-width since it's wider than the default `--card-min-width`. This is intentional — marketplace cards need more space.

- [ ] **Step 9b: Delete the @media block**

Delete lines 1036-1067 entirely. The filter-bar rules are now global in theme.css. The `marketplace-grid: grid-template-columns: 1fr` override at 768px is no longer needed since `auto-fill` with `minmax(340px, 1fr)` will naturally become `1fr` when the container is narrower than 340px.

- [ ] **Step 9c: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "refactor(PluginsView): migrate to token-driven grid, remove @media block"
```

---

## Task 10: Migrate SkillsView.vue

**Files:**
- Modify: `src/views/SkillsView.vue`

- [ ] **Step 10a: Update skills-grid base rule**

Find `.skills-grid` (line 397) and replace:
```css
.skills-grid {
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}
```

With:
```css
.skills-grid {
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
}
```

- [ ] **Step 10b: Delete all 3 @media blocks**

Delete lines 581-611 entirely (the 1024px, 768px, and 480px blocks). The filter-bar rules (`gap: 8px`, `.btn-group` overrides) are now global in theme.css. The `auto-fill` grid handles reflow naturally.

- [ ] **Step 10c: Commit**

```bash
git add src/views/SkillsView.vue
git commit -m "refactor(SkillsView): migrate responsive to token system"
```

---

## Task 11: Migrate AgentsView.vue

**Files:**
- Modify: `src/views/AgentsView.vue`

- [ ] **Step 11a: Update agents-grid base rule**

Find `.agents-grid` (line 301) and replace:
```css
.agents-grid {
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 16px;
}
```

With:
```css
.agents-grid {
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: var(--grid-gap);
}
```

Note: agents-grid keeps `380px` as its min-width — agent cards are wider than the default `--card-min-width`.

- [ ] **Step 11b: Delete all 3 @media blocks**

Delete lines 335-372 entirely (the 1024px, 768px, and 480px blocks). The `.section-header` stacking, `.filter-bar` rules, and `.btn-group` overrides are now global in theme.css.

- [ ] **Step 11c: Commit**

```bash
git add src/views/AgentsView.vue
git commit -m "refactor(AgentsView): migrate responsive to token system"
```

---

## Task 12: Migrate MCPView.vue

**Files:**
- Modify: `src/views/MCPView.vue`

- [ ] **Step 12a: Update tools-grid base rule**

Find `.tools-grid` (line 557) and replace:
```css
.tools-grid {
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}
```

With:
```css
.tools-grid {
  grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width), 1fr));
  gap: var(--grid-gap);
}
```

- [ ] **Step 12b: Delete the @media block**

Delete lines 650-681 entirely. The filter-bar stacking, `.tools-grid: 1fr`, and `.search-box` overrides are now handled globally. The `mcp-server-item` grid (`1fr auto auto`) becomes `1fr` naturally at narrow widths since the auto columns collapse.

- [ ] **Step 12c: Commit**

```bash
git add src/views/MCPView.vue
git commit -m "refactor(MCPView): migrate responsive to token system"
```

---

## Task 13: Migrate RulesView.vue

**Files:**
- Modify: `src/views/RulesView.vue`

- [ ] **Step 13a: Delete the @media block**

Delete lines 586-628 entirely. The `.view` padding is now handled by the global `.content` token. The `.filter-bar` stacking, `.section-header` stacking, and `.rule-item` reflow rules need to be preserved as scoped overrides since they are view-specific.

After deleting the old block, add back only the view-specific rules:

```css
@media (max-width: 768px) {
  .rule-item {
    flex-wrap: wrap;
  }
  .rule-info {
    order: 2;
    flex-basis: 100%;
    margin-top: 8px;
  }
  .rule-actions {
    order: 3;
    width: 100%;
    justify-content: flex-end;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.15);
  }
}
```

- [ ] **Step 13b: Commit**

```bash
git add src/views/RulesView.vue
git commit -m "refactor(RulesView): migrate responsive to token system"
```

---

## Task 14: Migrate BackupView.vue

**Files:**
- Modify: `src/views/BackupView.vue`

- [ ] **Step 14a: Remove max-width constraint**

Find `.backup-content` or the container with `max-width: 1200px` (line 333) and remove the `max-width: 1200px` property. Content should fill available space.

- [ ] **Step 14b: Update stats-row**

Find `.stats-row` (line 360) and replace:
```css
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}
```

With:
```css
.stats-row {
  display: grid;
  grid-template-columns: repeat(var(--stats-columns), 1fr);
  gap: var(--grid-gap);
  margin-bottom: 24px;
}
```

- [ ] **Step 14c: Delete the first @media block**

Delete lines 365-369 (the stats-row @media 768px block). Token-driven `--stats-columns` handles this.

- [ ] **Step 14d: Delete the second @media block and replace with view-specific rules only**

Delete lines 539-562. Add back only the view-specific backup-card rules:

```css
@media (max-width: 768px) {
  .backup-card {
    flex-direction: column;
    align-items: flex-start;
  }
  .backup-actions {
    width: 100%;
    margin-top: var(--spacing-sm);
  }
}
```

The `.filter-bar` and `.filter-select` rules are now global.

- [ ] **Step 14e: Commit**

```bash
git add src/views/BackupView.vue
git commit -m "refactor(BackupView): remove max-width cap, migrate responsive to tokens"
```

---

## Task 15: Migrate SettingsView.vue

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 15a: Update settings-grid**

Find `.settings-grid` base rule (it may be in the parent or in theme.css). If defined in this file, replace the fixed `1fr 1fr` with auto-fill:

The settings-grid class is defined globally in theme.css (already handled in Task 3). If the file has its own definition, replace with:

```css
.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--grid-gap);
}
```

- [ ] **Step 15b: Update theme-grid**

Find `.theme-grid` (line 188) and replace:
```css
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
```

With:
```css
.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 10px;
}
```

- [ ] **Step 15c: Delete the @media block**

Delete lines 378-382 entirely. The `auto-fill` grid handles reflow naturally.

- [ ] **Step 15d: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "refactor(SettingsView): migrate responsive to auto-fill grid"
```

---

## Task 16: Migrate PromptManagerView.vue

**Files:**
- Modify: `src/views/PromptManagerView.vue`

- [ ] **Step 16a: Verify no action needed**

This view has no `@media` queries and no grid layouts. Its `max-width: 480px` constraint is a content width limit (for a form), not a layout constraint — it stays as-is.

Verify the view renders correctly at all breakpoints by checking that:
- No horizontal overflow at 900px width
- Content area uses the global `.content` padding from tokens
- The `max-width: 480px` form doesn't cause overflow

If the view has a `.view` scoped padding override (like `padding: 24px`), remove it so it inherits from the global `.content` token.

- [ ] **Step 16b: Commit (if changes made)**

```bash
git add src/views/PromptManagerView.vue
git commit -m "refactor(PromptManagerView): align padding with token system"
```

---

## Task 17: Verification pass

- [ ] **Step 17a: Build check**

```bash
npm run build
```

Expected: Clean build with no errors.

- [ ] **Step 17b: Manual verification at each breakpoint**

Resize the Tauri window to each target width and verify:

| Width | Expected behavior |
|-------|------------------|
| 900px | Sidebar 200px, stats 2-col, content padding 20px |
| 1024px | Sidebar 200px, stats 2-col, content padding 20px |
| 1025px | Sidebar 240px, stats 4-col, content padding 28px |
| 1280px | Sidebar 240px, stats 4-col, content padding 28px |
| 1440px | Sidebar 240px, stats 4-col, content padding 28px |
| 1441px | Sidebar 240px, stats 4-col, content padding 32px |
| 1920px | Same as 1441px (no max-width cap) |

- [ ] **Step 17c: Check for horizontal scrollbar**

At each breakpoint, verify no horizontal scrollbar appears. Check the browser devtools for any `overflow-x` issues.

- [ ] **Step 17d: Verify each view page**

Navigate to every view (Dashboard, CLI Tools, Software, Plugins, Skills, Agents, MCP, Rules, Backup, Settings, Prompts) at 900px and 1440px and verify:
- No content overflow
- Grids reflow correctly
- Filter bars stack at ≤768px
- Sidebar hides at ≤768px

- [ ] **Step 17e: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "fix: responsive verification fixes"
```

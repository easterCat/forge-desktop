# Responsive Width Adaptation Design

**Date:** 2026-06-18
**Status:** Approved
**Scope:** All 11 views + 5 layout components

## Problem

The Forge desktop app (Tauri 2.x, min window 900x600) has inconsistent responsive behavior across pages. Breakpoint values vary (767 vs 768, 1023 vs 1024, 1280 vs 1440), padding and grid rules are hardcoded per-view, and there is no unified breakpoint system. When users resize the window, some pages reflow gracefully while others overflow or compress awkwardly.

## Goal

Unified responsive width adaptation across all pages, driven by CSS custom properties (tokens), covering four breakpoints for desktop window scaling.

## Constraints

- Tauri desktop app, minimum window 900x600
- No true mobile adaptation needed; breakpoints serve desktop window scaling
- Existing glassmorphism token system (6 layers) must be preserved
- No global max-width cap on content area — content fills available space

## Approach

**Token-driven responsive system** — new `tokens/responsive.css` defines breakpoint values and layout variables. All pages and components reference these variables instead of hardcoding pixel values. Existing per-view `@media` rules are migrated to the unified system.

---

## 1. Breakpoint Token Definitions

New file: `src/assets/tokens/responsive.css`

Four breakpoints for desktop window scaling:

| Tier | Range | Label |
|------|-------|-------|
| Mobile (small window) | ≤768px | `--bp-mobile` |
| Tablet | 769px – 1024px | `--bp-tablet` |
| Desktop | 1025px – 1440px | `--bp-desktop` |
| Large screen | ≥1441px | `--bp-large` |

Layout variables with per-breakpoint values:

```css
:root {
  /* Default values (large screen ≥1441px) */
  --content-padding-x: 32px;
  --content-padding-y: 28px;
  --sidebar-width: 240px;
  --grid-gap: 16px;
  --stats-columns: 4;
  --card-min-width: 300px;
}

@media (max-width: 1440px) {
  :root {
    --content-padding-x: 28px;
    --content-padding-y: 24px;
    --stats-columns: 4;
  }
}

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

Import in `src/assets/main.css`:
```css
@import './tokens/responsive.css';
```

## 2. Layout Component Adaptation

### AppFrame.vue

Replace hardcoded padding with token references. Remove all per-breakpoint padding `@media` rules from scoped style.

```css
.content {
  flex: 1;
  overflow-y: auto;
  padding: var(--content-padding-y) var(--content-padding-x);
  /* Mobile tabbar space: add 56px to bottom padding when sidebar is hidden */
  padding-bottom: calc(var(--content-padding-y) + 56px);
}
```

Note: The `+56px` bottom padding for MobileTabbar only applies at ≤768px. Since MobileTabbar is hidden above that breakpoint, the extra padding is harmless (the tabbar has `display:none` and doesn't occupy space). If visual precision is needed, a media query can gate it:

```css
@media (max-width: 768px) {
  .content {
    padding-bottom: calc(var(--content-padding-y) + 56px);
  }
}
```

### Sidebar.vue

Width driven by `--sidebar-width` token. At ≤768px, token is `0px` and `display:none` fully hides it.

```css
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  display: flex;
  flex-direction: column;
  transition: width 0.2s ease, min-width 0.2s ease;
}

@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
}
```

### Topbar.vue

No width changes needed — already flex-based and fills parent. Search bar visibility at ≤768px stays as-is.

### MobileTabbar.vue

No changes — already correctly shows at ≤768px and hides above.

## 3. Global Grid Utilities

Add to `theme.css`:

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

/* Horizontal overflow prevention */
.shell, .main, .content {
  overflow-x: hidden;
}

.stats-row, .auto-grid, .tool-grid,
.marketplace-grid, .sources-grid, .agents-grid {
  min-width: 0;
}
```

## 4. Per-Page Migration

### DashboardView

- `stats-row`: Remove scoped `@media` rules for grid-template-columns. Use global `.stats-row` class (driven by `--stats-columns`).
- `quick-actions-grid`: Replace custom column logic with `.auto-grid` class.

### CliToolsView

- `tool-grid`: Already uses vertical flex layout. No grid changes needed.
- Verify no hardcoded padding overrides.

### SoftwareManagementView

- Most granular breakpoints (479px, 767px, 1023px, 1280px). Full migration to unified four breakpoints.
- Grid columns: Replace custom media query cascade with `.auto-grid`.
- Filter bar: Migrate to global filter-bar rules.

### PluginsView

- `marketplace-grid` (`minmax(300px, 1fr)`): Replace with `.auto-grid`.
- `sources-grid` (`minmax(340px, 1fr)`): Replace with `.auto-grid`, override `--card-min-width: 340px` locally if needed.
- Filter bar: Remove scoped rules, use global.

### SkillsView

- Tool grid: Replace custom breakpoint logic with `.auto-grid`.
- Source tabs overflow handling: Add `overflow-x: auto` (already present).

### AgentsView

- `agents-grid` (`minmax(380px, 1fr)`): Replace with `.auto-grid`, override `--card-min-width: 380px` locally.
- Remove scoped `@media` rules for grid columns.

### MCPView

- `tool-grid`: Already uses `auto-fill, minmax(280px, 1fr)`. Replace with `.auto-grid`.
- Audit table: Ensure `overflow-x: auto` on container for narrow windows.

### RulesView

- `tool-grid`: Same pattern as MCPView.

### BackupView

- Remove `max-width: 1200px` constraint. Content fills available space.
- Backup grid: Replace custom 2/1 column logic with `.auto-grid`.

### SettingsView

- `settings-grid`: Change from `1fr 1fr` to `repeat(auto-fill, minmax(320px, 1fr))`.
- Theme grid: Keep `repeat(auto-fill, minmax(90px, 1fr))` — already naturally responsive.

### PromptManagerView

- Verify grid behavior and migrate if custom breakpoints exist.

## 5. Filter Bar Consolidation

Current state: filter-bar media queries are duplicated across 7+ views in scoped styles.

Target: Single global definition in `theme.css`:

```css
/* Filter bar — global responsive rules */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

@media (max-width: 1024px) {
  .filter-bar { gap: 8px 10px; }
  .filter-bar .search-input { flex: 1 1 160px; }
  .filter-bar .filter-select { max-width: 140px; font-size: 12px; }
  .filter-bar .btn { padding: 7px 12px; font-size: 11px; }
}

@media (max-width: 768px) {
  .filter-bar {
    gap: 8px;
    flex-direction: column;
    align-items: stretch;
  }
  .filter-bar .search-input { flex: 1 1 auto; min-width: 0; }
  .filter-bar .filter-select { max-width: none; width: 100%; }
  .filter-bar .btn-group { margin-left: 0; justify-content: flex-end; width: 100%; }
  .filter-bar > button:not(.filter-select) { width: 100%; justify-content: center; }
  .section-header { flex-direction: column; gap: 12px; align-items: flex-start; }
}
```

Remove all duplicate filter-bar media queries from per-view scoped styles.

## 6. Migration Execution Order

1. Create `src/assets/tokens/responsive.css`
2. Update `src/assets/main.css` to import it
3. Add global grid utilities and overflow prevention to `theme.css`
4. Add global filter-bar responsive rules to `theme.css`
5. Refactor `AppFrame.vue` — content padding to token references
6. Refactor `Sidebar.vue` — width to token reference
7. Migrate views in order:
   - DashboardView
   - CliToolsView
   - SoftwareManagementView
   - PluginsView
   - SkillsView
   - AgentsView
   - MCPView
   - RulesView
   - BackupView
   - SettingsView
   - PromptManagerView
8. Verification pass at 900px / 1024px / 1280px / 1440px / 1920px

## 7. What Does NOT Change

- **Topbar**: Already flex-based, naturally responsive
- **MobileTabbar**: Show/hide logic at ≤768px stays as-is
- **Modal / Toast**: Already use `%` and `max-width`
- **Theme switching**: Unaffected by responsive changes
- **Glassmorphism layers**: All `backdrop-filter` and opacity values preserved
- **Tauri window config**: Default/min sizes unchanged

## 8. Success Criteria

- No horizontal scrollbar at any window size from 900px to 3840px
- All grids reflow naturally through the four breakpoints
- Sidebar collapses correctly at each tier
- Content padding adjusts smoothly via token variables
- Single source of truth for all breakpoint values in `tokens/responsive.css`
- Zero duplicate `@media` rules across view files

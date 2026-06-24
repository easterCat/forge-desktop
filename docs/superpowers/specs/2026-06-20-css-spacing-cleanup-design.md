# CSS Spacing Cleanup — Padding Stacking & Margin Removal

**Date**: 2026-06-20
**Status**: Approved
**Scope**: Frontend CSS — global theme + scoped view styles

## Problem

1. **Padding stacking**: `.view` containers in some views have their own padding, and `.content` inside them also has padding (`24px 32px 40px`). When nested, vertical padding doubles — e.g., PluginsView `.view` (24px top) + `.content` (24px top) = 48px visual top padding.

2. **Unwanted margin-bottom**: `.tab-bar` has `margin-bottom: 20px` globally and in scoped overrides. `.section-header` has `margin-bottom: 24px` globally (via CSS variable) with various scoped overrides (8px–24px). Both produce excess vertical spacing.

## Design Decision

**Approach A — Single source of truth**: Only `.content` owns padding. `.view` never has padding. `.tab-bar` and `.section-header` have `margin-bottom: 0`.

Rationale: `.content` is the standard content container across all views. The design reference (`forge.css`) defines `.content{padding:24px 32px 40px}` and `.view` has no padding. This approach aligns with the design reference, has the smallest diff, and lowest risk.

## Changes

### 1. Padding stacking fix

**Keep unchanged:**
- `src/assets/theme.css` `.content` — global padding via CSS variables (`24px 32px 40px`)
- `src/assets/tokens/responsive.css` — responsive `--content-padding-*` variables

**Remove padding from:**

| File | Selector | Current | Action |
|------|----------|---------|--------|
| `src/components/layout/AppFrame.vue` | `.content` | `padding: 24px 32px 40px` | Remove padding declaration (inherits global) |
| `src/views/PluginsView.vue` | `.view` | `padding: 24px 32px 40px` | Remove padding declaration |
| `src/views/BackupView.vue` | `.view` | `padding: var(--spacing-lg)` | Remove padding declaration |

**No change needed:** SoftwareManagementView, RulesView — `.view` has no padding.

### 2. Margin-bottom removal

#### `.tab-bar`

| File | Current | Target |
|------|---------|--------|
| `src/assets/theme.css` (line ~1851) | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/assets/theme.css` (line ~2719, FEAT-024-B) | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/PluginsView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/SoftwareManagementView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/CliToolsView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |

#### `.section-header`

| File | Current | Target |
|------|---------|--------|
| `src/assets/theme.css` CSS variable | `--section-header-margin-bottom: 24px` | `--section-header-margin-bottom: 0` |
| `src/assets/theme.css` global rule | `margin-bottom: var(--section-header-margin-bottom)` | `margin-bottom: 0` |
| `src/views/SoftwareManagementView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/RulesView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/BackupView.vue` | `margin-bottom: var(--spacing-lg)` | `margin-bottom: 0` |
| `src/views/DashboardView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/AgentsView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |
| `src/views/SettingsView.vue` | `margin-bottom: 8px` | `margin-bottom: 0` |
| `src/views/CliToolsView.vue` | `margin-bottom: 20px` | `margin-bottom: 0` |

**No change needed:** PluginsView `.section-header` — already has no `margin-bottom` declaration.

## Files touched (10 total)

1. `src/assets/theme.css`
2. `src/components/layout/AppFrame.vue`
3. `src/views/PluginsView.vue`
4. `src/views/BackupView.vue`
5. `src/views/SoftwareManagementView.vue`
6. `src/views/RulesView.vue`
7. `src/views/DashboardView.vue`
8. `src/views/AgentsView.vue`
9. `src/views/SettingsView.vue`
10. `src/views/CliToolsView.vue`

## Expected outcome

1. `.view` → `.content` nesting: vertical padding contributed only by `.content` (24px top, 40px bottom), no stacking.
2. `.tab-bar` and `.section-header` produce no extra vertical spacing.
3. Responsive behavior unchanged — `responsive.css` tokens untouched.
4. Other `.section-header` properties (`border-bottom`, `padding-bottom`, `gap`, etc.) unchanged.

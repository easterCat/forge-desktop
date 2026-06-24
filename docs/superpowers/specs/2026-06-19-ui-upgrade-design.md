# UI Upgrade Design: Pixel-Perfect Glass-Morphism Alignment

**Date:** 2026-06-19
**Prototype:** `design/theme/forge-cross-platform-glass.html` (2164 lines, single-file reference)
**Approach:** Bottom-up Token Rewrite (Approach A)

---

## 1. Goals

1. All visual elements, layouts, colors, fonts, spacing, component states, and responsive behavior must be **pixel-perfect identical** to the prototype HTML file.
2. Expand theme system from **5 themes to 20 themes** (all themes defined in prototype's `THEMES` array).
3. **Delete legacy theme system** (`useTheme.ts`), consolidate on Pinia theme store + `useGlassTheme.ts`.
4. Ensure **all prototype features** are implemented in Vue components (sync chips, marketplace tabs, agent targets, etc.).
5. Replace all **hardcoded `rgba()` values** in component scoped styles with CSS variable references.

---

## 2. Token Layer Rewrite

### 2.1 colors.css

Source of truth: prototype `:root` block (lines 15-65).

Key variable corrections:
- `--bg`: `#F0EDE8` (current: `#F5F3F0`)
- `--fg-ghost`: `#7A7A7A` (current: `#9A9A9A`)
- `--glass-highlight`: `rgba(255,255,255,0.85)` (add new)
- `--border-hover`: `rgba(255,255,255,0.35)` (verify)
- `--border-window`: `rgba(255,255,255,0.18)` (verify)
- `--border-outer-glow`: `rgba(255,255,255,0.40)` (add new)

Also define: all shadow variables, tint variables, font stacks, easing curves, timing constants, z-index layers.

Naming unification:
- Keep `--sidebar-w` (match prototype), remove `--sidebar-width`
- Keep `--topbar-h`, define only in `colors.css`

### 2.2 glass.css

Source of truth: prototype lines 18-30.

6-layer glass system (warm/light baseline):
```
--glass-bg:          rgba(255,255,255,0.45)
--glass-bg-hover:    rgba(255,255,255,0.58)
--glass-sidebar:     rgba(255,255,255,0.35)
--glass-topbar:      rgba(255,255,255,0.38)
--glass-input:       rgba(255,255,255,0.40)
--glass-input-focus: rgba(255,255,255,0.60)
--glass-window:      rgba(255,255,255,0.25)
--glass-inner-glow:  rgba(255,255,255,0.70)
--glass-highlight:   rgba(255,255,255,0.85)
```

### 2.3 motion.css

Source of truth: prototype lines 1156-1166 and all `@keyframes`.

Animations to define:
- `tint-drift`: 8s ease-in-out infinite (stat card atmosphere drift)
- `tint-sweep`: 4.5s ease-in-out infinite (stat card highlight sweep)
- `shimmer`: 1.5s ease-in-out infinite (skeleton loading)
- `pulse`: 1.5s ease-in-out infinite (badge progress dot)
- `sync-spin`: 1s linear infinite (syncing chip icon)
- `toastIn` / `toastOut`: using `--t-slow`
- `modal-enter` / `modal-exit` (if needed)

Timing constants:
- `--t-fast`: 150ms
- `--t-base`: 200ms
- `--t-slow`: 300ms
- `--ease`: cubic-bezier(0.4, 0, 0.2, 1)

Reduced motion: `@media (prefers-reduced-motion: reduce)` disables all animations.

### 2.4 responsive.css

Source of truth: prototype lines 514-665.

Breakpoints:
- `1024px` (tablet): sidebar collapses to 60px, stats 2-col, grid adjusts
- `768px` (small tablet): sidebar hidden, mobile tabbar visible, stats 2-col
- `640px`: stats 1-col
- `480px` (phone): filter bar compact, reduced padding

Layout tokens per breakpoint:
- `--content-padding`: 32px → 24px → 16px
- `--sidebar-w`: 240px → 60px → 0px
- `--stats-columns`: 4 → 2 → 1
- `--card-grid-min`: 320px → 300px → 1fr

---

## 3. Theme System Expansion (5 → 20)

### 3.1 Theme Files

Create 15 new files in `src/assets/tokens/themes/`:

| File | ID | Name | Colors (bg, surface, border, fg, accent, warn) |
|------|----|------|------------------------------------------------|
| `cool-mist.css` | cool-mist | Cool Mist | `#EEF1F5, #DDE2EA, #B8C4D4, #1E2A3A, #4A8A6A, #6B7FAA` |
| `sakura.css` | sakura | Sakura | `#F8F0F2, #F0DDE2, #E4B8C4, #3A2028, #8A5A6A, #C47A8A` |
| `ocean.css` | ocean | Ocean | `#ECF4F4, #D4E8E8, #A8D4D4, #0A2828, #2A8A8A, #4A9AAA` |
| `ember.css` | ember | Ember | `#F8F2EC, #F0E0CC, #E0C8A0, #382010, #B87A3A, #D49A4A` |
| `slate.css` | slate | Slate | `#F0F0EE, #E0E0DC, #C8C8C0, #2A2A28, #6A7A6A, #8A8A7A` |
| `aurora.css` | aurora | Aurora | `#F0F4F8, #DAE4F0, #A8C4E0, #14202C, #4A8ACC, #8A6ACC` |
| `cream.css` | cream | Cream | `#FAF8F5, #F2EDE6, #E8DFD0, #2C2820, #7A8A5A, #AA9A60` |
| `rose-gold.css` | rose-gold | Rose Gold | `#F8F2F4, #F0DDE2, #E0C0C8, #301820, #B85A70, #D48A6A` |
| `cyberpunk.css` | cyberpunk | Cyberpunk | `#0E0E18, #1A1A2E, #2A2A40, #E0E0F0, #FF2E63, #08D9D6` |
| `forest.css` | forest | Forest | `#F0F4EE, #D8E4D0, #B0C8A0, #14200E, #3A6A2A, #5A8A4A` |
| `desert.css` | desert | Desert Sand | `#F5EEE4, #E8D8C4, #D0BC98, #2C2014, #A07848, #C49A58` |
| `cotton-candy.css` | cotton-candy | Cotton Candy | `#F8F0F8, #F0DAF0, #E0C0E8, #282030, #C060A0, #60A0C0` |
| `charcoal.css` | charcoal | Charcoal | `#181818, #252525, #333333, #E8E8E8, #6A8A6A, #8A6A6A` |
| `peach.css` | peach | Peach Fuzz | `#FBF0E8, #F5DCC8, #E8C0A0, #302018, #D48A58, #E8A070` |
| `nordic.css` | nordic | Nordic | `#F4F6F8, #E2E8EE, #C8D4E0, #1C2430, #5A7A9A, #8A6A5A` |

Update 5 existing theme files to match prototype exactly.

### 3.2 Theme Variable Derivation

Each theme file derives all CSS variables from its 6 core colors, following prototype's `selectTheme()` logic (lines 1537-1606):

1. Detect dark/light via perceived luminance formula: `0.299R + 0.587G + 0.114B`
2. Derive: `--fg-title`, `--fg-muted`, `--fg-ghost` (light vs dark variants)
3. Derive: `--accent-hover`, `--accent-press`, `--accent-glow`
4. Derive: all `--glass-*` variables (light vs dark alpha values)
5. Derive: all `--shadow-*` variables
6. Derive: `--border-*` variables
7. Derive: body `background-image` (multi-layer gradient with color zones and grid texture)
8. Derive: `::before` pseudo-element (3 radial color zones)
9. Derive: `::after` pseudo-element opacity (0.18 light, 0.15 dark)

### 3.3 Theme Store Update

File: `src/stores/theme.ts`

- `ThemeId` type: union of 20 string literals
- `themes` array: 20 objects with `{ id, name, desc, colors[6] }`
- `setTheme(id)`: sets `data-theme` attribute on `<html>`, persists to `localStorage`
- `initTheme()`: restores from `localStorage`, default to `warm`
- All 20 theme CSS files imported statically in `theme.css` via `@import`

### 3.4 Settings Theme Picker

Update `SettingsView.vue` theme grid:
- Display 20 theme cards in `grid-template-columns: repeat(auto-fill, minmax(90px, 1fr))`
- Each card: 6-color preview strip + name
- Active card: accent border + checkmark indicator
- Max height 240px with scroll

---

## 4. Component Style Alignment

### 4.1 Hardcoded Value Replacement

Audit all Vue components' `<style scoped>` blocks. Replace hardcoded `rgba()` with CSS variable references:

| Component | Hardcoded | Replace with |
|-----------|-----------|-------------|
| `Sidebar.vue` | `rgba(255,255,255,0.40)` | `var(--glass-sidebar)` |
| `Topbar.vue` | `rgba(255,255,255,0.40)` | `var(--glass-topbar)` |
| `AppFrame.vue` | `rgba(255,255,255,0.32)` | `var(--glass-window)` |
| `MobileTabbar.vue` | `rgba(255,255,255,0.50)` | `var(--glass-topbar)` |
| View components | Various card rgba | `var(--glass-bg)` |

### 4.2 theme.css Deduplication

Current issue: base styles and `[data-theme="warm"]` overrides define the same properties twice.

Solution: Base layer uses warm theme values directly (warm is default). Remove `[data-theme="warm"]` selector. Other themes override via their own `[data-theme="xxx"]` selectors.

### 4.3 Global Component Styles

Verify each global component class matches prototype exactly:

| Class | Key prototype values | File location |
|-------|---------------------|---------------|
| `.card` | `border-radius: var(--radius)`, `padding: 16px`, `background: rgba(255,255,255,0.42)` | theme.css |
| `.card:hover` | `transform: translateY(-3px)`, `box-shadow: 0 8px 32px` | theme.css |
| `.btn-primary` | `background: rgba(45,45,45,0.85)`, hover translateY(-2px) | theme.css |
| `.btn-secondary` | `backdrop-filter: blur(12px)`, hover translateY(-2px) | theme.css |
| `.badge` | success/warn/error/info/outline/progress variants | theme.css |
| `.stat-card` | `border-radius: 22px`, tint animations | theme.css |
| `.modal` | `backdrop-filter: blur(40px) saturate(1.4)`, `border-radius: 28px` | theme.css |
| `.toast` | `backdrop-filter: blur(30px) saturate(1.3)` | theme.css |
| `.toggle` | `width: 36px`, `height: 20px`, `border-radius: 10px` | theme.css |
| `.filter-bar` | flex-wrap, search-input, filter-select layout | main.css |
| `.dropdown` | glass menu styles | theme.css |
| `.confirm-dialog` | confirmation dialog styles | theme.css |
| `.empty-state` | empty state placeholder | theme.css |
| `.skeleton` | loading skeleton shimmer | theme.css |
| `.error-state` | error state display | theme.css |

### 4.4 Body Background

Verify `warm.css` body background matches prototype lines 67-97:
1. Base gradient (160deg)
2. 3 radial color zones
3. Fine grid (24px) + coarse grid (96px)
4. `::before` — 3 radial color blobs
5. `::after` — SVG noise texture at 0.18 opacity

---

## 5. Architecture Cleanup

### 5.1 Delete Legacy Theme System

- Delete: `src/composables/useTheme.ts`
- Search and replace all `import { useTheme }` with `import { useThemeStore }` or `import { useGlassTheme }`
- If `cycleTheme()` is needed, add equivalent to `useGlassTheme.ts`
- Remove `localStorage` key `aem-theme` references

### 5.2 CSS Variable Naming

Unify to prototype naming:
- `--sidebar-w` (remove `--sidebar-width`)
- `--topbar-h` (define only in `colors.css`)
- Remove duplicate definitions across tokens and theme.css

### 5.3 useGlassTheme.ts Update

- Extend `ThemeId` type to 20 themes
- Keep `setGlassVariant()` API unchanged (not exposed to UI)

### 5.4 Theme Loading Strategy

All 20 theme CSS files imported statically in `theme.css`:
```css
@import './tokens/themes/warm.css';
@import './tokens/themes/cool-mist.css';
/* ... all 20 */
```

Each theme file uses `[data-theme="id"]` selector. No dynamic loading needed.

---

## 6. Missing Feature Implementation

### 6.1 Plugin CLI Sync Chips

Source: prototype lines 389-485.

Three states:
- **unsynced**: `rgba(255,255,255,0.40)` bg, warn border, `+` icon
- **synced**: green tint bg, green border, checkmark icon
- **syncing**: blue tint bg, spinning icon animation

Features:
- Chip icon (CLI tool abbreviation with tool color)
- Click to toggle sync/unsync
- Sync count badge on plugin card name

### 6.2 Marketplace Source Tabs

Source: prototype lines 738-742.

- Tab bar above marketplace grid: All Sources / forge-official / community-hub / ai-tools-pack
- Each tab shows count in `.tab-count`
- Click filters marketplace list
- Dynamic count updates from filtered data

### 6.3 Agent Install Target Chips

Source: prototype lines 569-586, 2074-2097.

- `.target-chip` with dot + tool abbreviation
- Selected state: accent bg, bold weight
- Click toggles selection
- Count updates label: "Install to · N selected"
- Install button shows count

### 6.4 Skills Source Tabs

Source: prototype lines 779-785.

- Tab bar: All / Local / Anthropic / Marketplace / Skills.sh
- `.tab-count` on each tab
- Filters skills list on click

### 6.5 MCP Audit Log Table

Source: prototype lines 864-886.

- `.audit-table` with styled headers and rows
- Action badges (health_check, install, sync)
- Status badges (OK, FAIL)
- Mono font for timestamps and technical details
- Row hover highlight

### 6.6 MCP Groups

Source: prototype lines 852-863.

- `.group-chips` filter bar
- `.group-chip` toggle (All / Development / CI/CD)
- Server grid filtered by group

### 6.7 Import/Export Page

Source: prototype line 933.

Verify `ImportExportView.vue` matches prototype:
- Export: checkbox list, "Export as .forge" button
- Import: drop zone with dashed border, Git repo import button
- Setting groups with glass styling (no hover lift on import/export page per `#view-import-export .setting-group:hover` override)

### 6.8 Detail Modal Tabs

Source: prototype lines 960-1011.

- `.detail-tabs` + `.detail-tab` with bottom border indicator
- Tab content switching (Overview / Skills / Commands / Hooks / MCP / LSP)
- `.cap-grid` — 5-column grid showing capability counts

### 6.9 Other Prototype Components

| Component | Source | Description |
|-----------|--------|-------------|
| `.platform-pills` | lines 570-575 | Platform badges (mac/win/linux/cross) with color variants |
| `.health-timeline` | lines 589-592 | MCP health history dot bars |
| `.stats-mini` | lines 627-630 | Compact stats row |
| `.dropdown-menu` | lines 553-560 | Glass dropdown menu |
| `.confirm-dialog` | lines 563-567 | Confirmation dialog |
| `.error-state` | lines 547-550 | Error display with icon |
| `.skeleton` | lines 539-543 | Loading skeleton with shimmer |
| `.op-stage` | lines 498-505 | Operation stage badges (preparing/downloading/installing/etc.) |

---

## 7. Implementation Order

1. Token layer rewrite (colors → glass → motion → responsive)
2. Theme expansion (create 15 new themes, update 5 existing)
3. theme.css deduplication and alignment
4. Component scoped style fixes (hardcoded → variables)
5. Legacy system cleanup (delete useTheme.ts)
6. Missing feature implementation (sync chips, tabs, targets, etc.)
7. Global component style verification pass
8. Responsive behavior verification
9. Cross-browser testing (backdrop-filter support)

---

## 8. Success Criteria

- All 20 themes render identically to prototype's `selectTheme()` output
- No hardcoded `rgba()` values in component scoped styles
- Single theme system (Pinia store + useGlassTheme)
- All prototype interactive features functional in Vue
- Responsive behavior matches prototype at all breakpoints
- Zero visual regression on existing functionality

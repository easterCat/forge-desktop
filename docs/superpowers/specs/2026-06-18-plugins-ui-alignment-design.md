# Plugins UI Alignment Design

**Date:** 2026-06-18
**Status:** Draft
**Scope:** Sidebar + Plugins page (Installed / Marketplace / Sources) structural + visual alignment with HTML prototype

---

## 1. Overview

### Goal
Align the Vue component structure and visual styling with the HTML prototype (`design/theme/forge-cross-platform-glass.html`) for the following components:
1. Sidebar navigation
2. Plugins page (tab structure)
3. Installed plugin cards
4. Marketplace plugin cards
5. Sources management page (new)

### Approach
**Structural refactoring + visual alignment** — preserve existing business logic (Pinia stores, TypeScript types, event handling) while restructuring DOM and CSS to match the prototype.

### Key Decisions
- Replace "Updates" tab with "Sources" tab
- Create full Sources management page (list, search, filter, add/edit/delete)
- Keep `router-link` navigation (Vue Router) instead of prototype's `onclick`
- Reuse existing `CliSyncChip`, `FilterBar`, `SearchInput` components

---

## 2. Sidebar

### Files
- `src/components/layout/Sidebar.vue`

### Structural Changes
1. **Brand Logo**: Replace SVG with prototype's solid rectangle design
   - `#2D2D2D` fill, white inner rectangle, amber accent (`#B8944A`)
2. **Search input**: Add glass background and border
   - `background: rgba(255,255,255,0.30)`
   - `border: 1px solid rgba(255,255,255,0.32)`
   - `border-radius: 12px`
3. **Nav items**: Adjust padding to `10px 12px`
4. **Label fixes**: "Mcps" → "MCP Servers", remove "Prompts" menu item

### Visual Alignment
| Property | Prototype Value | Current Value |
|----------|----------------|---------------|
| Width | `240px` | `var(--sidebar-w)` ✅ |
| Background | `rgba(255,255,255,0.40)` | `var(--glass-sidebar)` ✅ |
| Backdrop | `blur(24px) saturate(1.2)` | ✅ |
| Border-right | `rgba(255,255,255,0.22)` | `var(--border-window)` ✅ |
| Nav hover | `rgba(255,255,255,0.30)` | `var(--glass-bg-hover)` ✅ |
| Nav active bg | `rgba(255,255,255,0.30)` | `var(--glass-bg-hover)` ✅ |
| Nav active border | `var(--accent)` | ✅ |
| Section title spacing | `0.06em` | `0.05em` ⚠️ |
| Footer border-top | `rgba(255,255,255,0.30)` | Missing ⚠️ |

### Preserved Logic
- `router-link` navigation
- Badge counts from stores
- Search input `v-model`
- Responsive hiding at 768px

---

## 3. PluginsView Tab Structure

### Files
- `src/views/PluginsView.vue`

### Changes
1. **Tab items**: Replace `updates` with `sources`
   - `[{ id: 'installed', ... }, { id: 'marketplace', ... }, { id: 'sources', ... }]`
2. **Section header**: Add count badge (`<span class="count">6 installed</span>`)
3. **Remove**: "Refresh manifest" button from header
4. **Tab bar**: Use global `tab-bar` style (underline, not chip-style)

### Tab Content
- **Installed**: Search + status filter → plugin cards
- **Marketplace**: Source tabs + search/category/sort filters → marketplace cards
- **Sources**: Search + status filter + Add Source button → source cards

---

## 4. Installed Plugin Cards

### Files
- `src/views/PluginsView.vue` (inline template, not separate PluginCard.vue)
- `src/components/common/CliSyncChip.vue` (reuse)

### DOM Structure
```html
<div class="card plugin-card">
  <div class="plugin-card-head">
    <div style="display:flex;align-items:center;gap:14px">
      <div class="plugin-icon">...</div>
      <div>
        <div class="plugin-name-row">
          <span>name</span>
          <span class="plugin-version">v1.0.0</span>
          <span class="sync-count">N synced</span>
        </div>
        <div class="plugin-meta">desc · author · software</div>
      </div>
    </div>
    <div class="btn-group">
      <div class="toggle">...</div>
      <button class="btn btn-secondary btn-sm">Update</button>
      <button class="btn-icon btn-sm">...</button>
    </div>
  </div>
  <div class="plugin-cli-row">
    <span class="plugin-cli-label">CLI Tools</span>
    <CliSyncChip ... />
  </div>
</div>
```

### Visual Details
| Property | Value |
|----------|-------|
| Card background | `rgba(255,255,255,0.42)` |
| Card backdrop | `blur(20px) saturate(1.2)` |
| Card border | `1px solid rgba(255,255,255,0.35)` |
| Card border-radius | `18px` |
| Card hover | `translateY(-3px)`, `box-shadow: 0 8px 32px rgba(0,0,0,0.08)` |
| Icon size | `40px × 40px` |
| Icon border-radius | `12px` |
| Icon background | `rgba(45,45,45,0.06)` + `border: 1px solid var(--border)` |
| Name font | `14px`, `600`, `var(--fg-title)` |
| Version font | `var(--font-mono)`, `11px`, `var(--fg-ghost)` |
| Meta font | `12px`, `var(--fg-ghost)` |
| Toggle size | `36px × 20px`, `border-radius: 10px` |
| CLI row border | `1px solid rgba(255,255,255,0.10)` |

---

## 5. Marketplace Plugin Cards

### Files
- `src/components/common/MarketplaceCard.vue`

### DOM Structure
```html
<div class="card marketplace-card">
  <div class="marketplace-card-head">
    <div class="marketplace-card-icon">...</div>
    <div class="marketplace-card-info">
      <div class="marketplace-card-name">
        name <span class="installed-dot" />
      </div>
      <div class="marketplace-card-author">
        by author <span class="source-badge">source</span>
      </div>
    </div>
  </div>
  <div class="marketplace-card-desc">description</div>
  <div class="marketplace-card-tags">
    <span class="tag">category</span>
  </div>
  <div class="marketplace-card-meta">
    <span>↓ downloads</span>
    <span>★ stars</span>
  </div>
  <div class="marketplace-card-footer">
    <span class="version">v1.0.0</span>
    <div class="btn-group">
      <button class="btn-icon">Details</button>
      <button class="btn btn-primary btn-sm">Install</button>
    </div>
  </div>
</div>
```

### Visual Details
| Property | Value |
|----------|-------|
| Card layout | `flex-direction: column`, `gap: 12px` |
| Icon size | `40px × 40px` |
| Icon border-radius | `12px` |
| Installed dot | `7px × 7px`, `border-radius: 50%`, `background: var(--success)` |
| Author font | `11px`, `var(--fg-ghost)` |
| Source badge | `font-family: var(--font-mono)`, `10px`, pill shape |
| Desc font | `12px`, `var(--fg-muted)`, `-webkit-line-clamp: 2` |
| Tag font | `10px`, pill shape, `rgba(255,255,255,0.40)` |
| Meta font | `11px`, `var(--fg-ghost)` |
| Footer border | `1px solid var(--border)` |
| Version font | `var(--font-mono)`, `11px`, `var(--fg-ghost)` |

---

## 6. Sources Management Page (New)

### Files
- `src/views/PluginsView.vue` (sources tab section)
- `src/components/plugins/SourceCard.vue` (new component)
- `src/components/plugins/AddRepoSourceDialog.vue` (reuse)
- `src/stores/plugin-marketplace.ts` (add source management methods)

### Page Structure
```html
<div id="plugins-sources">
  <div class="filter-bar">
    <div class="search-input">...</div>
    <select class="filter-select">All Status / Installed / Pending</select>
    <button class="btn btn-primary btn-sm">+ Add Source</button>
  </div>
  <div class="sources-grid">
    <SourceCard ... />
  </div>
</div>
```

### Source Card Structure
```html
<div class="card source-card">
  <div class="source-card-head">
    <div class="source-card-icon">
      <!-- Market or Resource icon -->
    </div>
    <div style="flex:1;min-width:0">
      <div class="source-card-title">
        <span class="source-name-text">name</span>
        <span class="badge success/warn">Installed/Pending</span>
      </div>
      <div class="source-card-subtitle">
        GitHub · N plugins
      </div>
    </div>
  </div>
  <div class="source-card-notes">notes text</div>
  <div class="source-card-url">
    <svg ... /> <span class="url-text">url</span>
    <a class="url-link" href="..." target="_blank">...</a>
  </div>
  <div class="source-card-path">
    <svg ... /> <span class="path-text">path</span>
  </div>
  <div class="source-card-footer">
    <div class="btn-group">
      <button class="btn-icon btn-sm" title="View details">...</button>
      <button class="btn-icon btn-sm" title="Edit notes">...</button>
      <button class="btn-icon btn-sm" title="More">...</button>
    </div>
    <button class="btn btn-secondary/primary btn-sm">Sync/Install</button>
  </div>
</div>
```

### Visual Details
| Property | Value |
|----------|-------|
| Grid | `grid-template-columns: repeat(auto-fill, minmax(340px, 1fr))`, `gap: 16px` |
| Card padding | `20px` |
| Card gap | `14px` |
| Icon size | `38px × 38px` |
| Icon border-radius | `12px` |
| Title font | `14px`, `600`, `var(--fg-title)` |
| Subtitle font | `11px`, `var(--fg-ghost)` |
| Notes font | `12px`, `var(--fg-muted)` |
| URL font | `var(--font-mono)`, `11px`, `var(--fg-ghost)` |
| Path font | `var(--font-mono)`, `11px`, `var(--fg-ghost)` |
| Footer border | `1px solid var(--border)` |

### Data Model
```typescript
interface PluginSource {
  id: string
  name: string
  url: string
  type: 'market' | 'resource'
  repoType: 'GitHub' | 'GitLab' | 'Gitee'
  installed: boolean
  pluginCount: number
  path: string
  notes: string
}
```

### Interactions
- **Search**: Filter by name and URL
- **Status filter**: All / Installed / Pending
- **Add Source**: Opens `AddRepoSourceDialog`
- **Sync**: Syncs installed source
- **Install**: Installs pending source
- **Details/Edit/More**: Opens respective dialogs (placeholder for now)

---

## 7. Implementation Order

1. **Sidebar** — Logo, search, nav items, footer
2. **PluginsView tabs** — Replace Updates with Sources, update tab structure
3. **Installed cards** — Restructure PluginCard in PluginsView
4. **Marketplace cards** — Update MarketplaceCard component
5. **Sources page** — New SourceCard component, filter bar, grid layout
6. **Integration** — Wire up store methods, test all interactions

---

## 8. Out of Scope

- Global CSS variable refactoring (existing variables work)
- Theme system changes
- Mobile responsive optimizations (existing breakpoints work)
- Plugin details dialog alignment (separate task)
- AddRepoSourceDialog visual alignment (reuse as-is)

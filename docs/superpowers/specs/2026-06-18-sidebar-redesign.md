# Sidebar Redesign — Complete Rewrite

**Date:** 2026-06-18
**Status:** Approved
**Scope:** `src/components/layout/Sidebar.vue`
**Reference:** `design/theme/forge-cross-platform-glass.html`

---

## 1. Goal

Completely rewrite `Sidebar.vue` to precisely match the design reference file (`forge-cross-platform-glass.html`). Every CSS property value, HTML structure detail, and interactive state must align with the reference.

## 2. Approach

Single-file Vue 3 component with scoped CSS using CSS variables. No Tailwind class names — all styling via CSS variables defined in `src/assets/theme.css` and `src/assets/tokens/`.

## 3. Layout Structure

```
sidebar (aside)
├── sidebar-brand          height: var(--topbar-h), border-bottom
├── sidebar-search         padding: 0 12px 12px, border-bottom
├── sidebar-nav (nav)      flex: 1, overflow-y: auto, padding: 0 12px
│   ├── nav-section: Overview
│   │   └── Dashboard
│   ├── nav-section: Manage
│   │   ├── CLI Tools
│   │   ├── Software
│   │   ├── Plugins
│   │   ├── Skills
│   │   ├── Agents
│   │   ├── MCP Servers
│   │   └── Rules
│   └── nav-section: Data
│       ├── Backup & Restore
│       └── Import / Export
└── sidebar-footer         padding: 16px, border-top
```

### Sidebar container

| Property | Value |
|----------|-------|
| width | `var(--sidebar-w)` (240px) |
| min-width | `var(--sidebar-w)` |
| background | `rgba(255,255,255,0.40)` |
| backdrop-filter | `blur(24px) saturate(1.2)` |
| border-right | `1px solid rgba(255,255,255,0.22)` |
| display | flex, column |
| z-index | 10 |

### Sidebar brand

| Property | Value |
|----------|-------|
| height | `var(--topbar-h)` (64px) |
| padding | `0 16px` |
| gap | 10px |
| border-bottom | `1px solid rgba(255,255,255,0.30)` |
| font-size | 15px |
| font-weight | 700 |
| letter-spacing | -0.02em |
| color | `var(--fg-title)` |

Logo SVG: 24×24, 4-layer nested rects (dark base → translucent white → light → amber accent).

### Sidebar search

| Property | Value |
|----------|-------|
| container padding | `0 12px 12px` |
| container border-bottom | `1px solid rgba(255,255,255,0.08)` |
| input padding | `8px 10px 8px 34px` |
| input font-size | 13px |
| input border-radius | `var(--radius-sm)` (12px) |
| input background | `rgba(255,255,255,0.30)` |
| input border | `1px solid rgba(255,255,255,0.32)` |
| input backdrop-filter | `blur(16px) saturate(1.2)` |
| icon position | absolute, left: 10px, vertical center |
| icon size | 14×14 |

### Sidebar nav

| Property | Value |
|----------|-------|
| flex | 1 |
| overflow-y | auto |
| padding | `0 12px` |

### Nav section

| Property | Value |
|----------|-------|
| margin-bottom | 4px |

### Nav section title

| Property | Value |
|----------|-------|
| font-size | 11px |
| font-weight | 600 |
| text-transform | uppercase |
| letter-spacing | 0.06em |
| color | `var(--fg-ghost)` |
| padding | `16px 12px 6px` |
| border-bottom | `1px solid rgba(255,255,255,0.32)` |

### Nav item

| Property | Value |
|----------|-------|
| display | flex, align-items: center |
| gap | 10px |
| padding | `10px 12px` |
| border-radius | `var(--radius-sm)` (12px) |
| font-size | 13px |
| font-weight | 500 |
| border-left | `3px solid transparent` |
| transition | `all var(--t-fast)` (150ms) |

### Nav badge

| Property | Value |
|----------|-------|
| margin-left | auto |
| font-size | 11px |
| font-weight | 600 |
| color | `var(--fg-ghost)` |
| font-family | `var(--font-mono)` |

### Sidebar footer

| Property | Value |
|----------|-------|
| padding | 16px |
| border-top | `1px solid rgba(255,255,255,0.30)` |
| display | flex, align-items: center |
| gap | 10px |
| font-size | 12px |
| color | `var(--fg-ghost)` |

### Avatar

| Property | Value |
|----------|-------|
| width/height | 28px |
| border-radius | `var(--radius-sm)` (12px) |
| background | `rgba(255,255,255,0.30)` |
| border | `1px solid rgba(255,255,255,0.32)` |
| color | `var(--accent)` |
| font-weight | 600 |
| font-size | 11px |
| backdrop-filter | `blur(12px)` |

## 4. Interactive States

### Nav item

| State | Background | Color | Border-left | SVG opacity |
|-------|-----------|-------|-------------|-------------|
| Default | transparent | `var(--fg-muted)` | 3px solid transparent | 0.5 |
| Hover | `rgba(255,255,255,0.30)` | `var(--fg)` | 3px solid transparent | 0.5 |
| Active | `rgba(255,255,255,0.30)` | `var(--fg-title)` | 3px solid `var(--accent)` | 1 |

Transition: `all 150ms cubic-bezier(0.4, 0, 0.2, 1)`

### Search input

| State | Background | Border |
|-------|-----------|--------|
| Default | `rgba(255,255,255,0.30)` | `rgba(255,255,255,0.32)` |
| Focus | `rgba(255,255,255,0.40)` | `rgba(255,255,255,0.40)` |

Transition: `border-color var(--t-fast), background var(--t-fast)`

## 5. Navigation Items

| Section | Label | Route | Badge source |
|---------|-------|-------|-------------|
| Overview | Dashboard | `/` | — |
| Manage | CLI Tools | `/cli-tools` | `cliToolsCount` |
| Manage | Software | `/software` | `softwareCount` |
| Manage | Plugins | `/plugins` | `pluginsCount` |
| Manage | Skills | `/skills` | `skillsCount` |
| Manage | Agents | `/agents` | `agentsCount` |
| Manage | MCP Servers | `/mcp` | `mcpCount` |
| Manage | Rules | `/rules` | `rulesCount` |
| Data | Backup & Restore | `/backup` | — |
| Data | Import / Export | `/import-export` | — |

Each nav item uses `router-link` with `text-decoration: none` and `color: inherit`.
Active state detection: `route.path === path` for exact match.

## 6. Responsive Behavior

| Breakpoint | Behavior |
|-----------|----------|
| > 768px | Sidebar visible, fixed left |
| ≤ 768px | Sidebar hidden (`display: none`), MobileTabbar handles navigation |

No collapse/expand toggle in the design reference.

## 7. Icon Reference

All icons are 18×18 SVG with `stroke="currentColor"`, `stroke-width="2"`, `stroke-linecap="round"`.

| Nav item | Icon |
|----------|------|
| Dashboard | 4-grid rects |
| CLI Tools | Terminal prompt + line |
| Software | Monitor + stand |
| Plugins | Layered diamonds |
| Skills | Star polygon |
| Agents | User circle + head |
| MCP Servers | Sun/circle + rays |
| Rules | Document + lines |
| Backup & Restore | Upload arrow + tray |
| Import / Export | Corner arrows + diagonal |

## 8. Changes from Current Implementation

1. **Nav structure**: Replace "Settings" in Data section with "Import / Export"
2. **SVG opacity**: Change from 0.6 to 0.5 (default), keep 1 for active
3. **Nav padding**: Change from `4px 0` to `0 12px`
4. **Brand border-bottom**: Add `1px solid rgba(255,255,255,0.30)` — currently missing
5. **Footer version**: Change from "v1.0.0" to match design or keep dynamic
6. **Import / Export route**: Add new route `/import-export` (requires router update)
7. **Footer version**: Display dynamic app version (e.g., from `package.json` or hardcoded constant), not the design mockup "v0.1.0"
8. **All CSS values**: Verify exact match with design reference

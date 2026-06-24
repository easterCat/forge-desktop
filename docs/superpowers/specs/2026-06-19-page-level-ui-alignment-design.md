# Page-Level UI Alignment: Pixel-Perfect Per-Page Fix

**Date:** 2026-06-19
**Prototype:** `design/theme/forge-cross-platform-glass.html`
**Scope:** All 10 page areas + token layer + global component styles
**Strategy:** Token layer first, then per-component fixes

---

## 1. Token Layer Fixes

| File | Variable | Current | Prototype |
|------|----------|---------|-----------|
| `glass.css` | `--glass-sidebar` | `rgba(255,255,255,0.35)` | `rgba(255,255,255,0.40)` |
| `glass.css` | `--glass-topbar` | `rgba(255,255,255,0.38)` | `rgba(255,255,255,0.40)` |

---

## 2. Global Component Style Fixes (theme.css base layer)

### 2.1 Buttons

| Property | Current | Prototype |
|----------|---------|-----------|
| `.btn` padding | `7px 16px` | `8px 16px` |
| `.btn` border-radius | `10px` | `var(--radius-sm)` (12px) |
| `.btn-primary` background | `var(--accent)` (opaque) | `rgba(45,45,45,0.85)` |
| `.btn-primary` box-shadow | `oklch(...)` | `0 1px 4px rgba(0,0,0,0.08)` |
| `.btn-secondary` background | `rgba(247,240,226,0.45)` | `rgba(255,255,255,0.30)` |
| `.btn-secondary` border | `rgba(255,255,255,0.15)` | `rgba(255,255,255,0.40)` |
| `.btn-sm` padding | `5px 12px` | `6px 12px` |
| `.btn-icon` background | `rgba(247,240,226,0.45)` | `rgba(255,255,255,0.30)` |

### 2.2 Modal

| Property | Current | Prototype |
|----------|---------|-----------|
| `.modal` background | `rgba(255,255,255,0.72)` | `rgba(255,255,255,0.48)` |
| `.modal` backdrop-filter | `blur(30px)` | `blur(40px) saturate(1.4)` |
| `.modal` border-radius | `var(--radius-lg)` (24px) | `var(--radius-xl)` (28px) |

### 2.3 Filter-Select

| Property | Current | Prototype |
|----------|---------|-----------|
| padding | `6px 12px` | `8px 32px 8px 12px` |
| font-size | `12px` | `13px` |
| background | `rgba(255,255,255,0.20)` | `rgba(255,255,255,0.32)` |
| border-radius | `var(--radius-md)` (14px) | `var(--radius-sm)` (12px) |
| backdrop-filter | `blur(16px)` | `blur(24px) saturate(1.2)` |

---

## 3. DashboardView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| section-header h2 font-size | `18px` | `20px` |
| section-header margin-bottom | none | `20px` |
| section-header padding-bottom | none | `14px` |
| section-header border-bottom | none | `1px solid rgba(255,255,255,0.30)` |
| section-header .count background | `var(--glass-bg)` (0.45) | `rgba(255,255,255,0.32)` |
| section-header .count border | `var(--border)` (0.20) | `rgba(255,255,255,0.32)` |
| stats-row margin-bottom | `32px` | `24px` |
| stats-row gap | `var(--grid-gap)` | `18px` |
| quick-action-card hover | `translateY(-2px)` | `translateY(-3px)` |
| quick-action-card padding | `16px 18px` | `20px` |

Structural: Change dashboard-header to standard section-header with "Environment Overview".

---

## 4. CliToolsView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| tab-item active color | `var(--fg-white)` | `var(--accent)` |
| tab-item padding | `10px 16px` | `12px 16px` |
| tool grid layout | `flex column` | `grid repeat(auto-fill, minmax(300px, 1fr))` |
| tool-card padding | `16px 20px` | `16px` |
| tool-card hover bg | `rgba(255,255,255,0.55)` | `rgba(255,255,255,0.58)` |
| tool-card hover | missing border-color + translateY(-3px) | full hover effect |

---

## 5. SoftwareManagementView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| **tab-bar** | **missing** | All / Detected / Not Found |
| section-header border-bottom | none | `1px solid rgba(255,255,255,0.30)` |
| section-header .count | plain text | styled badge with background |
| filter-bar background | glass panel | no background (plain flex) |
| filter-bar padding | `12px 16px` | none |
| software-item padding | `16px 18px` | `20px` |
| software-item border-radius | `var(--radius-md)` | `var(--radius)` (18px) |
| software-item hover | `filter:brightness(1.6)` | `translateY(-3px)` + shadow |
| software-icon size | `40px` | `42px` |

---

## 6. PluginsView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| source-tabs style | pill/chip (bg+border) | underline text tab (no bg) |
| source-tabs gap | `8px` | `0` |
| source-tabs border-bottom | none | `1px solid rgba(255,255,255,0.10)` |
| source-tab padding | `6px 12px` | `10px 16px` |
| source-tab font-size | `13px` | `12px` |
| source-tab active | bg+border change | underline color only |
| source-tab-count | badge style | inline text + opacity:0.5 |
| plugin card layout | single-column flex | `grid repeat(auto-fill, minmax(320px, 1fr))` |
| plugin card padding | `20px` | `16px` |
| plugin card gap | `0` | `10px` |
| plugin-icon size | `40px` | `42px` |
| sources-grid min-width | `340px` | `300px` |

---

## 7. SkillsView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| filter-bar margin-bottom | `20px` | `16px` |
| skill-card gap | `10px` | `12px` |
| skill-icon size | `40px` | `42px` |
| skill-icon border | `rgba(255,255,255,0.22)` | `var(--border)` (0.20) |
| tag font-size | `10px` | `11px` |
| tag padding | `2px 8px` | `3px 10px` |
| tag border-radius | `99px` | `var(--radius-sm)` (12px) |
| skill-desc font-size | `12px` | `13px` |
| skill-desc line-height | `1.5` | `1.6` |
| card-footer border-top | `rgba(255,255,255,0.18)` | `var(--border)` (0.20) |

---

## 8. AgentsView + AgentCard.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| agent-card padding | `24px` | `18px` |
| agent-card gap | `14px` | `12px` |
| agent-card hover | missing `translateY(-3px)` | has it |
| agent-icon border | `rgba(255,255,255,0.22)` | `var(--border)` (0.20) |
| agent-targets border-top | `rgba(255,255,255,0.10)` | `rgba(255,255,255,0.40)` |

---

## 9. MCPView.vue (Structural Refactor)

Refactor to match prototype tab structure:
- Add tab-bar: Services / Groups / Audit Log
- Services tab: filter-bar + card grid
- Groups tab: group chips + server grid
- Audit tab: audit table

Style fixes:
| Property | Current | Prototype |
|----------|---------|-----------|
| search-box background | `rgba(255,255,255,0.20)` | `rgba(255,255,255,0.32)` |
| search-box border-radius | `var(--radius-md)` | `var(--radius-sm)` |
| filter-select background | `rgba(255,255,255,0.20)` | `rgba(255,255,255,0.32)` |
| server-item border-radius | `var(--radius-md)` | `var(--radius)` (18px) |
| server-item hover | `translateY(-1px)` | `translateY(-3px)` |
| tool-item padding | `14px` | `16px` |
| tool-item background | `rgba(255,255,255,0.40)` | `rgba(255,255,255,0.42)` |

---

## 10. RulesView.vue

| Property | Current | Prototype |
|----------|---------|-----------|
| filter-bar margin-bottom | `24px` | `16px` |
| filter-select color | `var(--fg-muted)` | `var(--fg)` |
| rules-list gap | `10px` | `16px` |
| rule-item padding | `16px 20px` | `20px` |
| rule-item border-radius | `var(--radius-md)` | `var(--radius)` (18px) |
| rule-item hover | `translateY(-1px)` | `translateY(-3px)` |
| rule-icon size | `36px` | `42px` |
| rule-icon border-radius | `8px` | `12px` |
| rule-icon background | `rgba(255,255,255,0.30)` | `rgba(45,45,45,0.06)` |
| rule-icon border | `rgba(255,255,255,0.25)` | `var(--border)` (0.20) |
| rule-name font-family | `var(--font-mono)` | inherit body font |

---

## 11. Sidebar.vue + Topbar.vue

| Component | Property | Current | Prototype |
|-----------|----------|---------|-----------|
| Sidebar | background | `var(--glass-sidebar)` (0.35) | `rgba(255,255,255,0.40)` |
| Sidebar | border-right | `var(--border-window)` (0.18) | `rgba(255,255,255,0.22)` |
| Topbar | background | `var(--glass-topbar)` (0.38) | `rgba(255,255,255,0.40)` |
| Topbar | border-bottom | `var(--border-window)` (0.18) | `rgba(255,255,255,0.22)` |

---

## Implementation Order

1. Token layer (glass.css)
2. Global styles (theme.css base layer: buttons, modal, filter-select)
3. DashboardView.vue
4. CliToolsView.vue
5. SoftwareManagementView.vue + add tab-bar
6. PluginsView.vue (source-tabs style change)
7. SkillsView.vue
8. AgentsView + AgentCard.vue
9. MCPView.vue (structural refactor with tabs)
10. RulesView.vue
11. Sidebar.vue + Topbar.vue

## Success Criteria

- All 17 categories of differences resolved
- Every page matches prototype at pixel level
- All hover/active/selected states match
- All spacing, sizing, colors match prototype values
- Responsive behavior matches prototype breakpoints

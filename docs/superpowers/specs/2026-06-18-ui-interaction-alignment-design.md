# UI & Interaction Alignment Design

**Date:** 2026-06-18
**Reference:** `design/forge-cross-platform-glass.html` (authoritative prototype)
**Scope:** Align current Vue implementation to the reference prototype's glass design system

---

## Problem Statement

The current Vue implementation has significant visual and structural gaps compared to the reference HTML prototype:

1. **Token system duplication** — `theme.css` and `design/tokens/*.css` both define overlapping CSS custom properties with potentially conflicting values
2. **Hardcoded rgba values** — Components use `rgba(255,255,255,0.xx)` directly instead of `--glass-*` tokens, breaking dark theme
3. **Missing glass layering** — Only `.window-frame` has `backdrop-filter`; sidebar, topbar, and cards lack independent glass surfaces
4. **No background texture** — Prototype has 8-layer background (gradients + grids + noise); current is plain
5. **Missing micro-animations** — Stat card tint-drift/tint-sweep, card hover lifts, button hover effects absent
6. **Dimension mismatches** — `--topbar-h` defined as 56px in theme but components fallback to 64px
7. **Limited themes** — Only warm + midnight supported; prototype has 20

---

## Execution Priority

| Priority | Item | Effort | Dependencies |
|----------|------|--------|-------------|
| P0-1 | Token system dedup | Medium | None |
| P0-2 | Fix dimension mismatches | Small | P0-1 |
| P1-1 | Replace hardcoded rgba with glass tokens | Medium | P0-1 |
| P1-2 | Glass effect layering (sidebar, topbar, cards) | Medium | P1-1 |
| P2-1 | Background texture | Small | P1-1 |
| P2-2 | Interactive animations | Medium | P1-2 |
| P3 | Theme expansion | Medium | P0-1 |

Each item is executed and verified before moving to the next.

---

## P0-1: Token System Dedup

### Current State

Two parallel token sources:
- `src/assets/theme.css` — inline `:root` definitions (~80+ variables)
- `design/tokens/*.css` — structured token files (colors, glass, motion, themes)

### Target State

- `design/tokens/*.css` is the **single source of truth** for all design tokens
- `theme.css` only `@import`s token files + defines supplementary variables not in the token files (layout dimensions, z-index, component-level tokens)
- No duplicate definitions between the two sources

### Verified Conflicts Found

| Variable | `design/tokens/` value | `theme.css` value | Resolution |
|----------|----------------------|-------------------|------------|
| `--topbar-h` | 64px (colors.css:94) | 56px (line 77), 64px (line 257) | Remove both theme.css defs, keep token |
| `--border-window` | rgba(255,255,255,0.18) (colors.css:22) | rgba(255,255,255,0.12) (line 126) | Remove theme.css def, keep token |
| `--glass-bg` | Needs audit | 0.28 (theme.css) vs 0.45 (store) | Keep token file value |

### Implementation

1. Audit both sources, identify all overlapping variables
2. For each overlap: keep the `design/tokens/` value, remove from `theme.css` inline
3. Verify `theme.css` `@import` order is correct (colors → glass → motion → themes)
4. Confirm no regressions in warm theme appearance

### Files Modified

- `src/assets/theme.css` — remove duplicate inline definitions, ensure @import chain

---

## P0-2: Fix Dimension Mismatches

### Current State

- `theme.css` defines `--topbar-h: 56px`
- `Topbar.vue` uses `var(--topbar-h, 64px)` with fallback
- Reference prototype defines `--topbar-h: 64px`

### Target State

- All dimension tokens match the reference prototype values
- No component uses fallback values — all reference tokens directly

### Verified Conflicts

- `--topbar-h`: `design/tokens/colors.css` = `64px` (correct), `theme.css:77` = `56px` (bug), `theme.css:257` = `64px` (duplicate). **Resolution:** remove theme.css:77 definition, keep token file value (64px).
- `--border-window`: `design/tokens/colors.css` = `rgba(255,255,255,0.18)`, `theme.css:126` = `rgba(255,255,255,0.12)`. **Resolution:** remove theme.css:126 definition, keep token file value.

### Implementation

1. Remove duplicate/wrong `--topbar-h` and `--border-window` definitions from `theme.css`
2. Remove all fallback values from components: `var(--topbar-h, 64px)` → `var(--topbar-h)`
3. Audit other dimension tokens (`--sidebar-w`, `--radius-*`) for similar mismatches

### Files Modified

- `src/assets/theme.css` — fix token values
- `src/components/layout/Topbar.vue` — remove fallback
- Any other components with dimension fallbacks

---

## P1-1: Replace Hardcoded rgba with Glass Tokens

### Current State

Components hardcode glass-like values:
- Sidebar hover: `rgba(255,255,255,0.30)`
- Topbar secondary button hover: `rgba(255,255,255,0.20)`
- Announcement badge: `rgba(255,255,255,0.32)`
- Various card/input backgrounds

### Target State

All glass-like rgba values replaced with `--glass-*` tokens from the token system.

### Token Mapping (from reference prototype)

| Usage | Token | Light Value | Blur |
|-------|-------|------------|------|
| Window frame | `--glass-window` | `rgba(255,255,255,0.25)` | 40px |
| Sidebar | `--glass-sidebar` | `rgba(255,255,255,0.35)` | 24px |
| Topbar | `--glass-topbar` | `rgba(255,255,255,0.38)` | 24px |
| Card default | `--glass-bg` | `rgba(255,255,255,0.42)` | 20px |
| Card hover | `--glass-bg-hover` | `rgba(255,255,255,0.58)` | 20px |
| Input default | `--glass-input` | `rgba(255,255,255,0.40)` | 16px |
| Input focus | `--glass-input-focus` | `rgba(255,255,255,0.60)` | 16px |

Midnight theme overrides these with lower opacity values (e.g., `rgba(255,255,255,0.06)` for cards).

### Implementation

1. Grep all `.vue` and `.css` files for `rgba(255,255,255,` patterns
2. Replace each with the appropriate `--glass-*` token based on context
3. Verify each replacement in both warm and midnight themes

### Files Modified

- All View components in `src/views/`
- Layout components in `src/components/layout/`
- Common components in `src/components/common/`
- `src/assets/theme.css`

---

## P1-2: Glass Effect Layering

### Current State

Only `.window-frame` in AppFrame.vue has `backdrop-filter`. Sidebar and topbar are fully transparent, relying on the parent's blur.

### Target State

Each major surface has its own glass layer, matching the prototype:

- **Sidebar**: `background: var(--glass-sidebar)` + `backdrop-filter: blur(24px) saturate(1.2)` + `border-right: 1px solid var(--border-window)`
- **Topbar**: `background: var(--glass-topbar)` + `backdrop-filter: blur(24px) saturate(1.2)` + `border-bottom: 1px solid var(--border-window)`
- **Cards**: `background: var(--glass-bg)` + `backdrop-filter: blur(20px) saturate(1.2)` + `border: 1px solid var(--border)`
- **Inputs**: `background: var(--glass-input)` + `backdrop-filter: blur(16px) saturate(1.2)` + `border: 1px solid var(--border)`

### Implementation

1. Add glass background + backdrop-filter + border to Sidebar.vue
2. Add glass background + backdrop-filter + border to Topbar.vue
3. Verify card components already use glass tokens (from P1-1)
4. Verify input components use glass tokens
5. Test layered glass appearance — each surface should be visually distinct

### Files Modified

- `src/components/layout/Sidebar.vue`
- `src/components/layout/Topbar.vue`
- `src/components/layout/AppFrame.vue` (may need z-index adjustments)

---

## P2-1: Background Texture

### Current State

Body has a simple gradient background.

### Target State

8-layer background matching the prototype:

1. Base gradient: `linear-gradient(160deg, #EDEAE5 → #F2EFE9 → #F5F3EF → #F0EDE8 → #ECE9E3)`
2. Warm radial glow: `radial-gradient(ellipse at 0% 0%, rgba(190,175,155,0.20))`
3. Cool radial glow: `radial-gradient(ellipse at 100% 100%, rgba(160,170,190,0.18))`
4. Mid radial glow: `radial-gradient(ellipse at 60% 30%, rgba(200,195,185,0.12))`
5. Fine grid (24px): `linear-gradient(rgba(0,0,0,0.025) 1px, transparent 1px)` × 2
6. Coarse grid (96px): `linear-gradient(rgba(0,0,0,0.015) 1px, transparent 1px)` × 2
7. `body::before` — 3 radial-gradient color spots (fixed position)
8. `body::after` — SVG fractalNoise texture at `opacity: 0.18`

### Implementation

1. Define the multi-layer background in `theme.css` on `body`
2. Define `body::before` and `body::after` pseudo-elements
3. Midnight theme: adjust gradient colors to darker tones, reduce grid opacity, adjust noise opacity
4. Use CSS custom properties for theme-dependent color values where possible

### Files Modified

- `src/assets/theme.css`

---

## P2-2: Interactive Animations

### Animations to Implement

| Animation | Target | Duration | Description |
|-----------|--------|----------|-------------|
| tint-drift | Stat card `::before` | 8s infinite | Radial glow position drift with rotate |
| tint-sweep | Stat card `::after` | 4.5s infinite | Horizontal light sweep across card top |
| card-hover | `.card` | 200ms | `translateY(-3px)` + shadow deepen |
| btn-hover | `.btn-primary`, `.btn-secondary` | 150ms | `translateY(-2px)` + shadow spread |
| toast-in | `.toast` | 300ms | `translateY(12px) → 0` + fade in |
| toast-out | `.toast` | 300ms | Reverse of toast-in |
| modal-in | `.modal` | 300ms | Scale up + fade in |

### Implementation

1. Keyframes are already defined in `design/tokens/motion.css`
2. Apply animation properties to relevant component styles
3. `prefers-reduced-motion`: all animations disabled (already in motion.css)
4. Theme store's `shouldAnimate` composable available for JS-level control

### Files Modified

- `src/components/common/StatCard.vue` — tint-drift, tint-sweep
- `src/components/common/Toast.vue` — toast-in/out
- `src/components/common/Modal.vue` — modal-in
- `src/assets/theme.css` — card-hover, btn-hover base styles
- Various View components — card hover styles

---

## P3: Theme Expansion

### Target

Expand from 2 themes to 4+ themes, with infrastructure for easy addition.

### Initial Theme Set

| ID | Name | Description |
|----|------|-------------|
| `warm` | Warm Glass | Default, frosted warm paper (existing) |
| `midnight` | Midnight | Dark glass mode (existing) |
| `sage` | Sage | Muted green glass |
| `lavender` | Lavender | Purple frosted |
| `arctic` | Arctic | Cool blue-white |

### Infrastructure

Each theme is a CSS file in `design/tokens/themes/` defining `[data-theme="xxx"]` overrides. `theme.css` imports all theme files. `theme.ts` store's `ThemeId` type extends to include new IDs.

### Theme Card UI

Settings page theme-grid shows each theme as a card with:
- 6-color preview strip (bg, surface, border, fg, accent, secondary)
- Theme name
- Active indicator (checkmark or border highlight)

### Implementation

1. Create `design/tokens/themes/sage.css`, `lavender.css`, `arctic.css`
2. `@import` them in `theme.css`
3. Extend `ThemeId` type in `theme.ts`
4. Update `SettingsView.vue` theme-grid with new themes
5. Verify each theme's glass tokens, text contrast, and overall appearance

### Files Modified

- `design/tokens/themes/sage.css` (new)
- `design/tokens/themes/lavender.css` (new)
- `design/tokens/themes/arctic.css` (new)
- `src/assets/theme.css` — add @imports
- `src/stores/theme.ts` — extend ThemeId
- `src/views/SettingsView.vue` — theme grid UI

---

## Success Criteria

1. **No hardcoded rgba(255,255,255,...)** in any component — all go through `--glass-*` tokens
2. **Both themes (warm + midnight) render correctly** with proper glass layering
3. **Background texture matches prototype** — gradients, grids, noise all visible
4. **Stat cards animate** — tint-drift and tint-sweep visible on dashboard
5. **Card/button hover effects work** — lift + shadow transitions
6. **No duplicate token definitions** between theme.css and design/tokens/
7. **All dimension tokens consistent** — no fallback values in components
8. **4+ themes switchable** from Settings page

---

## Non-Goals

- Adding new Vue components or views
- Changing business logic or Tauri commands
- Modifying the router or store architecture
- Adding new npm dependencies
- Touching the Rust backend

/**
 * Forge Design Tokens — Index & TypeScript Types
 * Single source of truth for all design tokens
 */

// ================================================
// THEME IDs
// ================================================

export type ThemeId =
  | 'warm'
  | 'cool-mist'
  | 'midnight'
  | 'sakura'
  | 'sage'
  | 'lavender'
  | 'ocean'
  | 'ember'
  | 'slate'
  | 'aurora'
  | 'cream'
  | 'arctic'
  | 'rose-gold'
  | 'cyberpunk'
  | 'forest'
  | 'desert'
  | 'cotton-candy'
  | 'charcoal'
  | 'peach'
  | 'nordic'

export type GlassVariant =
  | 'default'
  | 'cool-mist'
  | 'sage'
  | 'lavender'
  | 'ember'
  | 'slate'
  | 'arctic'
  | 'rose-gold'

// ================================================
// GLASS LAYER INTERFACE
// ================================================

export interface GlassLayers {
  window: string
  sidebar: string
  topbar: string
  bg: string
  bgHover: string
  input: string
  inputFocus: string
}

// ================================================
// TINT VARIANT INTERFACE
// ================================================

export interface TintVariant {
  name: string
  driftDelay: string
  sweepDelay: string
}

// ================================================
// THEME TOKEN SETS
// ================================================

export interface ThemeTokens {
  id: ThemeId
  name: string
  baseline: 'light' | 'dark'
  colors: {
    bg: string
    fg: string
    accent: string
    success: string
    error: string
    info: string
    warn: string
    border: string
    borderHover: string
  }
  glass: GlassLayers
  tint: {
    warm: string
    cool: string
    soft: string
    amber: string
  }
}

// ================================================
// GLASS VARIANT INFO
// ================================================

export interface GlassVariantInfo {
  id: GlassVariant
  name: string
  desc: string
}

// ================================================
// PRE-DEFINED GLASS VARIANTS (API only, PM-D5)
// ================================================

export const GLASS_VARIANTS: GlassVariantInfo[] = [
  { id: 'default', name: 'Default', desc: 'Standard warm glass' },
  { id: 'cool-mist', name: 'Cool Mist', desc: 'Blue-grey frost' },
  { id: 'sage', name: 'Sage', desc: 'Muted green glass' },
  { id: 'lavender', name: 'Lavender', desc: 'Purple frosted' },
  { id: 'ember', name: 'Ember', desc: 'Warm amber glow' },
  { id: 'slate', name: 'Slate', desc: 'Neutral stone' },
  { id: 'arctic', name: 'Arctic', desc: 'Cool blue-white' },
  { id: 'rose-gold', name: 'Rose Gold', desc: 'Metallic warm pink' },
]

// ================================================
// TINT VARIANTS (with staggered delays)
// ================================================

export const TINT_VARIANTS: Record<string, TintVariant> = {
  warm: { name: 'Warm', driftDelay: '0s', sweepDelay: '0s' },
  cool: { name: 'Cool', driftDelay: '-2s', sweepDelay: '-1.5s' },
  soft: { name: 'Soft', driftDelay: '-4s', sweepDelay: '-3s' },
  amber: { name: 'Amber', driftDelay: '-6s', sweepDelay: '-4.5s' },
}

// ================================================
// CSS VARIABLE NAMES (for dynamic injection)
// ================================================

export const CSS_VAR = {
  // Background
  BG: '--bg',
  BG_CARD: '--bg-card',

  // Text
  FG: '--fg',
  FG_TITLE: '--fg-title',
  FG_MUTED: '--fg-muted',
  FG_GHOST: '--fg-ghost',

  // Accent
  ACCENT: '--accent',
  ACCENT_HOVER: '--accent-hover',

  // Semantic
  SUCCESS: '--success',
  ERROR: '--error',
  INFO: '--info',
  WARN: '--warn',

  // Borders
  BORDER: '--border',
  BORDER_HOVER: '--border-hover',

  // Glass
  GLASS_WINDOW: '--glass-window',
  GLASS_SIDEBAR: '--glass-sidebar',
  GLASS_TOPBAR: '--glass-topbar',
  GLASS_BG: '--glass-bg',
  GLASS_BG_HOVER: '--glass-bg-hover',
  GLASS_INPUT: '--glass-input',
  GLASS_INPUT_FOCUS: '--glass-input-focus',

  // Liquid Glass Sheen
  GLASS_SHEEN: '--glass-sheen',
  GLASS_SHEEN_HOVER: '--glass-sheen-hover',
  GLASS_SHEEN_ACTIVE: '--glass-sheen-active',

  // Tint
  TINT_WARM: '--tint-warm',
  TINT_COOL: '--tint-cool',
  TINT_SOFT: '--tint-soft',
  TINT_AMBER: '--tint-amber',

  // Layout
  SIDEBAR_W: '--sidebar-w',
  TOPBAR_H: '--topbar-h',

  // Radius
  RADIUS: '--radius',
  RADIUS_SM: '--radius-sm',
  RADIUS_LG: '--radius-lg',
  RADIUS_XL: '--radius-xl',
} as const

// ================================================
// STORAGE KEYS
// ================================================

export const STORAGE_KEYS = {
  THEME: 'forge-theme',
  GLASS_VARIANT: 'forge-glass-variant',
} as const

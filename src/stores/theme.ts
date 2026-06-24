/**
 * Forge Theme Store (Pinia)
 * Single source of truth for theme state management
 *
 * Tech-spec §3.1
 * PM-D4: localStorage key = 'forge-theme'
 * PM-D5: setGlassVariant() API only (not exposed to UI)
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ThemeId, GlassVariant } from '@/assets/tokens'

// Type exports for external use
export type { ThemeId, GlassVariant }

// ================================================
// STORAGE KEYS (PM-D4)
// ================================================

const THEME_STORAGE_KEY = 'forge-theme'
const GLASS_VARIANT_STORAGE_KEY = 'forge-glass-variant'

// ================================================
// GLASS LAYER BASE VALUES
// ================================================

const GLASS_BASES = {
  light: {
    window: 'rgba(255, 255, 255, 0.25)',
    sidebar: 'rgba(255, 255, 255, 0.35)',
    topbar: 'rgba(255, 255, 255, 0.38)',
    bg: 'rgba(255, 255, 255, 0.45)',
    bgHover: 'rgba(255, 255, 255, 0.58)',
    input: 'rgba(255, 255, 255, 0.40)',
    inputFocus: 'rgba(255, 255, 255, 0.60)',
  },
  dark: {
    window: 'rgba(255, 255, 255, 0.03)',
    sidebar: 'rgba(255, 255, 255, 0.04)',
    topbar: 'rgba(255, 255, 255, 0.04)',
    bg: 'rgba(255, 255, 255, 0.06)',
    bgHover: 'rgba(255, 255, 255, 0.10)',
    input: 'rgba(255, 255, 255, 0.08)',
    inputFocus: 'rgba(255, 255, 255, 0.12)',
  },
} as const

// ================================================
// STORE DEFINITION
// ================================================

export const useThemeStore = defineStore('theme', () => {
  // --- State ---
  const activeThemeId = ref<ThemeId>('warm')
  const activeGlassVariant = ref<GlassVariant | null>(null)
  const prefersReducedMotion = ref(false)

  // --- Getters ---
  const isDarkMode = computed(() => ['midnight', 'cyberpunk', 'charcoal'].includes(activeThemeId.value))
  const isLightMode = computed(() => activeThemeId.value !== 'midnight')
  const shouldAnimate = computed(() => !prefersReducedMotion.value)

  // Current glass background value (for dynamic usage)
  const currentGlassBg = computed(() => {
    const base = isDarkMode.value ? GLASS_BASES.dark : GLASS_BASES.light
    return base.bg
  })

  // --- Internal: Sync theme attribute (CSS is the single source of truth for glass variables) ---
  function applyThemeAttribute(themeId: ThemeId) {
    if (typeof document === 'undefined') return
    document.documentElement.setAttribute('data-theme', themeId)
  }

  // --- Actions ---
  function setTheme(themeId: ThemeId) {
    activeThemeId.value = themeId
    localStorage.setItem(THEME_STORAGE_KEY, themeId)
    applyThemeAttribute(themeId)
  }

  /**
   * Set glass variant (API only - PM-D5)
   * Not exposed to UI in this iteration
   */
  function setGlassVariant(variant: GlassVariant | null) {
    activeGlassVariant.value = variant
    localStorage.setItem(GLASS_VARIANT_STORAGE_KEY, variant ?? 'default')
    applyThemeAttribute(activeThemeId.value)
  }

  function initTheme() {
    // Restore saved theme
    const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as ThemeId | null
    const VALID_THEMES: ThemeId[] = [
      'warm', 'cool-mist', 'midnight', 'sakura', 'sage', 'lavender',
      'ocean', 'ember', 'slate', 'aurora', 'cream', 'arctic',
      'rose-gold', 'cyberpunk', 'forest', 'desert', 'cotton-candy',
      'charcoal', 'peach', 'nordic'
    ]
    if (savedTheme && VALID_THEMES.includes(savedTheme)) {
      setTheme(savedTheme)
    } else {
      setTheme('warm') // Default to warm
    }

    // Restore glass variant if saved
    const savedVariant = localStorage.getItem(GLASS_VARIANT_STORAGE_KEY)
    if (savedVariant && savedVariant !== 'default') {
      setGlassVariant(savedVariant as GlassVariant)
    }

    // Listen for prefers-reduced-motion
    if (typeof window !== 'undefined' && window.matchMedia) {
      const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
      prefersReducedMotion.value = mediaQuery.matches

      mediaQuery.addEventListener('change', (e) => {
        prefersReducedMotion.value = e.matches
      })
    }
  }

  return {
    // State
    activeThemeId,
    activeGlassVariant,
    prefersReducedMotion,
    // Getters
    isDarkMode,
    isLightMode,
    shouldAnimate,
    currentGlassBg,
    // Actions
    setTheme,
    setGlassVariant,
    initTheme,
  }
})

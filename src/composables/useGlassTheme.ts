/**
 * useGlassTheme — Thin wrapper around theme store
 *
 * PM-D1: All hardcoded glass values removed.
 * Now reads from CSS variables (single source of truth).
 * API形态保留 setGlassVariant() but not exposed to UI.
 */

import { computed } from 'vue'
import { useThemeStore } from '@/stores/theme'
import { GLASS_VARIANTS } from '@/assets/tokens'

export function useGlassTheme() {
  const store = useThemeStore()

  // Getters — read from store (which reads CSS variables)
  const isGlassTheme = computed(() => store.activeThemeId === 'warm')
  const isMidnightMode = computed(() => store.activeThemeId === 'midnight')
  const isDarkMode = computed(() => store.isDarkMode)
  const currentGlassBg = computed(() => store.currentGlassBg)

  // Backward compatibility: isWindowRounded always true for warm theme
  const isWindowRounded = computed(() => store.activeThemeId === 'warm')

  /**
   * Set glass variant (API only - PM-D5)
   * Note: UI does NOT expose this in FEAT-024-C
   */
  function setGlassVariant(variantId: string | null) {
    const validVariants = ['default', 'cool-mist', 'sage', 'lavender', 'ember', 'slate', 'arctic', 'rose-gold']
    const variant = variantId && validVariants.includes(variantId) ? variantId as any : null
    store.setGlassVariant(variant)
  }

  /**
   * Restore variant from localStorage (backward compat)
   */
  function restoreVariant() {
    const saved = localStorage.getItem('forge-glass-variant')
    if (saved && saved !== 'default') {
      setGlassVariant(saved)
    }
  }

  return {
    // Getters
    isGlassTheme,
    isMidnightMode,
    isDarkMode,
    currentGlassBg,
    // Backward compatibility
    isWindowRounded,
    // API
    setGlassVariant,
    restoreVariant,
    // Variants (API only, PM-D5)
    variants: GLASS_VARIANTS,
  }
}

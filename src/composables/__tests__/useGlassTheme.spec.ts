import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useThemeStore } from '@/stores/theme';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('useThemeStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    vi.clearAllMocks();
    vi.spyOn(localStorage, 'setItem').mockImplementation(() => {});
    vi.spyOn(localStorage, 'getItem').mockReturnValue(null);
  });

  it('defaults to warm theme', () => {
    const store = useThemeStore();
    expect(store.activeThemeId).toBe('warm');
  });

  it('isDarkMode returns true for midnight', () => {
    const store = useThemeStore();
    store.setTheme('midnight');
    expect(store.isDarkMode).toBe(true);
  });

  it('isDarkMode returns true for cyberpunk', () => {
    const store = useThemeStore();
    store.setTheme('cyberpunk');
    expect(store.isDarkMode).toBe(true);
  });

  it('isDarkMode returns false for warm', () => {
    const store = useThemeStore();
    store.setTheme('warm');
    expect(store.isDarkMode).toBe(false);
  });

  it('isLightMode returns false for midnight', () => {
    const store = useThemeStore();
    store.setTheme('midnight');
    expect(store.isLightMode).toBe(false);
  });

  it('isLightMode returns true for non-midnight themes', () => {
    const store = useThemeStore();
    store.setTheme('warm');
    expect(store.isLightMode).toBe(true);
  });

  it('setTheme calls localStorage.setItem', () => {
    const store = useThemeStore();
    store.setTheme('cool-mist');
    expect(localStorage.setItem).toHaveBeenCalledWith('forge-theme', 'cool-mist');
  });

  it('currentGlassBg returns light base for light mode', () => {
    const store = useThemeStore();
    store.setTheme('warm');
    expect(store.currentGlassBg).toMatch(/rgba\(255, 255, 255/);
  });

  it('setGlassVariant calls localStorage.setItem', () => {
    const store = useThemeStore();
    store.setGlassVariant('sage');
    expect(localStorage.setItem).toHaveBeenCalledWith('forge-glass-variant', 'sage');
  });

  it('initTheme restores saved theme from localStorage', () => {
    vi.spyOn(localStorage, 'getItem').mockImplementation((key) => {
      if (key === 'forge-theme') return 'cool-mist';
      return null;
    });
    const store = useThemeStore();
    store.initTheme();
    expect(store.activeThemeId).toBe('cool-mist');
  });

  it('initTheme ignores invalid theme values', () => {
    vi.spyOn(localStorage, 'getItem').mockImplementation((key) => {
      if (key === 'forge-theme') return 'not-a-real-theme';
      return null;
    });
    const store = useThemeStore();
    store.initTheme();
    expect(store.activeThemeId).toBe('warm');
  });
});

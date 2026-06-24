import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUiStore } from '@/stores/ui';

// Mock localStorage for node environment
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = String(value);
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    get length() {
      return Object.keys(store).length;
    },
    key: vi.fn((index: number) => Object.keys(store)[index] ?? null),
  };
})();

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  it('defaults sidebarCollapsed to false', () => {
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(false);
  });

  it('toggleSidebar flips the state', () => {
    const store = useUiStore();
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(true);
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(false);
  });

  it('persists sidebarCollapsed to localStorage', () => {
    const store = useUiStore();
    store.toggleSidebar();
    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      'forge-sidebar-collapsed',
      'true'
    );
  });

  it('restores sidebarCollapsed from localStorage on init', () => {
    // Pre-populate localStorage before creating the store
    localStorageMock.getItem.mockReturnValueOnce('true');
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(true);
  });

  it('handles invalid localStorage value gracefully', () => {
    localStorageMock.getItem.mockReturnValueOnce('invalid');
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(false);
  });
});

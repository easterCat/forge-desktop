import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useAppStore } from '@/stores/app';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@/stores/software', () => ({
  useSoftwareStore: vi.fn(() => ({
    fetchCliTools: vi.fn().mockResolvedValue(undefined),
    checkAllCliToolsStatus: vi.fn().mockResolvedValue(undefined),
  })),
}));

describe('useAppStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('defaults to dashboard view and empty error', () => {
    const store = useAppStore();
    expect(store.activeView).toBe('dashboard');
    expect(store.error).toBeNull();
    expect(store.isLoading).toBe(false);
    expect(store.hasError).toBe(false);
  });

  it('hasError is true when error is set', () => {
    const store = useAppStore();
    store.setError('some error');
    expect(store.hasError).toBe(true);
  });

  it('setLoading updates isLoading', () => {
    const store = useAppStore();
    store.setLoading(true);
    expect(store.isLoading).toBe(true);
    store.setLoading(false);
    expect(store.isLoading).toBe(false);
  });

  it('setError updates error', () => {
    const store = useAppStore();
    store.setError('test error');
    expect(store.error).toBe('test error');
  });

  it('clearError sets error to null', () => {
    const store = useAppStore();
    store.setError('test error');
    store.clearError();
    expect(store.error).toBeNull();
  });

  it('toggleSidebar flips sidebarCollapsed', () => {
    const store = useAppStore();
    expect(store.sidebarCollapsed).toBe(false);
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(true);
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(false);
  });

  it('setActiveView updates activeView', () => {
    const store = useAppStore();
    store.setActiveView('plugins');
    expect(store.activeView).toBe('plugins');
  });

  it('setSearchQuery updates searchQuery', () => {
    const store = useAppStore();
    store.setSearchQuery('claude');
    expect(store.searchQuery).toBe('claude');
  });

  it('initialize completes without error', async () => {
    const store = useAppStore();
    await store.initialize();
    expect(store.error).toBeNull();
    expect(store.isLoading).toBe(false);
  });

  it('initialize sets error on failure', async () => {
    const { useSoftwareStore } = await import('@/stores/software');
    vi.mocked(useSoftwareStore).mockReturnValueOnce({
      fetchCliTools: vi.fn().mockRejectedValue(new Error('init failed')),
      checkAllCliToolsStatus: vi.fn(),
    } as any);
    const store = useAppStore();
    await store.initialize();
    expect(store.error).toBe('init failed');
    expect(store.isLoading).toBe(false);
  });
});

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore } from '@/stores/settings';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.mocked(invoke).mockReset();
  });

  it('defaults to no token configured', () => {
    const store = useSettingsStore();
    expect(store.hasToken).toBe(false);
    expect(store.tokenPreview).toBeNull();
    expect(store.isLoading).toBe(false);
    expect(store.error).toBeNull();
  });

  it('isConfigured computed reflects hasToken', () => {
    const store = useSettingsStore();
    expect(store.isConfigured).toBe(false);
  });

  describe('refresh', () => {
    it('calls has_github_token and get_github_token_preview', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(true)
        .mockResolvedValueOnce('ghp_****xyz');
      const store = useSettingsStore();
      await store.refresh();
      expect(invoke).toHaveBeenCalledWith('has_github_token');
      expect(invoke).toHaveBeenCalledWith('get_github_token_preview');
    });

    it('sets hasToken and tokenPreview on success', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(true)
        .mockResolvedValueOnce('ghp_****xyz');
      const store = useSettingsStore();
      await store.refresh();
      expect(store.hasToken).toBe(true);
      expect(store.tokenPreview).toBe('ghp_****xyz');
    });

    it('sets hasToken to false on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('api error'));
      const store = useSettingsStore();
      await store.refresh();
      expect(store.hasToken).toBe(false);
      expect(store.tokenPreview).toBeNull();
      expect(store.error).toBeTruthy();
    });
  });

  describe('saveToken', () => {
    it('throws if token is empty', async () => {
      const store = useSettingsStore();
      await expect(store.saveToken('')).rejects.toThrow('Token cannot be empty');
    });

    it('throws if token is whitespace', async () => {
      const store = useSettingsStore();
      await expect(store.saveToken('   ')).rejects.toThrow('Token cannot be empty');
    });

    it('calls set_github_token and refreshes on success', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(true)
        .mockResolvedValueOnce('ghp_****xyz');
      const store = useSettingsStore();
      await store.saveToken('ghp_validtoken123');
      expect(invoke).toHaveBeenCalledWith('set_github_token', { token: 'ghp_validtoken123' });
    });

    it('throws on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('save failed'));
      const store = useSettingsStore();
      await expect(store.saveToken('bad token')).rejects.toThrow();
      expect(store.error).toBe('save failed');
    });
  });

  describe('clearToken', () => {
    it('calls clear_github_token and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce(false)
        .mockResolvedValueOnce(null);
      const store = useSettingsStore();
      await store.clearToken();
      expect(invoke).toHaveBeenCalledWith('clear_github_token');
    });

    it('throws on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('clear failed'));
      const store = useSettingsStore();
      await expect(store.clearToken()).rejects.toThrow();
      expect(store.error).toBe('clear failed');
    });
  });
});

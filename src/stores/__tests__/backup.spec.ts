import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useBackupStore } from '../backup';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('useBackupStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('starts with empty backups and no loading', () => {
    const store = useBackupStore();
    expect(store.backups).toEqual([]);
    expect(store.isLoading).toBe(false);
    expect(store.error).toBeNull();
  });

  describe('fetchBackups', () => {
    it('sets isLoading during fetch', async () => {
      let resolve: (v: any) => void;
      vi.mocked(invoke).mockImplementation(() => new Promise(r => (resolve = r)));
      const store = useBackupStore();
      const p = store.fetchBackups();
      expect(store.isLoading).toBe(true);
      resolve!([]);
      await p;
      expect(store.isLoading).toBe(false);
    });

    it('populates backups on success', async () => {
      vi.mocked(invoke).mockResolvedValue([
        { id: 'b1', name: 'backup1', createdAt: '2024-01-01', size: 1024 },
        { id: 'b2', name: 'backup2', createdAt: '2024-01-02', size: 2048 },
      ]);
      const store = useBackupStore();
      await store.fetchBackups();
      expect(store.backups).toHaveLength(2);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('db error'));
      const store = useBackupStore();
      await store.fetchBackups();
      expect(store.error).toBe('db error');
      expect(store.isLoading).toBe(false);
    });
  });

  describe('createBackup', () => {
    it('adds new backup to top of list', async () => {
      vi.mocked(invoke).mockResolvedValue({ id: 'b2', name: 'new backup', createdAt: '2024-01-03', size: 512, path: '/b2', fileCount: 1, includes: 'plugins' });
      const store = useBackupStore();
      store.backups = [{ id: 'b1', name: 'old', createdAt: '2024-01-01', size: 1024, path: '/b1', fileCount: 1, includes: 'plugins' }];
      await store.createBackup('new backup', ['plugins']);
      expect(store.backups[0].id).toBe('b2');
      expect(store.backups).toHaveLength(2);
    });

    it('throws and sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('create failed'));
      const store = useBackupStore();
      await expect(store.createBackup('name', [])).rejects.toThrow();
      expect(store.error).toBe('create failed');
    });
  });

  describe('restoreBackup', () => {
    it('throws on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('restore failed'));
      const store = useBackupStore();
      await expect(store.restoreBackup('b1')).rejects.toThrow();
      expect(store.error).toBe('restore failed');
    });
  });

  describe('deleteBackup', () => {
    it('removes from list on success', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      const store = useBackupStore();
      store.backups = [
        { id: 'b1', name: 'backup1', createdAt: '2024-01-01', size: 1024, path: '/b1', fileCount: 1, includes: 'plugins' },
        { id: 'b2', name: 'backup2', createdAt: '2024-01-02', size: 2048, path: '/b2', fileCount: 2, includes: 'skills' },
      ];
      await store.deleteBackup('b1');
      expect(store.backups).toHaveLength(1);
      expect(store.backups[0].id).toBe('b2');
    });

    it('throws on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('delete failed'));
      const store = useBackupStore();
      store.backups = [{ id: 'b1', name: 'backup1', createdAt: '2024-01-01', size: 1024, path: '/b1', fileCount: 1, includes: 'plugins' }];
      await expect(store.deleteBackup('b1')).rejects.toThrow();
      expect(store.error).toBe('delete failed');
    });
  });

  describe('formatSize', () => {
    it('returns 0 B for null', () => {
      const store = useBackupStore();
      expect(store.formatSize(null)).toBe('0 B');
    });

    it('returns 0 B for 0', () => {
      const store = useBackupStore();
      expect(store.formatSize(0)).toBe('0 B');
    });

    it('formats bytes', () => {
      const store = useBackupStore();
      expect(store.formatSize(512)).toBe('512 B');
    });

    it('formats KB', () => {
      const store = useBackupStore();
      expect(store.formatSize(1024)).toBe('1 KB');
    });

    it('formats MB', () => {
      const store = useBackupStore();
      expect(store.formatSize(1024 * 1024)).toBe('1 MB');
    });

    it('formats GB', () => {
      const store = useBackupStore();
      expect(store.formatSize(1024 * 1024 * 1024)).toBe('1 GB');
    });

    it('shows 2 decimal places', () => {
      const store = useBackupStore();
      const result = store.formatSize(1536);
      expect(result).toMatch(/1\.\d+ KB/);
    });
  });
});

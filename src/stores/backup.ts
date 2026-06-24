import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { BackupRecord } from '@/types';

export const useBackupStore = defineStore('backup', () => {
  const backups = ref<BackupRecord[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchBackups() {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<BackupRecord[]>('get_backups');
      backups.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch backups';
    } finally {
      isLoading.value = false;
    }
  }

  async function createBackup(
    name: string,
    includes: string[]
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const backup = await invoke<BackupRecord>('create_backup', {
        name,
        includes,
      });
      backups.value.unshift(backup);
      return backup;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create backup';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function restoreBackup(backupId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('restore_backup', { backupId });
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to restore backup';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteBackup(backupId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_backup', { backupId });
      backups.value = backups.value.filter(b => b.id !== backupId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete backup';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  function formatSize(bytes: number | null | undefined): string {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  return {
    backups,
    isLoading,
    error,
    fetchBackups,
    createBackup,
    restoreBackup,
    deleteBackup,
    formatSize,
  };
});

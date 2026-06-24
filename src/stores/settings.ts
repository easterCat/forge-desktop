// App-level settings (currently only the GitHub personal access token
// used to raise the GitHub REST API rate-limit when fetching / installing
// skills from Anthropic / Composio / other remote sources).

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  // State
  // We never store the raw token in the frontend. The Rust side returns
  // either `true` / `false` (whether a token is configured) and a masked
  // preview of the form "****abcd" so the UI can confirm what's loaded.
  const hasToken = ref(false);
  const tokenPreview = ref<string | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const isConfigured = computed(() => hasToken.value);

  async function refresh(): Promise<void> {
    isLoading.value = true;
    error.value = null;
    try {
      // Two cheap round-trips; called once on entering the Settings view.
      const [configured, preview] = await Promise.all([
        invoke<boolean>('has_github_token'),
        invoke<string | null>('get_github_token_preview'),
      ]);
      hasToken.value = configured;
      tokenPreview.value = preview;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      hasToken.value = false;
      tokenPreview.value = null;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveToken(token: string): Promise<void> {
    const trimmed = token.trim();
    if (!trimmed) {
      error.value = 'Token cannot be empty';
      throw new Error(error.value);
    }
    isLoading.value = true;
    error.value = null;
    try {
      await invoke<void>('set_github_token', { token: trimmed });
      await refresh();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function clearToken(): Promise<void> {
    isLoading.value = true;
    error.value = null;
    try {
      await invoke<void>('clear_github_token');
      await refresh();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    hasToken,
    tokenPreview,
    isLoading,
    error,
    isConfigured,
    refresh,
    saveToken,
    clearToken,
  };
});

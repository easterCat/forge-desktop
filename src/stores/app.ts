import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSoftwareStore } from './software';

export const useAppStore = defineStore('app', () => {
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const sidebarCollapsed = ref(false);
  const activeView = ref('dashboard');
  const searchQuery = ref('');

  const hasError = computed(() => error.value !== null);

  function setLoading(loading: boolean) {
    isLoading.value = loading;
  }

  function setError(err: string | null) {
    error.value = err;
  }

  function clearError() {
    error.value = null;
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function setActiveView(view: string) {
    activeView.value = view;
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query;
  }

  async function initialize() {
    console.log('[appStore] initialize called');
    try {
      setLoading(true);
      clearError();
      const softwareStore = useSoftwareStore();
      console.log('[appStore] About to call fetchCliTools...');
      await softwareStore.fetchCliTools();
      console.log('[appStore] About to call checkAllCliToolsStatus...');
      await softwareStore.checkAllCliToolsStatus();
      console.log('[appStore] initialize complete');
    } catch (e) {
      console.error('[appStore] initialize error:', e);
      setError(e instanceof Error ? e.message : 'Failed to initialize app');
    } finally {
      setLoading(false);
    }
  }

  return {
    isLoading,
    error,
    sidebarCollapsed,
    activeView,
    searchQuery,
    hasError,
    setLoading,
    setError,
    clearError,
    toggleSidebar,
    setActiveView,
    setSearchQuery,
    initialize,
  };
});

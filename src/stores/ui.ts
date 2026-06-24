import { defineStore } from 'pinia';
import { ref } from 'vue';

const STORAGE_KEY = 'forge-sidebar-collapsed';

export const useUiStore = defineStore('ui', () => {
  // --- State ---
  const sidebarCollapsed = ref(false);

  // --- Actions ---
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
    try {
      localStorage.setItem(STORAGE_KEY, String(sidebarCollapsed.value));
    } catch {
      // Silently ignore localStorage write failures
    }
  }

  // --- Init: restore from localStorage ---
  function initUi() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved === 'true') {
        sidebarCollapsed.value = true;
      }
    } catch {
      // Silently ignore localStorage read failures
    }
  }

  // Auto-init on store creation
  initUi();

  return {
    sidebarCollapsed,
    toggleSidebar,
    initUi,
  };
});

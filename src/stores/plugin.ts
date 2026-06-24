import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Plugin } from '@/types';

export const usePluginStore = defineStore('plugin', () => {
  const plugins = ref<Plugin[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchPlugins(softwareId: string = '') {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Plugin[]>('get_plugins', { softwareId });
      plugins.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch plugins';
    } finally {
      isLoading.value = false;
    }
  }

  async function installPlugin(plugin: Partial<Plugin> & { softwareId: string; name: string }) {
    try {
      isLoading.value = true;
      error.value = null;
      const now = new Date().toISOString();
      const newPlugin: Plugin = {
        id: crypto.randomUUID(),
        softwareId: plugin.softwareId,
        name: plugin.name,
        version: plugin.version || '',
        author: plugin.author || '',
        description: plugin.description || '',
        installedPath: plugin.installedPath || '',
        enabled: true,
        installedAt: now,
        lastUpdated: now,
      };
      await invoke('install_plugin', { plugin: newPlugin });
      plugins.value.push(newPlugin);
      return newPlugin;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to install plugin';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function uninstallPlugin(pluginId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('uninstall_plugin', { pluginId });
      plugins.value = plugins.value.filter(p => p.id !== pluginId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to uninstall plugin';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function togglePlugin(pluginId: string, enabled: boolean) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('toggle_plugin', { pluginId, enabled });
      const plugin = plugins.value.find(p => p.id === pluginId);
      if (plugin) {
        plugin.enabled = enabled;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to toggle plugin';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  function getPluginsBySoftware(softwareId: string): Plugin[] {
    return plugins.value.filter(p => p.softwareId === softwareId);
  }

  return {
    plugins,
    isLoading,
    error,
    fetchPlugins,
    installPlugin,
    uninstallPlugin,
    togglePlugin,
    getPluginsBySoftware,
  };
});

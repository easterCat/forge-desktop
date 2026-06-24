import { defineStore } from 'pinia';
import { ref, shallowRef } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Software } from '@/types';

// Development-only logging to avoid production blocking
const DEBUG = import.meta.env.DEV;
function debugLog(...args: unknown[]) {
  if (DEBUG) console.log('[softwareStore]', ...args);
}

// Idle callback helper for non-critical updates
function runWhenIdle(callback: () => void, timeout = 100) {
  if ('requestIdleCallback' in window) {
    requestIdleCallback(callback, { timeout });
  } else {
    setTimeout(callback, timeout);
  }
}

export interface CliToolInfo {
  id: string;
  key: string;
  name: string;
  icon: string;
  description: string;
  installMethods: { method: string; command: string; priority: number }[];
  npmPackage?: string;
  websiteUrl?: string;
}

export interface CliToolStatus {
  toolKey: string;
  isInstalled: boolean;
  installedVersion: string | null;
  installMethod: string | null;
  installPath: string | null;
  hasConflict: boolean;
  conflictInfo: string | null;
  latestVersion: string | null;
  needsUpgrade: boolean;
}

export interface UpgradeResult {
  success: boolean;
  message: string;
  newVersion: string | null;
  method: string;
}

export interface InstallResponse {
  success: boolean;
  message: string;
  installedVersion: string | null;
}

export interface UninstallResponse {
  success: boolean;
  message: string;
  needsManual: boolean;
  manualCommands: string[];
}

export interface UpdateCheckResult {
  success: boolean;
  hasUpdate: boolean;
  message: string;
  newVersion: string | null;
}

export const useSoftwareStore = defineStore('software', () => {
  const softwareList = ref<Software[]>([]);
  const cliTools = ref<CliToolInfo[]>([]);
  // Use shallowRef for Map to avoid deep reactivity tracking
  const cliToolStatuses = ref<Record<string, CliToolStatus>>({});
  const isLoading = ref(false);
  const isCheckingStatus = ref(false);
  const isUpgrading = ref(false);
  const isInstalling = ref(false);
  const isUninstalling = ref(false);
  const error = ref<string | null>(null);

  // Platform filter state
  const selectedPlatform = ref<string>(
    localStorage.getItem('forge-selected-platform') || 'auto'
  );

  function setSelectedPlatform(platform: string) {
    selectedPlatform.value = platform;
    localStorage.setItem('forge-selected-platform', platform);
  }

  async function fetchSoftware() {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Software[]>('get_software_list');
      softwareList.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch software';
    } finally {
      isLoading.value = false;
    }
  }

  async function detectSoftware() {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Software[]>('detect_software');
      softwareList.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to detect software';
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchCliTools() {
    try {
      isLoading.value = true;
      error.value = null;
      debugLog('Fetching CLI tools from Rust...');
      const result = await invoke<CliToolInfo[]>('get_cli_tools');
      cliTools.value = result;
      debugLog('Loaded', result.length, 'CLI tools');
    } catch (e) {
      debugLog('Error:', e);
      error.value = e instanceof Error ? e.message : 'Failed to fetch CLI tools';
    } finally {
      isLoading.value = false;
    }
  }

  async function checkCliToolStatus(toolKey: string) {
    try {
      const result = await invoke<CliToolStatus>('check_cli_tool_status', { toolKey });
      cliToolStatuses.value = { ...cliToolStatuses.value, [toolKey]: result };
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to check tool status';
      throw e;
    }
  }

  async function checkAllCliToolsStatus() {
    try {
      isCheckingStatus.value = true;
      error.value = null;
      debugLog('Checking all CLI tools status (parallel async)...');
      // Use the parallel async version to avoid blocking the main thread
      // This executes all tool checks concurrently instead of sequentially
      const result = await invoke<CliToolStatus[]>('check_all_cli_tools_status_parallel');
      // Batch update: build new Record, then assign once to trigger single reactive update
      const next: Record<string, CliToolStatus> = {};
      for (const status of result) {
        next[status.toolKey] = status;
      }
      cliToolStatuses.value = next;
      debugLog('Status check complete:', result.length, 'tools');
    } catch (e) {
      debugLog('Failed:', e);
      error.value = e instanceof Error ? e.message : 'Failed to check all tools status';
    } finally {
      isCheckingStatus.value = false;
    }
  }

  async function upgradeCliTool(toolKey: string, method: string) {
    try {
      isUpgrading.value = true;
      error.value = null;
      const result = await invoke<UpgradeResult>('upgrade_cli_tool', { toolKey, method });
      if (result.success) {
        await checkCliToolStatus(toolKey);
      }
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to upgrade tool';
      throw e;
    } finally {
      isUpgrading.value = false;
    }
  }

  async function installSoftware(softwareKey: string) {
    try {
      isInstalling.value = true;
      error.value = null;
      const result = await invoke<InstallResponse>('install_software', { softwareKey });
      if (result.success) {
        // Defer software detection to avoid blocking UI
        runWhenIdle(() => detectSoftware());
      }
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to install software';
      throw e;
    } finally {
      isInstalling.value = false;
    }
  }

  async function uninstallSoftware(softwareKey: string) {
    try {
      isUninstalling.value = true;
      error.value = null;
      const result = await invoke<UninstallResponse>('uninstall_software', { softwareKey });
      if (result.success) {
        // After uninstall, re-check status to confirm the tool is truly gone.
        // npm/brew may need a moment to fully remove symlinks, so retry with
        // back-off until the backend confirms isInstalled = false.
        await confirmUninstalled(softwareKey);
        runWhenIdle(() => detectSoftware());
      }
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to uninstall software';
      throw e;
    } finally {
      isUninstalling.value = false;
    }
  }

  /**
   * After a successful uninstall command, poll the status until the backend
   * confirms the tool is no longer installed. Retries up to 3 times with
   * increasing delays (500ms, 1000ms, 2000ms) to handle npm/brew cleanup lag.
   * Uses a generation counter to prevent concurrent chains from racing.
   */
  const uninstallGenerations = ref<Record<string, number>>({});

  async function confirmUninstalled(toolKey: string, attempt = 0) {
    const delays = [500, 1000, 2000];
    if (attempt >= delays.length) return;

    // Bump generation to cancel any prior chain for this key
    const gen = (uninstallGenerations.value[toolKey] ?? 0) + 1;
    uninstallGenerations.value = { ...uninstallGenerations.value, [toolKey]: gen };

    await new Promise(resolve => setTimeout(resolve, delays[attempt]));

    // If a newer chain started, abort this one
    if (uninstallGenerations.value[toolKey] !== gen) return;

    try {
      const status = await invoke<CliToolStatus>('check_cli_tool_status', { toolKey });

      // Re-check generation after async invoke
      if (uninstallGenerations.value[toolKey] !== gen) return;

      // Clear latestVersion when tool is not installed
      if (!status.isInstalled) {
        status.latestVersion = null;
      }
      cliToolStatuses.value = { ...cliToolStatuses.value, [toolKey]: status };
      // If still showing installed and retries remain, try again
      if (status.isInstalled && attempt + 1 < delays.length) {
        await confirmUninstalled(toolKey, attempt + 1);
      }
    } catch (e) {
      debugLog('Post-uninstall status check failed:', e);
    }
  }

  function getSoftwareById(id: string): Software | undefined {
    return softwareList.value.find(s => s.id === id);
  }

  function getSoftwareByKey(key: string): Software | undefined {
    return softwareList.value.find(s => s.key === key);
  }

  function getCliToolByKey(key: string): CliToolInfo | undefined {
    return cliTools.value.find(t => t.key === key);
  }

  function getCliToolStatus(key: string): CliToolStatus | undefined {
    return cliToolStatuses.value[key];
  }

  async function updateSoftware(softwareKey: string): Promise<UpdateCheckResult> {
    try {
      isInstalling.value = true;
      error.value = null;
      const result = await invoke<UpdateCheckResult>('update_software', { softwareKey });
      if (result.success) {
        runWhenIdle(() => detectSoftware());
      }
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update software';
      throw e;
    } finally {
      isInstalling.value = false;
    }
  }

  return {
    softwareList,
    cliTools,
    cliToolStatuses,
    isLoading,
    isCheckingStatus,
    isUpgrading,
    isInstalling,
    isUninstalling,
    error,
    fetchSoftware,
    detectSoftware,
    fetchCliTools,
    checkCliToolStatus,
    checkAllCliToolsStatus,
    upgradeCliTool,
    installSoftware,
    uninstallSoftware,
    getSoftwareById,
    getSoftwareByKey,
    getCliToolByKey,
    getCliToolStatus,
    selectedPlatform,
    setSelectedPlatform,
    updateSoftware,
  };
});

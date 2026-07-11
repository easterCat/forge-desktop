import { defineStore } from 'pinia';
import { ref } from 'vue';
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
  color?: string;
  installed?: boolean;
  needsUpdate?: boolean;
  pkg?: string;
  desc?: string;
  current?: string;
  latest?: string;
  /** Where this tool comes from: 'allagents' (23 tools) | 'custom' (user added) */
  displaySource?: 'allagents' | 'custom';
  /** If true, this tool requires manual download from website (no quick-install) */
  manualDownloadOnly?: boolean;
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
  /** Allagents 23 tools for the Default tab */
  const allagentsTools = ref<CliToolInfo[]>([]);
  // Use shallowRef for Map to avoid deep reactivity tracking
  const cliToolStatuses = ref<Record<string, CliToolStatus>>({});
  const isLoading = ref(false);
  const isCheckingStatus = ref(false);
  const isUpgrading = ref(false);
  const isInstalling = ref(false);
  const isUninstalling = ref(false);
  const error = ref<string | null>(null);

  // Store pre-installed version information for enhanced detection
  const installedVersions = ref<Record<string, string>>({});

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

  async function detectSoftwareWithVersions() {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Software[]>('detect_software', {
        installedVersions: Object.keys(installedVersions.value).length > 0 ? installedVersions.value : null
      });
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
      // Fetch both lists in parallel: allagents 23 tools + all tools for custom tab filter
      const [allagentsResult, allToolsResult] = await Promise.all([
        invoke<CliToolInfo[]>('get_allagents_cli_tools'),
        invoke<CliToolInfo[]>('get_cli_tools'),
      ]);
      allagentsTools.value = allagentsResult;
      // Filter custom tools from the full list (displaySource === 'custom').
      // Then de-duplicate by key as a defensive fallback: if the SQLite
      // table ever contains two rows with the same key (e.g. a stale
      // `mimo-code` row surviving a partial migration) we still only
      // render one card in the Custom tab. The authoritative fix lives in
      // the Rust migration in `db/connection.rs`; this is the UI belt.
      const seen = new Set<string>();
      cliTools.value = allToolsResult
        .filter(t => t.displaySource === 'custom')
        .filter(t => (seen.has(t.key) ? false : seen.add(t.key)));
      debugLog('Loaded', allagentsResult.length, 'allagents tools,', cliTools.value.length, 'custom tools');
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
      // Merge into the existing map instead of replacing it. The Rust
      // command can return an empty list (or one with empty `toolKey`
      // entries) when the parallel 60s timeout fires — in that case the
      // previous per-tool statuses are still valid and we must not blank
      // them, otherwise the UI flips every card from "Installed" to
      // "Not installed" until the user manually re-checks.
      const next: Record<string, CliToolStatus> = { ...cliToolStatuses.value };
      let updated = 0;
      for (const status of result) {
        // Drop empty `toolKey` sentinels emitted by the Rust fallback
        // (`CliToolStatus { tool_key: String::new(), ... }`); they would
        // otherwise be written under the literal "" key.
        if (!status.toolKey) continue;
        next[status.toolKey] = status;
        updated += 1;
      }
      cliToolStatuses.value = next;
      debugLog('Status check complete:', updated, 'updated,', result.length, 'received');
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
        // Store the installed version for enhanced detection
        if (result.installedVersion) {
          installedVersions.value = {
            ...installedVersions.value,
            [softwareKey]: result.installedVersion
          };
        }
        // Use enhanced detection with pre-installed version information
        await detectSoftwareWithVersions();
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

  async function addCustomCliTool(config: {
    key: string;
    name: string;
    installMethod: string;
    installCommand: string;
    detectCommand: string;
    websiteUrl?: string;
    pluginDir?: string;
  }) {
    try {
      isLoading.value = true;
      error.value = null;
      // Guard against duplicate keys at the store level. The Rust `add_custom_cli_tool`
      // command does not currently enforce uniqueness, so without this check a user
      // could end up with two rows (and therefore two cards) sharing the same key —
      // which is the bug that produced two MiMo Code cards in the Custom tab.
      const existing = [
        ...allagentsTools.value,
        ...cliTools.value,
      ].find(t => t.key === config.key);
      if (existing) {
        const msg = `Tool "${config.key}" already exists (${existing.name})`;
        error.value = msg;
        throw new Error(msg);
      }
      await invoke('add_custom_cli_tool', { config });
      await fetchCliTools();
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add custom tool';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function removeCustomCliTool(key: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('remove_custom_cli_tool', { key });
      await fetchCliTools();
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove custom tool';
      throw e;
    } finally {
      isLoading.value = false;
    }
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
    allagentsTools,
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
    addCustomCliTool,
    removeCustomCliTool,
  };
});

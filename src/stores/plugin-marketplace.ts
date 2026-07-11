// Plugin Marketplace Store - State management for plugin marketplace
// Backed by `$FORGE_HOME/plugins/marketplace.json` (default `~/.forge`),
// synced via `node scripts/plugins/install.mjs` (see FEAT-007 / FEAT-008).

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { resolvePluginVersion } from '@/utils/plugin-version';
import { sortMarketplaceSources } from '@/utils/plugin-source-sort';
import type {
  PluginSource,
  MarketplacePlugin,
  PluginInstallProgress,
  PluginCapabilities,
  SourceStatus,
  SourceInstallResult,
  SourceInstallProgress,
  PluginSyncStatus,
  PluginSyncProgress,
  PluginSyncResult,
  RepoType,
  MarketplaceManifest,
  ManifestSource,
  ManifestPlugin,
  RemovedEntry,
  CliToolMeta,
} from '@/types';
import { PRESET_MARKETPLACE_SOURCES } from '@/types';

export type { MarketplaceManifest, ManifestSource, ManifestPlugin, RemovedEntry };

export const usePluginMarketplaceStore = defineStore('pluginMarketplace', () => {
  // State
  const sources = ref<PluginSource[]>(PRESET_MARKETPLACE_SOURCES);
  const currentSource = ref<PluginSource | null>(null);
  const plugins = ref<MarketplacePlugin[]>([]);
  const installedPlugins = ref<MarketplacePlugin[]>([]);
  const manifest = ref<MarketplaceManifest | null>(null);

  // Filter state
  const searchKeyword = ref('');

  // Loading states
  const isLoadingSources = ref(false);
  const isLoadingPlugins = ref(false);
  const isInstalling = ref(false);
  const isUpdating = ref(false);
  const isUninstalling = ref(false);

  // Error state
  const error = ref<string | null>(null);

  // Capabilities cache
  const capabilitiesCache = ref<Map<string, PluginCapabilities>>(new Map());

  // Resolved version cache: keyed by `${sourceId}::${pluginName}`.
  // Populated by resolveAllPluginVersions() for installed plugins whose
  // `version` field is empty or missing from the marketplace manifest.
  const resolvedVersions = ref<Map<string, string>>(new Map());

  // Install progress tracking
  const installProgress = ref<Map<string, PluginInstallProgress>>(new Map());

  // Source installation state (FEAT-016)
  // Using Record instead of Map for better Vue reactivity
  const sourceStatus = ref<Record<string, SourceStatus>>({});
  const isInstallingSource = ref(false);
  const currentInstallingSourceId = ref<string | null>(null);
  const sourceInstallProgress = ref<Record<string, SourceInstallProgress>>({});

  // Source notes state (FEAT-020)
  const sourceNotes = ref<Record<string, string>>({});

  // Computed
  const installedPluginNames = computed(() =>
    new Set(installedPlugins.value.map(p => p.name))
  );

  const isPluginInstalled = (pluginName: string) =>
    installedPluginNames.value.has(pluginName);

  const installedPluginIds = computed(() =>
    new Set(installedPlugins.value.map(p => `${p.sourceId}::${p.name}`))
  );

  // Source list ordered by `pluginCount` descending so the most
  // content-rich repositories surface first in both the Marketplace
  // source tabs and the Sources install grid. Ties (or sources with no
  // count yet) fall back to the original list order to keep things
  // stable across renders.
  // Sort with the same two-criteria rule (preset-first, then
  // pluginCount desc) that `loadMarketplaceManifest` uses. The pure
  // helper `sortMarketplaceSources` keeps both call sites in sync and
  // is covered by `utils/__tests__/plugin-source-sort.spec.ts`.
  const sortedSources = computed(() => sortMarketplaceSources(sources.value));

  const getPluginStatus = (plugin: MarketplacePlugin): 'installed' | 'update-available' | 'not-installed' => {
    const key = `${plugin.sourceId}::${plugin.name}`;
    if (!installedPluginIds.value.has(key)) return 'not-installed';
    const installed = installedPlugins.value.find(p => `${p.sourceId}::${p.name}` === key);
    if (installed && installed.latestVersion && installed.version !== installed.latestVersion) {
      return 'update-available';
    }
    return 'installed';
  };

  // Actions

  /// Load the full manifest (sources + counts) from the backend.
  /// This also injects the live plugin count into each `PluginSource`.
  async function loadMarketplaceManifest() {
    try {
      isLoadingSources.value = true;
      error.value = null;
      const remote = await invoke<PluginSource[]>('get_marketplace_sources');
      // Merge: keep preset metadata (description, command) but use remote count.
      // For user-added sources (not in remote), preserve the existing pluginCount.
      const merged: PluginSource[] = sources.value.map(existing => {
        const live = remote.find(r => r.id === existing.id);
        return {
          ...existing,
          pluginCount: live?.pluginCount ?? existing.pluginCount ?? 0,
        };
      });
      sources.value = merged;
      // Default-select the first source if none selected. Prefer an
      // already-installed source so the Marketplace tab has data to
      // show on first open; fall back to the first preset otherwise.
      // The pick is made from the *sorted* source list (same order the
      // source-tabs are rendered in) so the visually-first tab is the
      // one that's active by default.
      if (!currentSource.value && merged.length > 0) {
        const ordered = sortMarketplaceSources(merged);
        const installedFirst = ordered.find(
          (s) => sourceStatus.value[s.id]?.isInstalled,
        );
        currentSource.value = installedFirst ?? ordered[0];
      }
      return merged;
    } catch (e) {
      console.error('Failed to load marketplace manifest:', e);
      error.value = e instanceof Error ? e.message : 'Failed to load marketplace manifest';
      return sources.value;
    } finally {
      isLoadingSources.value = false;
    }
  }

  async function fetchInstalledPlugins() {
    try {
      isLoadingPlugins.value = true;
      error.value = null;
      const result = await invoke<MarketplacePlugin[]>('get_marketplace_plugins');
      installedPlugins.value = result;
      // After refreshing the installed set, re-derive `isInstalled`
      // on the currently-displayed marketplace list. This covers the
      // case where the user installs a plugin and the marketplace tab
      // is still showing the stale "Install" badge because the
      // marketplace refetch hasn't happened yet (or the user
      // navigates back to the tab before it does).
      if (plugins.value.length > 0) {
        plugins.value = reconcileInstalledFlags(plugins.value);
      }
      // Resolve missing versions from plugin manifest files on disk
      await resolveAllPluginVersions();
      return result;
    } catch (e) {
      console.error('Failed to fetch installed plugins:', e);
      error.value = e instanceof Error ? e.message : 'Failed to fetch installed plugins';
      return [];
    } finally {
      isLoadingPlugins.value = false;
    }
  }

  /// Resolve the display version for installed plugins whose `version` field
  /// is empty. Reads manifest files from disk (marketplace.json → plugin.json
  /// → package.json → "unknown") via the Rust backend. Results are cached
  /// so repeated calls are a no-op.
  async function resolveAllPluginVersions(): Promise<void> {
    const toResolve = installedPlugins.value.filter(p => {
      const key = `${p.sourceId}::${p.name}`;
      // Skip if already resolved or if the plugin already has a version
      if (resolvedVersions.value.has(key)) return false;
      if (p.version && p.version.trim() !== '') return false;
      // Only resolve if we have an install path
      return !!p.installPath;
    });

    if (toResolve.length === 0) return;

    const results = await Promise.allSettled(
      toResolve.map(async (p) => {
        const key = `${p.sourceId}::${p.name}`;
        const version = await resolvePluginVersion(p.installPath!);
        return { key, version };
      })
    );

    // Build a new map to trigger Vue reactivity (Map.set() on a ref'd Map
    // doesn't trigger updates because the Map reference doesn't change).
    const next = new Map(resolvedVersions.value);
    for (const result of results) {
      if (result.status === 'fulfilled') {
        next.set(result.value.key, result.value.version);
      }
    }
    resolvedVersions.value = next;
  }

  /// Get the display version for a plugin: prefer the store's `version`
  /// field, fall back to the resolved version cache, then empty string.
  /// Returns the raw version string (without "v" prefix); callers add
  /// the prefix in the template.
  function getResolvedVersion(plugin: MarketplacePlugin): string {
    const key = `${plugin.sourceId}::${plugin.name}`;
    const resolved = plugin.version || resolvedVersions.value.get(key);
    if (!resolved || resolved === 'unknown') return '—';
    return resolved;
  }

  async function fetchPluginsBySource(sourceId: string) {
    if (sourceId === 'all') {
      return fetchAllSourcesPlugins();
    }

    const source = sources.value.find(s => s.id === sourceId);
    if (!source) return;

    currentSource.value = source;

    try {
      isLoadingPlugins.value = true;
      error.value = null;

      // The Rust command returns the full list of plugins for the
      // source — no pagination, no page numbers. The UI renders every
      // card at once.
      const result = await invoke<MarketplacePlugin[]>('fetch_marketplace_plugins', {
        sourceId: source.id,
        keyword: searchKeyword.value || null,
      });

      plugins.value = reconcileInstalledFlags(result);
    } catch (e) {
      console.error(`Failed to fetch plugins from ${sourceId}:`, e);
      error.value = e instanceof Error ? e.message : 'Failed to fetch plugins';
      plugins.value = [];
    } finally {
      isLoadingPlugins.value = false;
    }
  }

  /// Fetch plugins from ALL installed sources and merge them into a single
  /// list. Each source is fetched independently — plugins with the same
  /// name in different sources are kept as separate entries (their
  /// sourceId distinguishes them).
  async function fetchAllSourcesPlugins() {
    try {
      isLoadingPlugins.value = true;
      error.value = null;

      const installedSourceIds = sources.value
        .filter(s => sourceStatus.value[s.id]?.isInstalled)
        .map(s => s.id);

      const results = await Promise.all(
        installedSourceIds.map(sourceId =>
          invoke<MarketplacePlugin[]>('fetch_marketplace_plugins', {
            sourceId,
            keyword: searchKeyword.value || null,
          }).catch(e => {
            console.error(`Failed to fetch plugins from ${sourceId}:`, e);
            return [] as MarketplacePlugin[];
          })
        )
      );

      const merged = results.flat();
      plugins.value = reconcileInstalledFlags(merged);
    } catch (e) {
      console.error('Failed to fetch all source plugins:', e);
      error.value = e instanceof Error ? e.message : 'Failed to fetch plugins';
      plugins.value = [];
    } finally {
      isLoadingPlugins.value = false;
    }
  }

  /// Cross-reference the given marketplace list with the locally
  /// installed set and set `isInstalled` on each entry. The Rust
  /// backend now writes the canonical value, but the marketplace list
  /// and the installed list are fetched independently and can race
  /// (e.g. user installs a plugin, then quickly switches the source
  /// tab before the installed-list refetch resolves). Reconciling on
  /// the frontend guarantees the Installed tab and the card's
  /// "Install/Installed" badge stay in sync across reload, restart,
  /// and refresh — fixing the bug where freshly-installed plugins
  /// were reported as "not installed" after re-entering the page.
  function reconcileInstalledFlags(list: MarketplacePlugin[]): MarketplacePlugin[] {
    if (list.length === 0) return list;
    const installedKeys = new Set(
      installedPlugins.value.map(p => `${p.sourceId}::${p.name}`),
    );
    return list.map(p => ({
      ...p,
      isInstalled: p.isInstalled ?? installedKeys.has(`${p.sourceId}::${p.name}`),
    }));
  }

  function selectSource(source: PluginSource) {
    // Only allow selecting a source if it's installed
    const status = sourceStatus.value[source.id];
    if (!status?.isInstalled) {
      console.warn(`Source '${source.id}' is not installed yet`);
      return;
    }

    currentSource.value = source;
    searchKeyword.value = '';
    fetchPluginsBySource(source.id);
  }

  function setSearchKeyword(keyword: string) {
    searchKeyword.value = keyword;
    if (currentSource.value) {
      fetchPluginsBySource(currentSource.value.id);
    }
  }

  async function installPlugin(plugin: MarketplacePlugin) {
    // Find source by plugin's sourceId (works for both single-source and "All Sources" views)
    const source = sources.value.find(s => s.id === plugin.sourceId);
    if (!source) {
      console.error(`Source not found for plugin: ${plugin.name} (sourceId: ${plugin.sourceId})`);
      return { success: false, error: 'Source not found' };
    }

    const progress: PluginInstallProgress = {
      pluginId: plugin.id,
      pluginName: plugin.name,
      stage: 'downloading',
      progress: 0,
      message: '正在安装插件...',
      startedAt: new Date().toISOString(),
    };
    // Reassign the Map reference so Vue's reactivity picks up the new
    // entry — `Map.set()` alone does NOT trigger updates on a ref'd Map.
    installProgress.value = new Map(installProgress.value).set(plugin.id, progress);
    isInstalling.value = true;

    try {
      progress.progress = 30;
      progress.message = '正在准备插件文件...';
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      const installResult = await invoke<{ success: boolean; path?: string; error?: string }>(
        'install_marketplace_plugin',
        {
          plugin,
        }
      );

      if (!installResult.success) {
        throw new Error(installResult.error || 'Installation failed');
      }

      progress.progress = 100;
      progress.stage = 'success';
      progress.message = `插件已安装到 ${installResult.path}`;
      progress.completedAt = new Date().toISOString();
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      // Refresh installed plugins
      await fetchInstalledPlugins();

      // Refresh marketplace tab cards so is_installed status updates immediately
      if (currentSource.value) {
        await fetchPluginsBySource(currentSource.value.id);
      }

      return { success: true, path: installResult.path };
    } catch (e) {
      progress.stage = 'failed';
      progress.error = e instanceof Error ? e.message : String(e);
      progress.message = `安装失败: ${progress.error}`;
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      return { success: false, error: e };
    } finally {
      isInstalling.value = false;

      setTimeout(() => {
        const next = new Map(installProgress.value);
        next.delete(plugin.id);
        installProgress.value = next;
      }, 5000);
    }
  }

  async function uninstallPlugin(plugin: MarketplacePlugin) {
    const progress: PluginInstallProgress = {
      pluginId: plugin.id,
      pluginName: plugin.name,
      stage: 'downloading',
      progress: 0,
      message: '正在卸载插件...',
      startedAt: new Date().toISOString(),
    };
    installProgress.value = new Map(installProgress.value).set(plugin.id, progress);
    isUninstalling.value = true;

    try {
      progress.progress = 50;
      progress.message = '正在移除插件文件...';
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      const uninstallResult = await invoke<{ success: boolean; error?: string }>(
        'uninstall_marketplace_plugin',
        { pluginId: plugin.id }
      );

      if (!uninstallResult.success) {
        throw new Error(uninstallResult.error || 'Uninstallation failed');
      }

      progress.progress = 100;
      progress.stage = 'success';
      progress.message = '插件已卸载';
      progress.completedAt = new Date().toISOString();
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      // Refresh installed plugins
      await fetchInstalledPlugins();

      // Update plugin status in current list
      const pluginInList = plugins.value.find(p => p.id === plugin.id);
      if (pluginInList) {
        pluginInList.isInstalled = false;
      }

      return { success: true };
    } catch (e) {
      progress.stage = 'failed';
      progress.error = e instanceof Error ? e.message : String(e);
      progress.message = `卸载失败: ${progress.error}`;
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      return { success: false, error: e };
    } finally {
      isUninstalling.value = false;

      setTimeout(() => {
        const next = new Map(installProgress.value);
        next.delete(plugin.id);
        installProgress.value = next;
      }, 5000);
    }
  }

  async function updatePlugin(plugin: MarketplacePlugin) {
    // Find source by plugin's sourceId (works for both single-source and "All Sources" views)
    const source = sources.value.find(s => s.id === plugin.sourceId);
    if (!source) {
      console.error(`Source not found for plugin: ${plugin.name} (sourceId: ${plugin.sourceId})`);
      return { success: false, error: 'Source not found' };
    }

    const progress: PluginInstallProgress = {
      pluginId: plugin.id,
      pluginName: plugin.name,
      stage: 'updating',
      progress: 0,
      message: '正在更新插件...',
      startedAt: new Date().toISOString(),
    };
    installProgress.value = new Map(installProgress.value).set(plugin.id, progress);
    isUpdating.value = true;

    try {
      progress.progress = 30;
      progress.message = '正在下载新版本插件...';
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      const updateResult = await invoke<{ success: boolean; newVersion?: string; error?: string }>(
        'update_marketplace_plugin',
        {
          plugin,
        }
      );

      if (!updateResult.success) {
        throw new Error(updateResult.error || 'Update failed');
      }

      progress.progress = 100;
      progress.stage = 'success';
      progress.message = `插件已更新到 ${updateResult.newVersion}`;
      progress.completedAt = new Date().toISOString();
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      await fetchInstalledPlugins();

      // Refresh marketplace tab cards so is_installed/update status updates immediately
      if (currentSource.value) {
        await fetchPluginsBySource(currentSource.value.id);
      }

      return { success: true, newVersion: updateResult.newVersion };
    } catch (e) {
      progress.stage = 'failed';
      progress.error = e instanceof Error ? e.message : String(e);
      progress.message = `更新失败: ${progress.error}`;
      installProgress.value = new Map(installProgress.value).set(plugin.id, { ...progress });

      return { success: false, error: e };
    } finally {
      isUpdating.value = false;

      setTimeout(() => {
        const next = new Map(installProgress.value);
        next.delete(plugin.id);
        installProgress.value = next;
      }, 5000);
    }
  }

  function getInstallProgress(pluginId: string): PluginInstallProgress | undefined {
    return installProgress.value.get(pluginId);
  }

  function clearError() {
    error.value = null;
  }

  /// Fetch plugin capabilities from the backend (local first, remote fallback).
  /// Results are cached by `${sourceId}/${pluginName}`.
  async function fetchPluginCapabilities(plugin: MarketplacePlugin): Promise<PluginCapabilities> {
    const key = `${plugin.sourceId}/${plugin.name}`;
    if (capabilitiesCache.value.has(key)) {
      return capabilitiesCache.value.get(key)!;
    }
    try {
      const caps = await invoke<PluginCapabilities>('get_plugin_capabilities', {
        sourceId: plugin.sourceId,
        pluginName: plugin.name,
      });
      // Reassign the Map reference so Vue's reactivity picks up the new
      // entry — `Map.set()` on a ref'd Map does NOT trigger updates.
      const next = new Map(capabilitiesCache.value);
      next.set(key, caps);
      capabilitiesCache.value = next;
      return caps;
    } catch (e) {
      console.error('Failed to fetch plugin capabilities:', e);
      throw e;
    }
  }

  /// Clear the capabilities cache entry for a plugin.
  function clearCapabilitiesCache(pluginName: string, sourceId?: string) {
    const next = new Map(capabilitiesCache.value);
    if (sourceId) {
      next.delete(`${sourceId}/${pluginName}`);
    } else {
      // Clear all entries matching the plugin name
      for (const key of [...next.keys()]) {
        if (key.endsWith(`/${pluginName}`)) {
          next.delete(key);
        }
      }
    }
    capabilitiesCache.value = next;
  }

  /// Toggle the user-controlled enable/disable flag for an installed plugin.
  /// The plugin stays on disk; only the metadata in
  /// `$FORGE_HOME/plugins/marketplace.json` is updated. After persisting
  /// the new value on the Rust side we reconcile the local `installedPlugins`
  /// ref so the UI reflects the change immediately without a full refetch.
  async function setPluginDisabled(plugin: MarketplacePlugin, disabled: boolean): Promise<boolean> {
    try {
      const newValue = await invoke<boolean>('set_plugin_disabled', {
        sourceId: plugin.sourceId,
        pluginName: plugin.name,
        disabled,
      });
      // Mirror the new value into our local store so the toggle on the
      // Installed tab reflects the persisted state right away.
      const entry = installedPlugins.value.find(p => p.name === plugin.name && p.sourceId === plugin.sourceId);
      if (entry) {
        entry.disabled = newValue;
      }
      return newValue;
    } catch (e) {
      console.error('Failed to set plugin disabled state:', e);
      throw e;
    }
  }

  async function sweepInuse(): Promise<number> {
    try {
      const count = await invoke<number>('sweep_inuse_cmd');
      console.info(`sweep_inuse: updated ${count} plugin timestamps`);
      return count;
    } catch (e) {
      console.error('Failed to sweep inuse:', e);
      throw e;
    }
  }

  // ---------------------------------------------------------------------------
  // Source installation (FEAT-016)
  // ---------------------------------------------------------------------------

  async function loadSourceStatus(): Promise<void> {
    try {
      // FEAT-019: load user-added sources first so get_sources_status
      // can return the merged list (preset + user). The Rust command
      // already walks user_sources.json internally, so the
      // SourceStatus entries for user sources arrive via the same
      // `get_marketplace_source_status` call. We also keep the source
      // list in the store in sync by fetching user sources separately.
      const userSources = await invoke<PluginSource[]>('get_user_marketplace_sources');
      mergeUserSources(userSources);

      const statuses = await invoke<SourceStatus[]>('get_marketplace_source_status');

      // Use whole replacement to ensure Vue reactivity is properly triggered
      const next: Record<string, SourceStatus> = {};
      for (const status of statuses) {
        next[status.sourceId] = status;
      }
      sourceStatus.value = next;
    } catch (e) {
      console.error('loadSourceStatus error:', e);
    }

    // Load source notes (FEAT-020)
    await loadSourceNotes();
  }

  // ---------------------------------------------------------------------------
  // Source notes (FEAT-020)
  // ---------------------------------------------------------------------------

  async function loadSourceNotes(): Promise<void> {
    try {
      sourceNotes.value = await invoke<Record<string, string>>('get_source_notes');
    } catch (e) {
      console.error('loadSourceNotes error:', e);
    }
  }

  async function saveSourceNote(sourceId: string, note: string): Promise<void> {
    await invoke('save_source_note', { sourceId, note });
    if (note) {
      sourceNotes.value[sourceId] = note;
    } else {
      delete sourceNotes.value[sourceId];
    }
  }

  /// Merge persisted user sources into the in-memory sources list
  /// (FEAT-019). Preserves the "newest at top" ordering — the most
  /// recently added user source is at index 0. Deduplicates by id
  /// (preset wins on conflict).
  function mergeUserSources(userSources: PluginSource[]): void {
    if (!userSources || userSources.length === 0) return;
    const presetIds = new Set(PRESET_MARKETPLACE_SOURCES.map(s => s.id));
    const existingIds = new Set(sources.value.map(s => s.id));
    // Also build a URL set for dedup — user-added sources may have a
    // different id slug (e.g. "addyosmani-agent-skills") but the same
    // command URL as a preset source (e.g. id "addyosmani").
    const existingUrls = new Set(
      sources.value.map(s => s.command.replace(/\/+$/, '').replace(/\.git$/, ''))
    );
    // Keep the order returned by the backend (newest-first, since
    // add_user_marketplace_source appends to the end of the array —
    // we reverse it here so the most recent add stays at the top, the
    // same convention addSource() uses).
    const incoming = [...userSources].reverse();
    for (const s of incoming) {
      if (presetIds.has(s.id)) continue; // preset wins
      if (existingIds.has(s.id)) continue; // already in list
      const url = s.command.replace(/\/+$/, '').replace(/\.git$/, '');
      if (existingUrls.has(url)) continue; // same URL already exists (preset or earlier user source)
      sources.value = [s, ...sources.value];
      existingIds.add(s.id);
      existingUrls.add(url);
    }
  }

  async function installSource(sourceId: string): Promise<SourceInstallResult> {
    isInstallingSource.value = true;
    currentInstallingSourceId.value = sourceId;

    try {
      const progress: SourceInstallProgress = {
        sourceId,
        stage: 'preparing',
        progress: 10,
        message: '正在准备下载...',
      };
      sourceInstallProgress.value = { ...sourceInstallProgress.value, [sourceId]: progress };

      // Look up the source so we can pass its repo URL to the Rust
      // command for user-added sources (FEAT-018 fix). For preset
      // sources the URL is resolved server-side; for user-added ones
      // Rust needs the URL to derive the repo_name and clone target.
      const source = sources.value.find(s => s.id === sourceId);
      const repoUrl = source?.command;
      const isUserAdded = !!source && !PRESET_MARKETPLACE_SOURCES.some(p => p.id === sourceId);

      const result = await invoke<SourceInstallResult>('install_marketplace_source', {
        sourceId,
        repoUrl,
      });

      if (result.success) {
        progress.stage = 'success';
        progress.progress = 100;
        progress.message = result.installedPaths?.length === 2
          ? '已安装到 2 个位置'
          : '安装完成';
        sourceInstallProgress.value = { ...sourceInstallProgress.value, [sourceId]: { ...progress } };

        // For user-added sources the backend's get_marketplace_source_status
        // (which only walks PRESET_MARKETPLACE_SOURCES) won't know about
        // this id and would always report it as not installed. Inject
        // the success status into the local map so the UI reflects the
        // install immediately. For preset sources we still re-fetch to
        // get the canonical installed_paths from the backend.
        if (isUserAdded) {
          const installedPath = result.installedPath ?? result.installedPaths?.[0] ?? '';
          const next = { ...sourceStatus.value };
          next[sourceId] = {
            sourceId,
            name: source?.name ?? sourceId,
            nameZh: source?.nameZh,
            repoUrl: repoUrl ?? '',
            isInstalled: true,
            installedPath,
            installedPaths: result.installedPaths ?? (installedPath ? [installedPath] : []),
          };
          sourceStatus.value = next;
        } else {
          await loadSourceStatus();
        }
      } else {
        progress.stage = 'failed';
        progress.message = result.error || '安装失败';
        sourceInstallProgress.value = { ...sourceInstallProgress.value, [sourceId]: { ...progress } };
      }

      return result;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      const progress: SourceInstallProgress = {
        sourceId,
        stage: 'failed',
        progress: 0,
        message: `安装失败: ${errMsg}`,
      };
      sourceInstallProgress.value = { ...sourceInstallProgress.value, [sourceId]: progress };
      return { success: false, sourceId, installedPaths: [], error: errMsg };
    } finally {
      isInstallingSource.value = false;
      currentInstallingSourceId.value = null;
    }
  }

  async function installAllSources(): Promise<SourceInstallResult[]> {
    isInstallingSource.value = true;
    try {
      const results = await invoke<SourceInstallResult[]>('install_all_marketplace_sources');
      await loadSourceStatus();
      return results;
    } finally {
      isInstallingSource.value = false;
      currentInstallingSourceId.value = null;
    }
  }

  function getSourceStatus(sourceId: string): SourceStatus | undefined {
    return sourceStatus.value[sourceId];
  }

  function getSourceInstallProgress(sourceId: string): SourceInstallProgress | undefined {
    return sourceInstallProgress.value[sourceId];
  }

  function clearSourceInstallProgress(sourceId: string): void {
    const newProgress = { ...sourceInstallProgress.value };
    delete newProgress[sourceId];
    sourceInstallProgress.value = newProgress;
  }

  /// Add a new repository source. The source is appended to the
  /// persisted `user_sources.json` (FEAT-019) and the in-memory
  /// `sources` list. Returns `{ success: true, source }` on success
  /// or `{ success: false, error }` on failure.
  ///
  /// Validation matches the FEAT-018 spec (GitHub URL regex, dedup by
  /// `command`). The Rust backend performs a second dedup pass on
  /// persisted state to handle races where the user adds the same URL
  /// across two app instances.
  async function addSource(
    url: string,
    repoType: RepoType = 'market',
  ): Promise<{ success: boolean; source?: PluginSource; error?: string }> {
    const trimmed = url.trim().replace(/\/+$/, '').replace(/\.git$/, '');
    if (!trimmed) {
      return { success: false, error: '请输入仓库地址' };
    }
    const m = trimmed.match(/^https:\/\/github\.com\/([^/\s]+)\/([^/\s]+)$/);
    if (!m) {
      return { success: false, error: '请输入合法的 GitHub 仓库 URL（https://github.com/owner/repo）' };
    }
    const [, owner, repoName] = m;
    if (sources.value.some(s => s.command.replace(/\/+$/, '').replace(/\.git$/, '') === trimmed)) {
      return { success: false, error: '该仓库源已存在' };
    }
    let id = `${owner}-${repoName}`;
    let suffix = 2;
    while (sources.value.some(s => s.id === id)) {
      id = `${owner}-${repoName}-${suffix++}`;
    }
    const source: PluginSource = {
      id,
      name: repoName,
      command: trimmed,
      description: `User-added repository ${owner}/${repoName}`,
      repoName,
      pluginCount: repoType === 'res' ? 1 : 0,
      repoType,
    };

    // Persist to backend first (FEAT-019). On success, update UI.
    // On failure, surface the Rust error message and leave the UI
    // untouched.
    try {
      const persisted = await invoke<PluginSource>('add_user_marketplace_source', {
        source,
      });
      // Use the persisted copy (Rust may have normalized fields) and
      // place it at the top of the in-memory list.
      sources.value = [persisted, ...sources.value];
      return { success: true, source: persisted };
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      // Map known backend error messages to friendlier Chinese text.
      if (errMsg.includes('already exists')) {
        return { success: false, error: '该仓库源已存在' };
      }
      return { success: false, error: `保存失败: ${errMsg}` };
    }
  }

  /// Remove a user-added source by id (FEAT-019). No-op for preset
  /// sources — those cannot be removed from the UI. Returns true if a
  /// source was removed.
  async function removeSource(sourceId: string): Promise<boolean> {
    const isPreset = PRESET_MARKETPLACE_SOURCES.some(p => p.id === sourceId);
    if (isPreset) return false;
    try {
      const removed = await invoke<PluginSource | null>('remove_user_marketplace_source', {
        sourceId,
      });
      if (removed) {
        sources.value = sources.value.filter(s => s.id !== sourceId);
        // Also drop its SourceStatus entry (if any) so the UI doesn't
        // show a stale "installed" badge for a no-longer-existing source.
        const next = { ...sourceStatus.value };
        delete next[sourceId];
        sourceStatus.value = next;
        return true;
      }
      return false;
    } catch (e) {
      console.error('removeSource error:', e);
      return false;
    }
  }

  /// Switch the repo_type of a user-added source (market ↔ res).
  /// Preset sources cannot be modified. On success, refreshes the
  /// marketplace so the plugin list reflects the new type.
  async function switchSourceType(
    sourceId: string,
    newRepoType: RepoType,
  ): Promise<{ success: boolean; error?: string }> {
    try {
      const updated = await invoke<PluginSource>('update_source_repo_type', {
        sourceId,
        repoType: newRepoType,
      });
      // Update in-memory sources list
      const idx = sources.value.findIndex(s => s.id === sourceId);
      if (idx !== -1) {
        sources.value[idx] = { ...sources.value[idx], ...updated };
      }
      // Refresh source status and marketplace to reload plugin data for the new type
      await Promise.all([
        loadSourceStatus(),
        loadMarketplaceManifest(),
      ]);
      // Re-fetch plugins for the current source if it's the one being switched
      if (currentSource.value?.id === sourceId) {
        await fetchPluginsBySource(sourceId);
      }
      return { success: true };
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      console.error('switchSourceType error:', errMsg);
      return { success: false, error: errMsg };
    }
  }

  const installedSourcesCount = computed(() => {
    let count = 0;
    for (const sourceId of Object.keys(sourceStatus.value)) {
      if (sourceStatus.value[sourceId]?.isInstalled) count++;
    }
    return count;
  });

  const totalSourcesCount = computed(() => sources.value.length);

  const allSourcesInstalled = computed(() =>
    installedSourcesCount.value === totalSourcesCount.value && totalSourcesCount.value > 0
  );

  // ---------------------------------------------------------------------------
  // Plugin Sync to CLI Tool (FEAT-CLI-SYNC)
  // ---------------------------------------------------------------------------

  // Sync status: keyed by plugin_id (sourceId::pluginName)
  const syncStatuses = ref<Record<string, PluginSyncStatus>>({});
  // Sync progress: keyed by plugin_id
  const syncProgress = ref<Record<string, PluginSyncProgress>>({});
  // All supported CLI tools from CliToolManager (populated once per session)
  const supportedCliTools = ref<CliToolMeta[]>([]);

  /** Color palette for known CLI tool keys. */
  const CLI_TOOL_COLORS: Record<string, string> = {
    'claude-code': '#B8944A',
    'codex': '#059669',
    'gemini-cli': '#2563EB',
    'opencode': '#0891B2',
    'openclaw': '#7C3AED',
    'hermes': '#DC2626',
    'cursor': '#7C3AED',
    'deepseek-reasonix': '#4F46E5',
    'mimo-code': '#0891B2',
    'qwen-code': '#2563EB',
    'copilot': '#6E40C9',
  };

  /** Fallback color for unknown tool keys (hash-based). */
  function getCliToolColor(key: string): string {
    if (CLI_TOOL_COLORS[key]) return CLI_TOOL_COLORS[key];
    // Generate a consistent color from the key string
    let hash = 0;
    for (let i = 0; i < key.length; i++) {
      hash = key.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash) % 360;
    return `hsl(${hue}, 55%, 45%)`;
  }

  /// Fetch all supported CLI tools from the backend (once per session).
  async function fetchSupportedCliTools(): Promise<void> {
    if (supportedCliTools.value.length > 0) return;
    try {
      const list = await invoke<Array<{
        key: string;
        name: string;
        icon: string;
        pluginDir: string | null;
      }>>('get_supported_cli_tools');
      supportedCliTools.value = list.map(t => ({
        key: t.key,
        name: t.name,
        icon: t.icon.substring(0, 2).toUpperCase(),
        color: getCliToolColor(t.key),
        pluginDir: t.pluginDir,
      }));
    } catch (e) {
      console.error('Failed to fetch supported CLI tools:', e);
    }
  }

  /// Fetch sync statuses for a batch of plugins.
  async function fetchSyncStatuses(pluginIds: string[]): Promise<void> {
    if (pluginIds.length === 0) return;
    try {
      const results = await invoke<PluginSyncStatus[]>('get_plugin_sync_status', {
        pluginIds,
      });
      const next: Record<string, PluginSyncStatus> = {};
      for (const status of results) {
        // Backend returns pluginId as the 2-part key (sourceId::pluginName).
        // The view and syncPluginToCliTool both use 3-part keys
        // (sourceId::pluginName::cliToolKey), so we must append cliToolKey
        // here to match — otherwise the view's getSyncStatus() lookup fails.
        if (status.synced) {
          const syncKey = `${status.pluginId}::${status.cliToolKey}`;
          next[syncKey] = status;
        }
      }
      syncStatuses.value = next;
    } catch (e) {
      console.error('Failed to fetch sync statuses:', e);
    }
  }

  /// Sync a plugin to a CLI tool's plugin directory.
  async function syncPluginToCliTool(
    plugin: MarketplacePlugin,
    cliToolKey: string,
  ): Promise<{ success: boolean; error?: string }> {
    const pluginId = `${plugin.sourceId}::${plugin.name}`;
    const syncKey = `${pluginId}::${cliToolKey}`;
    syncProgress.value = {
      ...syncProgress.value,
      [syncKey]: {
        pluginId: syncKey,
        cliToolKey,
        stage: 'syncing',
        progress: 30,
        message: '正在同步插件...',
      },
    };

    try {
      // Derive repo name from source command URL (e.g. "claude-plugins-official")
      const source = sources.value.find(s => s.id === plugin.sourceId);
      const sourceRepoName = source?.repoName ?? source?.command?.split('/').pop() ?? null;

      const result = await invoke<PluginSyncResult>('sync_plugin_to_cli_tool', {
        pluginId,
        cliToolKey,
        sourceId: plugin.sourceId,
        pluginName: plugin.name,
        sourceRepoName,
        pluginVersion: plugin.version ?? null,
      });

      if (result.success) {
        syncProgress.value = {
          ...syncProgress.value,
          [syncKey]: {
            pluginId: syncKey,
            cliToolKey,
            stage: 'success',
            progress: 100,
            message: '同步完成',
          },
        };
        syncStatuses.value = {
          ...syncStatuses.value,
          [syncKey]: {
            pluginId: syncKey,
            cliToolKey,
            synced: true,
            syncedAt: new Date().toISOString(),
            targetPath: result.targetPath,
          },
        };
        return { success: true };
      } else {
        syncProgress.value = {
          ...syncProgress.value,
          [syncKey]: {
            pluginId: syncKey,
            cliToolKey,
            stage: 'failed',
            progress: 0,
            error: result.error,
            message: `同步失败: ${result.error}`,
          },
        };
        return { success: false, error: result.error };
      }
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      syncProgress.value = {
        ...syncProgress.value,
        [syncKey]: {
          pluginId: syncKey,
          cliToolKey,
          stage: 'failed',
          progress: 0,
          error: errMsg,
          message: `同步失败: ${errMsg}`,
        },
      };
      return { success: false, error: errMsg };
    } finally {
      setTimeout(() => {
        const next = { ...syncProgress.value };
        delete next[syncKey];
        syncProgress.value = next;
      }, 3000);
    }
  }

  /// Remove previously synced plugin data from a CLI tool's plugin directory.
  async function unsyncPluginFromCliTool(
    plugin: MarketplacePlugin,
    cliToolKey: string,
  ): Promise<{ success: boolean; error?: string }> {
    const pluginId = `${plugin.sourceId}::${plugin.name}`;
    const syncKey = `${pluginId}::${cliToolKey}`;
    syncProgress.value = {
      ...syncProgress.value,
      [syncKey]: {
        pluginId: syncKey,
        cliToolKey,
        stage: 'unsyncing',
        progress: 30,
        message: '正在取消同步...',
      },
    };

    try {
      const result = await invoke<PluginSyncResult>('unsync_plugin_from_cli_tool', {
        pluginId,
        cliToolKey,
      });

      if (result.success) {
        syncProgress.value = {
          ...syncProgress.value,
          [syncKey]: {
            pluginId: syncKey,
            cliToolKey,
            stage: 'success',
            progress: 100,
            message: '已取消同步',
          },
        };
        const next = { ...syncStatuses.value };
        delete next[syncKey];
        syncStatuses.value = next;
        return { success: true };
      } else {
        syncProgress.value = {
          ...syncProgress.value,
          [syncKey]: {
            pluginId: syncKey,
            cliToolKey,
            stage: 'failed',
            progress: 0,
            error: result.error,
            message: `取消同步失败: ${result.error}`,
          },
        };
        return { success: false, error: result.error };
      }
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : String(e);
      syncProgress.value = {
        ...syncProgress.value,
        [syncKey]: {
          pluginId: syncKey,
          cliToolKey,
          stage: 'failed',
          progress: 0,
          error: errMsg,
          message: `取消同步失败: ${errMsg}`,
        },
      };
      return { success: false, error: errMsg };
    } finally {
      setTimeout(() => {
        const next = { ...syncProgress.value };
        delete next[syncKey];
        syncProgress.value = next;
      }, 3000);
    }
  }

  function getSyncProgress(pluginId: string): PluginSyncProgress | undefined {
    return syncProgress.value[pluginId];
  }

  function getSyncStatus(pluginId: string): PluginSyncStatus | undefined {
    return syncStatuses.value[pluginId];
  }

  return {
    // State
    sources,
    currentSource,
    plugins,
    installedPlugins,
    manifest,
    searchKeyword,
    isLoadingSources,
    isLoadingPlugins,
    isInstalling,
    isUpdating,
    isUninstalling,
    error,
    installProgress,
    // Source installation state (FEAT-016)
    sourceStatus,
    isInstallingSource,
    currentInstallingSourceId,
    sourceInstallProgress,
    // Source notes state (FEAT-020)
    sourceNotes,
    // Plugin sync state
    syncStatuses,
    syncProgress,
    supportedCliTools,
    // Resolved version cache
    resolvedVersions,

    // Computed
    installedPluginNames,
    installedSourcesCount,
    totalSourcesCount,
    allSourcesInstalled,
    sortedSources,

    // Actions
    loadMarketplaceManifest,
    fetchInstalledPlugins,
    fetchPluginsBySource,
    selectSource,
    setSearchKeyword,
    installPlugin,
    uninstallPlugin,
    updatePlugin,
    getInstallProgress,
    clearError,
    isPluginInstalled,
    getPluginStatus,
    fetchPluginCapabilities,
    clearCapabilitiesCache,
    setPluginDisabled,
    sweepInuse,
    // Version resolution
    resolveAllPluginVersions,
    getResolvedVersion,
    // Source installation actions (FEAT-016)
    loadSourceStatus,
    // Source notes actions (FEAT-020)
    loadSourceNotes,
    saveSourceNote,
    installSource,
    installAllSources,
    getSourceStatus,
    getSourceInstallProgress,
    clearSourceInstallProgress,
    addSource,
    removeSource,
    switchSourceType,
    // Plugin sync actions
    fetchSyncStatuses,
    fetchSupportedCliTools,
    syncPluginToCliTool,
    unsyncPluginFromCliTool,
    getSyncProgress,
    getSyncStatus,
  };
});

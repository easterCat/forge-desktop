/**
 * Plugin 适配器 Store
 *
 * 在现有 PluginMarketplaceStore 基础上添加 AllAgentsService 同步能力，
 * 实现声明式管理和多客户端同步。保留现有的 marketplace 来源管理逻辑。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  UnifiedPlugin,
  ClientType,
} from '@/types/unified-plugin';


// ============================================================================
// 现有类型 (保留兼容)
// ============================================================================

export interface MarketplaceSource {
  id: string;
  name: string;
  url: string;
  branch?: string;
  enabled: boolean;
  lastSyncedAt?: string;
  repoType?: 'github' | 'local';
}

export interface MarketplacePluginData {
  id: string;
  name: string;
  description: string;
  version?: string;
  sourceId: string;
  categories: string[];
  tags: string[];
  installSource: 'local' | 'git-subdir' | 'url';
  installPath?: string;
  cliToolKeys?: string[];
  disabled?: boolean;
}

// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedPluginAdapterStore = defineStore('unified-plugin-adapter', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 插件市场来源 */
  const sources = ref<MarketplaceSource[]>([]);

  /** 市场插件列表 */
  const plugins = ref<MarketplacePluginData[]>([]);

  /** 已安装的插件 */
  const installedPlugins = ref<UnifiedPlugin[]>([]);

  /** 活动来源 ID */
  const activeSourceId = ref<string>('');

  /** 同步目标客户端 */
  const targetClients = ref<ClientType[]>(['claude', 'cursor', 'copilot']);

  /** 搜索关键词 */
  const searchKeyword = ref('');

  /** 加载状态 */
  const isLoading = ref(false);
  const isInstalling = ref(false);
  const isSyncing = ref(false);
  const error = ref<string | null>(null);

  /** 安装进度 */
  const installProgress = ref<Map<string, { progress: number; status: string }>>(new Map());

  // ==========================================================================
  // 计算属性
  // ==========================================================================

  /** 已安装插件名称集合 */
  const installedPluginNames = computed(() =>
    new Set(installedPlugins.value.map(p => p.name))
  );

  /** 检查插件是否已安装 */
  const isPluginInstalled = (pluginName: string) =>
    installedPluginNames.value.has(pluginName);

  /** 过滤后的市场插件 */
  const filteredPlugins = computed(() => {
    let result = plugins.value;

    // 来源过滤
    if (activeSourceId.value) {
      result = result.filter(p => p.sourceId === activeSourceId.value);
    }

    // 搜索过滤
    if (searchKeyword.value) {
      const kw = searchKeyword.value.toLowerCase();
      result = result.filter(p =>
        p.name.toLowerCase().includes(kw) ||
        p.description.toLowerCase().includes(kw) ||
        p.tags.some(t => t.toLowerCase().includes(kw))
      );
    }

    // 排除已禁用的
    result = result.filter(p => !p.disabled);

    return result;
  });

  /** 来源统计 */
  const sourceStats = computed(() => {
    const stats: Record<string, number> = {};
    for (const plugin of plugins.value) {
      stats[plugin.sourceId] = (stats[plugin.sourceId] || 0) + 1;
    }
    return stats;
  });

  // ==========================================================================
  // 来源管理 (保留现有逻辑)
  // ==========================================================================

  /** 加载来源列表 */
  async function fetchSources(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 allagents marketplace 获取来源
      const result = await invoke<{ success: boolean; data?: { output?: string }; error?: string }>(
        'allagents_marketplace_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.output) {
        sources.value = parseMarketplaceOutput(result.data.output);
      }

      // 同时加载本地预设来源
      const localResult = await invoke<{ success: boolean; data?: MarketplaceSource[] }>(
        'get_marketplace_sources'
      );

      if (localResult.success && localResult.data) {
        // 合并，避免重复
        const existingIds = new Set(sources.value.map(s => s.id));
        for (const source of localResult.data) {
          if (!existingIds.has(source.id)) {
            sources.value.push(source);
          }
        }
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 添加来源 */
  async function addSource(source: Omit<MarketplaceSource, 'id'>): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_marketplace_add',
        {
          workspacePath: '',
          source: source.url,
          name: source.name,
          branch: source.branch,
        }
      );

      if (result.success) {
        sources.value.push({
          ...source,
          id: source.name,
        });
        return true;
      } else {
        error.value = result.error ?? '添加来源失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 移除来源 */
  async function removeSource(sourceId: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_marketplace_remove',
        { workspacePath: '', name: sourceId }
      );

      if (result.success) {
        sources.value = sources.value.filter(s => s.id !== sourceId);
        return true;
      } else {
        error.value = result.error ?? '移除来源失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 切换活动来源 */
  function switchSource(sourceId: string) {
    activeSourceId.value = sourceId;
    searchKeyword.value = '';
  }

  // ==========================================================================
  // 插件管理
  // ==========================================================================

  /** 加载市场插件列表 */
  async function fetchPlugins(sourceId?: string): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: MarketplacePluginData[] }>(
        'fetch_marketplace_plugins',
        { sourceId: sourceId || activeSourceId.value }
      );

      if (result.success && result.data) {
        plugins.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 加载已安装插件 */
  async function fetchInstalledPlugins(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 allagents 获取已安装的插件
      const result = await invoke<{ success: boolean; data?: { plugins?: unknown[] } }>(
        'allagents_plugin_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.plugins) {
        installedPlugins.value = result.data.plugins.map((p) =>
          convertAllagentsPlugin(p)
        );
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 安装插件 */
  async function installPlugin(
    plugin: MarketplacePluginData,
    options: {
      scope?: 'user' | 'project';
      clients?: string[];
    } = {}
  ): Promise<boolean> {
    isInstalling.value = true;
    error.value = null;

    try {
      // 更新进度
      installProgress.value.set(plugin.id, {
        progress: 0,
        status: 'downloading',
      });

      const spec = `${plugin.name}@${sources.value.find(s => s.id === plugin.sourceId)?.name ?? plugin.sourceId}`;

      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_plugin_install',
        {
          workspacePath: '',
          pluginSpec: spec,
          scope: options.scope ?? 'project',
          skills: undefined,
        }
      );

      if (result.success) {
        // 添加到已安装列表
        installedPlugins.value.push({
          id: plugin.id,
          name: plugin.name,
          description: plugin.description,
          version: plugin.version,
          source: {
            type: 'marketplace',
            marketplace: plugin.sourceId,
          },
          scope: 'project',
          type: 'skill',
          tags: plugin.tags,
          categories: plugin.categories,
          installed: true,
          enabled: true,
          installedAt: new Date().toISOString(),
          syncTargets: [],
          syncStatus: 'pending',
          targetClients: options.clients ?? [],
          allagentsSpec: spec,
        });

        installProgress.value.set(plugin.id, {
          progress: 100,
          status: 'complete',
        });

        return true;
      } else {
        error.value = result.error ?? '安装插件失败';
        installProgress.value.set(plugin.id, {
          progress: 0,
          status: 'error',
        });
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isInstalling.value = false;
    }
  }

  /** 卸载插件 */
  async function uninstallPlugin(plugin: UnifiedPlugin): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const spec = plugin.allagentsSpec ?? plugin.name;

      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_plugin_uninstall',
        { workspacePath: '', pluginSpec: spec }
      );

      if (result.success) {
        installedPlugins.value = installedPlugins.value.filter(
          p => p.id !== plugin.id
        );
        return true;
      } else {
        error.value = result.error ?? '卸载插件失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  // ==========================================================================
  // 同步
  // ==========================================================================

  /** 同步所有插件 */
  async function syncAll(targetClient?: string): Promise<boolean> {
    isSyncing.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: unknown; error?: string }>(
        'allagents_update',
        {
          workspacePath: '',
          offline: false,
          dryRun: false,
          client: targetClient,
        }
      );

      if (result.success) {
        // 更新已安装插件的同步状态
        for (const plugin of installedPlugins.value) {
          plugin.syncStatus = 'synced';
        }
        return true;
      } else {
        error.value = result.error ?? '同步失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  // ==========================================================================
  // 工具函数
  // ==========================================================================

  /** 将 allagents 插件转换为 UnifiedPlugin */
  function convertAllagentsPlugin(raw: unknown): UnifiedPlugin {
    const r = raw as Record<string, unknown>;
    return {
      id: String(r.name ?? ''),
      name: String(r.name ?? ''),
      description: String(r.description ?? ''),
      version: String(r.version ?? ''),
      source: {
        type: 'marketplace',
        marketplace: String(r.source ?? ''),
      },
      scope: 'project',
      type: 'skill',
      tags: [],
      categories: [],
      installed: r.installed !== false,
      enabled: true,
      syncTargets: [],
      syncStatus: 'synced',
      targetClients: [],
      allagentsSpec: String(r.name ?? ''),
    };
  }

  /** 解析 allagents marketplace 输出 */
  function parseMarketplaceOutput(output: string): MarketplaceSource[] {
    const sources: MarketplaceSource[] = [];
    const lines = output.split('\n');
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('[')) {
        sources.push({
          id: trimmed,
          name: trimmed,
          url: `https://github.com/${trimmed}`,
          enabled: true,
        });
      }
    }
    return sources;
  }

  /** 清除错误 */
  function clearError() {
    error.value = null;
  }

  // ==========================================================================
  // 返回
  // ==========================================================================

  return {
    // 状态
    sources,
    plugins,
    installedPlugins,
    activeSourceId,
    targetClients,
    searchKeyword,
    isLoading,
    isInstalling,
    isSyncing,
    error,
    installProgress,

    // 计算属性
    installedPluginNames,
    isPluginInstalled,
    filteredPlugins,
    sourceStats,

    // 来源管理
    fetchSources,
    addSource,
    removeSource,
    switchSource,

    // 插件管理
    fetchPlugins,
    fetchInstalledPlugins,
    installPlugin,
    uninstallPlugin,

    // 同步
    syncAll,

    // 工具
    clearError,
  };
});

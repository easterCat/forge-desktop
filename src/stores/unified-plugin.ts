/**
 * 统一 Plugin Store
 *
 * 替代现有的 11 个分散 Store，提供统一的插件/技能/代理/规则/MCP 管理。
 * 通过 AllAgentsService 与 allagents CLI 交互。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  UnifiedPlugin,
  UnifiedMCP,
  PluginContentType,
  SyncStatus,
  ClientType,
  WorkspaceConfig,
} from '@/types/unified-plugin';
import { SUPPORTED_CLIENTS } from '@/types/unified-plugin';
import {
  generateWorkspaceConfig,
  configToYaml,
  mergeWorkspaceConfigs,
  validateWorkspaceConfig,
} from '@/services/allagents-config';

// ============================================================================
// Tauri Command 结果类型
// ============================================================================

interface CommandResult {
  success: boolean;
  data?: any;
  error?: string;
}

// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedPluginStore = defineStore('unified-plugin', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 所有已加载的插件 */
  const plugins = ref<UnifiedPlugin[]>([]);

  /** MCP 服务列表 */
  const mcpServers = ref<UnifiedMCP[]>([]);

  /** 同步目标客户端 */
  const targetClients = ref<ClientType[]>([...SUPPORTED_CLIENTS]);

  /** 工作区路径 */
  const workspacePath = ref<string>('');

  /** 加载状态 */
  const isLoading = ref(false);

  /** 错误信息 */
  const error = ref<string | null>(null);

  /** 同步状态 */
  const syncStatus = ref<{
    syncing: boolean;
    lastSyncAt?: string;
    syncedCount: number;
    errorCount: number;
  }>({
    syncing: false,
    syncedCount: 0,
    errorCount: 0,
  });

  /** workspace.yaml 配置缓存 */
  const configCache = ref<WorkspaceConfig | null>(null);

  // ==========================================================================
  // 计算属性
  // ==========================================================================

  /** 按类型分组的插件 */
  const pluginsByType = computed(() => {
    const grouped: Record<PluginContentType, UnifiedPlugin[]> = {
      skill: [],
      agent: [],
      rule: [],
      mcp: [],
      hook: [],
      command: [],
    };

    for (const plugin of plugins.value) {
      grouped[plugin.type].push(plugin);
    }

    return grouped;
  });

  /** 已安装的插件 */
  const installedPlugins = computed(() =>
    plugins.value.filter(p => p.installed)
  );

  /** 已安装的技能 */
  const installedSkills = computed(() =>
    plugins.value.filter(p => p.type === 'skill' && p.installed)
  );

  /** 已安装的代理 */
  const installedAgents = computed(() =>
    plugins.value.filter(p => p.type === 'agent' && p.installed)
  );

  /** 已安装的规则 */
  const installedRules = computed(() =>
    plugins.value.filter(p => p.type === 'rule' && p.installed)
  );

  /** 同步状态统计 */
  const syncStats = computed(() => {
    const synced = plugins.value.filter(p => p.syncStatus === 'synced').length;
    const pending = plugins.value.filter(p => p.syncStatus === 'pending').length;
    const errors = plugins.value.filter(p => p.syncStatus === 'error').length;
    const conflict = plugins.value.filter(p => p.syncStatus === 'conflict').length;

    return { synced, pending, errors, conflict };
  });

  /** 当前 workspace.yaml 内容 */
  const currentConfig = computed(() => {
    if (configCache.value) {
      return configCache.value;
    }

    return generateWorkspaceConfig(
      plugins.value,
      mcpServers.value,
      targetClients.value,
      { workspacePath: workspacePath.value }
    );
  });

  // ==========================================================================
  // Actions
  // ==========================================================================

  /** 初始化工作区 */
  async function initWorkspace(path: string, from?: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<CommandResult>('allagents_init', {
        workspacePath: path,
        from,
      });

      if (result.success) {
        workspacePath.value = path;
        return true;
      } else {
        error.value = result.error ?? 'Failed to initialize workspace';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 同步所有插件 */
  async function syncAll(
    options: {
      offline?: boolean;
      dryRun?: boolean;
      client?: string;
    } = {}
  ): Promise<boolean> {
    syncStatus.value.syncing = true;
    error.value = null;

    try {
      const result = await invoke<CommandResult>('allagents_update', {
        workspacePath: workspacePath.value,
        offline: options.offline ?? false,
        dryRun: options.dryRun ?? false,
        client: options.client,
      });

      if (result.success) {
        syncStatus.value.lastSyncAt = new Date().toISOString();
        syncStatus.value.syncedCount = result.data?.synced_count ?? 0;
        syncStatus.value.errorCount = result.data?.error_count ?? 0;

        // 更新插件同步状态
        for (const plugin of plugins.value) {
          plugin.syncStatus = 'synced';
        }

        return true;
      } else {
        error.value = result.error ?? 'Sync failed';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      syncStatus.value.syncing = false;
    }
  }

  /** 安装插件 */
  async function installPlugin(
    plugin: UnifiedPlugin,
    options: {
      scope?: 'user' | 'project';
      skills?: string[];
    } = {}
  ): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const spec = plugin.allagentsSpec ?? `${plugin.name}@${plugin.source.repo ?? plugin.source.marketplace ?? ''}`;

      const result = await invoke<CommandResult>('allagents_plugin_install', {
        workspacePath: workspacePath.value,
        pluginSpec: spec,
        scope: options.scope,
        skills: options.skills,
      });

      if (result.success) {
        plugin.installed = true;
        plugin.installedAt = new Date().toISOString();
        plugin.syncStatus = 'pending';
        return true;
      } else {
        error.value = result.error ?? 'Failed to install plugin';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 卸载插件 */
  async function uninstallPlugin(plugin: UnifiedPlugin): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const spec = plugin.allagentsSpec ?? plugin.name;

      const result = await invoke<CommandResult>('allagents_plugin_uninstall', {
        workspacePath: workspacePath.value,
        pluginSpec: spec,
      });

      if (result.success) {
        plugin.installed = false;
        plugin.syncStatus = 'pending';
        return true;
      } else {
        error.value = result.error ?? 'Failed to uninstall plugin';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 切换插件启用/禁用 */
  async function togglePlugin(plugin: UnifiedPlugin): Promise<boolean> {
    plugin.enabled = !plugin.enabled;
    plugin.syncStatus = 'pending';
    return true;
  }

  /** 添加 MCP 服务器 */
  async function addMcpServer(
    server: Omit<UnifiedMCP, 'groupIds' | 'tags' | 'healthStatus' | 'auditLog'>
  ): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const commandOrUrl = server.transport === 'http'
        ? server.url ?? ''
        : server.command ?? '';

      const result = await invoke<CommandResult>('allagents_mcp_add', {
        workspacePath: workspacePath.value,
        name: server.name,
        commandOrUrl,
        transport: server.transport,
        client: server.clients?.join(','),
      });

      if (result.success) {
        mcpServers.value.push({
          ...server,
          groupIds: [],
          tags: [],
          healthStatus: 'unknown',
          auditLog: [],
        });
        return true;
      } else {
        error.value = result.error ?? 'Failed to add MCP server';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 移除 MCP 服务器 */
  async function removeMcpServer(name: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<CommandResult>('allagents_mcp_remove', {
        workspacePath: workspacePath.value,
        name,
      });

      if (result.success) {
        mcpServers.value = mcpServers.value.filter(s => s.name !== name);
        return true;
      } else {
        error.value = result.error ?? 'Failed to remove MCP server';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 加载工作区状态 */
  async function loadStatus(): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<CommandResult>('allagents_status', {
        workspacePath: workspacePath.value,
      });

      if (result.success && result.data) {
        // 更新客户端列表
        if (result.data.clients) {
          targetClients.value = result.data.clients;
        }

        return true;
      } else {
        error.value = result.error ?? 'Failed to load status';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 生成 workspace.yaml */
  async function generateConfig(): Promise<string> {
    const config = generateWorkspaceConfig(
      plugins.value,
      mcpServers.value,
      targetClients.value,
      { workspacePath: workspacePath.value }
    );

    const errors = validateWorkspaceConfig(config);
    if (errors.some(e => e.severity === 'error')) {
      throw new Error(
        `Config validation failed: ${errors
          .filter(e => e.severity === 'error')
          .map(e => e.message)
          .join(', ')}`
      );
    }

    return configToYaml(config);
  }

  /** 更新目标客户端列表 */
  function setTargetClients(clients: ClientType[]) {
    targetClients.value = clients;
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
    plugins,
    mcpServers,
    targetClients,
    workspacePath,
    isLoading,
    error,
    syncStatus,

    // 计算属性
    pluginsByType,
    installedPlugins,
    installedSkills,
    installedAgents,
    installedRules,
    syncStats,
    currentConfig,

    // Actions
    initWorkspace,
    syncAll,
    installPlugin,
    uninstallPlugin,
    togglePlugin,
    addMcpServer,
    removeMcpServer,
    loadStatus,
    generateConfig,
    setTargetClients,
    clearError,
  };
});

/**
 * useAllAgents 组合式函数
 *
 * 提供 AllAgents 相关的响应式状态和操作方法，
 * 供 Vue 组件直接使用。
 */

import { computed, ref } from 'vue';
import { useUnifiedPluginStore } from '@/stores/unified-plugin';
import type {
  UnifiedPlugin,
  UnifiedMCP,
  ClientType,
  PluginContentType,
  WorkspaceConfig,
} from '@/types/unified-plugin';
import { SUPPORTED_CLIENTS, CLIENT_DISPLAY_NAMES } from '@/types/unified-plugin';

export function useAllAgents() {
  const store = useUnifiedPluginStore();

  // ==========================================================================
  // 响应式状态
  // ==========================================================================

  const isLoading = computed(() => store.isLoading);
  const error = computed(() => store.error);
  const syncStatus = computed(() => store.syncStatus);
  const syncStats = computed(() => store.syncStats);

  // ==========================================================================
  // 插件操作
  // ==========================================================================

  /** 按类型获取插件 */
  function getPluginsByType(type: PluginContentType): UnifiedPlugin[] {
    return store.pluginsByType[type];
  }

  /** 获取单个插件 */
  function getPluginById(id: string): UnifiedPlugin | undefined {
    return store.plugins.find(p => p.id === id);
  }

  /** 搜索插件 */
  function searchPlugins(query: string): UnifiedPlugin[] {
    const lowerQuery = query.toLowerCase();
    return store.plugins.filter(
      p =>
        p.name.toLowerCase().includes(lowerQuery) ||
        p.description?.toLowerCase().includes(lowerQuery) ||
        p.tags.some(t => t.toLowerCase().includes(lowerQuery))
    );
  }

  /** 安装插件并自动同步 */
  async function installAndSync(
    plugin: UnifiedPlugin,
    options: {
      scope?: 'user' | 'project';
      skills?: string[];
      autoSync?: boolean;
    } = {}
  ): Promise<boolean> {
    const installed = await store.installPlugin(plugin, options);
    if (installed && options.autoSync !== false) {
      await store.syncAll({ client: undefined });
    }
    return installed;
  }

  /** 卸载插件并自动同步 */
  async function uninstallAndSync(plugin: UnifiedPlugin): Promise<boolean> {
    const uninstalled = await store.uninstallPlugin(plugin);
    if (uninstalled) {
      await store.syncAll();
    }
    return uninstalled;
  }

  // ==========================================================================
  // MCP 操作
  // ==========================================================================

  /** 获取 MCP 服务器列表 */
  const mcpServers = computed(() => store.mcpServers);

  /** 添加 MCP 服务器并自动同步 */
  async function addMcpAndSync(
    server: Omit<UnifiedMCP, 'groupIds' | 'tags' | 'healthStatus' | 'auditLog'>
  ): Promise<boolean> {
    const added = await store.addMcpServer(server);
    if (added) {
      await store.syncAll();
    }
    return added;
  }

  /** 移除 MCP 服务器并自动同步 */
  async function removeMcpAndSync(name: string): Promise<boolean> {
    const removed = await store.removeMcpServer(name);
    if (removed) {
      await store.syncAll();
    }
    return removed;
  }

  // ==========================================================================
  // 客户端操作
  // ==========================================================================

  /** 获取支持的客户端列表 */
  const supportedClients = computed(() =>
    SUPPORTED_CLIENTS.map(c => ({
      id: c,
      name: CLIENT_DISPLAY_NAMES[c],
      selected: store.targetClients.includes(c),
    }))
  );

  /** 切换客户端选择 */
  function toggleClient(client: ClientType) {
    const current = [...store.targetClients];
    const index = current.indexOf(client);
    if (index >= 0) {
      current.splice(index, 1);
    } else {
      current.push(client);
    }
    store.setTargetClients(current);
  }

  /** 选择所有客户端 */
  function selectAllClients() {
    store.setTargetClients([...SUPPORTED_CLIENTS]);
  }

  /** 取消选择所有客户端 */
  function deselectAllClients() {
    store.setTargetClients([]);
  }

  // ==========================================================================
  // 同步操作
  // ==========================================================================

  /** 执行同步 */
  async function sync(
    options: {
      offline?: boolean;
      dryRun?: boolean;
      client?: string;
    } = {}
  ): Promise<boolean> {
    return store.syncAll(options);
  }

  /** 预览同步变更 (dry-run) */
  async function previewSync(): Promise<{
    syncedCount: number;
    errorCount: number;
  }> {
    const result = await store.syncAll({ dryRun: true });
    return {
      syncedCount: store.syncStatus.syncedCount,
      errorCount: store.syncStatus.errorCount,
    };
  }

  // ==========================================================================
  // 配置操作
  // ==========================================================================

  /** 生成 workspace.yaml 内容 */
  async function generateWorkspaceYaml(): Promise<string> {
    return store.generateConfig();
  }

  /** 获取当前配置 */
  const currentConfig = computed(() => store.currentConfig);

  /** 初始化工作区 */
  async function initWorkspace(
    path: string,
    from?: string
  ): Promise<boolean> {
    return store.initWorkspace(path, from);
  }

  /** 加载工作区状态 */
  async function loadWorkspaceStatus(): Promise<boolean> {
    return store.loadStatus();
  }

  // ==========================================================================
  // 统计信息
  // ==========================================================================

  /** 插件统计 */
  const stats = computed(() => ({
    total: store.plugins.length,
    installed: store.installedPlugins.length,
    skills: store.installedSkills.length,
    agents: store.installedAgents.length,
    rules: store.installedRules.length,
    mcpServers: store.mcpServers.length,
    syncStats: store.syncStats,
  }));

  // ==========================================================================
  // 返回
  // ==========================================================================

  return {
    // 状态
    isLoading,
    error,
    syncStatus,
    stats,

    // 插件操作
    getPluginsByType,
    getPluginById,
    searchPlugins,
    installAndSync,
    uninstallAndSync,

    // MCP 操作
    mcpServers,
    addMcpAndSync,
    removeMcpAndSync,

    // 客户端操作
    supportedClients,
    toggleClient,
    selectAllClients,
    deselectAllClients,

    // 同步操作
    sync,
    previewSync,

    // 配置操作
    generateWorkspaceYaml,
    currentConfig,
    initWorkspace,
    loadWorkspaceStatus,

    // 清除错误
    clearError: store.clearError,
  };
}

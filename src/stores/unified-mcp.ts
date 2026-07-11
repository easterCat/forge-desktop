/**
 * MCP 适配器 Store
 *
 * 在现有 MCPStore 基础上添加 AllAgentsService 同步能力。
 * 保留 Forge Desktop 的高级功能（健康检查、审计日志、分组），
 * 同步层委托给 AllAgentsService。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  UnifiedMCP,
  McpTransport,
  McpHealthStatus,
  AuditEntry,
  ClientType,
} from '@/types/unified-plugin';


// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedMcpStore = defineStore('unified-mcp', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 所有 MCP 服务器 */
  const servers = ref<UnifiedMCP[]>([]);

  /** MCP 分组 */
  const groups = ref<{ id: string; name: string; serverNames: string[] }[]>([]);

  /** 同步目标客户端 */
  const targetClients = ref<ClientType[]>(['claude', 'cursor', 'copilot']);

  /** 加载状态 */
  const isLoading = ref(false);
  const isSyncing = ref(false);
  const error = ref<string | null>(null);

  /** 搜索关键词 */
  const searchKeyword = ref('');

  /** 健康检查状态 */
  const healthCheckInterval = ref<number | null>(null);

  // ==========================================================================
  // 计算属性
  // ==========================================================================

  /** 按传输协议分组 */
  const serversByTransport = computed(() => {
    const grouped: Record<McpTransport, UnifiedMCP[]> = {
      http: [],
      stdio: [],
    };
    for (const server of servers.value) {
      grouped[server.transport].push(server);
    }
    return grouped;
  });

  /** 过滤后的服务器 */
  const filteredServers = computed(() => {
    if (!searchKeyword.value) {
      return servers.value;
    }
    const kw = searchKeyword.value.toLowerCase();
    return servers.value.filter(s =>
      s.name.toLowerCase().includes(kw) ||
      s.tags.some(t => t.toLowerCase().includes(kw))
    );
  });

  /** 健康状态统计 */
  const healthStats = computed(() => {
    const healthy = servers.value.filter(s => s.healthStatus === 'healthy').length;
    const unhealthy = servers.value.filter(s => s.healthStatus === 'unhealthy').length;
    const unknown = servers.value.filter(s => s.healthStatus === 'unknown').length;
    return { healthy, unhealthy, unknown, total: servers.value.length };
  });

  /** 分组统计 */
  const groupStats = computed(() => {
    const stats: Record<string, number> = {};
    for (const group of groups.value) {
      stats[group.name] = group.serverNames.length;
    }
    return stats;
  });

  // ==========================================================================
  // CRUD 操作
  // ==========================================================================

  /** 加载 MCP 服务器列表 */
  async function fetchServers(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 allagents 获取 MCP 列表
      const result = await invoke<{
        success: boolean;
        data?: { servers?: unknown[] };
        error?: string;
      }>(
        'allagents_mcp_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.servers) {
        servers.value = result.data.servers.map((s) =>
          convertAllagentsMcp(s)
        );
      }

      // 同时加载分组信息
      await fetchGroups();
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 添加 MCP 服务器 */
  async function addServer(
    server: Omit<UnifiedMCP, 'groupIds' | 'tags' | 'healthStatus' | 'auditLog'>
  ): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const commandOrUrl = server.transport === 'http'
        ? server.url ?? ''
        : server.command ?? '';

      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_mcp_add',
        {
          workspacePath: '',
          name: server.name,
          commandOrUrl,
          transport: server.transport,
          client: server.clients?.join(','),
        }
      );

      if (result.success) {
        servers.value.push({
          ...server,
          groupIds: [],
          tags: [],
          healthStatus: 'unknown',
          auditLog: [],
        });
        return true;
      } else {
        error.value = result.error ?? '添加 MCP 服务器失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 更新 MCP 服务器 */
  async function updateServer(
    name: string,
    updates: Partial<UnifiedMCP>
  ): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      // 先移除旧的
      await invoke('allagents_mcp_remove', {
        workspacePath: '',
        name,
      });

      // 再添加新的
      const server = servers.value.find(s => s.name === name);
      if (server) {
        const updated = { ...server, ...updates };
        const commandOrUrl = updated.transport === 'http'
          ? updated.url ?? ''
          : updated.command ?? '';

        const result = await invoke<{ success: boolean; error?: string }>(
          'allagents_mcp_add',
          {
            workspacePath: '',
            name: updated.name,
            commandOrUrl,
            transport: updated.transport,
            client: updated.clients?.join(','),
          }
        );

        if (result.success) {
          const idx = servers.value.findIndex(s => s.name === name);
          if (idx >= 0) {
            servers.value[idx] = updated;
          }
          return true;
        } else {
          error.value = result.error ?? '更新 MCP 服务器失败';
          return false;
        }
      }

      return false;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 删除 MCP 服务器 */
  async function removeServer(name: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_mcp_remove',
        { workspacePath: '', name }
      );

      if (result.success) {
        servers.value = servers.value.filter(s => s.name !== name);
        return true;
      } else {
        error.value = result.error ?? '删除 MCP 服务器失败';
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
  // 健康检查 (Forge Desktop 保留功能)
  // ==========================================================================

  /** 检查单个服务器健康状态 */
  async function checkHealth(serverName: string): Promise<McpHealthStatus> {
    try {
      const result = await invoke<{ success: boolean; data?: unknown }>(
        'check_mcp_service_health',
        { serviceName: serverName }
      );

      const status: McpHealthStatus = result.success ? 'healthy' : 'unhealthy';

      // 更新本地状态
      const server = servers.value.find(s => s.name === serverName);
      if (server) {
        server.healthStatus = status;
        server.lastHealthCheck = new Date().toISOString();
      }

      return status;
    } catch {
      const server = servers.value.find(s => s.name === serverName);
      if (server) {
        server.healthStatus = 'unhealthy';
      }
      return 'unhealthy';
    }
  }

  /** 检查所有服务器健康状态 */
  async function checkAllHealth(): Promise<void> {
    for (const server of servers.value) {
      await checkHealth(server.name);
    }
  }

  /** 开始定期健康检查 */
  function startHealthCheck(intervalMs: number = 60000) {
    stopHealthCheck();
    healthCheckInterval.value = window.setInterval(checkAllHealth, intervalMs);
  }

  /** 停止健康检查 */
  function stopHealthCheck() {
    if (healthCheckInterval.value) {
      clearInterval(healthCheckInterval.value);
      healthCheckInterval.value = null;
    }
  }

  // ==========================================================================
  // 审计日志 (Forge Desktop 保留功能)
  // ==========================================================================

  /** 记录审计日志 */
  function logAudit(serverName: string, action: string, detail: string) {
    const server = servers.value.find(s => s.name === serverName);
    if (server) {
      server.auditLog.push({
        timestamp: new Date().toISOString(),
        action,
        detail,
      });
    }
  }

  /** 获取审计日志 */
  function getAuditLog(serverName: string): AuditEntry[] {
    const server = servers.value.find(s => s.name === serverName);
    return server?.auditLog ?? [];
  }

  // ==========================================================================
  // 分组管理 (Forge Desktop 保留功能)
  // ==========================================================================

  /** 加载分组 */
  async function fetchGroups(): Promise<void> {
    try {
      const result = await invoke<{ success: boolean; data?: { id: string; name: string; serverNames: string[] }[] }>(
        'get_mcp_groups'
      );

      if (result.success && result.data) {
        groups.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  /** 创建分组 */
  async function createGroup(name: string, serverNames: string[]): Promise<boolean> {
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'create_mcp_group',
        { name, serverNames }
      );

      if (result.success) {
        await fetchGroups();
        return true;
      } else {
        error.value = result.error ?? '创建分组失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  /** 更新分组 */
  async function updateGroup(
    groupId: string,
    name: string,
    serverNames: string[]
  ): Promise<boolean> {
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'update_mcp_group',
        { groupId, name, serverNames }
      );

      if (result.success) {
        await fetchGroups();
        return true;
      } else {
        error.value = result.error ?? '更新分组失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  /** 删除分组 */
  async function deleteGroup(groupId: string): Promise<boolean> {
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'delete_mcp_group',
        { groupId }
      );

      if (result.success) {
        groups.value = groups.value.filter(g => g.id !== groupId);
        return true;
      } else {
        error.value = result.error ?? '删除分组失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  // ==========================================================================
  // AllAgents 同步
  // ==========================================================================

  /** 同步 MCP 配置到所有客户端 */
  async function syncAll(targetClient?: string): Promise<boolean> {
    isSyncing.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_update',
        {
          workspacePath: '',
          offline: false,
          dryRun: false,
          client: targetClient,
        }
      );

      if (result.success) {
        logAudit('system', 'sync', 'MCP 配置已同步到所有客户端');
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

  /** 将 allagents MCP 转换为 UnifiedMCP */
  function convertAllagentsMcp(raw: unknown): UnifiedMCP {
    const r = raw as Record<string, unknown>;
    const transport = String(r.type ?? 'stdio') as McpTransport;
    return {
      name: String(r.name ?? ''),
      transport,
      url: String(r.url ?? ''),
      command: String(r.command ?? ''),
      args: Array.isArray(r.args) ? (r.args as string[]) : [],
      env: (r.env && typeof r.env === 'object') ? (r.env as Record<string, string>) : {},
      headers: (r.headers && typeof r.headers === 'object') ? (r.headers as Record<string, string>) : {},
      clients: Array.isArray(r.clients) ? (r.clients as string[]) : [],
      groupIds: [],
      tags: [],
      healthStatus: 'unknown',
      auditLog: [],
    };
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
    servers,
    groups,
    targetClients,
    isLoading,
    isSyncing,
    error,
    searchKeyword,

    // 计算属性
    serversByTransport,
    filteredServers,
    healthStats,
    groupStats,

    // CRUD
    fetchServers,
    addServer,
    updateServer,
    removeServer,

    // 健康检查
    checkHealth,
    checkAllHealth,
    startHealthCheck,
    stopHealthCheck,

    // 审计日志
    logAudit,
    getAuditLog,

    // 分组
    fetchGroups,
    createGroup,
    updateGroup,
    deleteGroup,

    // 同步
    syncAll,

    // 工具
    clearError,
  };
});

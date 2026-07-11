/**
 * Agent 适配器 Store
 *
 * 在现有 AgentStore 基础上添加 AllAgentsService 同步能力，
 * 实现声明式管理和多客户端同步。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  SyncTarget,
  ClientType,
} from '@/types/unified-plugin';


// ============================================================================
// Agent 类型 (保留现有接口)
// ============================================================================

export interface Agent {
  id: string;
  name: string;
  description: string;
  emoji?: string;
  color?: string;
  department: string;
  content: string;
  source: string;
  tags?: string;
  installedTargets?: string;
  isCustom: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AgentDepartment {
  id: string;
  name: string;
  emoji: string;
  count: number;
}

/** 部门常量 (保留兼容) */
export const AGENT_DEPARTMENTS: AgentDepartment[] = [
  { id: 'academic', name: '学术研究', emoji: '📚', count: 0 },
  { id: 'design', name: '设计', emoji: '🎨', count: 0 },
  { id: 'engineering', name: '工程', emoji: '⚙️', count: 0 },
  { id: 'finance', name: '财务', emoji: '💰', count: 0 },
  { id: 'game-development', name: '游戏开发', emoji: '🎮', count: 0 },
  { id: 'hr', name: '人力资源', emoji: '👥', count: 0 },
  { id: 'legal', name: '法务', emoji: '⚖️', count: 0 },
  { id: 'marketing', name: '市场营销', emoji: '📢', count: 0 },
  { id: 'paid-media', name: '付费媒体', emoji: '📺', count: 0 },
  { id: 'product', name: '产品', emoji: '📦', count: 0 },
  { id: 'project-management', name: '项目管理', emoji: '📋', count: 0 },
  { id: 'sales', name: '销售', emoji: '💼', count: 0 },
  { id: 'spatial-computing', name: '空间计算', emoji: '🌐', count: 0 },
  { id: 'specialized', name: '专业领域', emoji: '🔬', count: 0 },
  { id: 'strategy', name: '战略', emoji: '🎯', count: 0 },
  { id: 'supply-chain', name: '供应链', emoji: '🔗', count: 0 },
  { id: 'support', name: '支持', emoji: '🤝', count: 0 },
  { id: 'testing', name: '测试', emoji: '🧪', count: 0 },
];

// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedAgentStore = defineStore('unified-agent', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 所有代理 */
  const agents = ref<Agent[]>([]);

  /** 选中的部门 */
  const selectedDepartment = ref<string | null>(null);

  /** 搜索关键词 */
  const searchQuery = ref('');

  /** 同步目标客户端 */
  const targetClients = ref<ClientType[]>(['claude', 'cursor', 'copilot']);

  /** 加载状态 */
  const isLoading = ref(false);
  const isSyncing = ref(false);
  const error = ref<string | null>(null);

  /** 同步状态 */
  const syncStatus = ref<Record<string, SyncTarget[]>>({});

  // ==========================================================================
  // 计算属性
  // ==========================================================================

  /** 部门统计 (动态计算) */
  const departments = computed(() => {
    const counts: Record<string, number> = {};
    for (const agent of agents.value) {
      counts[agent.department] = (counts[agent.department] || 0) + 1;
    }

    return AGENT_DEPARTMENTS.map(dept => ({
      ...dept,
      count: counts[dept.id] || 0,
    })).filter(d => d.count > 0);
  });

  /** 过滤后的代理 */
  const filteredAgents = computed(() => {
    let result = agents.value;

    if (selectedDepartment.value) {
      result = result.filter(a => a.department === selectedDepartment.value);
    }

    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(a =>
        a.name.toLowerCase().includes(q) ||
        a.description.toLowerCase().includes(q) ||
        a.department.toLowerCase().includes(q)
      );
    }

    return result;
  });

  /** 按部门分组 */
  const agentsByDepartment = computed(() => {
    const grouped: Record<string, Agent[]> = {};
    for (const agent of filteredAgents.value) {
      if (!grouped[agent.department]) {
        grouped[agent.department] = [];
      }
      grouped[agent.department].push(agent);
    }
    return grouped;
  });

  /** 已安装到目标的代理数量 */
  const installedCount = computed(() =>
    agents.value.filter(a => {
      const targets = getInstalledTargets(a);
      return targets.length > 0;
    }).length
  );

  // ==========================================================================
  // CRUD 操作 (保留现有 API)
  // ==========================================================================

  /** 加载代理列表 */
  async function fetchAgents(department?: string): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: Agent[]; error?: string }>(
        'get_agents',
        { department: department || null }
      );

      if (result.success && result.data) {
        agents.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 搜索代理 */
  async function searchAgents(query: string): Promise<void> {
    searchQuery.value = query;
  }

  /** 创建代理 */
  async function createAgent(agent: Omit<Agent, 'id' | 'createdAt' | 'updatedAt'>): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: Agent; error?: string }>(
        'create_agent',
        {
          agent: {
            ...agent,
            id: crypto.randomUUID(),
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          },
        }
      );

      if (result.success && result.data) {
        agents.value.push(result.data);
        return true;
      } else {
        error.value = result.error ?? '创建代理失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 更新代理 */
  async function updateAgent(agent: Agent): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'update_agent',
        {
          agent: {
            ...agent,
            updatedAt: new Date().toISOString(),
          },
        }
      );

      if (result.success) {
        const idx = agents.value.findIndex(a => a.id === agent.id);
        if (idx >= 0) {
          agents.value[idx] = agent;
        }
        return true;
      } else {
        error.value = result.error ?? '更新代理失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 删除代理 */
  async function deleteAgent(agentId: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'delete_agent',
        { agentId }
      );

      if (result.success) {
        agents.value = agents.value.filter(a => a.id !== agentId);
        return true;
      } else {
        error.value = result.error ?? '删除代理失败';
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
  // 安装/卸载 (保留现有 API)
  // ==========================================================================

  /** 安装代理到目标 */
  async function installAgent(agentId: string, target: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'install_agent_to_target',
        { agentId, target }
      );

      if (result.success) {
        // 更新本地状态
        const agent = agents.value.find(a => a.id === agentId);
        if (agent) {
          const targets = getInstalledTargets(agent);
          if (!targets.includes(target)) {
            targets.push(target);
            agent.installedTargets = JSON.stringify(targets);
          }
        }
        return true;
      } else {
        error.value = result.error ?? '安装代理失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 从目标卸载代理 */
  async function uninstallAgent(agentId: string, target: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'uninstall_agent_from_target',
        { agentId, target }
      );

      if (result.success) {
        const agent = agents.value.find(a => a.id === agentId);
        if (agent) {
          const targets = getInstalledTargets(agent).filter(t => t !== target);
          agent.installedTargets = JSON.stringify(targets);
        }
        return true;
      } else {
        error.value = result.error ?? '卸载代理失败';
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
  // AllAgents 同步 (新增)
  // ==========================================================================

  /** 通过 allagents 同步所有代理 */
  async function syncAllAgents(targetClient?: string): Promise<boolean> {
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
        // 更新同步状态
        for (const agent of agents.value) {
          const targets = getInstalledTargets(agent);
          for (const target of targets) {
            if (!syncStatus.value[agent.id]) {
              syncStatus.value[agent.id] = [];
            }
            const existing = syncStatus.value[agent.id].find(
              t => t.client === target
            );
            if (existing) {
              existing.status = 'synced';
              existing.lastSyncedAt = new Date().toISOString();
            } else {
              syncStatus.value[agent.id].push({
                client: target,
                path: '',
                status: 'synced',
                lastSyncedAt: new Date().toISOString(),
              });
            }
          }
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

  /** 解析已安装目标 */
  function getInstalledTargets(agent: Agent): string[] {
    if (!agent.installedTargets) return [];
    try {
      return JSON.parse(agent.installedTargets);
    } catch {
      return [];
    }
  }

  /** 检查是否已安装到目标 */
  function isInstalledTo(agent: Agent, target: string): boolean {
    return getInstalledTargets(agent).includes(target);
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
    agents,
    selectedDepartment,
    searchQuery,
    targetClients,
    isLoading,
    isSyncing,
    error,
    syncStatus,

    // 计算属性
    departments,
    filteredAgents,
    agentsByDepartment,
    installedCount,

    // CRUD
    fetchAgents,
    searchAgents,
    createAgent,
    updateAgent,
    deleteAgent,

    // 安装/卸载
    installAgent,
    uninstallAgent,
    isInstalledTo,
    getInstalledTargets,

    // 同步
    syncAllAgents,

    // 工具
    clearError,
  };
});

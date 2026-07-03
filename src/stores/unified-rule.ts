/**
 * Rule 适配器 Store
 *
 * 在现有 RuleStore 基础上添加 workspace.files 同步能力。
 * Rules 通过 allagents workspace.yaml 的 workspace.files 机制同步到客户端。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  UnifiedPlugin,
  SyncTarget,
  ClientType,
} from '@/types/unified-plugin';
import { SUPPORTED_CLIENTS } from '@/types/unified-plugin';

// ============================================================================
// Rule 类型 (保留现有接口)
// ============================================================================

export interface Rule {
  id: string;
  softwareId: string;
  name: string;
  type: string;
  filePath: string;
  content: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedRuleStore = defineStore('unified-rule', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 所有规则 */
  const rules = ref<Rule[]>([]);

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

  /** 按软件分组 */
  const rulesBySoftware = computed(() => {
    const grouped: Record<string, Rule[]> = {};
    for (const rule of rules.value) {
      if (!grouped[rule.softwareId]) {
        grouped[rule.softwareId] = [];
      }
      grouped[rule.softwareId].push(rule);
    }
    return grouped;
  });

  /** 按类型分组 */
  const rulesByType = computed(() => {
    const grouped: Record<string, Rule[]> = {};
    for (const rule of rules.value) {
      if (!grouped[rule.type]) {
        grouped[rule.type] = [];
      }
      grouped[rule.type].push(rule);
    }
    return grouped;
  });

  /** 已激活的规则 */
  const activeRules = computed(() =>
    rules.value.filter(r => r.isActive)
  );

  /** 已同步的规则数量 */
  const syncedCount = computed(() =>
    rules.value.filter(r => {
      const targets = syncStatus.value[r.id] ?? [];
      return targets.some(t => t.status === 'synced');
    }).length
  );

  // ==========================================================================
  // CRUD 操作 (保留现有 API)
  // ==========================================================================

  /** 加载规则列表 */
  async function fetchRules(softwareId?: string): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: any; error?: string }>(
        'get_rules',
        { softwareId: softwareId || '' }
      );

      if (result.success && result.data) {
        rules.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 创建规则 */
  async function createRule(
    softwareId: string,
    name: string,
    type: string,
    filePath: string,
    content: string
  ): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: Rule; error?: string }>(
        'create_rule',
        {
          rule: {
            id: crypto.randomUUID(),
            softwareId,
            name,
            type,
            filePath,
            content,
            isActive: true,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          },
        }
      );

      if (result.success && result.data) {
        rules.value.push(result.data);
        return true;
      } else {
        error.value = result.error ?? '创建规则失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 更新规则 */
  async function updateRule(rule: Rule): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'update_rule',
        {
          rule: {
            ...rule,
            updatedAt: new Date().toISOString(),
          },
        }
      );

      if (result.success) {
        const idx = rules.value.findIndex(r => r.id === rule.id);
        if (idx >= 0) {
          rules.value[idx] = rule;
        }
        return true;
      } else {
        error.value = result.error ?? '更新规则失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 删除规则 */
  async function deleteRule(ruleId: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'delete_rule',
        { ruleId }
      );

      if (result.success) {
        rules.value = rules.value.filter(r => r.id !== ruleId);
        return true;
      } else {
        error.value = result.error ?? '删除规则失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 切换规则激活状态 */
  async function toggleRule(ruleId: string, isActive: boolean): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'toggle_rule',
        { ruleId, isActive }
      );

      if (result.success) {
        const rule = rules.value.find(r => r.id === ruleId);
        if (rule) {
          rule.isActive = isActive;
        }
        return true;
      } else {
        error.value = result.error ?? '切换规则状态失败';
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

  /** 通过 workspace.files 同步规则 */
  async function syncRules(targetClient?: string): Promise<boolean> {
    isSyncing.value = true;
    error.value = null;

    try {
      // 生成包含规则的 workspace.yaml 配置
      const config = generateWorkspaceConfigWithRules();

      // 写入配置文件
      const writeResult = await invoke<{ success: boolean; error?: string }>(
        'allagents_generate_config',
        {
          workspacePath: '',
          clients: targetClients.value,
          plugins: [],
          mcpServers: null,
        }
      );

      if (!writeResult.success) {
        error.value = writeResult.error ?? '生成配置失败';
        return false;
      }

      // 执行同步
      const syncResult = await invoke<{ success: boolean; data?: any; error?: string }>(
        'allagents_update',
        {
          workspacePath: '',
          offline: false,
          dryRun: false,
          client: targetClient,
        }
      );

      if (syncResult.success) {
        // 更新同步状态
        for (const rule of rules.value) {
          if (!rule.isActive) continue;

          if (!syncStatus.value[rule.id]) {
            syncStatus.value[rule.id] = [];
          }

          for (const client of targetClients.value) {
            const existing = syncStatus.value[rule.id].find(
              t => t.client === client
            );
            if (existing) {
              existing.status = 'synced';
              existing.lastSyncedAt = new Date().toISOString();
            } else {
              syncStatus.value[rule.id].push({
                client,
                path: rule.filePath,
                status: 'synced',
                lastSyncedAt: new Date().toISOString(),
              });
            }
          }
        }
        return true;
      } else {
        error.value = syncResult.error ?? '同步失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  /** 生成包含规则的 workspace 配置 */
  function generateWorkspaceConfigWithRules() {
    const activeRulesList = rules.value.filter(r => r.isActive);

    return {
      workspace: {
        files: activeRulesList.map(rule => ({
          source: rule.filePath,
          dest: rule.name,
        })),
      },
    };
  }

  // ==========================================================================
  // 工具函数
  // ==========================================================================

  /** 按软件获取规则 */
  function getRulesBySoftware(softwareId: string): Rule[] {
    return rules.value.filter(r => r.softwareId === softwareId);
  }

  /** 按类型获取规则 */
  function getRulesByType(type: string): Rule[] {
    return rules.value.filter(r => r.type === type);
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
    rules,
    targetClients,
    isLoading,
    isSyncing,
    error,
    syncStatus,

    // 计算属性
    rulesBySoftware,
    rulesByType,
    activeRules,
    syncedCount,

    // CRUD
    fetchRules,
    createRule,
    updateRule,
    deleteRule,
    toggleRule,

    // 同步
    syncRules,

    // 工具
    getRulesBySoftware,
    getRulesByType,
    clearError,
  };
});

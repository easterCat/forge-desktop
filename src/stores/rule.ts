import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Rule } from '@/types';

export const useRuleStore = defineStore('rule', () => {
  const rules = ref<Rule[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchRules(softwareId: string = '') {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<Rule[]>('get_rules', { softwareId });
      rules.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch rules';
    } finally {
      isLoading.value = false;
    }
  }

  async function createRule(
    softwareId: string,
    name: string,
    ruleType: string,
    filePath: string,
    content: string = ''
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const now = new Date().toISOString();
      const rule: Rule = {
        id: crypto.randomUUID(),
        softwareId,
        name,
        type: ruleType,
        filePath,
        content,
        isActive: true,
        createdAt: now,
        updatedAt: now,
      };
      await invoke('create_rule', { rule });
      rules.value.push(rule);
      return rule;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create rule';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateRule(rule: Rule) {
    try {
      isLoading.value = true;
      error.value = null;
      rule.updatedAt = new Date().toISOString();
      await invoke('update_rule', { rule });
      const index = rules.value.findIndex(r => r.id === rule.id);
      if (index !== -1) {
        rules.value[index] = rule;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update rule';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteRule(ruleId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_rule', { ruleId });
      rules.value = rules.value.filter(r => r.id !== ruleId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete rule';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function toggleRule(ruleId: string, isActive: boolean) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('toggle_rule', { ruleId, isActive });
      const rule = rules.value.find(r => r.id === ruleId);
      if (rule) {
        rule.isActive = isActive;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to toggle rule';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  function getRulesBySoftware(softwareId: string): Rule[] {
    return rules.value.filter(r => r.softwareId === softwareId);
  }

  function getRulesByType(type: string): Rule[] {
    return rules.value.filter(r => r.type === type);
  }

  return {
    rules,
    isLoading,
    error,
    fetchRules,
    createRule,
    updateRule,
    deleteRule,
    toggleRule,
    getRulesBySoftware,
    getRulesByType,
  };
});

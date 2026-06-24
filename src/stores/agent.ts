import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Agent, AgentImportResult } from '@/types';

export const useAgentStore = defineStore('agent', () => {
  const agents = ref<Agent[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const selectedDepartment = ref<string | null>(null);
  const searchQuery = ref('');

  // Computed: departments with counts
  const departments = computed(() => {
    const counts: Record<string, number> = {};
    for (const agent of agents.value) {
      counts[agent.department] = (counts[agent.department] || 0) + 1;
    }
    return Object.entries(counts).map(([id, count]) => ({ id, count }));
  });

  // Computed: filtered agents
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
        (a.tags && a.tags.toLowerCase().includes(q))
      );
    }
    return result;
  });

  // Computed: agents grouped by department
  const agentsByDepartment = computed(() => {
    const groups: Record<string, Agent[]> = {};
    for (const agent of filteredAgents.value) {
      if (!groups[agent.department]) groups[agent.department] = [];
      groups[agent.department].push(agent);
    }
    return groups;
  });

  // Fetch all agents, auto-import from marketplace if empty
  async function fetchAgents(department?: string) {
    try {
      isLoading.value = true;
      error.value = null;
      agents.value = await invoke<Agent[]>('get_agents', { department: department || null });
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch agents';
    } finally {
      isLoading.value = false;
    }
  }

  // Search agents
  async function searchAgents(query: string) {
    try {
      isLoading.value = true;
      error.value = null;
      agents.value = await invoke<Agent[]>('search_agents', { query });
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to search agents';
    } finally {
      isLoading.value = false;
    }
  }

  // Create agent
  async function createAgent(agent: Agent) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('create_agent', { agent });
      agents.value.push(agent);
      return agent;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create agent';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  // Update agent
  async function updateAgent(agent: Agent) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('update_agent', { agent });
      const index = agents.value.findIndex(a => a.id === agent.id);
      if (index !== -1) {
        agents.value[index] = agent;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update agent';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  // Delete agent
  async function deleteAgent(agentId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_agent', { agentId });
      agents.value = agents.value.filter(a => a.id !== agentId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete agent';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  // Import from repo
  async function importFromRepo(sourceDir: string): Promise<AgentImportResult> {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<AgentImportResult>('import_agents_from_repo', { sourceDir });
      // Refresh agents after import
      await fetchAgents();
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import agents';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  // Install agent to target
  async function installAgent(agentId: string, target: string): Promise<string> {
    try {
      error.value = null;
      const path = await invoke<string>('install_agent_to_target', { agentId, target });
      // Update local state
      const agent = agents.value.find(a => a.id === agentId);
      if (agent) {
        const targets: string[] = agent.installedTargets
          ? JSON.parse(agent.installedTargets)
          : [];
        if (!targets.includes(target)) {
          targets.push(target);
          agent.installedTargets = JSON.stringify(targets);
        }
      }
      return path;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to install agent';
      throw e;
    }
  }

  // Uninstall agent from target
  async function uninstallAgent(agentId: string, target: string) {
    try {
      error.value = null;
      await invoke('uninstall_agent_from_target', { agentId, target });
      // Update local state
      const agent = agents.value.find(a => a.id === agentId);
      if (agent && agent.installedTargets) {
        const targets: string[] = JSON.parse(agent.installedTargets);
        agent.installedTargets = JSON.stringify(targets.filter(t => t !== target));
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to uninstall agent';
      throw e;
    }
  }

  // Helper: check if agent is installed to a target
  function isInstalledTo(agent: Agent, target: string): boolean {
    if (!agent.installedTargets) return false;
    try {
      const targets: string[] = JSON.parse(agent.installedTargets);
      return targets.includes(target);
    } catch {
      return false;
    }
  }

  // Helper: get installed targets list
  function getInstalledTargets(agent: Agent): string[] {
    if (!agent.installedTargets) return [];
    try {
      return JSON.parse(agent.installedTargets);
    } catch {
      return [];
    }
  }

  return {
    agents,
    isLoading,
    error,
    selectedDepartment,
    searchQuery,
    departments,
    filteredAgents,
    agentsByDepartment,
    fetchAgents,
    searchAgents,
    createAgent,
    updateAgent,
    deleteAgent,
    importFromRepo,
    installAgent,
    uninstallAgent,
    isInstalledTo,
    getInstalledTargets,
  };
});

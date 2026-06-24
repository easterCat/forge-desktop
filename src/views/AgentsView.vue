<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Agents</h2>
      <span class="count">{{ filteredAgents.length }} agents</span>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="search-input">
        <svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search agents…"
          aria-label="Search agents"
        />
        <button
          v-show="searchQuery"
          class="clear-btn"
          aria-label="Clear search"
          @click="searchQuery = ''"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="#9A9A9A" stroke-width="1.5" stroke-linecap="round">
            <line x1="2" y1="2" x2="8" y2="8" /><line x1="8" y1="2" x2="2" y2="8" />
          </svg>
        </button>
      </div>
      <select v-model="deptFilter" class="filter-select" aria-label="Filter by department">
        <option value="all">All Departments</option>
        <option value="engineering">Engineering</option>
        <option value="design">Design</option>
        <option value="product">Product</option>
        <option value="quality">Quality</option>
        <option value="custom">Custom</option>
      </select>
      <select v-model="sourceFilter" class="filter-select" aria-label="Filter by source">
        <option value="all">All Sources</option>
        <option value="agency-agents-zh">agency-agents-zh</option>
        <option value="custom">Custom</option>
      </select>
      <div class="btn-group">
        <button class="btn btn-secondary btn-sm" @click="handleCreateAgent">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          Create Agent
        </button>
        <button class="btn btn-primary btn-sm" @click="handleImportAgents">
          Import agency-agents-zh
        </button>
      </div>
    </div>

    <!-- Agents Card Grid (virtualized) -->
    <VirtualGrid
      v-if="filteredAgents.length > 0"
      ref="virtualGridRef"
      :items="filteredAgents"
      :column-width="380"
      :row-height="280"
      :gap="16"
    >
      <template #default="{ item }">
        <AgentCard
          :agent="item as Agent"
          @click="openAgentDetails(item as Agent)"
          @install="handleInstall(item as Agent, $event)"
        />
      </template>
    </VirtualGrid>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <svg aria-hidden="true" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" />
      </svg>
      <h3>No agents found</h3>
      <p>Try adjusting your search or filters.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useAgentStore } from '@/stores/agent'
import AgentCard from '@/components/agents/AgentCard.vue'
import VirtualGrid from '@/components/common/VirtualGrid.vue'
import type { Agent } from '@/types/agent'

// Mock data from design prototype (forge-cross-platform-glass.html line 1158)
const mockAgents: Agent[] = [
  { id: '1', name: 'Software Architect', description: 'Designs system architecture and technical solutions', emoji: 'SA', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '2', name: 'Frontend Director', description: 'Leads frontend architecture and Vue 3 implementation', emoji: 'FD', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '3', name: 'Backend Director', description: 'Oversees Rust backend and Tauri IPC layer', emoji: 'BD', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","opencode"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '4', name: 'Design Director', description: 'Guards design system consistency and visual quality', emoji: 'DD', department: 'design', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '5', name: 'Design UX', description: 'User experience research and interaction design', emoji: 'UX', department: 'design', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '6', name: 'QA Director', description: 'Test strategy and quality assurance oversight', emoji: 'QA', department: 'quality', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","deepseek"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '7', name: 'Performance Engineer', description: 'Performance profiling and optimization', emoji: 'PE', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","opencode"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '8', name: 'Product Manager', description: 'Product strategy and requirement management', emoji: 'PM', department: 'product', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '9', name: 'Review Expert', description: 'Code review and quality gate enforcement', emoji: 'RE', department: 'quality', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","codex","mimo-code"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '10', name: 'Deployment Engineer', description: 'CI/CD pipeline and release management', emoji: 'DE', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","openclaw"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '11', name: 'Custom Agent', description: 'User-defined agent role', emoji: 'CA', department: 'custom', content: '', source: 'custom', tags: '', installedTargets: '[]', isCustom: true, createdAt: '', updatedAt: '' },
]

// Store
const agentStore = useAgentStore()

// State
const useMock = ref(false)
const searchQuery = ref('')
const deptFilter = ref('all')
const sourceFilter = ref('all')

// Data source: store-first, mock fallback
const displayAgents = computed<Agent[]>(() => {
  if (useMock.value) return mockAgents
  return agentStore.agents
})

// Filtered agents
const filteredAgents = computed(() => {
  let result = displayAgents.value

  // Department filter
  if (deptFilter.value !== 'all') {
    result = result.filter(a => a.department === deptFilter.value)
  }

  // Source filter
  if (sourceFilter.value !== 'all') {
    result = result.filter(a => a.source === sourceFilter.value)
  }

  // Search filter
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(a =>
      a.name.toLowerCase().includes(q) ||
      a.description.toLowerCase().includes(q) ||
      (a.tags && a.tags.toLowerCase().includes(q))
    )
  }

  return result
})

// VirtualGrid ref for scroll reset
const virtualGridRef = ref<InstanceType<typeof VirtualGrid> | null>(null)

// Reset scroll position when filters change
watch([deptFilter, sourceFilter, searchQuery], () => {
  virtualGridRef.value?.resetScroll()
})

function openAgentDetails(agent: Agent): void {
  // TODO: Open agent details dialog
  console.log('Open agent details:', agent.name)
}

function handleInstall(agent: Agent, targets: string[]): void {
  // TODO: Install agent to selected targets
  console.log('Install', agent.name, 'to', targets)
}

function handleCreateAgent(): void {
  // TODO: Open create agent dialog
  console.log('Create agent')
}

function handleImportAgents(): void {
  // TODO: Import from agency-agents-zh
  console.log('Import agency-agents-zh')
}

// Store-first + mock fallback
onMounted(async () => {
  try {
    await agentStore.fetchAgents()
    if (agentStore.agents.length === 0) useMock.value = true
  } catch {
    useMock.value = true
  }
})
</script>

<style scoped>
/* ── Section Header ── */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0;
  padding-bottom: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.30);
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title);
  margin: 0;
}

.count {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-muted);
  background: rgba(255, 255, 255, 0.32);
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.32);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

/* ── Filter Bar ── */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1 1 180px;
  min-width: 0;
  position: relative;
}

.search-input input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.32);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.30);
  font-family: inherit;
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast), box-shadow var(--t-fast);
}

.search-input input:focus {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.40);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

.search-input input::placeholder {
  color: var(--fg-ghost);
}

.search-input svg {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.clear-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.32);
  cursor: pointer;
  border: none;
  padding: 0;
}

.filter-select {
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.32) url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%239A9A9A' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E") no-repeat right 12px center;
  border: 1px solid rgba(255, 255, 255, 0.30);
  appearance: none;
  cursor: pointer;
  font-family: inherit;
  color: var(--fg);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.filter-select:focus {
  background-color: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.40);
}

.btn-group {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-left: auto;
}

/* ── Empty State ── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 24px;
  text-align: center;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.3;
}

.empty-state h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg-title);
  margin-bottom: 6px;
}

.empty-state p {
  font-size: 13px;
  color: var(--fg-ghost);
  max-width: 320px;
  line-height: 1.5;
}

</style>

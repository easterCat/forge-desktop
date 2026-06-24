# Skills & Agents UI Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite SkillsView.vue and AgentsView.vue to match the design prototype in `design/theme/forge-cross-platform-glass.html`, with store-first + mock fallback data integration.

**Architecture:** View-inline rewrite for both views. Skills cards are inline in the view (no SkillCard component). Agents cards use refactored AgentCard.vue with two-letter abbreviations and target chip selector. Both views try Pinia stores first, fall back to design prototype mock data.

**Tech Stack:** Vue 3 Composition API (`<script setup lang="ts">`), Pinia stores, CSS custom properties from design token system, existing common components (FilterBar, SearchInput, Badge, Button).

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/views/SkillsView.vue` | Rewrite | Skills page: source tabs (5), filter bar (search + type + status + Import + Sync All), card grid with glass style, mock data fallback |
| `src/views/AgentsView.vue` | Rewrite | Agents page: filter bar (search + department + source + Create/Import buttons), card grid using AgentCard, mock data fallback |
| `src/components/agents/AgentCard.vue` | Refactor | Agent card: two-letter abbreviation icon, department+source label, target chip selector, Install/View/more actions |

**NOT modified:** `src/stores/skill.ts`, `src/stores/agent.ts`, `src/types/skill.ts`, `src/types/agent.ts`, `src/assets/theme.css`, `src/components/skills/SkillCard.vue` (unused).

---

### Task 1: Rewrite SkillsView.vue

**Files:**
- Modify: `src/views/SkillsView.vue` (full rewrite)

- [ ] **Step 1: Rewrite SkillsView.vue with design-aligned structure**

Replace the entire file with the following. Key changes: 5 source tabs (remove "Cursor" tab), filter bar with Import + Sync All buttons, card grid with glass card style matching design prototype, store-first + mock fallback data pattern.

```vue
<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Skills</h2>
      <span class="count">{{ filteredSkills.length }} skills</span>
    </div>

    <!-- Source Tabs (5 tabs matching design prototype) -->
    <div class="source-tabs">
      <div
        v-for="tab in sourceTabs"
        :key="tab.id"
        class="source-tab"
        :class="{ active: activeSource === tab.id }"
        @click="activeSource = tab.id"
      >
        {{ tab.label }}
        <span class="tab-count">{{ tab.count }}</span>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="search-input">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search skills…"
        />
        <button
          v-show="searchQuery"
          class="clear-btn"
          @click="searchQuery = ''"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="#9A9A9A" stroke-width="1.5" stroke-linecap="round">
            <line x1="2" y1="2" x2="8" y2="8" /><line x1="8" y1="2" x2="2" y2="8" />
          </svg>
        </button>
      </div>
      <select v-model="typeFilter" class="filter-select">
        <option value="all">All Types</option>
        <option value="agent">Agent</option>
        <option value="command">Command</option>
        <option value="automation">Automation</option>
      </select>
      <select v-model="statusFilter" class="filter-select">
        <option value="all">All Status</option>
        <option value="enabled">Enabled</option>
        <option value="disabled">Disabled</option>
      </select>
      <div class="btn-group">
        <button class="btn btn-secondary btn-sm" @click="handleImport">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="17 8 12 3 7 8" /><line x1="12" y1="3" x2="12" y2="15" />
          </svg>
          Import
        </button>
        <button class="btn btn-primary btn-sm" @click="handleSyncAll">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
          Sync All
        </button>
      </div>
    </div>

    <!-- Skills Card Grid -->
    <div v-if="filteredSkills.length > 0" class="skills-grid">
      <div
        v-for="skill in filteredSkills"
        :key="skill.name"
        class="card skill-card"
      >
        <!-- Card Header -->
        <div class="skill-card-head">
          <div class="skill-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
            </svg>
          </div>
          <div class="skill-card-info">
            <div class="skill-name">
              {{ skill.name }}
              <span class="tag" :style="{ color: typeColor(skill.type), borderColor: typeColor(skill.type) + '30' }">
                {{ skill.type }}
              </span>
            </div>
          </div>
        </div>

        <!-- Description -->
        <p class="skill-desc">{{ skill.desc }}</p>

        <!-- Card Footer -->
        <div class="skill-card-footer">
          <span class="skill-software">{{ skill.software }}</span>
          <div class="btn-group">
            <div
              class="toggle"
              :class="{ on: getSkillEnabled(skill) }"
              @click="toggleSkill(skill)"
            ></div>
            <button class="btn btn-secondary btn-sm">Edit</button>
            <button class="btn-icon btn-sm" title="More actions">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="5" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="12" cy="19" r="1" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
      </svg>
      <h3>No skills found</h3>
      <p>Try adjusting your search or filters.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSkillStore } from '@/stores'

// Display type matching design prototype data structure
interface DisplaySkill {
  name: string
  type: 'agent' | 'command' | 'automation'
  desc: string
  software: string
  source: string
  enabled: boolean
}

// Mock data from design prototype (forge-cross-platform-glass.html line 1145)
const mockSkills: DisplaySkill[] = [
  { name: 'imagegen', type: 'agent', desc: 'AI image generation skill', software: 'Cursor', source: 'local', enabled: true },
  { name: 'frontend-design', type: 'command', desc: 'Frontend UI design helper', software: 'Cursor', source: 'local', enabled: true },
  { name: 'brandkit', type: 'agent', desc: 'Brand guidelines generator', software: 'Cursor', source: 'local', enabled: true },
  { name: 'skill-creator', type: 'automation', desc: 'Skill creation helper', software: 'Cursor', source: 'local', enabled: true },
  { name: 'canvas', type: 'command', desc: 'Live React canvas builder', software: 'Cursor', source: 'local', enabled: false },
  { name: 'shell', type: 'automation', desc: 'Shell command executor', software: 'Claude Desktop', source: 'anthropic', enabled: true },
  { name: 'documents', type: 'agent', desc: 'Document artifact creator', software: 'Cursor', source: 'marketplace', enabled: true },
  { name: 'spreadsheets', type: 'command', desc: 'Spreadsheet builder', software: 'Cursor', source: 'skills-sh', enabled: false },
]

// Type color palette from design prototype
const TYPE_COLORS: Record<string, string> = {
  agent: '#B8944A',
  command: '#5A6B7A',
  automation: '#5A8A64',
}

function typeColor(type: string): string {
  return TYPE_COLORS[type] || '#B8944A'
}

// Store
const skillStore = useSkillStore()

// State
const useMock = ref(false)
const searchQuery = ref('')
const typeFilter = ref('all')
const statusFilter = ref('all')
const activeSource = ref('all')
const skillsState = ref<Record<string, boolean>>({})

// Data source: store-first, mock fallback
const displaySkills = computed<DisplaySkill[]>(() => {
  if (useMock.value) return mockSkills
  // Map store skills to display format
  return skillStore.skills.map(s => ({
    name: s.name,
    type: (s.type as DisplaySkill['type']) || 'command',
    desc: s.name, // store doesn't have desc, use name as fallback
    software: s.softwareId,
    source: 'local',
    enabled: true,
  }))
})

// Source tabs (5 tabs per design)
const sourceTabs = computed(() => [
  { id: 'all', label: 'All', count: displaySkills.value.length },
  { id: 'local', label: 'Local', count: displaySkills.value.filter(s => s.source === 'local').length },
  { id: 'anthropic', label: 'Anthropic', count: displaySkills.value.filter(s => s.source === 'anthropic').length },
  { id: 'marketplace', label: 'Marketplace', count: displaySkills.value.filter(s => s.source === 'marketplace').length },
  { id: 'skills-sh', label: 'Skills.sh', count: displaySkills.value.filter(s => s.source === 'skills-sh').length },
])

// Filtered skills
const filteredSkills = computed(() => {
  const keyword = searchQuery.value.toLowerCase()
  return displaySkills.value.filter(skill => {
    // Source filter
    if (activeSource.value !== 'all' && skill.source !== activeSource.value) return false
    // Search filter
    if (keyword && !skill.name.toLowerCase().includes(keyword) && !skill.desc.toLowerCase().includes(keyword)) return false
    // Type filter
    if (typeFilter.value !== 'all' && skill.type !== typeFilter.value) return false
    // Status filter
    const enabled = getSkillEnabled(skill)
    if (statusFilter.value === 'enabled' && !enabled) return false
    if (statusFilter.value === 'disabled' && enabled) return false
    return true
  })
})

function getSkillEnabled(skill: DisplaySkill): boolean {
  return skillsState.value[skill.name] ?? skill.enabled
}

function toggleSkill(skill: DisplaySkill): void {
  skillsState.value[skill.name] = !(skillsState.value[skill.name] ?? skill.enabled)
}

function handleImport(): void {
  // TODO: Open import dialog
  console.log('Import skill')
}

function handleSyncAll(): void {
  // TODO: Sync all repositories
  console.log('Sync all')
}

// Store-first + mock fallback
onMounted(async () => {
  try {
    await skillStore.fetchSkills()
    if (skillStore.skills.length === 0) useMock.value = true
  } catch {
    useMock.value = true
  }
})
</script>

<style scoped>
/* ── Source Tabs ── */
.source-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.10);
  margin-bottom: 16px;
  overflow-x: auto;
}

.source-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-ghost);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--t-fast);
  white-space: nowrap;
}

.source-tab:hover {
  color: var(--fg);
  background: rgba(255, 255, 255, 0.10);
}

.source-tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-count {
  font-family: var(--font-mono);
  font-size: 10px;
  opacity: 0.5;
}

/* ── Filter Bar ── */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  margin-bottom: 20px;
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

/* ── Skills Grid ── */
.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

/* ── Skill Card (glass style) ── */
.skill-card {
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 10px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  cursor: pointer;
}

.skill-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.skill-card-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.skill-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.skill-card-info {
  flex: 1;
  min-width: 0;
}

.skill-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.tag {
  font-size: 10px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 99px;
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid;
  font-family: var(--font-mono);
}

.skill-desc {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.skill-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.18);
  padding-top: 10px;
  margin-top: auto;
}

.skill-software {
  font-size: 11px;
  color: var(--fg-ghost);
}

/* Toggle switch */
.toggle {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.22);
  border: 1px solid rgba(255, 255, 255, 0.32);
  cursor: pointer;
  position: relative;
  transition: background var(--t-base);
  flex-shrink: 0;
}

.toggle::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: white;
  transition: transform var(--t-base);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.toggle.on {
  background: var(--accent);
}

.toggle.on::after {
  transform: translateX(16px);
}

/* Button icon */
.btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
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

/* ── Responsive ── */
@media (max-width: 1024px) {
  .skills-grid {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }
}

@media (max-width: 768px) {
  .skills-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .filter-bar {
    gap: 8px;
  }

  .search-input {
    flex: 1 1 100%;
  }

  .btn-group {
    margin-left: 0;
    width: 100%;
    justify-content: flex-end;
  }
}

@media (max-width: 480px) {
  .skills-grid {
    grid-template-columns: 1fr;
  }
}
</style>
```

- [ ] **Step 2: Verify the build compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors related to SkillsView.vue

- [ ] **Step 3: Commit**

```bash
git add src/views/SkillsView.vue
git commit -m "feat(skills): rewrite SkillsView with glass card grid and design-aligned layout"
```

---

### Task 2: Refactor AgentCard.vue

**Files:**
- Modify: `src/components/agents/AgentCard.vue` (full rewrite)

- [ ] **Step 1: Rewrite AgentCard.vue with design-aligned structure**

Replace the entire file. Key changes: two-letter abbreviation icon (from `agent.emoji` field), department + source label, target chip selector with colored dots, Install/View/more actions row.

```vue
<template>
  <div class="agent-card">
    <!-- Card Header -->
    <div class="agent-card-header">
      <div class="agent-icon">
        <span class="agent-abbr">{{ agent.emoji || agent.name?.charAt(0) }}</span>
      </div>
      <div class="agent-info">
        <div class="agent-name">{{ agent.name }}</div>
        <div class="agent-dept">{{ agent.department }} · {{ agent.source }}</div>
      </div>
    </div>

    <!-- Description -->
    <p class="agent-desc">{{ agent.description }}</p>

    <!-- Install Targets Section -->
    <div class="agent-targets-section">
      <div class="agent-targets-label">
        INSTALL TO · {{ selectedTargets.size }} selected
      </div>
      <div class="target-grid">
        <button
          v-for="tool in TARGET_TOOLS"
          :key="tool.key"
          class="target-chip"
          :class="{ selected: selectedTargets.has(tool.key) }"
          @click.stop="toggleTarget(tool.key)"
        >
          <span
            class="chip-dot"
            :style="{ background: selectedTargets.has(tool.key) ? tool.color : 'rgba(154,154,154,0.3)' }"
          ></span>
          <span class="chip-abbr">{{ tool.abbr }}</span>
        </button>
      </div>
    </div>

    <!-- Actions Row -->
    <div class="agent-actions-row">
      <button
        class="btn btn-primary btn-sm"
        :disabled="selectedTargets.size === 0"
        @click.stop="$emit('install', Array.from(selectedTargets))"
      >
        Install ({{ selectedTargets.size }})
      </button>
      <button class="btn btn-secondary btn-sm" @click.stop="$emit('click')">
        View
      </button>
      <button class="btn-icon btn-sm" @click.stop title="More actions">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="5" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="12" cy="19" r="1" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Agent } from '@/types/agent'

const props = defineProps<{
  agent: Agent
}>()

defineEmits<{
  (e: 'click'): void
  (e: 'install', targets: string[]): void
}>()

// Target tools from design prototype (forge-cross-platform-glass.html line 1146)
const TARGET_TOOLS = [
  { key: 'claude-code', abbr: 'CC', name: 'Claude Code', color: '#D97706' },
  { key: 'cursor', abbr: 'CU', name: 'Cursor', color: '#7C3AED' },
  { key: 'copilot', abbr: 'CO', name: 'Copilot', color: '#059669' },
  { key: 'gemini-cli', abbr: 'GM', name: 'Gemini CLI', color: '#2563EB' },
  { key: 'opencode', abbr: 'OC', name: 'OpenCode', color: '#0891B2' },
  { key: 'deepseek', abbr: 'DS', name: 'DeepSeek', color: '#4F46E5' },
  { key: 'kiro', abbr: 'KI', name: 'Kiro', color: '#DC2626' },
  { key: 'codex', abbr: 'CX', name: 'Codex', color: '#9333EA' },
  { key: 'openclaw', abbr: 'CL', name: 'OpenClaw', color: '#B45309' },
  { key: 'mimo-code', abbr: 'MI', name: 'MiMo Code', color: '#0D9488' },
] as const

const selectedTargets = ref<Set<string>>(new Set())

function toggleTarget(key: string) {
  const next = new Set(selectedTargets.value)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  selectedTargets.value = next
}

onMounted(() => {
  if (props.agent.installedTargets) {
    try {
      const parsed = JSON.parse(props.agent.installedTargets) as string[]
      selectedTargets.value = new Set(parsed)
    } catch {
      selectedTargets.value = new Set()
    }
  }
})
</script>

<style scoped>
.agent-card {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 24px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow: hidden;
}

.agent-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

/* Card Header */
.agent-card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.agent-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.agent-abbr {
  font-size: 16px;
  font-weight: 700;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg-title);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-dept {
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-top: 2px;
}

/* Description */
.agent-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin: 0;
}

/* Install Targets */
.agent-targets-section {
  margin-top: auto;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.10);
}

.agent-targets-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-bottom: 8px;
}

.target-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.target-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg-muted);
  border: 1px solid rgba(255, 255, 255, 0.32);
  cursor: pointer;
  transition: all var(--t-fast);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  user-select: none;
}

.target-chip:hover {
  border-color: rgba(255, 255, 255, 0.28);
  background: rgba(255, 255, 255, 0.22);
  color: var(--fg);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.target-chip.selected {
  background: rgba(45, 45, 45, 0.10);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}

.target-chip .chip-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1.5px solid rgba(255, 255, 255, 0.30);
}

.target-chip.selected .chip-dot {
  border-color: transparent;
}

.target-chip .chip-abbr {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

/* Actions Row */
.agent-actions-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 12px;
}

.agent-actions-row .btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Button icon */
.btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
</style>
```

- [ ] **Step 2: Verify the build compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors related to AgentCard.vue

- [ ] **Step 3: Commit**

```bash
git add src/components/agents/AgentCard.vue
git commit -m "feat(agents): refactor AgentCard with two-letter abbreviations and target chips"
```

---

### Task 3: Rewrite AgentsView.vue

**Files:**
- Modify: `src/views/AgentsView.vue` (full rewrite)

- [ ] **Step 1: Rewrite AgentsView.vue with design-aligned structure**

Replace the entire file. Key changes: filter bar with Department + Source selects and Create/Import buttons, card grid using AgentCard component, store-first + mock fallback, removed status filter.

```vue
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
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search agents…"
        />
        <button
          v-show="searchQuery"
          class="clear-btn"
          @click="searchQuery = ''"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="#9A9A9A" stroke-width="1.5" stroke-linecap="round">
            <line x1="2" y1="2" x2="8" y2="8" /><line x1="8" y1="2" x2="2" y2="8" />
          </svg>
        </button>
      </div>
      <select v-model="deptFilter" class="filter-select">
        <option value="all">All Departments</option>
        <option value="engineering">Engineering</option>
        <option value="design">Design</option>
        <option value="product">Product</option>
        <option value="quality">Quality</option>
        <option value="custom">Custom</option>
      </select>
      <select v-model="sourceFilter" class="filter-select">
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

    <!-- Agents Card Grid -->
    <div v-if="filteredAgents.length > 0" class="agents-grid">
      <AgentCard
        v-for="agent in filteredAgents"
        :key="agent.id"
        :agent="agent"
        @click="openAgentDetails(agent)"
        @install="handleInstall(agent, $event)"
      />
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" />
      </svg>
      <h3>No agents found</h3>
      <p>Try adjusting your search or filters.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAgentStore } from '@/stores/agent'
import AgentCard from '@/components/agents/AgentCard.vue'
import type { Agent } from '@/types/agent'

// Mock data from design prototype (forge-cross-platform-glass.html line 1158)
const mockAgents: Agent[] = [
  { id: '1', name: 'Software Architect', description: 'Designs system architecture and technical solutions', emoji: 'SA', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '2', name: 'Frontend Director', description: 'Leads frontend architecture and Vue 3 implementation', emoji: 'FD', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '3', name: 'Backend Director', description: 'Oversees Rust backend and Tauri IPC layer', emoji: 'BD', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","opencode"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '4', name: 'Design Director', description: 'Guards design system consistency and visual quality', emoji: 'DD', department: 'design', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '5', name: 'Design UX', description: 'User experience research and interaction design', emoji: 'UX', department: 'design', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '6', name: 'QA Director', description: 'Test strategy and quality assurance oversight', emoji: 'QA', department: 'quality', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","deepseek"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '7', name: 'Performance Engineer', description: 'Performance profiling and optimization', emoji: 'PE', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","opencode","kiro"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '8', name: 'Product Manager', description: 'Product strategy and requirement management', emoji: 'PM', department: 'product', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","copilot","gemini-cli"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '9', name: 'Review Expert', description: 'Code review and quality gate enforcement', emoji: 'RE', department: 'quality', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","cursor","codex","mimo-code"]', isCustom: false, createdAt: '', updatedAt: '' },
  { id: '10', name: 'Deployment Engineer', description: 'CI/CD pipeline and release management', emoji: 'DE', department: 'engineering', content: '', source: 'agency-agents-zh', tags: '', installedTargets: '["claude-code","openclaw","kiro"]', isCustom: false, createdAt: '', updatedAt: '' },
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
  margin-bottom: 20px;
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
  margin-bottom: 20px;
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

/* ── Agents Grid ── */
.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 16px;
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

/* ── Responsive ── */
@media (max-width: 1024px) {
  .agents-grid {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }
}

@media (max-width: 768px) {
  .section-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }

  .agents-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .filter-bar {
    gap: 8px;
  }

  .search-input {
    flex: 1 1 100%;
  }

  .btn-group {
    margin-left: 0;
    width: 100%;
    justify-content: flex-end;
  }
}

@media (max-width: 480px) {
  .agents-grid {
    grid-template-columns: 1fr;
  }
}
</style>
```

- [ ] **Step 2: Verify the build compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors related to AgentsView.vue

- [ ] **Step 3: Commit**

```bash
git add src/views/AgentsView.vue src/components/agents/AgentCard.vue
git commit -m "feat(agents): rewrite AgentsView with design-aligned filter bar and card grid"
```

---

### Task 4: Final verification and cleanup

**Files:**
- None (verification only)

- [ ] **Step 1: Run full type check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1`
Expected: No type errors across the project

- [ ] **Step 2: Run build**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -10`
Expected: Build succeeds with no errors

- [ ] **Step 3: Verify both routes render correctly**

Open the app in dev mode (`npm run dev`) and navigate to:
- `/skills` — should show 5 source tabs, filter bar with Import/Sync All, card grid with glass cards
- `/agents` — should show filter bar with dept/source selects, card grid with two-letter abbreviation icons and target chips

- [ ] **Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix(ui): resolve any remaining alignment issues"
```

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRuleStore } from '@/stores/rule'
import FilterBar from '@/components/common/FilterBar.vue'
import SearchInput from '@/components/common/SearchInput.vue'
import Badge from '@/components/common/Badge.vue'
import Button from '@/components/common/Button.vue'
import type { Rule } from '@/types'
import { confirm } from '@/utils/dialog'

const ruleStore = useRuleStore()

// State
const searchQuery = ref('')
const selectedSource = ref('all')
const selectedType = ref('all')
const selectedStatus = ref('all')

// Mock data for UI demo (PENDING: real API integration)
const mockRules: Array<Rule & { size: string; software: string }> = [
  { id: '1', softwareId: 'cursor', name: 'AGENTS.md', type: 'md', filePath: '.cursor/rules/AGENTS.md', content: '', isActive: true, createdAt: '2026-06-12T20:28:00Z', updatedAt: '2026-06-12T20:28:00Z', size: '4.2 KB', software: 'Cursor' },
  { id: '2', softwareId: 'cursor', name: 'coding-standards.mdc', type: 'mdc', filePath: '.cursor/rules/coding-standards.mdc', content: '', isActive: true, createdAt: '2026-06-11T14:00:00Z', updatedAt: '2026-06-11T14:00:00Z', size: '2.1 KB', software: 'Cursor' },
  { id: '3', softwareId: 'cursor', name: 'commit-rules.md', type: 'md', filePath: '.cursor/rules/commit-rules.md', content: '', isActive: true, createdAt: '2026-06-10T09:15:00Z', updatedAt: '2026-06-10T09:15:00Z', size: '1.8 KB', software: 'Cursor' },
  { id: '4', softwareId: 'claude-desktop', name: 'project-context.md', type: 'md', filePath: '.claude/rules/project-context.md', content: '', isActive: false, createdAt: '2026-06-09T16:42:00Z', updatedAt: '2026-06-09T16:42:00Z', size: '3.4 KB', software: 'Claude Desktop' },
]

// Source options
const sourceOptions = [
  { value: 'all', label: 'All Sources' },
  { value: 'Cursor', label: 'Cursor' },
  { value: 'Global', label: 'Global' },
  { value: 'Project', label: 'Project' },
]

// Type options
const typeOptions = [
  { value: 'all', label: 'All Types' },
  { value: 'md', label: 'Markdown (.md)' },
  { value: 'mdc', label: 'MDC (.mdc)' },
]

// Status options
const statusOptions = [
  { value: 'all', label: 'All Status' },
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
]

// Computed
const rules = computed(() => {
  // PENDING: Replace mock with ruleStore.rules when backend is ready
  return mockRules
})

const filteredRules = computed(() => {
  let result = rules.value

  // Filter by search
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(rule =>
      rule.name.toLowerCase().includes(query) ||
      rule.software.toLowerCase().includes(query)
    )
  }

  // Filter by source
  if (selectedSource.value !== 'all') {
    result = result.filter(rule => rule.software === selectedSource.value)
  }

  // Filter by type
  if (selectedType.value !== 'all') {
    result = result.filter(rule => rule.type === selectedType.value)
  }

  // Filter by status
  if (selectedStatus.value !== 'all') {
    const isActive = selectedStatus.value === 'active'
    result = result.filter(rule => rule.isActive === isActive)
  }

  return result
})

const activeCount = computed(() => rules.value.filter(r => r.isActive).length)

// Actions
function formatDate(iso: string): string {
  const date = new Date(iso)
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

function getTypeLabel(type: string): string {
  return type === 'mdc' ? '.mdc' : `.${type}`
}

function getSourceVariant(source: string): 'success' | 'warn' | 'info' {
  if (source === 'Cursor') return 'success'
  if (source === 'Global') return 'warn'
  return 'info'
}

async function handleToggle(rule: Rule) {
  // PENDING: Connect to ruleStore.toggleRule when backend is ready
  console.log('Toggle rule:', rule.id, !rule.isActive)
  // Optimistic update for demo
  rule.isActive = !rule.isActive
}

function handleEdit(rule: Rule) {
  // PENDING: Open edit modal
  console.log('Edit rule:', rule.id)
}

async function handleDelete(rule: Rule) {
  openDropdown.value = null
  if (await confirm(`确认删除此规则？`)) {
    console.log('Delete rule:', rule.id)
  }
}

// More options dropdown
const openDropdown = ref<string | null>(null)

function toggleDropdown(key: string) {
  openDropdown.value = openDropdown.value === key ? null : key
}

function handleDuplicate(rule: Rule) {
  openDropdown.value = null
  console.log('Duplicate rule:', rule.id)
}

function handleExportRule(rule: Rule) {
  openDropdown.value = null
  console.log('Export rule:', rule.id)
}

function handleCreateRule() {
  // PENDING: Open create rule modal
  console.log('Create new rule')
}

onMounted(async () => {
  // PENDING: Uncomment when backend is ready
  // await ruleStore.fetchRules()
})
</script>

<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Rules</h2>
      <span class="count">{{ filteredRules.length }} files</span>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <SearchInput
        v-model="searchQuery"
        placeholder="Search rules…"
      />
      <select
        v-model="selectedSource"
        class="filter-select"
      >
        <option
          v-for="opt in sourceOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </option>
      </select>
      <select
        v-model="selectedType"
        class="filter-select"
      >
        <option
          v-for="opt in typeOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </option>
      </select>
      <select
        v-model="selectedStatus"
        class="filter-select"
      >
        <option
          v-for="opt in statusOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </option>
      </select>
      <Button
        variant="primary"
        size="sm"
        @click="handleCreateRule"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        >
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        Create Rule
      </Button>
    </div>

    <!-- Rules List -->
    <div class="card-grid">
      <div
        v-for="rule in filteredRules"
        :key="rule.id"
        class="card rule-card"
      >
        <div class="card-head">
          <div class="card-icon" :style="{ background: rule.type === 'md' ? 'rgba(90,107,122,0.10)' : 'rgba(90,138,100,0.10)', color: 'var(--fg-muted)' }">
            <svg v-if="rule.type === 'md'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" />
            </svg>
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M16 18l6-6-6-6" /><path d="M8 6l-6 6 6 6" />
            </svg>
          </div>
          <div style="flex: 1; min-width: 0">
            <div class="card-title">
              {{ rule.name }}
              <span class="badge" :class="rule.isActive ? 'success' : 'outline'">
                {{ rule.isActive ? 'Active' : 'Inactive' }}
              </span>
            </div>
            <div class="card-subtitle">{{ rule.software }} · {{ rule.size }}</div>
          </div>
        </div>
        <div class="card-meta">
          <div class="card-meta-item"><span class="label">Type</span><span class="value">.{{ rule.type }}</span></div>
          <div class="card-meta-item"><span class="label">Category</span><span class="value">{{ rule.category || 'general' }}</span></div>
          <div class="card-meta-item"><span class="label">Size</span><span class="value">{{ rule.size }}</span></div>
          <div class="card-meta-item"><span class="label">Modified</span><span class="value">{{ rule.modified || formatDate(rule.updatedAt) }}</span></div>
        </div>
        <div class="card-divider"></div>
        <div class="card-footer">
          <div class="card-footer-left">
            <span class="badge" :class="rule.isActive ? 'success' : 'outline'">
              {{ rule.isActive ? 'Active' : 'Inactive' }}
            </span>
          </div>
          <div class="card-footer-right">
            <button class="btn btn-secondary btn-sm" @click="handleEdit(rule)">Edit</button>
            <div class="dropdown-wrapper" @click.stop>
              <button class="btn-icon btn-sm" @click.stop="toggleDropdown(rule.id)" aria-label="More actions">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                </svg>
              </button>
              <Transition name="dropdown">
                <div v-if="openDropdown === rule.id" class="dropdown-menu">
                  <button class="dropdown-item" @click.stop="handleDuplicate(rule)">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                    Duplicate
                  </button>
                  <button class="dropdown-item" @click.stop="handleExportRule(rule)">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                    Export
                  </button>
                  <div class="dropdown-divider"></div>
                  <button class="dropdown-item danger" @click.stop="handleDelete(rule)">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                    Delete
                  </button>
                </div>
              </Transition>
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- Empty State -->
      <div
        v-if="filteredRules.length === 0"
        class="empty-state"
      >
        <svg
          width="48"
          height="48"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--fg-ghost)"
          stroke-width="1.5"
          stroke-linecap="round"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
        </svg>
        <p>No rules found</p>
        <span>Try adjusting your filters or create a new rule</span>
      </div>
  </div>
</template>

<style scoped>
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
}

.section-header .count {
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

.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  flex-wrap: wrap;
}

.filter-select {
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius-sm);
  color: var(--fg);
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1L5 5L9 1' stroke='%239A9A9A' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  transition: border-color var(--t-fast), background var(--t-fast);
}

.filter-select:hover {
  border-color: rgba(255, 255, 255, 0.40);
}

.filter-select:focus {
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

/* Card Grid */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  align-items: stretch;
}

.card-head {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.card-icon {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border);
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.card-subtitle {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  row-gap: 4px;
  word-break: break-word;
}

.card-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.card-meta-item .value {
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
  max-width: 160px;
}

.card-divider {
  height: 1px;
  background: var(--border);
  margin-top: auto;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 2px;
  margin-top: auto;
  width: 100%;
}

.card-footer-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.card-footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  width: fit-content;
}

.btn-ghost {
  background: transparent;
  border: none;
  color: var(--fg-ghost);
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--t-fast);
}

.btn-ghost:hover {
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg);
}

.btn-ghost:active {
  background: rgba(255, 255, 255, 0.40);
}

.toggle-btn.active {
  color: var(--success);
}

.delete-btn:hover {
  background: rgba(184, 90, 66, 0.15);
  color: var(--error);
}

/* Dropdown */
.dropdown-wrapper { position: relative; }
.dropdown-menu { position: absolute; bottom: 100%; right: -4px; margin-bottom: 6px; min-width: 160px; background: rgba(255,255,255,0.52); backdrop-filter: blur(40px) saturate(1.4); -webkit-backdrop-filter: blur(40px) saturate(1.4); border: 1px solid rgba(255,255,255,0.35); border-radius: var(--radius); box-shadow: 0 8px 32px rgba(0,0,0,0.12), inset 0 1px 0 rgba(255,255,255,0.50); padding: 4px; z-index: var(--z-dropdown); }
.dropdown-item { display: flex; align-items: center; gap: 8px; width: 100%; padding: 7px 10px; font-size: 12px; color: var(--fg-muted); border: none; background: none; border-radius: 6px; cursor: pointer; transition: all 150ms ease; text-align: left; }
.dropdown-item:hover { background: rgba(255,255,255,0.40); color: var(--fg); }
.dropdown-item.danger { color: var(--error); }
.dropdown-item.danger:hover { background: rgba(184,90,66,0.12); }
.dropdown-divider { height: 1px; background: rgba(255,255,255,0.20); margin: 4px 0; }
.dropdown-enter-active, .dropdown-leave-active { transition: opacity 150ms ease, transform 150ms ease; }
.dropdown-enter-from, .dropdown-leave-to { opacity: 0; transform: translateY(4px); }

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  text-align: center;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-muted);
  margin-bottom: 4px;
}

.empty-state span {
  font-size: 13px;
  color: var(--fg-ghost);
}

/* List transition */
.list-enter-active,
.list-leave-active {
  transition: all var(--t-base);
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

/* Responsive */
@media (max-width: 768px) {
  .rule-item {
    flex-wrap: wrap;
  }

  .rule-info {
    order: 2;
    flex-basis: 100%;
    margin-top: 8px;
  }

  .rule-actions {
    order: 3;
    width: 100%;
    justify-content: flex-end;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.15);
  }
}
</style>

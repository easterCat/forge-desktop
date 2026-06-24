<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Skills</h2>
      <span class="count">{{ filteredSkills.length }} skills</span>
    </div>

    <!-- Source Tabs -->
    <SourceTabs v-model="activeSource" :tabs="sourceTabs" />

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
    <div v-if="filteredSkills.length > 0" class="card-grid">
      <div
        v-for="skill in filteredSkills"
        :key="skill.name"
        class="card skill-card"
        @click="openSkillDetails(skill)"
      >
        <!-- Card Header -->
        <div class="card-head">
          <div class="card-icon" :style="{ background: typeBg(skill.type), color: typeColor(skill.type) }">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
            </svg>
          </div>
          <div style="flex: 1; min-width: 0">
            <div class="card-title">
              {{ skill.name }}
              <span class="tag" :style="{ color: typeColor(skill.type), borderColor: typeColor(skill.type) + '30' }">
                {{ skill.type }}
              </span>
            </div>
            <div class="card-subtitle">{{ skill.software }} · {{ skill.source }}</div>
          </div>
        </div>

        <!-- Description -->
        <div class="card-desc">{{ skill.desc }}</div>

        <!-- Card Meta -->
        <div class="card-meta">
          <div class="card-meta-item"><span class="label">Type</span><span class="value">{{ skill.type }}</span></div>
          <div class="card-meta-item"><span class="label">Software</span><span class="value">{{ skill.software }}</span></div>
          <div class="card-meta-item"><span class="label">Status</span><span class="value">{{ getSkillEnabled(skill) ? 'Enabled' : 'Disabled' }}</span></div>
        </div>

        <!-- CLI Tools sync chips -->
        <div v-if="cliTools.length > 0" class="plugin-cli-row">
          <CliSyncChip
            v-for="tool in cliTools"
            :key="tool.key"
            :tool-key="tool.key"
            :tool-name="tool.name"
            :tool-icon="tool.icon"
            :tool-color="tool.color"
            :state="isToolSynced(skill.name, tool.key) ? 'synced' : 'unsynced'"
            :show-label="false"
            @click="toggleSkillTool(skill.name, tool.key)"
          />
        </div>

        <!-- Card Divider -->
        <div class="card-divider"></div>

        <!-- Card Footer -->
        <div class="card-footer">
          <div class="card-footer-left">
            <ProgressSlot
              v-if="getOperation(`skill-${skill.name}`) && getOperation(`skill-${skill.name}`)?.stage !== 'idle'"
              :stage="getOperation(`skill-${skill.name}`)?.stage || 'idle'"
              :progress="getOperation(`skill-${skill.name}`)?.progress || 0"
            />
            <span v-else class="badge" :class="getSkillEnabled(skill) ? 'success' : 'outline'">
              {{ getSkillEnabled(skill) ? 'Active' : 'Inactive' }}
            </span>
          </div>
          <div class="card-footer-right">
            <button class="btn btn-secondary btn-sm" @click.stop>Edit</button>
            <DropdownMenu :model-value="openDropdown === skill.name" @update:model-value="(v: boolean) => openDropdown = v ? skill.name : null" :min-width="160">
              <template #trigger>
                <button class="btn-icon btn-sm" @click.stop="toggleDropdown(skill.name)" aria-label="More actions">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                  </svg>
                </button>
              </template>
              <button class="dropdown-item" @click.stop="handleDuplicate(skill)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                Duplicate
              </button>
              <button class="dropdown-item" @click.stop="handleExport(skill)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                Export
              </button>
              <div class="dropdown-divider"></div>
              <button class="dropdown-item danger" @click.stop="handleDeleteSkill(skill)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                Delete
              </button>
            </DropdownMenu>
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
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useSkillStore, useSoftwareStore } from '@/stores'
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import SourceTabs from '@/components/common/SourceTabs.vue'
import CliSyncChip from '@/components/common/CliSyncChip.vue'
import ProgressSlot from '@/components/common/ProgressSlot.vue'
import { useOperationProgress } from '@/composables/useOperationProgress'
import { confirm } from '@/utils/dialog'

const { startOperation, updateProgress, completeOperation, getOperation } = useOperationProgress()

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
  { name: 'imagegen', type: 'agent', desc: 'AI 图像生成技能，支持文生图、图生图与多种风格迁移转换。', software: 'Cursor', source: 'local', enabled: true },
  { name: 'frontend-design', type: 'command', desc: '前端设计辅助技能，提供组件建议、布局优化与响应式适配。', software: 'Cursor', source: 'local', enabled: true },
  { name: 'brandkit', type: 'agent', desc: '品牌规范生成器，输出配色方案、字体搭配与使用指南。', software: 'Cursor', source: 'local', enabled: true },
  { name: 'skill-creator', type: 'automation', desc: '技能创建向导，自动生成模板、配置文件与发布元数据。', software: 'Cursor', source: 'local', enabled: true },
  { name: 'canvas', type: 'command', desc: '实时 React 画布构建器，支持组件拖拽与即时代码导出。', software: 'Cursor', source: 'local', enabled: false },
  { name: 'shell', type: 'automation', desc: 'Shell 命令执行器，安全封装终端操作并记录执行历史。', software: 'Claude Desktop', source: 'anthropic', enabled: true },
  { name: 'documents', type: 'agent', desc: '文档生成器，自动创建技术文档、README 与 API 手册。', software: 'Cursor', source: 'marketplace', enabled: true },
  { name: 'spreadsheets', type: 'command', desc: '电子表格工具，支持数据导入、公式生成与图表输出。', software: 'Cursor', source: 'skills-sh', enabled: false },
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

function typeBg(type: string): string {
  const bg: Record<string, string> = {
    agent: 'rgba(184,148,74,0.10)',
    command: 'rgba(90,107,122,0.10)',
    automation: 'rgba(90,138,100,0.10)',
  }
  return bg[type] || 'rgba(45,45,45,0.06)'
}

// Store
const skillStore = useSkillStore()
const softwareStore = useSoftwareStore()

// CLI tools for tool icon sync
const cliTools = computed(() => {
  return softwareStore.cliTools.slice(0, 8).map((t: any) => ({
    key: t.key,
    icon: t.icon || t.name.charAt(0),
    color: t.color || '#5C5C5C',
    name: t.name,
  }))
})

// State
const useMock = ref(false)
const searchQuery = ref('')
const typeFilter = ref('all')
const statusFilter = ref('all')
const activeSource = ref('all')
const skillsState = ref<Record<string, boolean>>({})
const skillSyncState = ref<Record<string, Set<string>>>({})

// Data source: store-first, mock fallback
const displaySkills = computed<DisplaySkill[]>(() => {
  if (useMock.value) return mockSkills
  // Map store skills to display format
  return skillStore.skills.map(s => ({
    name: s.name,
    type: (['agent', 'command', 'automation'].includes(s.type) ? s.type : 'command') as DisplaySkill['type'],
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

function openSkillDetails(skill: DisplaySkill): void {
  // TODO: Open skill details dialog
  console.log('Open skill details:', skill.name)
}

function isToolSynced(skillName: string, toolKey: string): boolean {
  return skillSyncState.value[skillName]?.has(toolKey) ?? false
}

function toggleSkillTool(skillName: string, toolKey: string): void {
  if (!skillSyncState.value[skillName]) {
    skillSyncState.value[skillName] = new Set()
  }
  const synced = skillSyncState.value[skillName]
  if (synced.has(toolKey)) {
    synced.delete(toolKey)
  } else {
    synced.add(toolKey)
  }
}

function handleImport(): void {
  // TODO: Open import dialog
  console.log('Import skill')
}

function handleSyncAll(): void {
  // TODO: Sync all repositories
  console.log('Sync all')
}

// More options dropdown
const openDropdown = ref<string | null>(null)
const deleteTimers = ref<number[]>([])

function toggleDropdown(key: string) {
  openDropdown.value = openDropdown.value === key ? null : key
}

function closeDropdown() {
  openDropdown.value = null
}

function handleDuplicate(skill: DisplaySkill) {
  closeDropdown()
  console.log('Duplicate skill:', skill.name)
}

function handleExport(skill: DisplaySkill) {
  closeDropdown()
  console.log('Export skill:', skill.name)
}

async function handleDeleteSkill(skill: DisplaySkill) {
  closeDropdown()
  if (await confirm(`确认删除 ${skill.name}？`)) {
    const key = `skill-${skill.name}`
    startOperation(key)
    updateProgress(key, 'preparing', 10, `Deleting ${skill.name}...`)
    // Simulate delete progress
    const t1 = window.setTimeout(() => updateProgress(key, 'installing', 50, 'Removing files...'), 300)
    const t2 = window.setTimeout(() => {
      completeOperation(key, true, `${skill.name} deleted`)
      console.log('Delete skill:', skill.name)
    }, 800)
    deleteTimers.value.push(t1, t2)
  }
}

// Store-first + mock fallback
onMounted(async () => {
  document.addEventListener('click', handleGlobalClick)
  try {
    await skillStore.fetchSkills()
    if (skillStore.skills.length === 0) useMock.value = true
  } catch {
    useMock.value = true
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleGlobalClick)
  deleteTimers.value.forEach(id => window.clearTimeout(id))
  deleteTimers.value = []
})

function handleGlobalClick(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.dropdown-wrapper')) {
    openDropdown.value = null
  }
}
</script>

<style scoped>
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

/* ── Skills Grid ── */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  align-items: stretch;
}

/* ── Skill Card (glass style) ── */
.skill-card {
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 12px;
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

/* Card structure - Prototype aligned */
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

.tag {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.32);
  color: var(--fg-muted);
  border: 1px solid rgba(255, 255, 255, 0.32);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.card-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  min-height: 2.1em;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  row-gap: 4px;
}

.card-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.card-meta-item .label {
  color: var(--fg-ghost);
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

/* Badge */
.badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.badge.success {
  background: rgba(90, 138, 100, 0.15);
  border: 1px solid rgba(90, 138, 100, 0.20);
  color: var(--success);
}

.badge.success::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--success);
}

.badge.outline {
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-muted);
}

/* ── CLI Sync Row ───────────────────────────────────────────────────────────── */
.plugin-cli-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 12px;
  margin-top: 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.10);
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

</style>

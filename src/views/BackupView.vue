<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Backup & Restore</h2>
      <div class="btn-group">
        <Button variant="secondary" size="sm" @click="handleConfigure">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9"/>
          </svg>
          Settings
        </Button>
        <Button variant="primary" size="sm" @click="handleCreateBackup">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          Create Backup
        </Button>
      </div>
    </div>

    <!-- Stats Row -->
    <div class="stats-row">
      <StatCard label="Total Backups" :value="stats.totalBackups" tint="warm" />
      <StatCard label="Storage Used" :value="stats.storageUsed" tint="cool" />
      <StatCard label="Auto-Backup" :value="stats.autoBackup" tint="soft" :value-color="'var(--success)'" />
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <SearchInput
        v-model="searchQuery"
        placeholder="Search backups…"
      />
      <select v-model="typeFilter" class="filter-select">
        <option value="all">All Types</option>
        <option value="scheduled">Scheduled</option>
        <option value="manual">Manual</option>
        <option value="incremental">Incremental</option>
        <option value="pre-update">Pre-update</option>
        <option value="system">System</option>
      </select>
    </div>

    <!-- Backup Grid -->
    <div class="settings-grid">
      <div
        v-for="backup in filteredBackups"
        :key="backup.id"
        class="backup-card"
      >
        <!-- Type icon + label -->
        <div class="backup-card-top">
          <div class="backup-icon" :style="{ color: typeColors[backup.type] }">
            <svg v-if="backup.type === 'scheduled'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
            <svg v-else-if="backup.type === 'manual'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M12 20h9"/>
              <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
            </svg>
            <svg v-else-if="backup.type === 'incremental'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
              <polyline points="17 6 23 6 23 12"/>
            </svg>
            <svg v-else-if="backup.type === 'pre-update'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
          </div>
          <span class="type-badge" :style="{ color: typeColors[backup.type] }">
            {{ typeLabels[backup.type] }}
          </span>
        </div>

        <!-- Name -->
        <h4 class="backup-name">{{ backup.name }}</h4>

        <!-- Date -->
        <div class="backup-date">{{ formatDate(backup.createdAt) }}</div>

        <!-- Size · Files · Status -->
        <div class="backup-meta">
          <span>{{ formatSize(backup.size) }}</span>
          <span class="meta-sep">·</span>
          <span>{{ backup.fileCount }} files</span>
          <span class="backup-status">
            <span class="status-dot"></span>
            OK
          </span>
        </div>

        <!-- Include tags -->
        <div class="backup-includes">
          <span
            v-for="item in parseIncludes(backup.includes)"
            :key="item"
            class="include-tag"
          >
            {{ item }}
          </span>
        </div>

        <!-- Actions -->
        <div class="backup-actions">
          <Button variant="secondary" size="sm" class="restore-btn" @click="handleRestoreSingle(backup)">
            Restore
          </Button>
          <Button variant="ghost" size="sm" @click="handleDelete(backup)">
            Delete
          </Button>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredBackups.length === 0" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        <h3>No backups found</h3>
        <p>Try adjusting your search or filter criteria.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import Button from '@/components/common/Button.vue'
import StatCard from '@/components/common/StatCard.vue'
import SearchInput from '@/components/common/SearchInput.vue'
import { useBackupStore } from '@/stores/backup'
import type { BackupRecord } from '@/types'

// Extended backup type for view-specific fields
interface ViewBackupRecord extends BackupRecord {
  type: 'scheduled' | 'manual' | 'incremental' | 'pre-update' | 'system'
  status: 'completed' | 'pending' | 'error'
}

const backupStore = useBackupStore()

// Type colors and labels
const typeColors: Record<string, string> = {
  scheduled: 'var(--info)',
  manual: 'var(--accent)',
  incremental: 'var(--success)',
  system: 'var(--error)',
  'pre-update': 'var(--warn)',
}

const typeLabels: Record<string, string> = {
  scheduled: 'Scheduled',
  manual: 'Manual',
  incremental: 'Incremental',
  system: 'System',
  'pre-update': 'Pre-update',
}

// Search and filter state
const searchQuery = ref('')
const typeFilter = ref('all')

// Mock stats (PENDING: integrate with real API)
const stats = ref({
  totalBackups: 12,
  storageUsed: '378 MB',
  autoBackup: 'Daily 03:00',
})

// Mock backup data (PENDING: fetch from useBackupStore)
const backups = ref<ViewBackupRecord[]>([
  {
    id: '1',
    name: 'Scheduled Full',
    path: '/backups/scheduled-2026-06-18',
    size: 54945089,
    fileCount: 168,
    createdAt: '2026-06-18T03:00:00Z',
    includes: 'Cursor,Claude Desktop,Windsurf,CLI Tools,MCP Servers',
    type: 'scheduled',
    status: 'completed',
  },
  {
    id: '2',
    name: 'Manual Snapshot',
    path: '/backups/manual-2026-06-17',
    size: 53575680,
    fileCount: 164,
    createdAt: '2026-06-17T14:22:00Z',
    includes: 'Cursor,Claude Desktop,Windsurf,CLI Tools',
    type: 'manual',
    status: 'completed',
  },
  {
    id: '3',
    name: 'Incremental',
    path: '/backups/incremental-2026-06-17',
    size: 3984588,
    fileCount: 12,
    createdAt: '2026-06-17T09:15:00Z',
    includes: 'Cursor,Claude Desktop',
    type: 'incremental',
    status: 'completed',
  },
  {
    id: '4',
    name: 'Scheduled Full',
    path: '/backups/scheduled-2026-06-16',
    size: 52109312,
    fileCount: 160,
    createdAt: '2026-06-16T03:00:00Z',
    includes: 'Cursor,Claude Desktop,Windsurf,CLI Tools,MCP Servers',
    type: 'scheduled',
    status: 'completed',
  },
  {
    id: '5',
    name: 'Pre-update v2.4.0',
    path: '/backups/pre-update-2026-06-15',
    size: 51234567,
    fileCount: 158,
    createdAt: '2026-06-15T22:10:00Z',
    includes: 'Cursor,Claude Desktop,Windsurf,CLI Tools',
    type: 'pre-update',
    status: 'completed',
  },
])

// Computed: filter backups
const filteredBackups = computed(() => {
  let result = backups.value

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(
      (b) =>
        b.name.toLowerCase().includes(query) ||
        b.includes?.toLowerCase().includes(query)
    )
  }

  if (typeFilter.value !== 'all') {
    result = result.filter((b) => b.type === typeFilter.value)
  }

  return result
})

// Helper: format date
function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).replace(',', '')
}

// Helper: format size
function formatSize(bytes: number | null): string {
  if (!bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

// Helper: parse includes string to array
function parseIncludes(includes: string | null): string[] {
  if (!includes) return []
  return includes.split(',').map((s) => s.trim())
}

// Actions
function handleCreateBackup() {
  console.log('Creating backup...')
}

function handleConfigure() {
  console.log('Configure backup...')
}

function handleRestoreSingle(backup: ViewBackupRecord) {
  console.log('Restoring from:', backup.name)
}

function handleDelete(backup: ViewBackupRecord) {
  console.log('Deleting:', backup.name)
}
</script>

<style scoped>
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.30);
  flex-wrap: wrap;
  gap: var(--spacing-sm);
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title);
  margin: 0;
}

.btn-group {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-left: auto;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 18px;
  margin-bottom: 24px;
}

.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  flex-wrap: wrap;
}

.filter-bar :deep(.search-input) {
  flex: 1 1 180px;
  min-width: 0;
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
  flex: 0 1 auto;
  min-width: 0;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  outline: none;
}

.filter-select:focus {
  border-color: rgba(255, 255, 255, 0.30);
}

.backup-card {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  position: relative;
  transition: all var(--t-base);
}

.backup-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

/* Icon + type badge row: gap 10, mb 12 */
.backup-card-top {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.backup-icon {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.05);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(45, 45, 45, 0.04);
  padding: 2px 8px;
  border-radius: 99px;
  border: 1px solid var(--border);
}

/* Name: mb 6 */
.backup-name {
  margin: 0 0 6px;
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
}

/* Date: mb 4 */
.backup-date {
  font-size: 12px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
  margin-bottom: 4px;
}

/* Meta (size · files · status): gap 8, mb 10 */
.backup-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--fg-muted);
  margin-bottom: 10px;
}

.meta-sep {
  color: var(--border);
}

.backup-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  color: var(--success);
}

.status-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--success);
}

/* Includes tags: gap 4, mb 14 */
.backup-includes {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}

.include-tag {
  padding: 3px 10px;
  font-size: 11px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.32);
  color: var(--fg-muted);
  border: 1px solid rgba(255, 255, 255, 0.32);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

/* Actions: gap 8 */
.backup-actions {
  display: flex;
  gap: 8px;
}

.backup-actions .restore-btn {
  flex: 1;
}

.empty-state {
  grid-column: 1 / -1;
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

@media (max-width: 768px) {
  .backup-grid {
    grid-template-columns: 1fr;
  }

  .section-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
}
</style>

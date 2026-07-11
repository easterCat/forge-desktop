<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Plugins</h2>
      <span class="count">{{ installedPlugins.length }} installed</span>
    </div>

    <!-- Tab Bar: Installed / Marketplace / Sources -->
    <TabBar v-model="activeTab" :tabs="tabItems" />

    <!-- Installed Toolbar -->
    <div v-if="activeTab === 'installed'" class="installed-toolbar">
      <SearchInput
        v-model="installedSearch"
        placeholder="Search installed plugins…"
        class="installed-search"
      />
      <select v-model="statusFilter" class="filter-select">
        <option value="all">All Status</option>
        <option value="enabled">Enabled</option>
        <option value="disabled">Disabled</option>
      </select>
    </div>

    <!-- ─── Tab 1: Installed ─── -->
    <div v-if="activeTab === 'installed'" class="tab-content">
      <!-- Loading skeletons -->
      <div v-if="isLoadingInstalled" class="plugin-list">
        <div v-for="i in 3" :key="`skel-${i}`" class="card plugin-card skeleton-card">
          <div class="plugin-card-head">
            <div class="skeleton-left">
              <div class="skeleton skeleton-icon"></div>
              <div class="skeleton-info">
                <div class="skeleton skeleton-name"></div>
                <div class="skeleton skeleton-meta"></div>
              </div>
            </div>
            <div class="skeleton-actions">
              <div class="skeleton skeleton-btn"></div>
            </div>
          </div>
          <div class="skeleton skeleton-chips"></div>
        </div>
      </div>

      <!-- Empty: no installed plugins -->
      <div v-else-if="filteredInstalled.length === 0" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
        <h3>No plugins installed</h3>
        <p>Browse the Marketplace to discover and install plugins.</p>
        <button class="btn btn-primary btn-sm" @click="activeTab = 'marketplace'">
          Go to Marketplace
        </button>
      </div>

      <!-- Plugin cards grid -->
      <div v-else class="plugin-list">
        <div
          v-for="{ plugin, syncChips, syncCount, resolvedVersion, sourceName, uninstallStage, uninstallProgress } in filteredInstalledWithSync"
          :key="`${plugin.sourceId}::${plugin.name}`"
          class="card plugin-card"
        >
          <div class="plugin-card-head">
            <div style="flex:1;min-width:0">
              <div style="font-weight:600;color:var(--fg-title);font-size:14px;display:flex;align-items:center;gap:8px">
                <span>{{ plugin.name }}</span>
                <span v-if="syncCount > 0" class="sync-count">
                  {{ syncCount }} synced
                </span>
              </div>
              <div class="plugin-card-meta">
                <template v-if="resolvedVersion">
                  v{{ resolvedVersion }}
                </template>
              </div>
            </div>
            <div class="plugin-source-label" :title="plugin.repository || plugin.sourceId">
              {{ sourceName }}
            </div>
          </div>

          <div class="plugin-card-body">
            <div class="plugin-desc-clamp">
              {{ plugin.description || 'No description' }}
            </div>

            <!-- CLI Tools sync chips — all 23 allagents tools per card -->
            <div v-if="syncChips.length > 0" class="plugin-cli-row">
              <span class="plugin-cli-label">Sync to</span>
              <CliSyncChip
                v-for="chip in syncChips"
                :key="chip.key"
                :tool-key="chip.key"
                :tool-name="chip.name"
                :tool-icon="chip.icon"
                :tool-color="chip.color"
                :state="chip.state"
                :show-label="false"
                @click="handleCliSync(plugin, $event as any)"
              />
              <button
                v-if="syncCount > 0"
                class="sync-count-badge"
                :data-testid="`sync-badge-${plugin.sourceId}-${plugin.name}`"
                @click.stop="openSyncDialog(plugin)"
              >
                <span class="sync-count-badge-text">{{ syncCount }} synced</span>
              </button>
            </div>
          </div>

          <div class="card-footer">
            <div class="card-footer-left">
              <ProgressSlot
                :stage="uninstallStage"
                :progress="uninstallProgress"
              />
            </div>
            <div class="card-footer-right">
              <DropdownMenu
                :model-value="openDropdown === `${plugin.sourceId}::${plugin.name}`"
                :min-width="140"
                @update:model-value="(v: boolean) => openDropdown = v ? `${plugin.sourceId}::${plugin.name}` : null"
              >
                <template #trigger>
                  <button class="btn-icon btn-sm" title="More actions" aria-label="More actions" @click.stop="togglePluginDropdown(plugin)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                    </svg>
                  </button>
                </template>
                <button class="dropdown-item danger" @click.stop="handleUninstall(plugin)">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                  Uninstall
                </button>
              </DropdownMenu>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Marketplace: Source Tabs -->
    <div v-if="activeTab === 'marketplace'" class="source-tabs-wrap">
      <SourceTabs v-model="currentSourceId" :tabs="sourceTabItems" />
    </div>

    <!-- Marketplace: Filter Bar -->
    <div v-if="activeTab === 'marketplace'" class="filter-bar">
      <SearchInput
        v-model="marketplaceSearch"
        placeholder="Search marketplace…"
        class="marketplace-search"
      />
      <select v-model="categoryFilter" class="filter-select">
        <option value="all">All Categories</option>
        <option value="automation">Automation</option>
        <option value="ai">AI / ML</option>
        <option value="devtools">Dev Tools</option>
        <option value="ui">UI / Theme</option>
        <option value="data">Data</option>
        <option value="productivity">Productivity</option>
      </select>
      <select v-model="sortBy" class="filter-select">
        <option value="popular">Most Popular</option>
        <option value="newest">Newest</option>
        <option value="name">A–Z</option>
      </select>
      <button class="btn btn-secondary btn-sm" @click="refreshMarketplace">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
        <span>Refresh</span>
      </button>
    </div>

    <!-- ─── Tab 2: Marketplace ─── -->
    <div v-if="activeTab === 'marketplace'" class="tab-content">
      <!-- Loading -->
      <div v-if="isLoadingMarketplace" class="marketplace-grid">
        <div v-for="i in 6" :key="`mskel-${i}`" class="card marketplace-card skeleton-marketplace-card">
          <div class="skeleton skeleton-mcard-icon"></div>
          <div class="skeleton skeleton-mcard-name"></div>
          <div class="skeleton skeleton-mcard-desc"></div>
          <div class="skeleton skeleton-mcard-footer"></div>
        </div>
      </div>

      <!-- Empty: no plugins -->
      <div v-else-if="filteredMarketplace.length === 0" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          <line x1="8" y1="8" x2="14" y2="14" stroke="var(--fg-ghost)" stroke-width="1.5"/>
          <line x1="14" y1="8" x2="8" y2="14" stroke="var(--fg-ghost)" stroke-width="1.5"/>
        </svg>
        <h3>No plugins found</h3>
        <p>Try adjusting your search or filters.</p>
      </div>

      <!-- Marketplace grid -->
      <div v-else class="marketplace-grid">
        <MarketplaceCard
          v-for="plugin in filteredMarketplace"
          :key="`${plugin.sourceId}::${plugin.name}`"
          :name="plugin.name"
          :author="plugin.author"
          :description="plugin.description"
          :version="plugin.latestVersion || plugin.version"
          :downloads="plugin.downloads"
          :stars="plugin.stars"
          :installed="plugin.isInstalled ?? false"
          :icon="plugin.name.substring(0, 2).toUpperCase()"
          :icon-color="getPluginColor(plugin)"
          :tags="plugin.tags"
          @click="openPluginDetails(plugin)"
          @install="handleInstall(plugin)"
        />
      </div>
    </div>

    <!-- ─── Tab 3: Sources ─── -->
    <div v-if="activeTab === 'sources'" class="tab-content">
      <!-- Filter bar -->
      <div class="filter-bar">
        <SearchInput
          v-model="sourcesSearch"
          placeholder="Search sources…"
          class="sources-search"
        />
        <select v-model="sourcesStatusFilter" class="filter-select">
          <option value="all">All Status</option>
          <option value="installed">Installed</option>
          <option value="pending">Pending</option>
        </select>
        <button class="btn btn-primary btn-sm" @click="showAddSourceDialog = true">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          <span>Add Source</span>
        </button>
      </div>

      <!-- Source cards grid -->
      <div class="sources-grid">
        <SourceCard
          v-for="{ source, status, notes, installProgress } in filteredSources"
          :key="source.id"
          :source="source"
          :status="status"
          :notes="notes"
          :is-installing="installingSourceId === source.id"
          :install-progress="installProgress"
          @view-details="openSourceDetails(source)"
          @install="handleInstallSource(source)"
          @sync="handleSyncSource(source)"
          @edit-notes="openSourceNotes(source.id)"
          @delete="handleDeleteSource(source)"
        />
      </div>
    </div>

    <!-- Plugin Details Dialog -->
    <PluginDetailsDialog
      v-if="detailsPlugin"
      :plugin="detailsPlugin"
      @close="detailsPlugin = null"
    />

    <!-- Add Source Dialog -->
    <AddRepoSourceDialog
      v-model="showAddSourceDialog"
      @confirm="handleAddSource"
    />

    <!-- Source Note Dialog -->
    <SourceNoteDialog
      v-if="editingSourceId"
      :visible="!!editingSourceId"
      :source-id="editingSourceId"
      :source-name="sources.find(s => s.id === editingSourceId)?.name ?? ''"
      :initial-note="getSourceNotes(editingSourceId)"
      @update:visible="editingSourceId = null"
      @save="handleSaveSourceNote"
    />

    <!-- Plugin CLI Sync Dialog -->
    <PluginCliSyncDialog
      :plugin="syncDialogPlugin"
      :is-open="!!syncDialogPlugin"
      @update:is-open="(v) => { if (!v) syncDialogPlugin = null }"
      @synced="loadAllSyncStatuses"
      @unsynced="loadAllSyncStatuses"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, inject } from 'vue'
import { storeToRefs } from 'pinia'
import { usePluginMarketplaceStore } from '@/stores/plugin-marketplace'
import { useSoftwareStore } from '@/stores/software'
import TabBar from '@/components/common/TabBar.vue'
import SearchInput from '@/components/common/SearchInput.vue'
import CliSyncChip from '@/components/common/CliSyncChip.vue'
import MarketplaceCard from '@/components/common/MarketplaceCard.vue'
import PluginCliSyncDialog from '@/components/plugins/PluginCliSyncDialog.vue'
import PluginDetailsDialog from '@/components/plugins/PluginDetailsDialog.vue'
import SourceCard from '@/components/plugins/SourceCard.vue'
import SourceNoteDialog from '@/components/plugins/SourceNoteDialog.vue'
import AddRepoSourceDialog from '@/components/plugins/AddRepoSourceDialog.vue'
import SourceTabs from '@/components/common/SourceTabs.vue'
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import { confirm } from '@/utils/dialog'
import ProgressSlot from '@/components/common/ProgressSlot.vue'
import type { MarketplacePlugin, RepoType, PluginSource } from '@/types'
import type { OperationStage } from '@/composables/useOperationProgress'

// ── Store ─────────────────────────────────────────────────────────────────────
const marketplaceStore = usePluginMarketplaceStore()
const softwareStore = useSoftwareStore()
const showNotification = inject<(msg: string, type?: string) => void>('showNotification')

// All 23 allagents tools — displayed in the Sync to row on every card
const allSyncToolKeys = [
  // Universal clients
  'copilot', 'codex', 'opencode', 'gemini', 'ampcode', 'vscode', 'replit', 'kimi',
  // Provider clients
  'claude', 'cursor', 'factory', 'windsurf', 'cline', 'continue',
  'roo', 'kilo', 'trae', 'augment', 'zencoder', 'junie', 'openhands', 'kiro',
]

const {
  plugins,
  installedPlugins,
  sources,
} = storeToRefs(marketplaceStore)

// ── Tab state ─────────────────────────────────────────────────────────────────
const activeTab = ref('installed')

const tabItems = computed(() => [
  { id: 'installed', label: 'Installed', count: installedPlugins.value.length || undefined },
  { id: 'marketplace', label: 'Marketplace', count: totalMarketplacePlugins.value || undefined },
  { id: 'sources', label: 'Sources', count: sources.value.length || undefined },
])

// ── Tab 1: Installed ──────────────────────────────────────────────────────────
const installedSearch = ref('')
const statusFilter = ref<'all' | 'enabled' | 'disabled'>('all')
const isLoadingInstalled = ref(false)
const openDropdown = ref<string | null>(null)
const syncDialogPlugin = ref<MarketplacePlugin | null>(null)
const uninstallProgress = ref<Map<string, { stage: string; progress: number; message: string }>>(new Map())

// PENDING: usePluginsStore().installed — real API not yet available
const mockInstalled: MarketplacePlugin[] = []

const effectiveInstalled = computed(() => {
  // Prefer real store data; fall back to mock
  return installedPlugins.value.length > 0
    ? installedPlugins.value
    : mockInstalled
})

const filteredInstalled = computed(() => {
  const q = installedSearch.value.toLowerCase().trim()
  return effectiveInstalled.value.filter(p => {
    const matchSearch = !q || p.name.toLowerCase().includes(q) ||
      p.description?.toLowerCase().includes(q) ||
      p.author?.toLowerCase().includes(q)
    const matchStatus =
      statusFilter.value === 'all' ? true :
      statusFilter.value === 'enabled' ? !p.disabled :
      p.disabled
    return matchSearch && matchStatus
  })
})

// Pre-compute plugin sync data to avoid O(n×m) method calls in template.
// Audit item 2.2: the original code called `getCliMeta`, `getPluginSyncState`,
// `marketplaceStore.getResolvedVersion`, `getSourceName`, `getPluginStage`,
// and `getPluginProgress` inside the `v-for` body. Each call hit a reactive
// source (a `ref`, a `Map.get`, a store getter), and each one re-ran on every
// render. For a list of 30 installed plugins × 23 sync tools × 6 helper
// calls, that was ~4,140 reactive reads per render — even when nothing
// visible changed. We collapse all of that to O(n) pre-computation here.
interface PluginWithSyncData {
  plugin: MarketplacePlugin
  syncChips: Array<{
    key: string
    name: string
    icon: string
    color: string
    state: 'synced' | 'unsynced' | 'syncing'
  }>
  syncCount: number
  resolvedVersion: string
  sourceName: string
  uninstallStage: OperationStage
  uninstallProgress: number
}
const filteredInstalledWithSync = computed<PluginWithSyncData[]>(() => {
  const syncKeys = syncState.value
  const uninstallMap = uninstallProgress.value
  return filteredInstalled.value.map(plugin => {
    const version = marketplaceStore.getResolvedVersion(plugin)
    const versionLabel = version !== '—' ? version : ''
    const sourceKey = `${plugin.sourceId}::${plugin.name}`
    const uninstallEntry = uninstallMap.get(sourceKey)
    return {
      plugin,
      syncChips: allSyncToolKeys.map(toolKey => {
        const meta = getCliMeta(toolKey)
        const fullKey = `${plugin.sourceId}::${plugin.name}::${toolKey}`
        return {
          key: toolKey,
          name: meta.name,
          icon: meta.icon,
          color: meta.color,
          state: syncKeys[fullKey] ?? 'unsynced',
        }
      }),
      syncCount: getPluginSyncCount(plugin),
      resolvedVersion: versionLabel,
      sourceName: getSourceName(plugin.sourceId),
      uninstallStage: (uninstallEntry?.stage ?? 'idle') as OperationStage,
      uninstallProgress: uninstallEntry?.progress ?? 0,
    }
  })
})

// ── Tab 2: Marketplace ────────────────────────────────────────────────────────
const marketplaceSearch = ref('')
const categoryFilter = ref('all')
const sortBy = ref<'popular' | 'newest' | 'name'>('popular')
const currentSourceId = ref('all')
const isLoadingMarketplace = ref(false)

// Re-fetch plugins when the source tab changes (skip initial value —
// the mount handler takes care of the first fetch after sources load).
let initialSourceTabDone = false
watch(currentSourceId, async (newSourceId) => {
  if (!initialSourceTabDone) return
  if (activeTab.value !== 'marketplace') return
  isLoadingMarketplace.value = true
  try {
    await marketplaceStore.fetchPluginsBySource(newSourceId)
  } finally {
    isLoadingMarketplace.value = false
  }
})

const installedSources = computed(() =>
  sources.value.filter(s => s.pluginCount && s.pluginCount > 0)
)

const totalMarketplacePlugins = computed(() =>
  sources.value.reduce((sum, s) => sum + (s.pluginCount ?? 0), 0)
)

const sourceTabItems = computed(() => [
  ...installedSources.value.map(s => ({ id: s.id, label: s.name, count: s.pluginCount ?? 0 })),
  { id: 'all', label: 'All Sources', count: totalMarketplacePlugins.value },
])

function getPluginColor(plugin: MarketplacePlugin): string {
  const colors = ['#5A8A64', '#5A6B7A', '#B8944A', '#B85A42', '#7A5A6B']
  let hash = 0
  for (let i = 0; i < plugin.name.length; i++) {
    hash = plugin.name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
}

const filteredMarketplace = computed(() => {
  let list = currentSourceId.value === 'all'
    ? plugins.value
    : plugins.value.filter(p => p.sourceId === currentSourceId.value)

  const q = marketplaceSearch.value.toLowerCase().trim()
  if (q) {
    list = list.filter(p =>
      p.name.toLowerCase().includes(q) ||
      p.description?.toLowerCase().includes(q) ||
      p.author?.toLowerCase().includes(q)
    )
  }

  if (categoryFilter.value !== 'all') {
    list = list.filter(p => p.categories?.includes(categoryFilter.value))
  }

  // Sort
  if (sortBy.value === 'popular') {
    list = [...list].sort((a, b) => (b.downloads ?? 0) - (a.downloads ?? 0))
  } else if (sortBy.value === 'newest') {
    list = [...list].sort((a, b) =>
      new Date(b.lastUpdated ?? 0).getTime() - new Date(a.lastUpdated ?? 0).getTime()
    )
  } else {
    list = [...list].sort((a, b) => a.name.localeCompare(b.name))
  }

  return list
})

// ── Tab 3: Sources ───────────────────────────────────────────────────────────
const sourcesSearch = ref('')
const sourcesStatusFilter = ref<'all' | 'installed' | 'pending'>('all')
const showAddSourceDialog = ref(false)
const editingSourceId = ref<string | null>(null)
const installingSourceId = ref<string | null>(null)

const filteredSources = computed(() => {
  const q = sourcesSearch.value.toLowerCase().trim()
  // Audit item 2.2 (round 2): the template re-evaluated
  // `getSourceStatus`, `getSourceNotes`, and `marketplaceStore.getSourceInstallProgress`
  // for every source on every render. We compute them once here.
  const notesMap = marketplaceStore.sourceNotes
  return marketplaceStore.sortedSources
    .map(source => {
      const status = marketplaceStore.getSourceStatus(source.id)
      const installProgress = marketplaceStore.getSourceInstallProgress(source.id)
      return {
        source,
        status,
        notes: notesMap[source.id] ?? '',
        installProgress,
      }
    })
    .filter(({ source, status }) => {
      const isInstalled = status?.isInstalled ?? false
      if (sourcesStatusFilter.value === 'installed' && !isInstalled) return false
      if (sourcesStatusFilter.value === 'pending' && isInstalled) return false
      if (q) {
        const nameMatch = source.name.toLowerCase().includes(q)
        const urlMatch = source.command?.toLowerCase().includes(q) ?? false
        if (!nameMatch && !urlMatch) return false
      }
      return true
    })
})

// `getSourceNotes` is still needed for the SourceNoteDialog binding
// (single-call site, not in a v-for, so the pre-compute above doesn't
// cover it).
function getSourceNotes(sourceId: string): string {
  return marketplaceStore.sourceNotes[sourceId] ?? ''
}

function getSourceName(sourceId: string): string {
  const source = sources.value.find(s => s.id === sourceId)
  return source?.name || sourceId
}

async function handleInstallSource(source: PluginSource) {
  installingSourceId.value = source.id
  try {
    await marketplaceStore.installSource(source.id)
  } finally {
    installingSourceId.value = null
  }
}

async function handleSyncSource(source: PluginSource) {
  await marketplaceStore.installSource(source.id)
}

function openSourceNotes(sourceId: string) {
  editingSourceId.value = sourceId
}

function openSourceDetails(source: PluginSource) {
  if (source.command) {
    window.open(source.command, '_blank')
  }
}

async function handleSaveSourceNote(sourceId: string, note: string) {
  await marketplaceStore.saveSourceNote(sourceId, note)
  editingSourceId.value = null
}

async function handleDeleteSource(source: PluginSource) {
  if (!await confirm(`确定要删除源 "${source.name}" 吗？`)) return
  try {
    await marketplaceStore.removeSource(source.id)
  } catch (e) {
    console.error('Failed to delete source:', e)
  }
}

async function handleAddSource(url: string, repoType: RepoType) {
  try {
    const result = await marketplaceStore.addSource(url, repoType)
    if (result.success) {
      showAddSourceDialog.value = false
    }
  } catch (e) {
    console.error('Failed to add source:', e)
  }
}

// ── CLI sync helpers ─────────────────────────────────────────────────────────
const syncState = ref<Record<string, 'synced' | 'unsynced' | 'syncing'>>({})

/// Fetch sync statuses for all installed plugins and populate local syncState
async function loadAllSyncStatuses() {
  try {
    const pluginIds = installedPlugins.value.map(p => `${p.sourceId}::${p.name}`)
    if (pluginIds.length === 0) return

    await marketplaceStore.fetchSyncStatuses(pluginIds)

    // Map store's syncStatuses to local syncState, preserving any in-flight syncing entries
    const next: Record<string, 'synced' | 'unsynced' | 'syncing'> = {}
    for (const key of Object.keys(marketplaceStore.syncStatuses)) {
      const status = marketplaceStore.syncStatuses[key]
      if (status.synced) {
        next[key] = 'synced'
      }
    }
    // Preserve syncing entries that are currently in-flight
    for (const [key, value] of Object.entries(syncState.value)) {
      if (value === 'syncing' && !(key in next)) {
        next[key] = 'syncing'
      }
    }
    syncState.value = next
  } catch (e) {
    console.error('Failed to load sync statuses:', e)
  }
}

// Maps toolKey → { name, icon, color } for CliSyncChip
// Covers all 23 allagents client keys + 6 local CLI tool keys
const CLI_TOOL_META: Record<string, { name: string; icon: string; color: string }> = {
  // AllAgents universal clients
  copilot:   { name: 'GitHub Copilot',  icon: 'Co', color: '#6E40C9' },
  codex:     { name: 'OpenAI Codex',    icon: 'Cx', color: '#10A37F' },
  opencode:  { name: 'OpenCode',         icon: 'OC', color: '#3B82F6' },
  gemini:    { name: 'Gemini',          icon: 'Gm', color: '#4285F4' },
  ampcode:   { name: 'Amp Code',        icon: 'AC', color: '#8B5CF6' },
  vscode:    { name: 'VS Code',         icon: 'VS', color: '#007ACC' },
  replit:   { name: 'Replit',          icon: 'Rp', color: '#F97316' },
  kimi:      { name: 'Kimi',            icon: 'Ki', color: '#8B5CF6' },
  // AllAgents provider clients
  claude:    { name: 'Claude Code',     icon: 'CC', color: '#D97706' },
  cursor:    { name: 'Cursor',         icon: 'Cu', color: '#7C3AED' },
  factory:   { name: 'Factory',         icon: 'Fa', color: '#F59E0B' },
  windsurf:  { name: 'Windsurf',       icon: 'WS', color: '#3B82F6' },
  cline:     { name: 'Cline',          icon: 'Cl', color: '#10B981' },
  continue:  { name: 'Continue',       icon: 'Cn', color: '#6366F1' },
  roo:       { name: 'Roo',            icon: 'Ro', color: '#EC4899' },
  kilo:      { name: 'Kilo',           icon: 'Kl', color: '#8B5CF6' },
  trae:      { name: 'Trae',           icon: 'Tr', color: '#F97316' },
  augment:   { name: 'Augment',        icon: 'Ag', color: '#6366F1' },
  zencoder:  { name: 'Zencoder',       icon: 'Zc', color: '#14B8A6' },
  junie:     { name: 'Junie',          icon: 'Jn', color: '#84CC16' },
  openhands: { name: 'OpenHands',      icon: 'OH', color: '#F59E0B' },
  kiro:      { name: 'Kiro',           icon: 'Kr', color: '#3B82F6' },
  // Local CLI tools
  'claude-code':    { name: 'Claude Code CLI', icon: 'CC', color: '#B8944A' },
  'gemini-cli':     { name: 'Gemini CLI',     icon: 'Gm', color: '#2563EB' },
  'openclaw':       { name: 'OpenClaw',       icon: 'OC', color: '#7C3AED' },
  'deepseek-reasonix': { name: 'DeepSeek',    icon: 'DS', color: '#4F46E5' },
  'mimo-code':      { name: 'Mimo Code',       icon: 'MC', color: '#0891B2' },
  'qwen-code':      { name: 'Qwen Code',       icon: 'QC', color: '#2563EB' },
}

function getCliMeta(key: string) {
  return CLI_TOOL_META[key] ?? { name: key, icon: key.substring(0, 2).toUpperCase(), color: '#5C5C5C' }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function getPluginSyncState(plugin: MarketplacePlugin, toolKey: string): 'synced' | 'unsynced' | 'syncing' {
  const key = `${plugin.sourceId}::${plugin.name}::${toolKey}`
  return syncState.value[key] ?? 'unsynced'
}

function getPluginSyncCount(plugin: MarketplacePlugin): number {
  const prefix = `${plugin.sourceId}::${plugin.name}::`
  return Object.keys(syncState.value).filter(k => k.startsWith(prefix) && syncState.value[k] === 'synced').length
}

// ── Actions ──────────────────────────────────────────────────────────────────
const detailsPlugin = ref<MarketplacePlugin | null>(null)

async function refreshMarketplace() {
  isLoadingMarketplace.value = true
  try {
    await marketplaceStore.fetchPluginsBySource(currentSourceId.value)
  } finally {
    isLoadingMarketplace.value = false
  }
}

function openPluginDetails(plugin: MarketplacePlugin) {
  detailsPlugin.value = plugin
}

function openSyncDialog(plugin: MarketplacePlugin) {
  syncDialogPlugin.value = plugin
}

async function handleInstall(plugin: MarketplacePlugin) {
  const result = await marketplaceStore.installPlugin(plugin)
  if (result.success) {
    // Refresh sync statuses after installing a new plugin
    await loadAllSyncStatuses()
  } else {
    console.error('Install failed:', result.error)
  }
}

function togglePluginDropdown(plugin: MarketplacePlugin) {
  const key = `${plugin.sourceId}::${plugin.name}`
  openDropdown.value = openDropdown.value === key ? null : key
}

async function handleUninstall(plugin: MarketplacePlugin) {
  openDropdown.value = null
  if (!await confirm(`确定要卸载插件 "${plugin.name}" 吗？同步的内容将被移除。`)) return

  const pluginKey = `${plugin.sourceId}::${plugin.name}`
  uninstallProgress.value.set(pluginKey, { stage: 'running', progress: 10, message: '正在移除同步内容...' })

  try {
    // 1. 先移除所有同步内容
    if (plugin.cliToolKeys?.length) {
      for (const toolKey of plugin.cliToolKeys) {
        const syncKey = `${pluginKey}::${toolKey}`
        if (syncState.value[syncKey] === 'synced') {
          const result = await marketplaceStore.unsyncPluginFromCliTool(plugin, toolKey)
          if (result.success) {
            syncState.value[syncKey] = 'unsynced'
          } else {
            console.error(`Failed to unsync ${toolKey}:`, result.error)
          }
        }
      }
    }

    // 2. 再卸载插件
    uninstallProgress.value.set(pluginKey, { stage: 'running', progress: 60, message: '正在卸载插件...' })
    const result = await marketplaceStore.uninstallPlugin(plugin)

    if (result.success) {
      uninstallProgress.value.set(pluginKey, { stage: 'completed', progress: 100, message: '已卸载' })
      if (showNotification) showNotification(`插件 "${plugin.name}" 已卸载`, 'success')
    } else {
      uninstallProgress.value.set(pluginKey, { stage: 'failed', progress: 0, message: '卸载失败' })
      if (showNotification) showNotification(`卸载失败: ${result.error}`, 'error')
    }
  } catch (e) {
    uninstallProgress.value.set(pluginKey, { stage: 'failed', progress: 0, message: '卸载失败' })
    if (showNotification) showNotification(`卸载失败: ${e}`, 'error')
  } finally {
    setTimeout(() => uninstallProgress.value.delete(pluginKey), 5000)
  }
}

async function handleCliSync(plugin: MarketplacePlugin, toolKey: string) {
  const key = `${plugin.sourceId}::${plugin.name}::${toolKey}`
  const current = syncState.value[key] ?? 'unsynced'
  if (current === 'syncing') return

  syncState.value[key] = 'syncing'
  try {
    if (current === 'synced') {
      const result = await marketplaceStore.unsyncPluginFromCliTool(plugin, toolKey)
      syncState.value[key] = result.success ? 'unsynced' : current
    } else {
      const result = await marketplaceStore.syncPluginToCliTool(plugin, toolKey)
      syncState.value[key] = result.success ? 'synced' : current
    }
  } catch {
    syncState.value[key] = current
  }
}

// ── Mount ─────────────────────────────────────────────────────────────────────
onMounted(async () => {
  isLoadingInstalled.value = true
  isLoadingMarketplace.value = true
  try {
    await Promise.all([
      marketplaceStore.fetchInstalledPlugins(),
      marketplaceStore.loadMarketplaceManifest(),
      marketplaceStore.loadSourceStatus(),
      marketplaceStore.fetchSupportedCliTools(),
      softwareStore.fetchCliTools(),
      softwareStore.checkAllCliToolsStatus(),
    ])
    await marketplaceStore.fetchPluginsBySource(currentSourceId.value)
    // Load sync statuses for all installed plugins after they are fetched
    await loadAllSyncStatuses()
    initialSourceTabDone = true
  } finally {
    isLoadingInstalled.value = false
    isLoadingMarketplace.value = false
  }
})
</script>

<style scoped>
/* ── Layout ─────────────────────────────────────────────────────────────────── */
.view {
  position: relative;
}

.view > * {
  position: relative;
  z-index: 1;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ── Section Header ──────────────────────────────────────────────────────────── */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.30);
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title, #111111);
}

.section-header .count {
  font-size: 12px;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

/* ── Tab Bar (uses global .tab-bar from theme.css) ─────────────────────────── */

/* ── Installed ──────────────────────────────────────────────────────────────── */
.installed-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.installed-search {
  width: 280px;
}

/* ── Plugin Card ───────────────────────────────────────────────────────────── */
.plugin-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  align-items: stretch;
}

.card.plugin-card {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 16px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  overflow: visible;
  position: relative;
}

.card.plugin-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.plugin-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.plugin-icon {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.plugin-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.plugin-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
}

.plugin-version {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-ghost);
}

.sync-count {
  font-size: 10px;
  font-weight: 700;
  font-family: var(--font-mono);
  padding: 2px 8px;
  border-radius: 99px;
  background: rgba(90, 138, 100, 0.10);
  color: var(--success);
  border: 1px solid rgba(90, 138, 100, 0.20);
}

.plugin-card-meta {
  font-size: 12px;
  color: var(--fg-ghost);
  line-height: 1.5;
  margin-top: 2px;
}

.plugin-desc-clamp {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* ── Plugin Source Label ─────────────────────────────────────────────────────── */
.plugin-source-label {
  font-size: 11px;
  color: var(--fg-ghost);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 120px;
  text-align: right;
  flex-shrink: 0;
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

.plugin-cli-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-ghost);
  margin-right: 2px;
  white-space: nowrap;
}

.sync-count-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #22c55e;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}

.sync-count-badge:hover {
  background: rgba(34, 197, 94, 0.25);
}

/* ── Card Footer ────────────────────────────────────────────────────────────── */
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

/* ── Marketplace ────────────────────────────────────────────────────────────── */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.marketplace-search {
  flex: 1;
  min-width: 200px;
  max-width: 360px;
}

.marketplace-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

/* ── Sources ──────────────────────────────────────────────────────────────── */
.sources-search {
  flex: 1;
  min-width: 200px;
  max-width: 360px;
}

.sources-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 14px;
}

/* ── Empty State ────────────────────────────────────────────────────────────── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  gap: 12px;
  text-align: center;
}

.empty-state svg {
  color: var(--fg-ghost, #9A9A9A);
  opacity: 0.6;
}

.empty-state h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-title, #111111);
}

.empty-state p {
  font-size: 14px;
  color: var(--fg-muted, #5C5C5C);
  max-width: 480px;
  line-height: 1.5;
}

/* ── Skeleton ────────────────────────────────────────────────────────────────── */
@keyframes shimmer {
  0% { background-position: -400px 0; }
  100% { background-position: 400px 0; }
}

.skeleton {
  background: linear-gradient(90deg, rgba(255, 255, 255, 0.20) 25%, rgba(255, 255, 255, 0.40) 50%, rgba(255, 255, 255, 0.20) 75%);
  background-size: 800px 100%;
  animation: shimmer 1.5s ease-in-out infinite;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.20);
}

.skeleton-card {
  padding: 18px 20px;
  gap: 0;
}

.skeleton-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
}

.skeleton-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm, 12px);
  flex-shrink: 0;
}

.skeleton-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-name {
  width: 40%;
  height: 14px;
}

.skeleton-meta {
  width: 60%;
  height: 12px;
}

.skeleton-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skeleton-btn {
  width: 60px;
  height: 28px;
  border-radius: var(--radius-sm, 12px);
}

.skeleton-chips {
  height: 32px;
  margin-top: 12px;
  border-radius: 8px;
}

.skeleton-marketplace-card {
  height: 200px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
}

.skeleton-mcard-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm, 12px);
}

.skeleton-mcard-name {
  width: 60%;
  height: 14px;
}

.skeleton-mcard-desc {
  width: 90%;
  height: 12px;
}

.skeleton-mcard-footer {
  width: 40%;
  height: 24px;
  margin-top: auto;
  border-radius: var(--radius-sm, 12px);
}

/* ── Spinner ────────────────────────────────────────────────────────────────── */
.spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

</style>

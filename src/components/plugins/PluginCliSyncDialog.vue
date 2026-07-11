<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="handleOverlayClick">
      <div class="dialog-content">
        <!-- Header -->
        <header class="dialog-header">
          <h3 class="dialog-title">
            Sync
            <span class="plugin-name">{{ plugin?.name }}</span>
            to
          </h3>
          <button class="close-btn" aria-label="Close" @click="close">×</button>
        </header>

        <!-- Search -->
        <div class="dialog-search">
          <input
            v-model="searchQuery"
            type="search"
            placeholder="Search tools…"
            class="search-input"
          />
        </div>

        <!-- Tool list -->
        <div class="dialog-body">
          <!-- AllAgents: Universal Clients -->
          <template v-if="filteredUniversalClients.length">
            <div class="section-label">AllAgents · Universal Clients</div>
            <div class="tool-list">
              <div
                v-for="client in filteredUniversalClients"
                :key="client.key"
                :data-testid="`tool-${client.key}`"
                :class="['tool-item', { syncing: syncingKeys.has(client.key) }]"
                @click="handleSync(client)"
              >
                <ToolIcon :tool-key="client.key" :size="28" />
                <span class="tool-name">{{ client.name }}</span>
                <span v-if="client.pluginDir === null" class="unsupported-tag">No plugin dir</span>
                <div class="tool-status">
                  <span v-if="syncingKeys.has(client.key)" class="sync-spinner" />
                  <span v-else-if="getToolSyncState(client.key) === 'synced'" class="sync-badge synced">
                    Synced
                  </span>
                  <span v-else class="sync-badge unsynced">Not synced</span>
                </div>
              </div>
            </div>
          </template>

          <!-- AllAgents: Provider Clients -->
          <template v-if="filteredProviderClients.length">
            <div class="section-label">AllAgents · Provider Clients</div>
            <div class="tool-list">
              <div
                v-for="client in filteredProviderClients"
                :key="client.key"
                :data-testid="`tool-${client.key}`"
                :class="['tool-item', { syncing: syncingKeys.has(client.key) }]"
                @click="handleSync(client)"
              >
                <ToolIcon :tool-key="client.key" :size="28" />
                <span class="tool-name">{{ client.name }}</span>
                <span v-if="client.pluginDir === null" class="unsupported-tag">No plugin dir</span>
                <div class="tool-status">
                  <span v-if="syncingKeys.has(client.key)" class="sync-spinner" />
                  <span v-else-if="getToolSyncState(client.key) === 'synced'" class="sync-badge synced">
                    Synced
                  </span>
                  <span v-else class="sync-badge unsynced">Not synced</span>
                </div>
              </div>
            </div>
          </template>

          <!-- Local CLI Tools -->
          <template v-if="filteredLocalTools.length">
            <div class="section-label">Local CLI Tools</div>
            <div class="tool-list">
              <div
                v-for="tool in filteredLocalTools"
                :key="tool.key"
                :data-testid="`tool-${tool.key}`"
                :class="['tool-item', { syncing: syncingKeys.has(tool.key) }]"
                @click="handleSync(tool)"
              >
                <ToolIcon :tool-key="tool.key" :size="28" />
                <span class="tool-name">{{ tool.name }}</span>
                <span v-if="tool.pluginDir === null" class="unsupported-tag">No plugin dir</span>
                <div class="tool-status">
                  <span v-if="syncingKeys.has(tool.key)" class="sync-spinner" />
                  <span v-else-if="getToolSyncState(tool.key) === 'synced'" class="sync-badge synced">
                    Synced
                  </span>
                  <span v-else class="sync-badge unsynced">Not synced</span>
                </div>
              </div>
            </div>
          </template>

          <!-- Empty search state -->
          <div v-if="isEmpty" class="empty-state">
            No tools match "{{ searchQuery }}"
          </div>
        </div>

        <!-- Footer -->
        <footer class="dialog-footer">
          <span class="footer-hint">
            {{ syncedCount }} / {{ totalCount }} synced
          </span>
          <div class="footer-actions">
            <button
              v-if="hasSynced"
              class="btn btn-ghost"
              :disabled="isBusy"
              @click="handleUnsyncAll"
            >
              Unsync All
            </button>
            <button
              v-if="hasUnsynced"
              class="btn btn-primary"
              :disabled="isBusy"
              @click="handleSyncAll"
            >
              {{ isBusy ? 'Syncing…' : 'Sync All' }}
            </button>
          </div>
        </footer>
      </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch } from 'vue'
import ToolIcon from '@/components/common/ToolIcon.vue'
import {
  CLIENT_DISPLAY_NAMES,
  CLIENT_COLORS,
  type ClientType,
} from '@/types/unified-plugin'
import { usePluginMarketplaceStore } from '@/stores/plugin-marketplace'
import type { MarketplacePlugin } from '@/types'
import type { CliToolMeta } from '@/types'

type ShowNotification = (message: string, type?: string) => void

interface ToolEntry {
  key: string
  name: string
  icon: string
  color: string
  pluginDir: string | null
}

const props = defineProps<{
  plugin: MarketplacePlugin | null
  isOpen: boolean
}>()

const emit = defineEmits<{
  'update:isOpen': [value: boolean]
  synced: []
  unsynced: []
}>()

const store = usePluginMarketplaceStore()
const showNotification = inject<ShowNotification>('showNotification')
const toast = {
  success: (msg: string) => showNotification?.(msg, 'success'),
  error: (msg: string) => showNotification?.(msg, 'error'),
  warning: (msg: string) => showNotification?.(msg, 'warn'),
}

// -- State --
const searchQuery = ref('')
const syncingKeys = ref(new Set<string>())

// Derived tool lists
const universalClients: ToolEntry[] = [
  'copilot', 'codex', 'opencode', 'gemini', 'ampcode', 'vscode', 'replit', 'kimi',
].map(key => ({
  key,
  name: CLIENT_DISPLAY_NAMES[key as ClientType],
  icon: key.substring(0, 2).toUpperCase(),
  color: CLIENT_COLORS[key as ClientType],
  pluginDir: null, // allagents clients don't use plugin_dir sync
}))

const providerClients: ToolEntry[] = [
  'claude', 'cursor', 'factory', 'windsurf', 'cline', 'continue',
  'roo', 'kilo', 'trae', 'augment', 'zencoder', 'junie', 'openhands', 'kiro',
].map(key => ({
  key,
  name: CLIENT_DISPLAY_NAMES[key as ClientType],
  icon: key.substring(0, 2).toUpperCase(),
  color: CLIENT_COLORS[key as ClientType],
  pluginDir: null,
}))

const localTools = computed<ToolEntry[]>(() =>
  (store.supportedCliTools as CliToolMeta[]).map(t => ({
    key: t.key,
    name: t.name,
    icon: t.icon,
    color: t.color,
    pluginDir: t.pluginDir,
  }))
)

const filteredUniversalClients = computed(() =>
  filterBySearch(universalClients)
)
const filteredProviderClients = computed(() =>
  filterBySearch(providerClients)
)
const filteredLocalTools = computed(() =>
  filterBySearch(localTools.value)
)

const isEmpty = computed(() =>
  !filteredUniversalClients.value.length &&
  !filteredProviderClients.value.length &&
  !filteredLocalTools.value.length
)

const totalCount = computed(() =>
  filteredUniversalClients.value.length +
  filteredProviderClients.value.length +
  filteredLocalTools.value.length
)

const syncedCount = computed(() => {
  if (!props.plugin) return 0
  const all = [...filteredUniversalClients.value, ...filteredProviderClients.value, ...filteredLocalTools.value]
  return all.filter(t => getToolSyncState(t.key) === 'synced').length
})

const hasSynced = computed(() => syncedCount.value > 0)
const hasUnsynced = computed(() => syncedCount.value < totalCount.value)
const isBusy = computed(() => syncingKeys.value.size > 0)

// -- Helpers --
function filterBySearch(list: ToolEntry[]): ToolEntry[] {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return list
  return list.filter(t =>
    t.key.toLowerCase().includes(q) ||
    t.name.toLowerCase().includes(q)
  )
}

function getToolSyncState(key: string): 'synced' | 'unsynced' {
  if (!props.plugin) return 'unsynced'
  const syncKey = `${props.plugin.sourceId}::${props.plugin.name}::${key}`
  return store.syncStatuses[syncKey]?.synced ? 'synced' : 'unsynced'
}

function close() {
  emit('update:isOpen', false)
}

function handleOverlayClick() {
  close()
}

// -- Actions --
async function handleSync(tool: ToolEntry) {
  if (!props.plugin) return
  if (syncingKeys.value.has(tool.key)) return

  const current = getToolSyncState(tool.key)
  syncingKeys.value.add(tool.key)

  try {
    if (current === 'synced') {
      const result = await store.unsyncPluginFromCliTool(props.plugin, tool.key)
      if (result.success) {
        toast.success(`Unsynced from ${tool.name}`)
        emit('unsynced')
      } else {
        toast.error(result.error || 'Unsync failed')
      }
    } else {
      if (tool.pluginDir === null) {
        // Tool has no plugin directory — still try, backend will return error
        toast.warning(`${tool.name} has no plugin directory; sync will not copy files.`)
      }
      const result = await store.syncPluginToCliTool(props.plugin, tool.key)
      if (result.success) {
        toast.success(`Synced to ${tool.name}`)
        emit('synced')
      } else {
        toast.error(result.error || 'Sync failed')
      }
    }
  } finally {
    syncingKeys.value.delete(tool.key)
  }
}

async function handleSyncAll() {
  if (!props.plugin) return
  const unsynced = [
    ...filteredUniversalClients.value,
    ...filteredProviderClients.value,
    ...filteredLocalTools.value,
  ].filter(t => getToolSyncState(t.key) === 'unsynced')

  for (const tool of unsynced) {
    await handleSync(tool)
  }
}

async function handleUnsyncAll() {
  if (!props.plugin) return
  const synced = [
    ...filteredUniversalClients.value,
    ...filteredProviderClients.value,
    ...filteredLocalTools.value,
  ].filter(t => getToolSyncState(t.key) === 'synced')

  for (const tool of synced) {
    await handleSync(tool)
  }
}

// Reset search when dialog closes
watch(() => props.isOpen, (open) => {
  if (!open) {
    searchQuery.value = ''
    syncingKeys.value = new Set()
  }
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-content {
  background: var(--bg-primary);
  border-radius: 14px;
  width: 90%;
  max-width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.dialog-title {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--fg-primary);
}

.plugin-name {
  color: var(--accent);
}

.close-btn {
  background: none;
  border: none;
  font-size: 22px;
  cursor: pointer;
  color: var(--fg-muted);
  padding: 0 4px;
  line-height: 1;
}

.close-btn:hover {
  color: var(--fg-primary);
}

.dialog-search {
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--fg-primary);
  font-size: 14px;
  outline: none;
  box-sizing: border-box;
}

.search-input:focus {
  border-color: var(--accent);
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.section-label {
  padding: 8px 16px 4px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--fg-muted);
}

.tool-list {
  display: flex;
  flex-direction: column;
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 16px;
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
}

.tool-item:hover {
  background: var(--bg-hover);
}

.tool-item.syncing {
  opacity: 0.7;
  cursor: not-allowed;
}

.tool-name {
  flex: 1;
  font-size: 14px;
  color: var(--fg-primary);
}

.unsupported-tag {
  font-size: 11px;
  color: var(--fg-muted);
  background: var(--bg-secondary);
  border-radius: 4px;
  padding: 1px 6px;
}

.tool-status {
  flex-shrink: 0;
}

.sync-badge {
  font-size: 11px;
  border-radius: 4px;
  padding: 2px 7px;
  font-weight: 500;
}

.sync-badge.synced {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.sync-badge.unsynced {
  background: var(--bg-secondary);
  color: var(--fg-muted);
}

.sync-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  padding: 32px;
  text-align: center;
  color: var(--fg-muted);
  font-size: 14px;
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

.footer-hint {
  font-size: 13px;
  color: var(--fg-muted);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 6px 14px;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: opacity 0.15s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--accent);
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.88;
}

.btn-ghost {
  background: transparent;
  color: var(--fg-secondary);
  border: 1px solid var(--border-color);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-hover);
}
</style>

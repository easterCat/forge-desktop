<script setup lang="ts">
import { computed } from 'vue'
import type { PluginSource, SourceStatus, SourceInstallProgress } from '@/types'
import { PRESET_MARKETPLACE_SOURCES } from '@/types'
import ProgressSlot from '@/components/common/ProgressSlot.vue'
import { open as openExternal } from '@tauri-apps/plugin-shell'

interface Props {
  source: PluginSource
  status?: SourceStatus
  notes?: string
  isInstalling?: boolean
  installProgress?: SourceInstallProgress
}

const props = withDefaults(defineProps<Props>(), {
  status: undefined,
  notes: '',
  isInstalling: false,
  installProgress: undefined,
})

const emit = defineEmits<{
  'view-details': []
  install: []
  sync: []
  'edit-notes': []
  delete: []
}>()

const isInstalled = computed(() => props.status?.isInstalled ?? false)
const isPreset = computed(() => PRESET_MARKETPLACE_SOURCES.some(p => p.id === props.source.id))

const repoTypeLabel = computed(() => {
  switch (props.source.repoType) {
    case 'market': return 'Market'
    case 'res': return 'Resource'
    default: return props.source.repoType ?? 'Unknown'
  }
})

function mapInstallStage(stage: string): 'idle' | 'preparing' | 'downloading' | 'installing' | 'verifying' | 'completed' | 'failed' | 'cancelled' {
  switch (stage) {
    case 'preparing': return 'preparing'
    case 'cloning': return 'downloading'
    case 'extracting': return 'installing'
    case 'success': return 'completed'
    case 'failed': return 'failed'
    default: return 'preparing'
  }
}

const sourceUrl = computed(() => {
  return props.source.command ?? ''
})

const installedPath = computed(() => {
  return props.status?.installedPath ?? props.status?.installedPaths?.[0] ?? '—'
})

async function openExternalUrl(url: string) {
  try {
    await openExternal(url)
  } catch (e) {
    console.warn('shell.open failed, falling back to window.open:', e)
    try {
      window.open(url, '_blank', 'noopener,noreferrer')
    } catch (err) {
      console.error('Failed to open URL:', err)
    }
  }
}
</script>

<template>
  <div class="card source-card">
    <div class="source-card-head">
      <div class="source-card-icon">
        <svg v-if="source.repoType === 'market'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
          <polyline points="9 22 9 12 15 12 15 22"/>
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
      </div>
      <div style="flex:1;min-width:0">
        <div class="source-card-title">
          <span class="source-name-text">{{ source.name }}</span>
          <span v-if="isInstalled" class="badge success">Installed</span>
          <span v-else class="badge warn">Pending</span>
        </div>
        <div class="source-card-subtitle">
          {{ repoTypeLabel }} · {{ source.pluginCount ?? 0 }} plugins
        </div>
      </div>
    </div>

    <div class="source-card-notes" :class="{ 'is-empty': !notes }">{{ notes || '暂无' }}</div>

    <div v-if="sourceUrl" class="source-card-url" :title="sourceUrl">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
      </svg>
      <span class="url-text">{{ sourceUrl }}</span>
      <button class="url-link" title="Open in browser" @click.stop="openExternalUrl(sourceUrl)">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
          <polyline points="15 3 21 3 21 9"/>
          <line x1="10" y1="14" x2="21" y2="3"/>
        </svg>
      </button>
    </div>

    <div class="source-card-path">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <span class="path-text">{{ installedPath }}</span>
    </div>

    <div class="source-card-footer">
      <!-- Installing state with progress -->
      <template v-if="isInstalling">
        <ProgressSlot
          v-if="installProgress"
          :stage="mapInstallStage(installProgress.stage)"
          :progress="installProgress.progress"
          compact
        />
        <button class="btn btn-primary btn-sm" disabled>
          <span class="spinner"></span>
          <span>{{ installProgress?.message || 'Installing...' }}</span>
        </button>
      </template>

      <div v-else class="footer-actions">
        <button class="btn btn-sm btn-secondary" @click="emit('view-details')">
          Details
        </button>
        <button class="btn btn-sm btn-secondary" @click="emit('edit-notes')">
          Edit
        </button>
        <button v-if="!isPreset" class="btn btn-sm btn-secondary btn-delete" @click="emit('delete')">
          Delete
        </button>
        <!-- Installed state -->
        <button
          v-if="isInstalled"
          class="btn btn-primary btn-sm"
          @click="emit('sync')"
        >
          Sync
        </button>
        <!-- Not installed state -->
        <button
          v-else
          class="btn btn-primary btn-sm"
          @click="emit('install')"
        >
          Install
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.source-card {
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 14px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  overflow: hidden;
  position: relative;
}

.source-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.source-card-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.source-card-icon {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.source-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.source-name-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-subtitle {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
}

.source-card-notes {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-notes.is-empty {
  color: var(--fg-ghost);
  font-style: italic;
}

.source-card-url {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.source-card-url svg {
  flex-shrink: 0;
  opacity: 0.4;
}

.source-card-url .url-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-url .url-link {
  flex-shrink: 0;
  color: var(--fg-ghost);
  opacity: 0.4;
  transition: all var(--t-fast);
  cursor: pointer;
  display: flex;
  align-items: center;
}

.source-card-url .url-link:hover {
  opacity: 1;
  color: var(--accent);
}

.source-card-path {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.source-card-path svg {
  flex-shrink: 0;
  opacity: 0.4;
}

.source-card-path .path-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  border-top: 1px solid var(--border);
  padding-top: 14px;
  gap: 10px;
  flex-wrap: wrap;
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.footer-actions .btn {
  padding: 0 10px;
  font-size: 11px;
  font-weight: 500;
}

.footer-actions .btn-delete:hover {
  color: var(--error, #e53e3e);
  border-color: rgba(229, 62, 62, 0.25);
  background: rgba(229, 62, 62, 0.08);
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.01em;
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

.badge.warn {
  background: rgba(184, 148, 74, 0.15);
  border: 1px solid rgba(184, 148, 74, 0.20);
  color: var(--warn);
}

.badge.warn::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--warn);
}

.spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>

<template>
  <BaseCard variant="glass" padding="md" class="plugin-card" :class="{ installed: isInstalled, 'update-available': hasUpdate }" @click="onCardClick">
    <!-- Header with icon and badges -->
    <div class="card-header">
      <div class="plugin-icon">
        <BaseIcon size="md">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </BaseIcon>
      </div>
      <div class="card-badges">
        <span v-if="isInstalled && !hasUpdate" class="badge installed-badge">
          <BaseIcon size="xs">
            <polyline points="20 6 9 17 4 12"/>
          </BaseIcon>
          Installed
        </span>
        <span v-if="hasUpdate" class="badge update-badge">
          <BaseIcon size="xs">
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </BaseIcon>
          Update
        </span>
      </div>
    </div>

    <!-- Title and description -->
    <div class="card-content">
      <h3 class="plugin-name">{{ plugin.name }}</h3>
      <p class="plugin-desc">{{ plugin.description }}</p>
    </div>

    <!-- Categories -->
    <div v-if="plugin.categories.length || plugin.tags.length" class="card-categories">
      <span
        v-for="cat in plugin.categories.slice(0, 2)"
        :key="cat"
        class="category-tag"
      >
        {{ cat }}
      </span>
      <span
        v-for="tag in plugin.tags.slice(0, 2)"
        :key="tag"
        class="category-tag tag"
      >
        {{ tag }}
      </span>
    </div>

    <!-- Progress bar -->
    <div v-if="progress" class="card-progress-slot">
      <ProgressSlot
        :stage="mappedStage"
        :progress="progress.progress"
      />
    </div>

    <!-- CLI Sync Chips (only for installed plugins) -->
    <div v-if="plugin.isInstalled" class="plugin-cli-row">
      <span class="plugin-cli-label">CLI Tools</span>
      <CliSyncChip
        v-for="tool in installedCliTools"
        :key="tool.key"
        :tool-key="tool.key"
        :tool-name="tool.name"
        :tool-icon="tool.icon"
        :tool-color="tool.color"
        :state="getSyncStatus(plugin, tool.key)"
        :show-icon="true"
        :show-label="false"
        @click="() => $emit('sync', plugin, tool.key)"
      />
    </div>

    <!-- Actions -->
    <div class="card-actions">
      <!-- Details -->
      <button
        class="btn btn-detail btn-sm"
        @click.stop="$emit('view-details', plugin)"
      >
        <BaseIcon size="sm">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="16" x2="12" y2="12"/>
          <line x1="12" y1="8" x2="12.01" y2="8"/>
        </BaseIcon>
        Details
      </button>

      <!-- Uninstall -->
      <button
        v-if="isInstalled && !progress"
        class="btn btn-icon btn-sm"
        title="Uninstall"
        @click.stop="$emit('uninstall', plugin)"
      >
        <BaseIcon size="sm">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          <line x1="10" y1="11" x2="10" y2="17"/>
          <line x1="14" y1="11" x2="14" y2="17"/>
        </BaseIcon>
      </button>

      <!-- Not installed -->
      <button
        v-if="!isInstalled && !progress"
        class="btn btn-primary btn-sm"
        @click.stop="$emit('install', plugin)"
      >
        <BaseIcon size="sm">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </BaseIcon>
        Install
      </button>

      <!-- Update available -->
      <button
        v-else-if="hasUpdate && !progress"
        class="btn btn-primary btn-sm"
        @click.stop="$emit('update', plugin)"
      >
        <BaseIcon size="sm">
          <polyline points="23 4 23 10 17 10"/>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
        </BaseIcon>
        Update
      </button>

      <!-- Already installed -->
      <button
        v-else-if="isInstalled && !hasUpdate && !progress"
        class="btn btn-secondary btn-sm installed-btn"
        disabled
      >
        <BaseIcon size="sm">
          <polyline points="20 6 9 17 4 12"/>
        </BaseIcon>
        Installed
      </button>

      <!-- Installing/Updating/Uninstalling -->
      <button
        v-if="progress"
        class="btn btn-secondary btn-sm"
        disabled
      >
        <span v-if="progress.stage === 'downloading'">Downloading...</span>
        <span v-else-if="progress.stage === 'installing'">Installing...</span>
        <span v-else-if="progress.stage === 'success'">Done</span>
        <span v-else-if="progress.stage === 'failed'">Failed</span>
        <span v-else>Processing...</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { MarketplacePlugin, PluginInstallProgress } from '@/types';
import { useSoftwareStore } from '@/stores/software';
import CliSyncChip from '@/components/common/CliSyncChip.vue';
import ProgressSlot from '@/components/common/ProgressSlot.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseIcon from '@/components/common/BaseIcon.vue';
import type { OperationStage } from '@/composables/useOperationProgress';

const props = defineProps<{
  plugin: MarketplacePlugin;
  progress?: PluginInstallProgress;
}>();

const isInstalled = computed(() => props.plugin.isInstalled);
const hasUpdate = computed(() => props.plugin.hasUpdate);

const emit = defineEmits<{
  (e: 'install', plugin: MarketplacePlugin): void;
  (e: 'uninstall', plugin: MarketplacePlugin): void;
  (e: 'update', plugin: MarketplacePlugin): void;
  (e: 'view-details', plugin: MarketplacePlugin): void;
  (e: 'sync', plugin: any, toolKey: string): void;
}>();

const installedCliTools = computed(() => {
  const softwareStore = useSoftwareStore()
  return softwareStore.cliTools
    .filter(t => softwareStore.cliToolStatuses[t.key]?.isInstalled)
    .map(t => ({
      key: t.key,
      icon: t.key.substring(0, 2).toUpperCase(),
      name: t.name,
      color: CLI_TOOL_COLORS[t.key] ?? '#5C5C5C',
    }))
})

const CLI_TOOL_COLORS: Record<string, string> = {
  'claude-code': '#B8944A',
  'cursor': '#7C3AED',
  'codex': '#059669',
  'gemini-cli': '#2563EB',
  'opencode': '#0891B2',
  'deepseek': '#4F46E5',
}

function getSyncStatus(plugin: any, toolKey: string): 'synced' | 'unsynced' {
  if (plugin.syncedWith?.includes(toolKey)) return 'synced'
  return 'unsynced'
}

function onCardClick() {
  emit('view-details', props.plugin);
}

const mappedStage = computed(() => {
  if (!props.progress) return 'idle' as OperationStage
  const map: Record<string, OperationStage> = {
    pending: 'preparing',
    downloading: 'downloading',
    installing: 'installing',
    updating: 'installing',
    success: 'completed',
    failed: 'failed',
  }
  return map[props.progress.stage] || 'preparing'
})
</script>

<style scoped>
.plugin-card {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.plugin-card.installed {
  border-color: var(--success);
}

.plugin-card.update-available {
  border-color: var(--warn);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: var(--space-3, 12px);
}

.plugin-icon {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--fg-muted);
  flex-shrink: 0;
}

.card-badges {
  display: flex;
  gap: var(--space-1-5, 6px);
  flex-wrap: wrap;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1, 4px);
  padding: var(--space-1, 4px) var(--space-2-5, 10px);
  border-radius: var(--radius-full, 9999px);
  font-size: var(--text-xs, 11px);
  font-weight: 600;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.installed-badge {
  background: var(--success-bg);
  color: var(--success);
}

.update-badge {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warn);
}

.card-content {
  flex: 1;
  margin-bottom: var(--space-3, 12px);
}

.plugin-name {
  font-size: var(--text-base, 14px);
  font-weight: 600;
  margin-bottom: var(--space-1, 4px);
  color: var(--fg-title);
}

.plugin-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: var(--leading-normal, 1.5);
  min-height: 2.1em;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-categories {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1-5, 6px);
  margin-bottom: var(--space-3, 12px);
}

.category-tag {
  padding: var(--space-1, 4px) var(--space-2, 8px);
  font-size: var(--text-xs, 11px);
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  color: var(--fg-muted);
}

.category-tag.tag {
  background: var(--accent-bg);
  color: var(--accent);
}

.card-progress-slot {
  margin-bottom: var(--space-3, 12px);
}

.card-actions {
  display: flex;
  gap: var(--space-2, 8px);
  align-items: center;
}

.card-actions .btn:not(.btn-icon) {
  flex: 0 0 auto;
}

.card-actions .btn-primary,
.card-actions .btn-secondary {
  margin-left: auto;
}

.card-actions .btn-icon {
  flex: 0 0 auto;
  padding: 6px;
}

.card-actions .btn-icon:hover {
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.installed-btn {
  opacity: 0.7;
  cursor: not-allowed;
}

.plugin-cli-row {
  display: flex;
  align-items: center;
  gap: var(--space-2, 8px);
  flex-wrap: wrap;
  padding-top: var(--space-3, 12px);
  border-top: 1px solid var(--border);
  margin-top: var(--space-1, 4px);
}

.plugin-cli-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-right: var(--space-0-5, 2px);
  white-space: nowrap;
}

.cli-sync-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px 5px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all 150ms ease;
  border: 1px solid transparent;
  line-height: 1;
}

.cli-sync-chip .chip-icon {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 700;
  flex-shrink: 0;
}

.cli-sync-chip .chip-label { white-space: nowrap }

.cli-sync-chip .chip-status {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.cli-sync-chip.unsynced {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(184, 148, 74, 0.20);
  color: var(--fg-muted);
}

.cli-sync-chip.unsynced .chip-icon {
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid rgba(0, 0, 0, 0.06);
}

.cli-sync-chip.unsynced .chip-status {
  background: rgba(184, 148, 74, 0.15);
  color: var(--warn);
}

.cli-sync-chip.unsynced:hover {
  background: rgba(184, 148, 74, 0.10);
  border-color: rgba(184, 148, 74, 0.35);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.cli-sync-chip.synced {
  background: rgba(90, 138, 100, 0.06);
  border-color: rgba(90, 138, 100, 0.15);
  color: var(--fg-muted);
}

.cli-sync-chip.synced .chip-icon {
  background: rgba(90, 138, 100, 0.08);
  border: 1px solid rgba(90, 138, 100, 0.12);
}

.cli-sync-chip.synced .chip-status {
  background: rgba(90, 138, 100, 0.18);
  color: var(--success);
}

.cli-sync-chip.synced:hover {
  background: rgba(90, 138, 100, 0.12);
  border-color: rgba(90, 138, 100, 0.30);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
</style>

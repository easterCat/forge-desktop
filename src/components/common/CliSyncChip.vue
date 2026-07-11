<script setup lang="ts">
import { computed, onMounted } from 'vue'
import ToolIcon from './ToolIcon.vue'
import ClientSyncDialog from '../plugins/ClientSyncDialog.vue'
import { useClientSync } from '@/composables/useClientSync'

export type CliSyncState = 'unsynced' | 'syncing' | 'synced'

interface Props {
  /** Tool identifier key */
  toolKey: string
  /** Tool display name */
  toolName: string
  /** Tool icon abbreviation (e.g., 'CC', 'GM') */
  toolIcon: string
  /** Tool accent color for icon */
  toolColor?: string
  /** Sync state */
  state?: CliSyncState
  /** Show tool icon chip */
  showIcon?: boolean
  /** Show tool name label */
  showLabel?: boolean
  /** Show sync count chip */
  showSyncCount?: boolean
  /** Disabled state */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  toolColor: 'var(--accent)',
  state: 'unsynced',
  showIcon: true,
  showLabel: true,
  showSyncCount: false,
  disabled: false
})

const emit = defineEmits<{
  click: [toolKey: string, state: CliSyncState]
}>()

// 使用组合式函数
const clientSync = useClientSync()
const {
  clients,
  totalSyncedCount,
  isDialogOpen,
  syncingClient,
  toggleDialog,
  toggleSync,
  syncAll,
  initClients
} = clientSync

const totalClients = computed(() => clients.value.length)

// 初始化客户端列表
onMounted(() => {
  if (props.showSyncCount) {
    initClients()
  }
})

const handleChipClick = () => {
  if (!props.disabled) {
    emit('click', props.toolKey, props.state)
  }
}

const handleWrapperClick = () => {
  if (props.showSyncCount && !props.disabled) {
    toggleDialog()
  }
}

const handleToggleSync = (clientKey: string) => {
  toggleSync(clientKey)
}
</script>

<template>
  <div class="cli-sync-chip-wrapper" @click="handleWrapperClick">
    <span
      :class="['cli-sync-chip', state]"
      @click.stop="handleChipClick"
    >
      <span v-if="showIcon" class="chip-icon">
        <ToolIcon :tool-key="toolKey" :size="24" />
      </span>
      <span v-if="showLabel" class="chip-label">{{ toolName }}</span>
      <span class="chip-status">
        <!-- Synced checkmark -->
        <svg v-if="state === 'synced'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <!-- Unsynced sync icon -->
        <svg v-else-if="state === 'unsynced'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
        <!-- Syncing spinner (CSS-driven) -->
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
      </span>
    </span>

    <!-- 总数 chip -->
    <span v-if="showSyncCount" class="chip-badge" @click.stop="handleWrapperClick">
      {{ totalSyncedCount }}/{{ totalClients }}
      <span class="chip-sync-label">同步</span>
    </span>

    <!-- 客户端同步管理模态框 -->
    <ClientSyncDialog
      v-if="isDialogOpen"
      :clients="clients"
      :syncing-client="syncingClient"
      @toggle-sync="handleToggleSync"
      @close="toggleDialog"
      @sync-all="syncAll"
    />
  </div>
</template>

<style scoped>
.cli-sync-chip-wrapper {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  position: relative;
}

.chip-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  background: rgba(59, 130, 246, 0.10);
  border: 1px solid rgba(59, 130, 246, 0.25);
  color: var(--info);
  cursor: pointer;
  transition: all var(--t-fast);
}

.chip-badge:hover {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.35);
  transform: translateY(-1px);
}

.chip-sync-label {
  opacity: 0.8;
}

/* 原有样式保持不变 */
.cli-sync-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px 4px 6px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all var(--t-fast);
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
  line-height: 1;
}

/* Unsynced */
.cli-sync-chip.unsynced {
  background: rgba(156, 163, 175, 0.10);
  border-color: rgba(156, 163, 175, 0.25);
  color: var(--fg-ghost);
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.unsynced .chip-icon {
  background: rgba(156, 163, 175, 0.08);
  border: 1px solid rgba(156, 163, 175, 0.20);
  opacity: 0.6;
}

.cli-sync-chip.unsynced .chip-status {
  background: rgba(156, 163, 175, 0.15);
  color: var(--fg-ghost);
  opacity: 0.6;
}

.cli-sync-chip.unsynced:hover {
  background: rgba(156, 163, 175, 0.15);
  border-color: rgba(156, 163, 175, 0.35);
  opacity: 1;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
}

.cli-sync-chip.unsynced:hover .chip-icon {
  opacity: 0.8;
}

.cli-sync-chip.unsynced:hover .chip-status {
  background: rgba(156, 163, 175, 0.20);
  opacity: 0.8;
}

/* Synced */
.cli-sync-chip.synced {
  background: rgba(34, 197, 94, 0.12);
  border-color: rgba(34, 197, 94, 0.35);
  color: var(--success);
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.synced .chip-icon {
  background: rgba(34, 197, 94, 0.12);
  border: 1px solid rgba(34, 197, 94, 0.25);
}

.cli-sync-chip.synced .chip-status {
  background: rgba(34, 197, 94, 0.20);
  color: var(--success);
}

.cli-sync-chip.synced:hover {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.40);
  color: var(--error);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.15);
}

.cli-sync-chip.synced:hover .chip-icon {
  background: rgba(239, 68, 68, 0.10);
  border: 1px solid rgba(239, 68, 68, 0.25);
}

.cli-sync-chip.synced:hover .chip-status {
  background: rgba(239, 68, 68, 0.20);
  color: var(--error);
}

/* Syncing */
.cli-sync-chip.syncing {
  background: rgba(59, 130, 246, 0.08);
  border-color: rgba(59, 130, 246, 0.25);
  color: var(--info);
  pointer-events: none;
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.syncing .chip-icon {
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.18);
}

.cli-sync-chip.syncing .chip-status {
  background: rgba(59, 130, 246, 0.15);
  color: var(--info);
  animation: sync-spin 1s linear infinite;
}

@keyframes sync-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.chip-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--t-fast);
  overflow: hidden;
}

.chip-icon :deep(.tool-icon-img) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.chip-icon :deep(.tool-icon-fallback) {
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  font-weight: 700;
  font-size: 10px;
  color: var(--fg-muted, #71717A);
  letter-spacing: 0.02em;
  line-height: 1;
}

.chip-label {
  white-space: nowrap;
}

.chip-status {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--t-fast);
}

.chip-status svg {
  width: 8px;
  height: 8px;
}

@media (prefers-reduced-motion: reduce) {
  .cli-sync-chip.syncing .chip-status {
    animation: none;
  }
}
</style>

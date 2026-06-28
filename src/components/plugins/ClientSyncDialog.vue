<template>
  <div class="modal-overlay" @click.self="handleOverlayClick">
    <div class="modal-content">
      <header class="modal-header">
        <h3>客户端同步管理</h3>
        <button class="close-button" @click="$emit('close')">×</button>
      </header>

      <div class="client-list">
        <div
          v-for="client in clients"
          :key="client.key"
          :data-testid="`client-${client.key}`"
          :class="['client-item', { disabled: !client.isInstalled }]"
          @click="handleClientClick(client)"
        >
          <CliSyncChip
            :tool-key="client.key"
            :tool-name="client.name"
            :tool-icon="client.icon"
            :tool-color="client.color"
            :state="getClientState(client)"
            :disabled="!client.isInstalled"
          />

          <!-- 加载动画 -->
          <div
            v-if="syncingClient === client.key"
            :data-testid="`spinner-${client.key}`"
            class="loading-spinner"
          />

          <!-- 安装状态标签 -->
          <span v-if="!client.isInstalled" class="status-tag">未安装</span>
        </div>
      </div>

      <footer class="modal-footer">
        <button
          data-testid="sync-all-button"
          class="sync-all-button"
          :disabled="!hasUnsyncedClients"
          @click="$emit('syncAll')"
        >
          全部同步
        </button>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import CliSyncChip from '../common/CliSyncChip.vue'
import type { ClientInfo } from '@/composables/useClientSync'

const props = defineProps<{
  clients: ClientInfo[]
  syncingClient: string | null
}>()

const emit = defineEmits<{
  toggleSync: [clientKey: string]
  close: []
  syncAll: []
}>()

const hasUnsyncedClients = computed(() =>
  props.clients.some(c => c.isInstalled && !c.isSynced)
)

const getClientState = (client: ClientInfo) => {
  if (props.syncingClient === client.key) return 'syncing'
  if (client.isSynced) return 'synced'
  return 'unsynced'
}

const handleClientClick = (client: ClientInfo) => {
  if (!client.isInstalled) return
  emit('toggleSync', client.key)
}

const handleOverlayClick = (event: MouseEvent) => {
  // 只在点击 overlay 本身时关闭，不包括子元素
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-primary);
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-button {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--fg-muted);
  padding: 0;
  line-height: 1;
}

.close-button:hover {
  color: var(--fg-primary);
}

.client-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.client-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.client-item:hover:not(.disabled) {
  background: var(--bg-hover);
}

.client-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.status-tag {
  font-size: 12px;
  color: var(--fg-muted);
  margin-left: auto;
}

.modal-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.sync-all-button {
  padding: 8px 16px;
  border-radius: 6px;
  background: var(--accent);
  color: white;
  border: none;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.2s;
}

.sync-all-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sync-all-button:hover:not(:disabled) {
  opacity: 0.9;
}
</style>

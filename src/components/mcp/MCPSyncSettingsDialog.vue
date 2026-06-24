<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>MCP Sync Settings</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <p class="description">
          Configure which agent platforms to sync installed MCP servers to. Servers will be added to the MCP configuration files of selected targets.
        </p>

        <!-- Existing Targets -->
        <div class="targets-section">
          <h4>Sync Targets</h4>
          <div class="targets-list">
            <div
              v-for="target in targets"
              :key="target.id"
              class="target-item"
            >
              <div class="target-info">
                <div class="target-header">
                  <span class="target-name">{{ target.name }}</span>
                  <span class="method-badge" :class="target.method">
                    {{ target.method }}
                  </span>
                </div>
                <code class="target-path">{{ target.path }}</code>
              </div>
              <div class="target-actions">
                <span v-if="target.exists" class="status-badge success">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                  Config found
                </span>
                <span v-else class="status-badge warning">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="12" y1="8" x2="12" y2="12"/>
                    <line x1="12" y1="16" x2="12.01" y2="16"/>
                  </svg>
                  Not found
                </span>
                <button
                  class="remove-btn"
                  @click="$emit('remove', target.id)"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Add New Target -->
        <div class="add-target-section">
          <h4>Add New Target</h4>
          <div class="add-form">
            <div class="form-row">
              <div class="form-group">
                <label>Agent Name</label>
                <input
                  type="text"
                  v-model="newTarget.name"
                  placeholder="e.g., Claude Desktop"
                />
              </div>
              <div class="form-group">
                <label>Sync Method</label>
                <select v-model="newTarget.method">
                  <option value="copy">Copy</option>
                  <option value="symlink">Symlink</option>
                </select>
              </div>
            </div>
            <div class="form-group">
              <label>Config File Path</label>
              <input
                type="text"
                v-model="newTarget.path"
                placeholder="e.g., ~/.config/claude/mcp.json"
              />
            </div>
            <button
              class="btn btn-primary btn-sm"
              :disabled="!newTarget.name || !newTarget.path"
              @click="addTarget"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Add Target
            </button>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          Close
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { MCPSyncTarget } from '@/types';

interface Props {
  targets: MCPSyncTarget[];
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'add', target: MCPSyncTarget): void;
  (e: 'remove', targetId: string): void;
}>();

const newTarget = ref({
  name: '',
  path: '',
  method: 'copy' as 'copy' | 'symlink',
});

function addTarget() {
  const target: MCPSyncTarget = {
    id: `custom-${Date.now()}`,
    name: newTarget.value.name,
    path: newTarget.value.path,
    method: newTarget.value.method,
    isValid: true,
    configFile: newTarget.value.path.split('/').pop() || 'mcp.json',
  };
  
  emit('add', target);
  
  // Reset form
  newTarget.value = {
    name: '',
    path: '',
    method: 'copy',
  };
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.dialog {
  width: 100%;
  max-width: 560px;
  max-height: 80vh;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-secondary);
  color: var(--fg);
}

.dialog-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.description {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.targets-section {
  margin-bottom: 24px;
}

.targets-section h4,
.add-target-section h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

.targets-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.target-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.target-info {
  flex: 1;
  min-width: 0;
}

.target-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.target-name {
  font-size: 14px;
  font-weight: 500;
}

.method-badge {
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 500;
  border-radius: 3px;
  text-transform: uppercase;
}

.method-badge.copy {
  background: var(--info-bg);
  color: var(--info);
}

.method-badge.symlink {
  background: var(--success-bg);
  color: var(--success);
}

.target-path {
  font-size: 12px;
  font-family: monospace;
  color: var(--fg-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.target-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 12px;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
}

.status-badge.success {
  background: var(--success-bg);
  color: var(--success);
}

.status-badge.warning {
  background: var(--warn-bg);
  color: var(--warn);
}

.remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.remove-btn:hover {
  background: var(--error-bg);
  color: var(--error);
}

.add-target-section {
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.add-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg);
}

.form-group input,
.form-group select {
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--accent);
}

.form-group input::placeholder {
  color: var(--fg-muted);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
</style>

<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>Sync Settings</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Description -->
      <div class="dialog-content">
        <p class="description">
          Configure sync targets to automatically copy or symlink installed skills to different agent directories.
        </p>

        <!-- Existing Targets -->
        <div class="targets-section">
          <h4>Current Targets</h4>
          <div class="targets-list">
            <div
              v-for="target in targets"
              :key="target.id"
              class="target-item"
            >
              <div class="target-info">
                <div class="target-name">{{ target.name }}</div>
                <div class="target-path">{{ target.path }}</div>
              </div>
              <div class="target-method">
                <span class="method-badge" :class="target.method">
                  {{ target.method === 'copy' ? 'Copy' : 'Symlink' }}
                </span>
              </div>
              <div class="target-status">
                <span v-if="target.exists" class="status-dot valid"></span>
                <span v-else class="status-dot invalid"></span>
              </div>
              <button
                class="btn btn-icon btn-sm"
                @click="$emit('remove', target.id)"
                title="Remove target"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6"/>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                </svg>
              </button>
            </div>
            <div v-if="targets.length === 0" class="empty-targets">
              No sync targets configured
            </div>
          </div>
        </div>

        <!-- Add New Target -->
        <div class="add-section">
          <h4>Add New Target</h4>
          <div class="add-form">
            <div class="form-row">
              <div class="form-group">
                <label>Name</label>
                <input
                  type="text"
                  v-model="newTarget.name"
                  placeholder="e.g., Claude Desktop"
                />
              </div>
              <div class="form-group method-group">
                <label>Method</label>
                <select v-model="newTarget.method">
                  <option value="copy">Copy</option>
                  <option value="symlink">Symlink</option>
                </select>
              </div>
            </div>
            <div class="form-group">
              <label>Path</label>
              <div class="path-input">
                <input
                  type="text"
                  v-model="newTarget.path"
                  placeholder="~/.claude/skills/"
                />
                <button class="btn btn-secondary btn-sm" @click="browsePath">
                  Browse
                </button>
              </div>
              <span class="hint">Use ~ for home directory (e.g., ~/.cursor/skills/)</span>
            </div>
            <button
              class="btn btn-secondary"
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

        <!-- Method Explanation -->
        <div class="method-explanation">
          <h4>Sync Methods</h4>
          <div class="method-cards">
            <div class="method-card">
              <div class="method-icon copy">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                </svg>
              </div>
              <div class="method-info">
                <h5>Copy</h5>
                <p>Creates a full copy of the skill files. Recommended for production use.</p>
              </div>
            </div>
            <div class="method-card">
              <div class="method-icon symlink">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
                  <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
                </svg>
              </div>
              <div class="method-info">
                <h5>Symlink</h5>
                <p>Creates a symbolic link. Saves disk space but requires original files.</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-primary" @click="$emit('close')">
          Done
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { SyncTarget } from '@/types';

interface Props {
  targets: SyncTarget[];
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'add', target: SyncTarget): void;
  (e: 'remove', targetId: string): void;
}>();

const newTarget = ref({
  name: '',
  path: '',
  method: 'copy' as 'copy' | 'symlink',
});

function browsePath() {
  // TODO: Implement path browser via Tauri
}

function addTarget() {
  if (newTarget.value.name && newTarget.value.path) {
    const target: SyncTarget = {
      id: `custom-${Date.now()}`,
      name: newTarget.value.name,
      path: newTarget.value.path,
      method: newTarget.value.method,
      isValid: true,
      exists: true,
    };
    emit('add', target);
    newTarget.value = { name: '', path: '', method: 'copy' };
  }
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
  position: relative;
  width: 100%;
  max-width: 600px;
  max-height: 80vh;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  z-index: 1;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
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
  margin-bottom: 20px;
  line-height: 1.5;
}

.targets-section,
.add-section,
.method-explanation {
  margin-bottom: 24px;
}

h4 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--fg);
}

.targets-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.target-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.target-info {
  flex: 1;
}

.target-name {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 2px;
}

.target-path {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: 'SF Mono', Monaco, monospace;
}

.method-badge {
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
}

.method-badge.copy {
  background: var(--accent-bg);
  color: var(--accent);
}

.method-badge.symlink {
  background: var(--warn-bg);
  color: var(--warn);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.valid {
  background: var(--success);
}

.status-dot.invalid {
  background: var(--error);
}

.empty-targets {
  padding: 24px;
  text-align: center;
  color: var(--fg-muted);
  font-size: 13px;
  background: var(--bg-secondary);
  border: 1px dashed var(--border);
  border-radius: 8px;
}

.add-form {
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.form-row {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  margin-bottom: 6px;
  color: var(--fg);
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
}

.method-group {
  width: 120px;
  flex-shrink: 0;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
}

.hint {
  display: block;
  margin-top: 4px;
  font-size: 11px;
  color: var(--fg-muted);
}

.method-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.method-card {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.method-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  flex-shrink: 0;
}

.method-icon.copy {
  background: var(--accent-bg);
  color: var(--accent);
}

.method-icon.symlink {
  background: var(--warn-bg);
  color: var(--warn);
}

.method-info h5 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 4px;
}

.method-info p {
  font-size: 11px;
  color: var(--fg-muted);
  line-height: 1.4;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
</style>

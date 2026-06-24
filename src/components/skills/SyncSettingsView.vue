<template>
  <div class="sync-settings-view">
    <!-- Sub-view Header with Breadcrumb -->
    <div class="subview-header">
      <button class="back-btn" @click="$emit('back')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
        <span>Back to Skills</span>
      </button>
      <div class="subview-title-row">
        <div class="subview-title-block">
          <h2>Sync Settings</h2>
          <p class="subview-description">
            Configure sync targets to automatically copy or symlink installed skills to different agent directories.
          </p>
        </div>
      </div>
    </div>

    <!-- Sub-view Content -->
    <div class="subview-content">
      <!-- Current Targets -->
      <section class="settings-section">
        <div class="section-label">
          <h3>Current Targets</h3>
          <span class="section-count">{{ targets.length }} configured</span>
        </div>
        <div v-if="targets.length > 0" class="targets-list">
          <div
            v-for="target in targets"
            :key="target.id"
            class="target-item"
          >
            <div class="target-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </div>
            <div class="target-info">
              <div class="target-name-row">
                <span class="target-name">{{ target.name }}</span>
                <span class="method-badge" :class="target.method">
                  {{ target.method === 'copy' ? 'Copy' : 'Symlink' }}
                </span>
                <span class="status-indicator" :class="target.exists ? 'valid' : 'invalid'">
                  <span class="status-dot"></span>
                  {{ target.exists ? 'Active' : 'Missing' }}
                </span>
              </div>
              <div class="target-path">{{ target.path }}</div>
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
        </div>
        <div v-else class="empty-state-inline">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          <p>No sync targets configured</p>
          <span>Add a target below to begin syncing skills to external directories</span>
        </div>
      </section>

      <!-- Add New Target -->
      <section class="settings-section">
        <div class="section-label">
          <h3>Add New Target</h3>
        </div>
        <div class="add-form">
          <div class="form-row">
            <div class="form-group flex-1">
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
              <button class="btn btn-secondary" @click="browsePath">
                Browse
              </button>
            </div>
            <span class="hint">Use ~ for home directory (e.g., ~/.cursor/skills/)</span>
          </div>
          <div class="form-actions">
            <button
              class="btn btn-primary"
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
      </section>

      <!-- Method Explanation -->
      <section class="settings-section">
        <div class="section-label">
          <h3>Sync Methods</h3>
        </div>
        <div class="method-cards">
          <div class="method-card">
            <div class="method-icon copy">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </div>
            <div class="method-info">
              <h4>Copy</h4>
              <p>Creates a full copy of the skill files. Recommended for production use — target stays self-contained.</p>
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
              <h4>Symlink</h4>
              <p>Creates a symbolic link. Saves disk space and updates live — but breaks if source path changes.</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { SyncTarget } from '@/types';

interface Props {
  targets: SyncTarget[];
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'back'): void;
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
.sync-settings-view {
  display: flex;
  flex-direction: column;
  gap: 0;
  width: 100%;
  height: 100%;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Sub-view Header */
.subview-header {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 24px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  align-self: flex-start;
  padding: 4px 10px 4px 6px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.back-btn:hover {
  background: var(--bg-secondary);
  border-color: var(--border);
  color: var(--fg);
}

.back-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.subview-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.subview-title-block {
  flex: 1;
  min-width: 0;
}

.subview-title-block h2 {
  font-size: 1.5rem;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg);
  margin-bottom: 6px;
}

.subview-description {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  max-width: 640px;
}

/* Sub-view Content */
.subview-content {
  display: flex;
  flex-direction: column;
  gap: 28px;
  max-width: 880px;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-label {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.section-label h3 {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg);
}

.section-count {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: 'SF Mono', Monaco, 'JetBrains Mono', monospace;
}

/* Targets List */
.targets-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.target-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  transition: all 0.15s ease;
}

.target-item:hover {
  border-color: var(--border-hover);
}

.target-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-radius: 8px;
  color: var(--fg-muted);
  flex-shrink: 0;
}

.target-info {
  flex: 1;
  min-width: 0;
}

.target-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}

.target-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--fg);
}

.method-badge {
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-radius: 4px;
}

.method-badge.copy {
  background: var(--accent-bg);
  color: var(--accent);
}

.method-badge.symlink {
  background: rgba(245, 158, 11, 0.12);
  color: var(--warn, #D97706);
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 500;
  color: var(--fg-muted);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-indicator.valid {
  color: var(--success);
}

.status-indicator.valid .status-dot {
  background: var(--success);
  box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.15);
}

.status-indicator.invalid {
  color: var(--error);
}

.status-indicator.invalid .status-dot {
  background: var(--error);
}

.target-path {
  font-size: 12px;
  color: var(--fg-muted);
  font-family: 'SF Mono', Monaco, 'JetBrains Mono', monospace;
  word-break: break-all;
}

.empty-state-inline {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 20px;
  background: var(--bg-card);
  border: 1px dashed var(--border);
  border-radius: 10px;
  color: var(--fg-muted);
  text-align: center;
}

.empty-state-inline svg {
  opacity: 0.5;
}

.empty-state-inline p {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
}

.empty-state-inline span {
  font-size: 12px;
  max-width: 360px;
}

/* Add Form */
.add-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 18px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group.flex-1 {
  flex: 1;
  min-width: 0;
}

.form-group label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-muted);
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 9px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
  transition: all 0.15s ease;
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--accent);
  background: var(--bg-card);
}

.form-group input::placeholder {
  color: var(--fg-ghost);
}

.method-group {
  width: 130px;
  flex-shrink: 0;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
  font-family: 'SF Mono', Monaco, 'JetBrains Mono', monospace;
  font-size: 12px;
}

.hint {
  font-size: 11px;
  color: var(--fg-muted);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
}

/* Method Cards */
.method-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.method-card {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  transition: all 0.15s ease;
}

.method-card:hover {
  border-color: var(--border-hover);
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
  background: rgba(245, 158, 11, 0.12);
  color: var(--warn, #D97706);
}

.method-info h4 {
  font-size: 13px;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 4px;
}

.method-info p {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
}

/* Responsive */
@media (max-width: 768px) {
  .form-row {
    flex-direction: column;
  }

  .method-group {
    width: 100%;
  }

  .method-cards {
    grid-template-columns: 1fr;
  }
}
</style>

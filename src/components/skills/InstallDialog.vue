<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>Install Skill</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Skill Info -->
      <div class="skill-preview">
        <div class="skill-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
        </div>
        <div class="skill-info">
          <h4>{{ skill.name }}</h4>
          <p>{{ skill.description }}</p>
        </div>
      </div>

      <!-- Local Path -->
      <div class="form-group">
        <label>Install to</label>
        <div class="path-input">
          <input
            type="text"
            v-model="localPath"
            placeholder="Select installation path..."
            readonly
          />
          <button class="btn btn-secondary btn-sm" @click="browsePath">
            Browse
          </button>
        </div>
        <span class="hint">Skills will be installed to this project's ./skills/ folder</span>
      </div>

      <!-- Sync Targets -->
      <div class="form-group">
        <label>Sync to targets</label>
        <div class="sync-targets-list">
          <div
            v-for="target in syncTargets"
            :key="target.id"
            class="sync-target-item"
          >
            <label class="checkbox-label">
              <input
                type="checkbox"
                v-model="selectedTargets"
                :value="target"
              />
              <span class="checkbox-custom"></span>
              <span class="target-name">{{ target.name }}</span>
            </label>
            <select v-model="target.method" class="method-select">
              <option value="copy">Copy</option>
              <option value="symlink">Symlink</option>
            </select>
          </div>
        </div>
        <span class="hint">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
          Copy: Full file copy | Symlink: Create symbolic link
        </span>
      </div>

      <!-- Install Command Preview -->
      <div v-if="skill.installCommand" class="form-group">
        <label>Install Command</label>
        <div class="command-preview">
          <code>{{ skill.installCommand }}</code>
          <button class="copy-btn" @click="copyCommand">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          Cancel
        </button>
        <button
          class="btn btn-primary"
          :disabled="!localPath"
          @click="handleInstall"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Install
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { MarketplaceSkill, SyncTarget } from '@/types';

interface Props {
  skill: MarketplaceSkill;
  syncTargets: SyncTarget[];
  /**
   * Default install path to prefill. This is the same path the
   * Installed tab uses to list local skills, so installing here
   * immediately shows up in the Installed tab. Resolved by the
   * parent from `get_default_skills_dir`.
   */
  defaultLocalPath?: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'install', data: { skill: MarketplaceSkill; syncTargets: SyncTarget[]; localPath: string }): void;
}>();

const localPath = ref(props.defaultLocalPath ?? '');
const selectedTargets = ref<SyncTarget[]>([]);

// If the parent resolves the default path asynchronously after mount
// and we were opened before that finished, update when it arrives.
watch(() => props.defaultLocalPath, (next) => {
  if (next && !localPath.value) {
    localPath.value = next;
  }
});

// Default to first target
watch(() => props.syncTargets, (targets) => {
  if (targets.length > 0) {
    selectedTargets.value = [targets[0]];
  }
}, { immediate: true });

async function browsePath() {
  // TODO: Implement file browser dialog via Tauri
  // For now, keep the default path that the parent provided.
}

function copyCommand() {
  if (props.skill.installCommand) {
    navigator.clipboard.writeText(props.skill.installCommand);
  }
}

function handleInstall() {
  emit('install', {
    skill: props.skill,
    syncTargets: selectedTargets.value,
    localPath: localPath.value,
  });
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
  max-width: 500px;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  z-index: 1;
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

.skill-preview {
  display: flex;
  gap: 12px;
  padding: 16px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.skill-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent);
  border-radius: 10px;
  color: white;
  flex-shrink: 0;
}

.skill-info h4 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.skill-info p {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.4;
}

.form-group {
  padding: 16px 20px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--fg);
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
}

.hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.hint svg {
  flex-shrink: 0;
}

.sync-targets-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sync-target-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.checkbox-label input {
  display: none;
}

.checkbox-custom {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border);
  border-radius: 4px;
  transition: all 0.2s;
  position: relative;
}

.checkbox-label input:checked + .checkbox-custom {
  background: var(--accent);
  border-color: var(--accent);
}

.checkbox-label input:checked + .checkbox-custom::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.target-name {
  font-size: 13px;
}

.method-select {
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 12px;
  color: var(--fg);
  cursor: pointer;
}

.command-preview {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.command-preview code {
  flex: 1;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 12px;
  color: var(--accent);
  word-break: break-all;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--bg-secondary);
  color: var(--fg);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
</style>

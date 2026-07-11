<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>Install MCP Server</h3>
        <button class="close-btn" aria-label="关闭" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Server Info -->
      <div class="server-preview">
        <div class="server-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
            <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
            <line x1="6" y1="6" x2="6.01" y2="6"/>
            <line x1="6" y1="18" x2="6.01" y2="18"/>
          </svg>
        </div>
        <div class="server-info">
          <h4>{{ server.name }}</h4>
          <p>{{ server.description }}</p>
        </div>
        <div class="protocol-badge" :class="server.protocol">
          {{ server.protocol.toUpperCase() }}
        </div>
      </div>

      <!-- Required Env Vars -->
      <div v-if="server.requiredEnvVars?.length" class="form-group env-section">
        <label>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
          Required Environment Variables
        </label>
        <div class="env-vars-list">
          <div
            v-for="env in server.requiredEnvVars"
            :key="env.name"
            class="env-var-item"
          >
            <div class="env-header">
              <code class="env-name">{{ env.name }}</code>
              <span v-if="env.required" class="required-badge">Required</span>
              <span v-else class="optional-badge">Optional</span>
            </div>
            <p v-if="env.description" class="env-desc">{{ env.description }}</p>
            <div v-if="env.example" class="env-example">
              <span>Example:</span>
              <code>{{ env.example }}</code>
            </div>
          </div>
        </div>
      </div>

      <!-- Install Directory -->
      <div class="form-group">
        <label>Install Directory</label>
        <div class="path-input">
          <input
            v-model="installDir"
            type="text"
            placeholder="Select installation directory..."
          />
          <button class="btn btn-secondary btn-sm" @click="browsePath">
            Browse
          </button>
        </div>
        <span class="hint">MCP servers will be installed to this directory's ./mcp/servers/ folder</span>
      </div>

      <!-- Sync Targets -->
      <div class="form-group">
        <label>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          Sync to Agent Targets
        </label>
        <div class="sync-targets-list">
          <div
            v-for="target in syncTargets"
            :key="target.id"
            class="sync-target-item"
          >
            <label class="checkbox-label">
              <input
                v-model="selectedTargets"
                type="checkbox"
                :value="target"
              />
              <span class="checkbox-custom"></span>
              <span class="target-name">{{ target.name }}</span>
              <code v-if="target.configFile" class="target-config">{{ target.configFile }}</code>
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
          Copy: Full file copy | Symlink: Create symbolic link (saves disk space)
        </span>
      </div>

      <!-- Install Command Preview -->
      <div v-if="server.installCommand" class="form-group">
        <label>Install Command</label>
        <div class="command-preview">
          <code>{{ server.installCommand }}</code>
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
          :disabled="!installDir"
          @click="handleInstall"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Install Server
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { MCPServerUnion, MarketplaceMCPSyncTarget } from '@/types';

interface Props {
  server: MCPServerUnion;
  syncTargets: MarketplaceMCPSyncTarget[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'install', data: { server: MCPServerUnion; syncTargets: MarketplaceMCPSyncTarget[]; installDir: string }): void;
}>();

const installDir = ref('');
const selectedTargets = ref<MarketplaceMCPSyncTarget[]>([]);

// Default to first target
watch(() => props.syncTargets, (targets) => {
  if (targets.length > 0) {
    selectedTargets.value = [targets[0]];
  }
}, { immediate: true });

async function browsePath() {
  // TODO: Implement file browser dialog via Tauri
}

function copyCommand() {
  if (props.server.installCommand) {
    navigator.clipboard.writeText(props.server.installCommand);
  }
}

function handleInstall() {
  emit('install', {
    server: props.server,
    syncTargets: selectedTargets.value,
    installDir: installDir.value,
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
  width: 100%;
  max-width: 560px;
  max-height: 90vh;
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  display: flex;
  flex-direction: column;
  overflow: hidden;
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

.server-preview {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.server-icon {
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

.server-info {
  flex: 1;
  min-width: 0;
}

.server-info h4 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.server-info p {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.protocol-badge {
  padding: 4px 8px;
  font-size: 11px;
  font-weight: 600;
  font-family: monospace;
  border-radius: 4px;
  flex-shrink: 0;
}

.protocol-badge.stdio {
  background: var(--success-bg);
  color: var(--success);
}

.protocol-badge.sse {
  background: var(--warn-bg);
  color: var(--warn);
}

.protocol-badge.http {
  background: var(--info-bg);
  color: var(--info);
}

.form-group {
  padding: 16px 20px;
}

.form-group label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--fg);
}

.form-group label svg {
  color: var(--fg-muted);
}

.env-section {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.env-vars-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.env-var-item {
  padding: 10px 12px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.env-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.env-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
}

.required-badge {
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 500;
  background: var(--error-bg);
  color: var(--error);
  border-radius: 3px;
}

.optional-badge {
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 500;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
  border-radius: 3px;
}

.env-desc {
  font-size: 12px;
  color: var(--fg-muted);
  margin-bottom: 4px;
}

.env-example {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-muted);
}

.env-example code {
  padding: 2px 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: monospace;
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
  max-height: 200px;
  overflow-y: auto;
}

.sync-target-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
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
  flex-shrink: 0;
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

.target-config {
  font-size: 11px;
  color: var(--fg-muted);
  padding: 2px 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
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

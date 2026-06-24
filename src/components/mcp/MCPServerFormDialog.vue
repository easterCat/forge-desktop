<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
      <!-- Header -->
      <div class="dialog-header">
        <h3 id="dialog-title">{{ mode === 'add' ? 'Add MCP Server' : 'Edit MCP Server' }}</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="Close dialog">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <form @submit.prevent="handleSubmit" class="server-form">
          <!-- Name -->
          <div class="form-group" :class="{ 'has-error': errors.name }">
            <label for="name">Server Name *</label>
            <input
              id="name"
              v-model="form.name"
              type="text"
              placeholder="e.g., My MCP Server"
              @blur="validateName"
            />
            <span v-if="errors.name" class="error-text">{{ errors.name }}</span>
          </div>

          <!-- Endpoint -->
          <div class="form-group" :class="{ 'has-error': errors.endpoint }">
            <label for="endpoint">Endpoint *</label>
            <input
              id="endpoint"
              v-model="form.endpoint"
              type="text"
              placeholder="e.g., https://api.example.com/mcp or /path/to/socket"
              @blur="validateEndpoint"
            />
            <span v-if="errors.endpoint" class="error-text">{{ errors.endpoint }}</span>
          </div>

          <!-- Port -->
          <div class="form-group" :class="{ 'has-error': errors.port }">
            <label for="port">Port (optional)</label>
            <input
              id="port"
              v-model.number="form.port"
              type="number"
              min="1"
              max="65535"
              placeholder="e.g., 3000"
              @blur="validatePort"
            />
            <span v-if="errors.port" class="error-text">{{ errors.port }}</span>
          </div>

          <!-- Protocol -->
          <div class="form-group">
            <label for="protocol">Protocol</label>
            <select id="protocol" v-model="form.protocol">
              <option value="http">HTTP</option>
              <option value="sse">SSE (Server-Sent Events)</option>
              <option value="stdio">STDIO</option>
            </select>
          </div>

          <!-- Auth Type -->
          <div class="form-group">
            <label for="authType">Authentication</label>
            <select id="authType" v-model="form.authType">
              <option value="none">None</option>
              <option value="bearer">Bearer Token</option>
              <option value="api-key">API Key</option>
            </select>
          </div>

          <!-- Configuration (JSON) -->
          <div class="form-group config-group" :class="{ 'has-error': errors.config }">
            <label>Configuration (JSON)</label>
            <div class="config-editor">
              <textarea
                v-model="form.config"
                class="config-textarea"
                placeholder='{"key": "value"}'
                rows="6"
                @input="validateConfig"
              ></textarea>
            </div>
            <span v-if="errors.config" class="error-text">{{ errors.config }}</span>
          </div>

          <!-- Tags -->
          <div class="form-group">
            <label for="tags">Tags (comma separated)</label>
            <input
              id="tags"
              v-model="tagsInput"
              type="text"
              placeholder="e.g., ai, productivity, api"
            />
          </div>

          <!-- Test Connection Result -->
          <div v-if="testResult" class="test-result" :class="testResult">
            <span class="test-icon">
              <svg v-if="testResult === 'success'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
            </span>
            <span>{{ testResult === 'success' ? 'Connection successful!' : 'Connection failed' }}</span>
          </div>
        </form>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button
          type="button"
          class="btn btn-secondary"
          :disabled="isTesting"
          @click="handleTestConnection"
        >
          <svg v-if="isTesting" class="spinner" width="14" height="14" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
          </svg>
          <span v-if="isTesting">Testing...</span>
          <span v-else>Test Connection</span>
        </button>
        <button class="btn btn-secondary" @click="$emit('close')">
          Cancel
        </button>
        <button
          type="submit"
          class="btn btn-primary"
          :disabled="isLoading"
          @click="handleSubmit"
        >
          <svg v-if="isLoading" class="spinner" width="14" height="14" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
          </svg>
          <span>{{ mode === 'add' ? 'Add Server' : 'Save Changes' }}</span>
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { reactive, ref, watch, computed } from 'vue';
import type { MCPService, MCPServiceFormData, ServerProtocol, AuthType } from '@/types';

interface Props {
  mode: 'add' | 'edit';
  service?: MCPService | null;
  isLoading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  service: null,
  isLoading: false,
});

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'submit', data: MCPServiceFormData): void;
  (e: 'test-connection', data: MCPServiceFormData): void;
}>();

const form = reactive({
  name: props.service?.name ?? '',
  endpoint: props.service?.endpoint ?? '',
  port: props.service?.port,
  protocol: (props.service?.protocol ?? 'http') as ServerProtocol,
  authType: (props.service?.authType ?? 'none') as AuthType,
  config: props.service?.config ?? '{}',
  groupIds: props.service?.groupIds ?? [],
});

const tagsInput = ref(props.service?.tags?.join(', ') ?? '');

const errors = reactive<Record<string, string>>({});
const isTesting = ref(false);
const testResult = ref<'success' | 'error' | null>(null);

// Watch for service changes in edit mode
watch(
  () => props.service,
  (newService) => {
    if (newService) {
      form.name = newService.name;
      form.endpoint = newService.endpoint;
      form.port = newService.port;
      form.protocol = newService.protocol;
      form.authType = newService.authType;
      form.config = newService.config;
      form.groupIds = newService.groupIds;
      tagsInput.value = newService.tags?.join(', ') ?? '';
    }
  }
);

function validateName(): boolean {
  if (!form.name.trim()) {
    errors.name = 'Name is required';
    return false;
  }
  if (form.name.length > 64) {
    errors.name = 'Name must be 64 characters or less';
    return false;
  }
  if (!/^[a-zA-Z0-9_-]+$/.test(form.name)) {
    errors.name = 'Name can only contain letters, numbers, hyphens, and underscores';
    return false;
  }
  delete errors.name;
  return true;
}

function validateEndpoint(): boolean {
  if (!form.endpoint.trim()) {
    errors.endpoint = 'Endpoint is required';
    return false;
  }
  delete errors.endpoint;
  return true;
}

function validatePort(): boolean {
  if (form.port !== undefined && form.port !== null) {
    if (form.port < 1 || form.port > 65535) {
      errors.port = 'Port must be between 1 and 65535';
      return false;
    }
  }
  delete errors.port;
  return true;
}

function validateConfig(): boolean {
  if (!form.config.trim()) {
    return true; // Optional
  }
  try {
    JSON.parse(form.config);
    delete errors.config;
    return true;
  } catch {
    errors.config = 'Invalid JSON format';
    return false;
  }
}

function validateAll(): boolean {
  return (
    validateName() &&
    validateEndpoint() &&
    validatePort() &&
    validateConfig()
  );
}

function parseTags(input: string): string[] {
  return input
    .split(',')
    .map(tag => tag.trim())
    .filter(tag => tag.length > 0);
}

function handleSubmit() {
  if (!validateAll()) return;

  const formData: MCPServiceFormData = {
    name: form.name.trim(),
    endpoint: form.endpoint.trim(),
    port: form.port,
    protocol: form.protocol,
    authType: form.authType,
    config: form.config.trim() || '{}',
    groupIds: form.groupIds,
    tags: parseTags(tagsInput.value),
  };

  emit('submit', formData);
}

async function handleTestConnection() {
  if (!validateEndpoint()) return;

  isTesting.value = true;
  testResult.value = null;

  const formData: MCPServiceFormData = {
    name: form.name.trim() || 'Test Connection',
    endpoint: form.endpoint.trim(),
    port: form.port,
    protocol: form.protocol,
    authType: form.authType,
    config: form.config.trim() || '{}',
    groupIds: form.groupIds,
    tags: parseTags(tagsInput.value),
  };

  emit('test-connection', formData);

  // Reset after a timeout (actual result should come from parent)
  setTimeout(() => {
    isTesting.value = false;
  }, 3000);
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
  max-width: 540px;
  max-height: 90vh;
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  display: flex;
  flex-direction: column;
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

.server-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
}

.form-group input,
.form-group select,
.form-group textarea {
  padding: 10px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 14px;
  color: var(--fg);
  transition: border-color 0.2s;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  outline: none;
  border-color: var(--accent);
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: var(--fg-muted);
}

.form-group select {
  cursor: pointer;
}

.form-group.has-error input,
.form-group.has-error select,
.form-group.has-error textarea {
  border-color: var(--error);
}

.error-text {
  font-size: 12px;
  color: var(--error);
}

.config-group .config-editor {
  position: relative;
}

.config-textarea {
  font-family: 'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace;
  font-size: 12px;
  resize: vertical;
  min-height: 120px;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
}

.test-result.success {
  background: var(--success-bg);
  color: var(--success);
}

.test-result.error {
  background: var(--error-bg);
  color: var(--error);
}

.test-icon {
  display: flex;
  align-items: center;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover, var(--accent));
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.btn-secondary:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}
</style>

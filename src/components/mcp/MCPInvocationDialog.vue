<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')">
    <div class="dialog" role="dialog" aria-modal="true" :aria-labelledby="`tool-dialog-${tool.name}`">
      <!-- Header -->
      <div class="dialog-header">
        <div class="header-info">
          <h3 :id="`tool-dialog-${tool.name}`">{{ tool.name }}</h3>
          <span class="tool-service">{{ service.name }}</span>
        </div>
        <button class="close-btn" @click="$emit('close')" aria-label="Close dialog">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Tool description -->
        <div v-if="tool.description" class="tool-description">
          <p>{{ tool.description }}</p>
        </div>

        <!-- Arguments Form -->
        <form @submit.prevent="handleInvoke" class="args-form">
          <div
            v-for="field in formFields"
            :key="field.name"
            class="form-group"
            :class="{ 'has-error': fieldErrors[field.name] }"
          >
            <label :for="`arg-${field.name}`">
              {{ field.label }}
              <span v-if="field.required" class="required">*</span>
            </label>

            <!-- Text input -->
            <input
              v-if="field.type === 'text'"
              :id="`arg-${field.name}`"
              v-model="args[field.name]"
              type="text"
              :placeholder="field.description || ''"
            />

            <!-- Number input -->
            <input
              v-else-if="field.type === 'number'"
              :id="`arg-${field.name}`"
              v-model.number="args[field.name]"
              type="number"
              :placeholder="field.description || ''"
            />

            <!-- Checkbox -->
            <label v-else-if="field.type === 'checkbox'" class="checkbox-label">
              <input
                :id="`arg-${field.name}`"
                v-model="args[field.name]"
                type="checkbox"
              />
              <span class="checkbox-custom"></span>
              <span class="checkbox-text">{{ field.description || 'Enable' }}</span>
            </label>

            <!-- Select -->
            <select
              v-else-if="field.type === 'select' && field.enum"
              :id="`arg-${field.name}`"
              v-model="args[field.name]"
            >
              <option value="">Select...</option>
              <option v-for="opt in field.enum" :key="opt" :value="opt">{{ opt }}</option>
            </select>

            <!-- Textarea for arrays/objects -->
            <textarea
              v-else
              :id="`arg-${field.name}`"
              v-model="args[field.name]"
              :placeholder="field.description || (field.type === 'textarea' ? 'Enter JSON...' : '')"
              :class="{ 'is-json': field.type === 'textarea' }"
              rows="3"
            ></textarea>

            <span v-if="fieldErrors[field.name]" class="error-text">{{ fieldErrors[field.name] }}</span>
          </div>

          <!-- Invoke button -->
          <button
            type="submit"
            class="btn btn-primary invoke-btn"
            :disabled="isInvoking"
          >
            <svg v-if="isInvoking" class="spinner" width="14" height="14" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="5 3 19 12 5 21 5 3"/>
            </svg>
            <span>{{ isInvoking ? 'Invoking...' : 'Invoke Tool' }}</span>
          </button>
        </form>

        <!-- Result Display -->
        <div v-if="error || result" class="result-section">
          <h4>Result</h4>

          <!-- Error display -->
          <div v-if="error" class="error-box">
            <span class="error-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
            </span>
            <pre class="error-message">{{ error }}</pre>
          </div>

          <!-- Result display -->
          <div v-else-if="result" class="result-box">
            <div class="result-meta">
              <span class="duration">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <polyline points="12 6 12 12 16 14"/>
                </svg>
                {{ result.durationMs }}ms
              </span>
              <button class="copy-btn" @click="copyResult" title="Copy to clipboard">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                </svg>
                <span>{{ copied ? 'Copied!' : 'Copy' }}</span>
              </button>
            </div>
            <div class="result-content">
              <pre><code>{{ formatResult(result) }}</code></pre>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">Close</button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue';
import { useMCPStore } from '@/stores/mcp';
import { schemaToFormFields } from '@/utils/mcp-schema-to-form';
import type { MCPService, MCPTool, MCPInvocationResult, FormField } from '@/types';

interface Props {
  service: MCPService;
  tool: MCPTool;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useMCPStore();

const args = reactive<Record<string, unknown>>({});
const fieldErrors = reactive<Record<string, string>>({});
const isInvoking = ref(false);
const result = ref<MCPInvocationResult | null>(null);
const error = ref<string | null>(null);
const copied = ref(false);

// Generate form fields from tool's input schema
const formFields = computed<FormField[]>(() => {
  if (!props.tool.inputSchema) return [];
  return schemaToFormFields(props.tool.inputSchema);
});

// Initialize args with defaults
formFields.value.forEach(field => {
  if (field.default !== undefined) {
    args[field.name] = field.default;
  }
});

function validateFields(): boolean {
  let valid = true;
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key]);

  formFields.value.forEach(field => {
    if (field.required && !args[field.name]) {
      fieldErrors[field.name] = `${field.label} is required`;
      valid = false;
    }

    // Try to parse JSON for textarea fields
    if (field.type === 'textarea' && args[field.name]) {
      try {
        const value = args[field.name];
        if (typeof value === 'string') {
          JSON.parse(value);
        }
      } catch {
        fieldErrors[field.name] = 'Invalid JSON format';
        valid = false;
      }
    }
  });

  return valid;
}

async function handleInvoke() {
  if (!validateFields()) return;

  isInvoking.value = true;
  error.value = null;
  result.value = null;

  // Parse JSON values for textarea fields
  const parsedArgs: Record<string, unknown> = {};
  formFields.value.forEach(field => {
    if (field.type === 'textarea' && args[field.name]) {
      try {
        parsedArgs[field.name] = JSON.parse(args[field.name] as string);
      } catch {
        parsedArgs[field.name] = args[field.name];
      }
    } else {
      parsedArgs[field.name] = args[field.name];
    }
  });

  try {
    result.value = await store.invokeTool(props.service.id, props.tool.name, parsedArgs);
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Invocation failed';
  } finally {
    isInvoking.value = false;
  }
}

function formatResult(invocationResult: MCPInvocationResult): string {
  if (invocationResult.content) {
    return invocationResult.content
      .map(block => {
        if (block.type === 'text' && block.text) {
          return block.text;
        }
        if (block.type === 'image' && block.data) {
          return `[Image: ${block.mimeType || 'image'}]`;
        }
        if (block.type === 'resource' && block.uri) {
          return `[Resource: ${block.uri}]`;
        }
        return JSON.stringify(block, null, 2);
      })
      .join('\n');
  }
  return JSON.stringify(invocationResult, null, 2);
}

async function copyResult() {
  if (!result.value) return;

  try {
    await navigator.clipboard.writeText(formatResult(result.value));
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    // Clipboard API failed
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
  width: 100%;
  max-width: 600px;
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
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
}

.tool-service {
  font-size: 12px;
  color: var(--fg-muted);
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

.tool-description {
  margin-bottom: 20px;
  padding: 12px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.tool-description p {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  margin: 0;
}

.args-form {
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

.required {
  color: var(--error);
}

.form-group input[type="text"],
.form-group input[type="number"],
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

.form-group textarea.is-json {
  font-family: 'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace;
  font-size: 12px;
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
  background: var(--bg-secondary);
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

.checkbox-text {
  font-size: 13px;
  color: var(--fg-muted);
}

.invoke-btn {
  margin-top: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
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

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.result-section {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border);
}

.result-section h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

.error-box {
  display: flex;
  gap: 10px;
  padding: 12px 14px;
  background: var(--error-bg);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
}

.error-icon {
  color: var(--error);
  flex-shrink: 0;
}

.error-message {
  font-size: 13px;
  color: var(--error);
  font-family: 'JetBrains Mono', monospace;
  white-space: pre-wrap;
  margin: 0;
}

.result-box {
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  overflow: hidden;
  box-shadow: var(--shadow-md);
}

.result-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.duration {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.copy-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: none;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

.result-content {
  padding: 14px;
  background: var(--bg-tertiary);
  max-height: 300px;
  overflow: auto;
}

.result-content pre {
  margin: 0;
}

.result-content code {
  font-family: 'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace;
  font-size: 12px;
  color: var(--fg);
  white-space: pre-wrap;
  word-break: break-word;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
</style>

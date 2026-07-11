<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="export-dialog-title">
      <!-- Header -->
      <div class="dialog-header">
        <h3 id="export-dialog-title">Export Mcps</h3>
        <button class="close-btn" aria-label="Close dialog" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Format Selector -->
        <div class="section">
          <h4>Export Format</h4>
          <div class="format-selector">
            <label class="radio-option">
              <input v-model="format" type="radio" value="json" />
              <span class="radio-custom"></span>
              <span class="radio-label">
                <strong>JSON</strong>
                <small>Full metadata, includes all fields</small>
              </span>
            </label>
            <label class="radio-option">
              <input v-model="format" type="radio" value="yaml" />
              <span class="radio-custom"></span>
              <span class="radio-label">
                <strong>YAML</strong>
                <small>Human-readable, concise format</small>
              </span>
            </label>
          </div>
        </div>

        <!-- Scope Selector -->
        <div class="section">
          <h4>Select Servers</h4>
          <div class="scope-selector">
            <label class="select-all">
              <input v-model="selectAll" type="checkbox" @change="toggleAll" />
              <span class="checkbox-custom"></span>
              <span>Select All ({{ services.length }})</span>
            </label>

            <div class="server-list">
              <label
                v-for="service in services"
                :key="service.id"
                class="checkbox-option"
              >
                <input
                  v-model="selectedIds"
                  type="checkbox"
                  :value="service.id"
                />
                <span class="checkbox-custom"></span>
                <span class="service-name">{{ service.name }}</span>
                <span class="service-protocol">{{ service.protocol.toUpperCase() }}</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Preview -->
        <div class="section preview-section">
          <h4>Preview</h4>
          <div class="preview-pane">
            <pre><code>{{ preview }}</code></pre>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn btn-primary"
          :disabled="selectedIds.length === 0 || isExporting"
          @click="handleExport"
        >
          <svg v-if="isExporting" class="spinner" width="14" height="14" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
          </svg>
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <span>{{ isExporting ? 'Exporting...' : `Download ${format.toUpperCase()}` }}</span>
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useMCPStore } from '@/stores/mcp';
import type { MCPService, MCPExportFormat } from '@/types';

interface Props {
  services: MCPService[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'export', data: string): void;
}>();

const store = useMCPStore();

const format = ref<MCPExportFormat>('json');
const selectedIds = ref<string[]>([]);
const selectAll = ref(true);
const isExporting = ref(false);

// Initialize with all selected
watch(
  () => props.services,
  (newServices) => {
    selectedIds.value = newServices.map(s => s.id);
  },
  { immediate: true }
);

function toggleAll() {
  if (selectAll.value) {
    selectedIds.value = props.services.map(s => s.id);
  } else {
    selectedIds.value = [];
  }
}

// Watch for individual checkbox changes
watch(selectedIds, (ids) => {
  selectAll.value = ids.length === props.services.length;
}, { deep: true });

// Generate preview
const preview = computed(() => {
  const selected = props.services.filter(s => selectedIds.value.includes(s.id));
  const exportData = {
    version: '1.0',
    exportedAt: new Date().toISOString(),
    services: selected.map(s => ({
      name: s.name,
      endpoint: s.endpoint,
      protocol: s.protocol,
      authType: s.authType,
      config: s.config ? JSON.parse(s.config) : {},
      groups: s.groupIds,
      tags: s.tags,
    })),
  };

  if (format.value === 'json') {
    return JSON.stringify(exportData, null, 2);
  }

  // Simple YAML preview
  return yamlPreview(exportData);
});

function yamlPreview(data: Record<string, unknown>): string {
  const lines: string[] = [];
  lines.push(`version: "${data.version}"`);
  lines.push(`exportedAt: "${data.exportedAt}"`);
  lines.push('services:');

  const services = data.services as Array<Record<string, unknown>>;
  services.forEach((service) => {
    const indent = '  ';

    lines.push(`${indent}- name: "${service.name}"`);
    lines.push(`${indent}  endpoint: "${service.endpoint}"`);
    lines.push(`${indent}  protocol: "${service.protocol}"`);
    lines.push(`${indent}  authType: "${service.authType}"`);

    if (service.config && Object.keys(service.config as object).length > 0) {
      lines.push(`${indent}  config:`);
      Object.entries(service.config as Record<string, unknown>).forEach(([key, value]) => {
        lines.push(`${indent}    ${key}: ${JSON.stringify(value)}`);
      });
    }

    if (service.tags && (service.tags as string[]).length > 0) {
      lines.push(`${indent}  tags:`);
      (service.tags as string[]).forEach(tag => {
        lines.push(`${indent}    - "${tag}"`);
      });
    }
  });

  return lines.join('\n');
}

async function handleExport() {
  if (selectedIds.value.length === 0) return;

  isExporting.value = true;

  try {
    const ids = selectAll.value ? null : selectedIds.value;
    const exportData = await store.exportServices(ids, format.value);

    // Create and trigger download
    const blob = new Blob([exportData], { type: format.value === 'json' ? 'application/json' : 'text/yaml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mcp-servers-export.${format.value}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    emit('export', exportData);
    emit('close');
  } catch (e) {
    console.error('Export failed:', e);
  } finally {
    isExporting.value = false;
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
  max-width: 640px;
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

.section {
  margin-bottom: 24px;
}

.section h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

.format-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  cursor: pointer;
  transition: all 0.2s;
}

.radio-option:hover {
  border-color: rgba(255, 255, 255, 0.50);
  background: var(--bg-card-hover);
}

.radio-option input {
  display: none;
}

.radio-custom {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border);
  border-radius: 50%;
  background: var(--bg-primary);
  flex-shrink: 0;
  transition: all 0.2s;
  position: relative;
}

.radio-option input:checked + .radio-custom {
  border-color: var(--accent);
}

.radio-option input:checked + .radio-custom::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 4px;
  width: 6px;
  height: 6px;
  background: var(--accent);
  border-radius: 50%;
}

.radio-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.radio-label strong {
  font-size: 14px;
  color: var(--fg);
}

.radio-label small {
  font-size: 12px;
  color: var(--fg-muted);
}

.scope-selector {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.select-all {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}

.select-all input,
.checkbox-option input {
  display: none;
}

.checkbox-custom {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  flex-shrink: 0;
  transition: all 0.2s;
  position: relative;
}

.select-all input:checked + .checkbox-custom,
.checkbox-option input:checked + .checkbox-custom {
  background: var(--accent);
  border-color: var(--accent);
}

.select-all input:checked + .checkbox-custom::after,
.checkbox-option input:checked + .checkbox-custom::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.server-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 200px;
  overflow-y: auto;
  padding: 4px;
}

.checkbox-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.checkbox-option:hover {
  background: var(--bg-secondary);
}

.service-name {
  flex: 1;
  font-size: 13px;
}

.service-protocol {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  color: var(--fg-muted);
  font-family: monospace;
}

.preview-section {
  margin-bottom: 0;
}

.preview-pane {
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  overflow: hidden;
  box-shadow: var(--shadow-md);
}

.preview-pane pre {
  padding: 14px;
  margin: 0;
  overflow-x: auto;
}

.preview-pane code {
  font-family: 'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace;
  font-size: 11px;
  color: var(--fg);
  white-space: pre;
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
  padding: 10px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
</style>

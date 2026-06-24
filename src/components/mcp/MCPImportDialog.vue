<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="handleClose" @keydown.escape="handleClose">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="import-dialog-title">
      <!-- Header -->
      <div class="dialog-header">
        <h3 id="import-dialog-title">Import Mcps</h3>
        <button class="close-btn" @click="handleClose" aria-label="Close dialog">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Step 1: Upload -->
        <div v-if="step === 'upload'" class="step upload-step">
          <div
            class="drop-zone"
            :class="{ 'drag-over': isDragOver, 'has-file': selectedFile }"
            @dragover.prevent="isDragOver = true"
            @dragleave.prevent="isDragOver = false"
            @drop.prevent="handleDrop"
            @click="openFilePicker"
          >
            <input
              ref="fileInput"
              type="file"
              accept=".json,.yaml,.yml"
              @change="handleFileSelect"
              hidden
            />

            <div v-if="!selectedFile" class="drop-content">
              <svg class="upload-icon" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              <p class="drop-title">Drag and drop your config file here</p>
              <span class="drop-hint">or click to browse</span>
              <p class="drop-formats">Supports JSON and YAML formats</p>
            </div>

            <div v-else class="file-info">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              <div class="file-details">
                <span class="file-name">{{ selectedFile.name }}</span>
                <span class="file-size">{{ formatFileSize(selectedFile.size) }}</span>
              </div>
              <button class="btn-icon" @click.stop="clearFile">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
          </div>

          <div class="upload-actions">
            <button class="btn btn-secondary" @click="handleClose">Cancel</button>
            <button
              class="btn btn-primary"
              :disabled="!selectedFile"
              @click="processFile"
            >
              Continue
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="9 18 15 12 9 6"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Step 2: Validate -->
        <div v-if="step === 'validate'" class="step validate-step">
          <div class="validation-summary">
            <div class="stat-card valid">
              <span class="stat-value">{{ importData.services.length }}</span>
              <span class="stat-label">Valid</span>
            </div>
            <div class="stat-card duplicate">
              <span class="stat-value">{{ importData.duplicates.length }}</span>
              <span class="stat-label">Duplicates</span>
            </div>
            <div class="stat-card error">
              <span class="stat-value">{{ importData.errors.length }}</span>
              <span class="stat-label">Errors</span>
            </div>
          </div>

          <!-- Validation errors -->
          <div v-if="importData.errors.length > 0" class="validation-errors">
            <h4>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              Validation Errors
            </h4>
            <div class="error-list">
              <div v-for="(err, i) in importData.errors" :key="i" class="error-item">
                {{ err }}
              </div>
            </div>
          </div>

          <!-- Duplicate services -->
          <div v-if="importData.duplicates.length > 0" class="duplicate-section">
            <h4>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="8" y="8" width="12" height="12" rx="2" ry="2"/>
                <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>
              </svg>
              Duplicate Services
            </h4>
            <div class="duplicate-list">
              <div v-for="dup in importData.duplicates" :key="dup.name" class="duplicate-item">
                <span class="service-name">{{ dup.name }}</span>
                <span class="duplicate-hint">Already exists</span>
              </div>
            </div>
          </div>

          <!-- Import mode selector -->
          <div v-if="importData.services.length > 0 || importData.duplicates.length > 0" class="import-mode">
            <label>Duplicate handling:</label>
            <div class="mode-options">
              <label class="radio-option" :class="{ selected: mode === 'skip' }">
                <input type="radio" v-model="mode" value="skip" />
                <span class="radio-custom"></span>
                <div class="mode-content">
                  <strong>Skip</strong>
                  <small>Skip services that already exist</small>
                </div>
              </label>
              <label class="radio-option" :class="{ selected: mode === 'overwrite' }">
                <input type="radio" v-model="mode" value="overwrite" />
                <span class="radio-custom"></span>
                <div class="mode-content">
                  <strong>Overwrite</strong>
                  <small>Replace existing services</small>
                </div>
              </label>
            </div>
          </div>

          <div class="validate-actions">
            <button class="btn btn-secondary" @click="step = 'upload'">Back</button>
            <button
              class="btn btn-primary"
              :disabled="importData.services.length === 0"
              @click="startImport"
            >
              Import {{ importData.services.length + (mode === 'overwrite' ? importData.duplicates.length : 0) }} Servers
            </button>
          </div>
        </div>

        <!-- Step 3: Importing -->
        <div v-if="step === 'importing'" class="step importing-step">
          <div class="progress-container">
            <div class="progress-header">
              <span class="progress-label">Importing services...</span>
              <span class="progress-percent">{{ progress }}%</span>
            </div>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
            </div>
            <p class="progress-message">{{ progressMessage }}</p>
          </div>
        </div>

        <!-- Step 4: Complete -->
        <div v-if="step === 'complete'" class="step complete-step">
          <div class="result-summary">
            <div class="result-icon success">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
            <h4>Import Complete!</h4>

            <div class="result-stats">
              <div class="result-stat">
                <span class="stat-value">{{ importResult.imported }}</span>
                <span class="stat-label">Imported</span>
              </div>
              <div v-if="importResult.overwritten > 0" class="result-stat">
                <span class="stat-value">{{ importResult.overwritten }}</span>
                <span class="stat-label">Overwritten</span>
              </div>
              <div v-if="importResult.skipped > 0" class="result-stat">
                <span class="stat-value">{{ importResult.skipped }}</span>
                <span class="stat-label">Skipped</span>
              </div>
            </div>

            <div v-if="importResult.errors.length > 0" class="import-errors">
              <h5>Errors:</h5>
              <div v-for="(err, i) in importResult.errors" :key="i" class="error-item">
                {{ err }}
              </div>
            </div>
          </div>

          <div class="complete-actions">
            <button class="btn btn-primary" @click="handleClose">Done</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { useMCPStore } from '@/stores/mcp';
import type { MCPService, MCPImportResult } from '@/types';

interface Props {
  existingServices: MCPService[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'import', result: MCPImportResult): void;
}>();

const store = useMCPStore();

type ImportDialogStep = 'upload' | 'validate' | 'importing' | 'complete';

const step = ref<ImportDialogStep>('upload');
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const isDragOver = ref(false);
const mode = ref<'skip' | 'overwrite'>('skip');
const progress = ref(0);
const progressMessage = ref('');

const importData = reactive<{
  services: MCPService[];
  errors: string[];
  duplicates: MCPService[];
}>({
  services: [],
  errors: [],
  duplicates: [],
});

const importResult = reactive<MCPImportResult>({
  imported: 0,
  skipped: 0,
  overwritten: 0,
  errors: [],
});

function handleClose() {
  if (step.value === 'importing') return; // Don't close during import
  emit('close');
}

function openFilePicker() {
  fileInput.value?.click();
}

function handleDrop(e: DragEvent) {
  isDragOver.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) {
    selectedFile.value = file;
  }
}

function handleFileSelect(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    selectedFile.value = file;
  }
}

function clearFile() {
  selectedFile.value = null;
  if (fileInput.value) {
    fileInput.value.value = '';
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

async function processFile() {
  if (!selectedFile.value) return;

  const text = await selectedFile.value.text();
  let data: Record<string, unknown>;

  try {
    // Try JSON first
    if (selectedFile.value.name.endsWith('.json')) {
      data = JSON.parse(text);
    } else {
      // Simple YAML parsing (for basic structure)
      data = parseYaml(text);
    }
  } catch (e) {
    importData.errors.push(`Failed to parse file: ${e instanceof Error ? e.message : 'Unknown error'}`);
    step.value = 'validate';
    return;
  }

  // Validate and categorize
  importData.services = [];
  importData.errors = [];
  importData.duplicates = [];

  if (!Array.isArray(data?.services) && !Array.isArray(data)) {
    importData.errors.push('Invalid format: "services" array not found');
    step.value = 'validate';
    return;
  }

  const servicesArray = Array.isArray(data) ? data : data.services;

  for (const item of servicesArray) {
    if (!item.name || typeof item.name !== 'string') {
      importData.errors.push(`Entry missing or invalid "name" field`);
      continue;
    }

    if (!item.endpoint) {
      importData.errors.push(`Entry "${item.name}": missing "endpoint"`);
      continue;
    }

    // Check for duplicates
    if (props.existingServices.some(s => s.name === item.name)) {
      importData.duplicates.push(item as MCPService);
      continue;
    }

    importData.services.push(normalizeImportService(item));
  }

  step.value = 'validate';
}

function parseYaml(text: string): Record<string, unknown> {
  // Simple YAML to JSON conversion for basic export format
  const lines = text.split('\n');
  const result: Record<string, unknown> = { services: [] };
  let currentService: Record<string, unknown> | null = null;
  let inServices = false;

  for (const line of lines) {
    const trimmed = line.trim();

    if (trimmed === 'services:' || trimmed === 'services |') {
      inServices = true;
      continue;
    }

    if (inServices && trimmed.startsWith('- ')) {
      if (currentService) {
        (result.services as Record<string, unknown>[]).push(currentService);
      }
      currentService = {};
      const keyValue = trimmed.slice(2).split(': ');
      if (keyValue.length >= 2) {
        currentService[keyValue[0]] = keyValue.slice(1).join(': ').replace(/^["']|["']$/g, '');
      }
    } else if (currentService && trimmed.includes(':')) {
      const colonIndex = trimmed.indexOf(':');
      const key = trimmed.slice(0, colonIndex).trim();
      let value = trimmed.slice(colonIndex + 1).trim().replace(/^["']|["']$/g, '');

      // Try to parse numbers
      if (/^\d+$/.test(value)) {
        currentService[key] = parseInt(value, 10);
      } else if (value === 'true') {
        currentService[key] = true;
      } else if (value === 'false') {
        currentService[key] = false;
      } else {
        currentService[key] = value;
      }
    }
  }

  if (currentService) {
    (result.services as Record<string, unknown>[]).push(currentService);
  }

  return result;
}

function normalizeImportService(item: Record<string, unknown>): MCPService {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    softwareId: '',
    name: item.name as string,
    endpoint: item.endpoint as string,
    protocol: (item.protocol as 'http' | 'sse' | 'stdio') || 'http',
    authType: (item.authType as 'none' | 'bearer' | 'api-key') || 'none',
    config: item.config ? JSON.stringify(item.config) : '{}',
    groupIds: (item.groups as string[]) || [],
    tags: (item.tags as string[]) || [],
    isHealthy: false,
    lastChecked: now,
    createdAt: now,
    updatedAt: now,
  };
}

async function startImport() {
  step.value = 'importing';
  progress.value = 0;
  progressMessage.value = 'Preparing import...';

  // Simulate progress for UX
  const progressInterval = setInterval(() => {
    if (progress.value < 90) {
      progress.value += Math.random() * 15;
    }
  }, 200);

  try {
    // Create import data for backend
    const importPayload = JSON.stringify({
      version: '1.0',
      services: [
        ...importData.services,
        ...(mode.value === 'overwrite' ? importData.duplicates : []),
      ],
    });

    const result = await store.importServices(importPayload, mode.value);

    importResult.imported = result.imported;
    importResult.skipped = result.skipped;
    importResult.overwritten = result.overwritten;
    importResult.errors = result.errors;

    clearInterval(progressInterval);
    progress.value = 100;
    progressMessage.value = 'Import complete!';

    setTimeout(() => {
      step.value = 'complete';
      emit('import', result);
    }, 500);
  } catch (e) {
    clearInterval(progressInterval);
    importResult.errors.push(e instanceof Error ? e.message : 'Import failed');
    step.value = 'complete';
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

.step {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* Upload Step */
.drop-zone {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  border: 2px dashed rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: var(--shadow-md);
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.drop-zone.has-file {
  border-style: solid;
}

.drop-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px;
}

.upload-icon {
  color: var(--fg-muted);
  margin-bottom: 16px;
}

.drop-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
  margin: 0 0 4px;
}

.drop-hint {
  font-size: 13px;
  color: var(--fg-muted);
}

.drop-formats {
  font-size: 12px;
  color: var(--fg-muted);
  margin: 12px 0 0;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
}

.file-info svg {
  color: var(--accent);
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
}

.file-size {
  font-size: 12px;
  color: var(--fg-muted);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.btn-icon:hover {
  background: var(--bg-tertiary);
  color: var(--error);
}

.upload-actions,
.validate-actions,
.complete-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 8px;
}

/* Validate Step */
.validation-summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--fg);
}

.stat-label {
  font-size: 11px;
  color: var(--fg-muted);
  margin-top: 4px;
}

.stat-card.valid .stat-value { color: var(--success); }
.stat-card.duplicate .stat-value { color: var(--warn); }
.stat-card.error .stat-value { color: var(--error); }

.validation-errors,
.duplicate-section {
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 14px;
  box-shadow: var(--shadow-md);
}

.validation-errors h4,
.duplicate-section h4 {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 10px;
}

.validation-errors h4 { color: var(--error); }
.duplicate-section h4 { color: var(--warn); }

.error-list,
.duplicate-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 120px;
  overflow-y: auto;
}

.error-item {
  font-size: 12px;
  color: var(--fg-muted);
  padding: 6px 10px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.duplicate-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
}

.duplicate-hint {
  font-size: 11px;
  color: var(--fg-muted);
}

.import-mode {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.import-mode > label {
  font-size: 13px;
  font-weight: 500;
}

.mode-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-option {
  display: flex;
  align-items: center;
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

.radio-option.selected {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.radio-option input {
  display: none;
}

.radio-custom {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-radius: 50%;
  flex-shrink: 0;
  position: relative;
}

.radio-option.selected .radio-custom {
  border-color: var(--accent);
}

.radio-option.selected .radio-custom::after {
  content: '';
  position: absolute;
  left: 3px;
  top: 3px;
  width: 6px;
  height: 6px;
  background: var(--accent);
  border-radius: 50%;
}

.mode-content {
  display: flex;
  flex-direction: column;
}

.mode-content strong {
  font-size: 13px;
}

.mode-content small {
  font-size: 11px;
  color: var(--fg-muted);
}

/* Importing Step */
.progress-container {
  padding: 24px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.progress-label {
  font-size: 13px;
  color: var(--fg);
}

.progress-percent {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
}

.progress-bar {
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-message {
  font-size: 12px;
  color: var(--fg-muted);
  margin: 10px 0 0;
  text-align: center;
}

/* Complete Step */
.result-summary {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px;
}

.result-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  margin-bottom: 16px;
}

.result-icon.success {
  background: var(--success-bg);
  color: var(--success);
}

.result-summary h4 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 20px;
}

.result-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 16px;
}

.result-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.result-stat .stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--success);
}

.result-stat .stat-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.import-errors {
  margin-top: 16px;
  padding: 12px;
  background: var(--error-bg);
  border-radius: 8px;
  width: 100%;
  text-align: left;
}

.import-errors h5 {
  font-size: 12px;
  font-weight: 600;
  color: var(--error);
  margin: 0 0 8px;
}

.import-errors .error-item {
  background: var(--error-bg);
}

/* Buttons */
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
</style>

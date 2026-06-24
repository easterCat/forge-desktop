<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>从 ZIP 安装技能</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Drop Zone / File Select -->
        <div 
          class="drop-zone"
          :class="{ active: isDragging, disabled: isInstalling }"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          @drop.prevent="handleDrop"
          @click="selectFile"
        >
          <div v-if="isInstalling" class="installing-state">
            <div class="spinner"></div>
            <p>正在安装技能...</p>
            <span class="progress-text">{{ installProgress }}%</span>
          </div>
          <div v-else-if="selectedFile" class="selected-file">
            <div class="file-icon">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="12" y1="18" x2="12" y2="12"/>
                <line x1="9" y1="15" x2="15" y2="15"/>
              </svg>
            </div>
            <div class="file-info">
              <span class="file-name">{{ fileName }}</span>
              <span class="file-size">{{ fileSize }}</span>
            </div>
            <button class="clear-btn" @click.stop="clearSelection">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <div v-else class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
            <p>点击选择 ZIP 文件或拖拽到此处</p>
            <span>支持 .zip 格式的技能包</span>
          </div>
        </div>

        <!-- Target Directory -->
        <div class="form-group">
          <label>安装到</label>
          <div class="path-input">
            <input
              type="text"
              v-model="targetDir"
              placeholder="选择安装目录..."
              readonly
            />
            <button class="btn btn-secondary btn-sm" @click="browseTargetDir">
              浏览
            </button>
          </div>
        </div>

        <!-- Result Message -->
        <div v-if="resultMessage" class="result-message" :class="resultType">
          <svg v-if="resultType === 'success'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          {{ resultMessage }}
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          取消
        </button>
        <button
          class="btn btn-primary"
          :disabled="!selectedFile || !targetDir || isInstalling"
          @click="handleInstall"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          安装
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useSkillImportStore } from '@/stores/skill-import';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'installed', skillName: string): void;
}>();

const store = useSkillImportStore();

const isDragging = ref(false);
const selectedFile = ref<string | null>(null);
const targetDir = ref('');
const isInstalling = ref(false);
const installProgress = ref(0);
const resultMessage = ref('');
const resultType = ref<'success' | 'error'>('success');

const fileName = computed(() => {
  if (!selectedFile.value) return '';
  const parts = selectedFile.value.split(/[/\\]/);
  return parts[parts.length - 1];
});

const fileSize = computed(() => {
  // This would be populated from the actual file
  return '';
});

async function selectFile() {
  if (isInstalling.value) return;
  
  const selected = await open({
    multiple: false,
    filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
  });
  
  if (selected) {
    selectedFile.value = selected as string;
    resultMessage.value = '';
  }
}

function handleDrop(event: DragEvent) {
  if (isInstalling.value) return;
  
  isDragging.value = false;
  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.name.endsWith('.zip')) {
      // Note: In Tauri, we can't directly get the path from drag-drop
      // The user needs to use the file picker
      resultMessage.value = '请使用文件选择器选择 ZIP 文件';
      resultType.value = 'error';
    }
  }
}

function clearSelection() {
  selectedFile.value = null;
  resultMessage.value = '';
}

async function browseTargetDir() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  
  if (selected) {
    targetDir.value = selected as string;
  }
}

async function handleInstall() {
  if (!selectedFile.value || !targetDir.value) return;
  
  try {
    isInstalling.value = true;
    installProgress.value = 0;
    resultMessage.value = '';
    
    // Simulate progress
    const progressInterval = setInterval(() => {
      if (installProgress.value < 90) {
        installProgress.value += 10;
      }
    }, 200);
    
    const result = await store.installFromZip(selectedFile.value, targetDir.value);
    
    clearInterval(progressInterval);
    installProgress.value = 100;
    
    if (result.success) {
      resultMessage.value = result.message;
      resultType.value = 'success';
      emit('installed', result.skill_name);
      
      // Auto close after success
      setTimeout(() => {
        emit('close');
      }, 1500);
    } else {
      resultMessage.value = result.message;
      resultType.value = 'error';
    }
  } catch (e) {
    resultMessage.value = e instanceof Error ? e.message : '安装失败';
    resultType.value = 'error';
  } finally {
    isInstalling.value = false;
  }
}

// Initialize default directory
onMounted(async () => {
  targetDir.value = await store.initDefaultSkillsDir();
});
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  position: relative;
  width: 100%;
  max-width: 480px;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  animation: slideIn 0.2s ease;
  z-index: 1;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
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
  padding: 20px;
}

.drop-zone {
  border: 2px dashed var(--border);
  border-radius: 12px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--bg-secondary);
  margin-bottom: 16px;
}

.drop-zone:hover:not(.disabled) {
  border-color: var(--accent);
  background: var(--bg-tertiary);
}

.drop-zone.active {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.drop-zone.disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.drop-zone .empty-state {
  color: var(--fg-muted);
}

.drop-zone .empty-state svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.drop-zone .empty-state p {
  font-size: 14px;
  margin-bottom: 4px;
}

.drop-zone .empty-state span {
  font-size: 12px;
}

.selected-file {
  display: flex;
  align-items: center;
  gap: 12px;
  text-align: left;
}

.file-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-bg);
  border-radius: 8px;
  color: var(--accent);
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  display: block;
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  display: block;
  font-size: 12px;
  color: var(--fg-muted);
  margin-top: 2px;
}

.clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.clear-btn:hover {
  background: var(--error);
  color: white;
}

.installing-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.progress-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--accent);
}

.form-group {
  margin-top: 16px;
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

.result-message {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 12px;
  border-radius: 8px;
  font-size: 13px;
}

.result-message.success {
  background: var(--success-bg);
  color: var(--success);
}

.result-message.error {
  background: var(--error-bg);
  color: var(--error);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
</style>

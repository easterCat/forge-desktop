<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>导入本地已有技能</h3>
        <button class="close-btn" aria-label="关闭" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Source Path Selection -->
        <div class="form-group">
          <label>扫描目录</label>
          <div class="path-select">
            <select v-model="selectedPath" :disabled="isScanning" @change="handlePathChange">
              <option value="">-- 选择目录 --</option>
              <option v-for="path in detectedPaths" :key="path" :value="path">
                {{ formatPath(path) }}
              </option>
              <option value="custom">自定义目录...</option>
            </select>
            <button class="btn btn-secondary btn-sm" :disabled="!selectedPath || isScanning" @click="scanDirectory">
              <svg v-if="isScanning" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
              </svg>
              {{ isScanning ? '扫描中...' : '扫描' }}
            </button>
          </div>
          <span v-if="detectedPaths.length === 0" class="hint">
            未检测到 Agent CLI skills 目录，请手动选择
          </span>
        </div>

        <!-- Skills List -->
        <div v-if="localSkills.length > 0" class="skills-list">
          <div class="list-header">
            <label class="checkbox-label">
              <input v-model="selectAll" type="checkbox" @change="handleSelectAll" />
              <span class="checkbox-custom"></span>
              <span>全选</span>
            </label>
            <span class="selected-count">已选择 {{ selectedSkills.length }} 项</span>
          </div>

          <div class="skills-items">
            <div 
              v-for="skill in localSkills" 
              :key="skill.path"
              class="skill-item"
              :class="{ selected: isSelected(skill.path) }"
              @click="toggleSelection(skill.path)"
            >
              <label class="checkbox-label">
                <input 
                  type="checkbox" 
                  :checked="isSelected(skill.path)"
                  @click.stop
                  @change="toggleSelection(skill.path)"
                />
                <span class="checkbox-custom"></span>
              </label>
              <div class="skill-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                </svg>
              </div>
              <div class="skill-info">
                <span class="skill-name">{{ skill.name }}</span>
                <span v-if="skill.description" class="skill-desc">{{ skill.description }}</span>
                <span v-else class="skill-desc skill-no-md">无描述文件</span>
              </div>
              <span v-if="skill.has_skill_md" class="has-md-badge">有效</span>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else-if="hasScanned && !isScanning" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
          </svg>
          <p>未找到技能</p>
          <span>该目录下没有找到有效的技能包</span>
        </div>

        <!-- Import Method -->
        <div v-if="selectedSkills.length > 0" class="form-group">
          <label>导入方式</label>
          <div class="import-methods">
            <label class="method-option" :class="{ active: importMethod === 'copy' }">
              <input v-model="importMethod" type="radio" value="copy" />
              <div class="method-content">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                </svg>
                <span class="method-name">复制</span>
                <span class="method-desc">完整复制文件</span>
              </div>
            </label>
            <label class="method-option" :class="{ active: importMethod === 'symlink' }">
              <input v-model="importMethod" type="radio" value="symlink" />
              <div class="method-content">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/>
                  <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/>
                </svg>
                <span class="method-name">符号链接</span>
                <span class="method-desc">创建快捷方式</span>
              </div>
            </label>
          </div>
        </div>

        <!-- Target Directory -->
        <div v-if="selectedSkills.length > 0" class="form-group">
          <label>安装到</label>
          <div class="path-input">
            <input
              v-model="targetDir"
              type="text"
              placeholder="选择安装目录..."
              readonly
            />
            <button class="btn btn-secondary btn-sm" @click="browseTargetDir">
              浏览
            </button>
          </div>
        </div>

        <!-- Progress -->
        <div v-if="isImporting" class="import-progress">
          <div class="progress-info">
            <span>{{ importStatus }}</span>
            <span>{{ importProgress }}/{{ selectedSkills.length }}</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: (importProgress / selectedSkills.length * 100) + '%' }"></div>
          </div>
        </div>

        <!-- Results -->
        <div v-if="importResults.length > 0" class="import-results">
          <div class="results-summary">
            <span class="success-count">{{ successCount }} 成功</span>
            <span v-if="failCount > 0" class="fail-count">{{ failCount }} 失败</span>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          取消
        </button>
        <button
          class="btn btn-primary"
          :disabled="selectedSkills.length === 0 || !targetDir || isImporting"
          @click="handleImport"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          导入选中 ({{ selectedSkills.length }})
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
import type { LocalSkill, ImportResult } from '@/types';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'imported', count: number): void;
}>();

const store = useSkillImportStore();

// State
const detectedPaths = ref<string[]>([]);
const selectedPath = ref('');
const localSkills = ref<LocalSkill[]>([]);
const selectedSkills = ref<string[]>([]);
const selectAll = ref(false);
const hasScanned = ref(false);
const isScanning = ref(false);
const isImporting = ref(false);
const importMethod = ref<'copy' | 'symlink'>('copy');
const targetDir = ref('');
const importProgress = ref(0);
const importStatus = ref('');
const importResults = ref<ImportResult[]>([]);

// Computed
const successCount = computed(() => importResults.value.filter(r => r.success).length);
const failCount = computed(() => importResults.value.filter(r => !r.success).length);

// Methods
function formatPath(path: string): string {
  // Use Tauri path module or fallback to common home directory patterns
  const home = path.includes('/.cursor/') || path.includes('/.agents/') 
    ? (path.startsWith('/Users/') ? '/Users/' + path.split('/')[2] : 
       path.startsWith('/home/') ? '/home/' + path.split('/')[2] : '')
    : '';
  if (home && path.startsWith(home)) {
    return '~' + path.slice(home.length);
  }
  return path;
}

async function handlePathChange() {
  if (selectedPath.value === 'custom') {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected) {
      selectedPath.value = selected as string;
    } else {
      selectedPath.value = '';
    }
  }
  selectedSkills.value = [];
  localSkills.value = [];
  hasScanned.value = false;
}

async function scanDirectory() {
  if (!selectedPath.value) return;
  
  try {
    isScanning.value = true;
    const skills = await store.scanLocalSkills(selectedPath.value);
    localSkills.value = skills;
    hasScanned.value = true;
  } catch (e) {
    console.error('Scan failed:', e);
  } finally {
    isScanning.value = false;
  }
}

function handleSelectAll() {
  if (selectAll.value) {
    selectedSkills.value = localSkills.value.map(s => s.path);
  } else {
    selectedSkills.value = [];
  }
}

function isSelected(path: string): boolean {
  return selectedSkills.value.includes(path);
}

function toggleSelection(path: string) {
  const index = selectedSkills.value.indexOf(path);
  if (index === -1) {
    selectedSkills.value.push(path);
  } else {
    selectedSkills.value.splice(index, 1);
  }
  
  // Update selectAll
  selectAll.value = selectedSkills.value.length === localSkills.value.length;
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

async function handleImport() {
  if (selectedSkills.value.length === 0 || !targetDir.value) return;
  
  try {
    isImporting.value = true;
    importResults.value = [];
    importProgress.value = 0;
    
    for (let i = 0; i < selectedSkills.value.length; i++) {
      const sourcePath = selectedSkills.value[i];
      importStatus.value = `正在导入: ${sourcePath.split('/').pop()}`;
      importProgress.value = i + 1;
      
      const result = await store.importLocalSkill(sourcePath, targetDir.value, importMethod.value);
      importResults.value.push(result);
    }
    
    importProgress.value = selectedSkills.value.length;
    emit('imported', successCount.value);
    
    // Close after short delay
    setTimeout(() => {
      emit('close');
    }, 1500);
  } catch (e) {
    console.error('Import failed:', e);
  } finally {
    isImporting.value = false;
  }
}

// Lifecycle
onMounted(async () => {
  try {
    // Initialize default skills directory
    targetDir.value = await store.initDefaultSkillsDir();
    
    // Detect CLI skills paths
    const paths = await store.detectCliSkillsPaths();
    detectedPaths.value = paths;
    if (paths.length > 0) {
      selectedPath.value = paths[0];
    }
  } catch (e) {
    console.error('Failed to initialize:', e);
  }
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
  max-width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
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
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--fg);
}

.path-select {
  display: flex;
  gap: 8px;
}

.path-select select {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
  cursor: pointer;
}

.hint {
  display: block;
  margin-top: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.skills-list {
  margin: 16px 0;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
  margin-bottom: 8px;
}

.selected-count {
  font-size: 12px;
  color: var(--fg-muted);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
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

.skills-items {
  max-height: 240px;
  overflow-y: auto;
}

.skill-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.skill-item:hover {
  background: var(--bg-secondary);
}

.skill-item.selected {
  background: var(--accent-bg);
}

.skill-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border-radius: 6px;
  color: var(--accent);
  flex-shrink: 0;
}

.skill-info {
  flex: 1;
  min-width: 0;
}

.skill-name {
  display: block;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.skill-desc {
  display: block;
  font-size: 11px;
  color: var(--fg-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.skill-no-md {
  font-style: italic;
}

.has-md-badge {
  padding: 2px 6px;
  font-size: 10px;
  background: var(--success-bg);
  color: var(--success);
  border-radius: 4px;
  flex-shrink: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: var(--fg-muted);
  text-align: center;
}

.empty-state svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 14px;
  margin-bottom: 4px;
}

.empty-state span {
  font-size: 12px;
}

.import-methods {
  display: flex;
  gap: 12px;
}

.method-option {
  flex: 1;
  cursor: pointer;
}

.method-option input {
  display: none;
}

.method-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 16px;
  background: var(--bg-secondary);
  border: 2px solid var(--border);
  border-radius: 10px;
  transition: all 0.2s;
}

.method-option.active .method-content {
  border-color: var(--accent);
  background: var(--accent-bg);
}

.method-content svg {
  color: var(--fg-muted);
}

.method-option.active .method-content svg {
  color: var(--accent);
}

.method-name {
  font-size: 13px;
  font-weight: 500;
}

.method-desc {
  font-size: 11px;
  color: var(--fg-muted);
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

.import-progress {
  margin-top: 16px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--fg-muted);
}

.progress-bar {
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.import-results {
  margin-top: 12px;
}

.results-summary {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.success-count {
  color: var(--success);
}

.fail-count {
  color: var(--error);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>

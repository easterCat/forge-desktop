<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>仓库管理</h3>
        <button class="close-btn" @click="$emit('close')" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Add Repository Form -->
        <div class="add-repository-form" v-if="!isAdding">
          <button class="btn btn-secondary" @click="isAdding = true">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            添加仓库
          </button>
        </div>

        <div class="add-form" v-else>
          <div class="form-row">
            <input
              type="text"
              v-model="newRepoUrl"
              placeholder="输入 Git 仓库地址..."
              class="repo-input"
              @keyup.enter="handleValidate"
            />
            <button 
              class="btn btn-secondary" 
              @click="handleValidate"
              :disabled="!newRepoUrl || isValidating"
            >
              <svg v-if="isValidating" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56"/>
              </svg>
              {{ isValidating ? '验证中...' : '验证' }}
            </button>
          </div>

          <!-- Validation Result -->
          <div v-if="validationResult" class="validation-result" :class="{ valid: validationResult.valid, invalid: !validationResult.valid }">
            <template v-if="validationResult.valid">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
                <polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
              <div class="validation-info">
                <span class="repo-name">{{ validationResult.name }}</span>
                <span class="branch-count">{{ validationResult.branch_count }} 个分支</span>
              </div>
              </Teleport>
</template>
            <template v-else>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="15" y1="9" x2="9" y2="15"/>
                <line x1="9" y1="9" x2="15" y2="15"/>
              </svg>
              <span>{{ validationResult.error_message }}</span>
              </Teleport>
</template>
          </div>

          <div class="form-actions">
            <button class="btn btn-secondary" @click="cancelAdd">取消</button>
            <button 
              class="btn btn-primary" 
              @click="handleAdd"
              :disabled="!validationResult?.valid"
            >
              添加
            </button>
          </div>
        </div>

        <!-- Repository List -->
        <div class="repository-list">
          <div v-if="repositories.length === 0 && !isLoading" class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
            </svg>
            <p>暂无仓库</p>
            <span>添加一个 Git 仓库以发现更多技能</span>
          </div>

          <div 
            v-for="repo in repositories" 
            :key="repo.id"
            class="repository-item"
          >
            <div class="repo-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
              </svg>
            </div>
            <div class="repo-info">
              <div class="repo-header">
                <span class="repo-name">{{ repo.name }}</span>
                <span class="repo-status" :class="repo.status">
                  {{ statusText(repo.status) }}
                </span>
              </div>
              <span class="repo-url">{{ repo.url }}</span>
              <div class="repo-meta">
                <span v-if="repo.skill_count > 0">{{ repo.skill_count }} 个技能</span>
                <span v-if="repo.last_sync_at">上次同步: {{ formatTime(repo.last_sync_at) }}</span>
              </div>
              <div v-if="repo.error_message" class="repo-error">
                {{ repo.error_message }}
              </div>
            </div>
            <div class="repo-actions">
              <button 
                class="btn-icon" 
                @click="handleSync(repo.id)"
                :disabled="isSyncing"
                :title="'同步仓库'"
              >
                <svg 
                  class="sync-icon" 
                  :class="{ spinning: syncingRepoId === repo.id }"
                  width="14" 
                  height="14" 
                  viewBox="0 0 24 24" 
                  fill="none" 
                  stroke="currentColor" 
                  stroke-width="2"
                >
                  <polyline points="23 4 23 10 17 10"/>
                  <polyline points="1 20 1 14 7 14"/>
                  <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
                </svg>
              </button>
              <button 
                class="btn-icon btn-danger" 
                @click="handleRemove(repo.id)"
                :title="'删除仓库'"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6"/>
                  <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          关闭
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useSkillImportStore } from '@/stores/skill-import';
import type { SkillRepository, RepositoryValidation } from '@/types';
import { confirm } from '@/utils/dialog';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'added', repo: SkillRepository): void;
  (e: 'removed', repoId: string): void;
}>();

const store = useSkillImportStore();

// State
const repositories = ref<SkillRepository[]>([]);
const isLoading = ref(false);
const isAdding = ref(false);
const isValidating = ref(false);
const isSyncing = ref(false);
const syncingRepoId = ref<string | null>(null);
const newRepoUrl = ref('');
const validationResult = ref<RepositoryValidation | null>(null);

// Lifecycle
onMounted(async () => {
  await fetchRepositories();
});

// Methods
async function fetchRepositories() {
  try {
    isLoading.value = true;
    repositories.value = await store.fetchRepositories();
  } catch (e) {
    console.error('Failed to fetch repositories:', e);
  } finally {
    isLoading.value = false;
  }
}

async function handleValidate() {
  if (!newRepoUrl.value) return;
  
  try {
    isValidating.value = true;
    validationResult.value = null;
    validationResult.value = await store.validateRepository(newRepoUrl.value);
  } catch (e) {
    console.error('Validation failed:', e);
  } finally {
    isValidating.value = false;
  }
}

async function handleAdd() {
  if (!validationResult.value?.valid) return;
  
  try {
    const repo = await store.addRepository(newRepoUrl.value, validationResult.value.name);
    repositories.value.push(repo);
    emit('added', repo);
    cancelAdd();
  } catch (e) {
    console.error('Failed to add repository:', e);
  }
}

function cancelAdd() {
  isAdding.value = false;
  newRepoUrl.value = '';
  validationResult.value = null;
}

async function handleSync(repoId: string) {
  try {
    syncingRepoId.value = repoId;
    isSyncing.value = true;
    await store.syncRepository(repoId);
    await fetchRepositories();
  } catch (e) {
    console.error('Sync failed:', e);
  } finally {
    syncingRepoId.value = null;
    isSyncing.value = false;
  }
}

async function handleRemove(repoId: string) {
  if (!await confirm('确定要删除此仓库吗？')) return;

  try {
    await store.removeRepository(repoId);
    repositories.value = repositories.value.filter(r => r.id !== repoId);
    emit('removed', repoId);
  } catch (e) {
    console.error('Failed to remove repository:', e);
  }
}

function statusText(status: string): string {
  const map: Record<string, string> = {
    pending: '待同步',
    syncing: '同步中',
    synced: '已同步',
    error: '失败',
  };
  return map[status] || status;
}

function formatTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
    return date.toLocaleDateString();
  } catch {
    return '';
  }
}
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
  width: 100%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  animation: slideIn 0.2s ease;
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

.add-repository-form {
  margin-bottom: 16px;
}

.add-form {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 10px;
  margin-bottom: 16px;
}

.form-row {
  display: flex;
  gap: 8px;
}

.repo-input {
  flex: 1;
  padding: 10px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
}

.repo-input:focus {
  outline: none;
  border-color: var(--accent);
}

.validation-result {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 13px;
}

.validation-result.valid {
  background: var(--success-bg);
  color: var(--success);
}

.validation-result.invalid {
  background: var(--error-bg);
  color: var(--error);
}

.validation-info {
  display: flex;
  flex-direction: column;
}

.validation-info .repo-name {
  font-weight: 500;
}

.validation-info .branch-count {
  font-size: 11px;
  opacity: 0.8;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

.repository-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.repository-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  background: var(--bg-secondary);
  border-radius: 10px;
  transition: all 0.15s;
}

.repository-item:hover {
  background: var(--bg-tertiary);
}

.repo-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--accent);
  flex-shrink: 0;
}

.repo-info {
  flex: 1;
  min-width: 0;
}

.repo-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.repo-name {
  font-size: 14px;
  font-weight: 500;
}

.repo-status {
  padding: 2px 6px;
  font-size: 10px;
  border-radius: 4px;
  text-transform: uppercase;
}

.repo-status.pending {
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.repo-status.syncing {
  background: var(--warn-bg);
  color: var(--warn);
}

.repo-status.synced {
  background: var(--success-bg);
  color: var(--success);
}

.repo-status.error {
  background: var(--error-bg);
  color: var(--error);
}

.repo-url {
  display: block;
  font-size: 12px;
  color: var(--fg-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.repo-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--fg-muted);
}

.repo-error {
  margin-top: 6px;
  padding: 6px 8px;
  background: var(--error-bg);
  border-radius: 4px;
  font-size: 11px;
  color: var(--error);
}

.repo-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.15s;
}

.btn-icon:hover {
  background: var(--accent);
  color: white;
}

.btn-icon.btn-danger:hover {
  background: var(--error);
  color: white;
}

.btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sync-icon.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
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
</style>

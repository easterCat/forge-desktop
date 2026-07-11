<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal import-modal">
      <div class="modal-header">
        <h3>导入 agency-agents-zh</h3>
        <button class="btn-icon" aria-label="关闭" @click="$emit('close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div v-if="!importing && !result" class="import-form">
          <p class="import-desc">
            从本地克隆的 agency-agents-zh 仓库导入 216 个 AI 专家角色。
          </p>
          <p class="import-hint">
            如果还没有克隆，先运行：<br/>
            <code>git clone https://github.com/jnMetaCode/agency-agents-zh.git</code>
          </p>

          <div class="path-input">
            <label>仓库路径</label>
            <div class="input-row">
              <input
                v-model="repoPath"
                type="text"
                placeholder="例如: ~/agency-agents-zh"
                class="path-field"
              />
              <button class="btn btn-secondary btn-sm" @click="handleBrowse">
                浏览…
              </button>
            </div>
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>
        </div>

        <div v-if="importing" class="importing-state">
          <div class="spinner"></div>
          <span>正在导入，解析 Markdown 文件中…</span>
        </div>

        <div v-if="result" class="import-result">
          <div class="result-icon">✓</div>
          <div class="result-title">导入完成</div>
          <div class="result-stats">
            <span class="stat imported">{{ result.imported }} 个已导入</span>
            <span class="stat skipped">{{ result.skipped }} 个已跳过</span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="$emit('close')">
          {{ result ? '关闭' : '取消' }}
        </button>
        <button
          v-if="!result"
          class="btn btn-primary"
          :disabled="!repoPath.trim() || importing"
          @click="handleImport"
        >
          {{ importing ? '导入中…' : '开始导入' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue';
import { useAgentStore } from '@/stores/agent';
import type { AgentImportResult } from '@/types/agent';

defineEmits<{
  (e: 'close'): void;
}>();

const agentStore = useAgentStore();
const showNotification = inject<(msg: string, type?: string) => void>('showNotification');

const repoPath = ref('');
const importing = ref(false);
const result = ref<AgentImportResult | null>(null);
const error = ref<string | null>(null);

async function handleBrowse() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 agency-agents-zh 仓库目录',
    });
    if (selected) {
      repoPath.value = selected as string;
    }
  } catch {
    // Dialog plugin not available, user can type path manually
  }
}

async function handleImport() {
  if (!repoPath.value.trim()) return;

  importing.value = true;
  error.value = null;

  try {
    result.value = await agentStore.importFromRepo(repoPath.value.trim());
    showNotification?.(`导入完成：${result.value.imported} 个 Agent`, 'success');
  } catch (e) {
    error.value = e instanceof Error ? e.message : '导入失败';
    showNotification?.('导入失败', 'error');
  } finally {
    importing.value = false;
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.import-modal {
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  width: 90%;
  max-width: 520px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--fg-muted);
  padding: 4px;
  border-radius: 4px;
}

.btn-icon:hover {
  background: var(--bg-tertiary);
}

.modal-body {
  padding: 20px 24px;
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.import-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  margin: 0;
}

.import-hint {
  font-size: 12px;
  color: var(--fg-ghost);
  margin: 0;
  line-height: 1.6;
}

.import-hint code {
  background: var(--bg-secondary);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  color: var(--accent);
}

.path-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.path-input label {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
}

.input-row {
  display: flex;
  gap: 8px;
}

.path-field {
  flex: 1;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 8px 12px;
  font-size: 13px;
  color: var(--fg);
  outline: none;
}

.path-field:focus {
  border-color: var(--accent);
}

.path-field::placeholder {
  color: var(--fg-ghost);
}

.error-msg {
  font-size: 12px;
  color: var(--error);
  background: var(--error-bg, var(--bg-secondary));
  padding: 8px 12px;
  border-radius: var(--radius);
}

.importing-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 40px 0;
  color: var(--fg-muted);
  font-size: 13px;
}

.spinner {
  width: 28px;
  height: 28px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.import-result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 0;
}

.result-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--success-bg, var(--bg-tertiary));
  color: var(--success);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 600;
}

.result-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
}

.result-stats {
  display: flex;
  gap: 16px;
}

.stat {
  font-size: 13px;
  color: var(--fg-muted);
}

.stat.imported {
  color: var(--success);
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 24px;
  border-top: 1px solid var(--border);
}

.btn {
  padding: 6px 14px;
  border-radius: var(--radius);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border);
  transition: all var(--transition-fast);
}

.btn-primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.btn-primary:hover {
  background: var(--accent-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.btn-secondary:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
}
</style>

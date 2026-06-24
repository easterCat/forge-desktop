<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="handleClose">
      <div class="dialog-container">
        <!-- Header -->
        <div class="dialog-header">
          <div class="header-content">
            <div class="skill-icon" :style="getIconStyle(skill.name)">
              {{ skill.name.charAt(0).toUpperCase() }}
            </div>
            <div class="header-info">
              <h2>{{ skill.name }}</h2>
              <div class="header-badges">
                <span v-if="skill.version" class="badge version-badge">v{{ skill.version }}</span>
                <span v-if="skill.author" class="badge author-badge">{{ skill.author }}</span>
                <span v-if="isInstalled" class="badge installed-badge">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  已安装
                </span>
              </div>
            </div>
          </div>
          <button class="close-btn" @click="handleClose" aria-label="关闭">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <!-- Content -->
        <div class="dialog-content">
          <!-- Description -->
          <div class="section">
            <h3>描述</h3>
            <p class="description">{{ skill.description || '暂无描述' }}</p>
          </div>

          <!-- Metadata -->
          <div class="section">
            <h3>信息</h3>
            <div class="metadata-grid">
              <div class="metadata-item">
                <span class="label">ID</span>
                <span class="value">{{ skill.id }}</span>
              </div>
              <div class="metadata-item">
                <span class="label">仓库</span>
                <button class="value link" @click.stop="openExternalUrl(skill.repository)">{{ skill.repository }}</button>
              </div>
              <div class="metadata-item">
                <span class="label">路径</span>
                <span class="value path">{{ skill.subdirectory }}</span>
              </div>
              <div class="metadata-item" v-if="skill.file_count > 0">
                <span class="label">文件数</span>
                <span class="value">{{ skill.file_count }}</span>
              </div>
              <div v-if="isInstalled && skill.installed_at" class="metadata-item">
                <span class="label">安装时间</span>
                <span class="value">{{ formatDate(skill.installed_at) }}</span>
              </div>
              <div v-if="isInstalled && skill.installed_path" class="metadata-item">
                <span class="label">安装路径</span>
                <span class="value path" :title="skill.installed_path">{{ skill.installed_path }}</span>
              </div>
            </div>
          </div>

          <!-- Tags -->
          <div v-if="skill.tags.length > 0" class="section">
            <h3>标签</h3>
            <div class="tags-list">
              <span v-for="tag in skill.tags" :key="tag" class="tag">{{ tag }}</span>
            </div>
          </div>

          <!-- Dependencies -->
          <div v-if="skill.dependencies.length > 0" class="section">
            <h3>依赖项</h3>
            <div class="deps-list">
              <div v-for="dep in skill.dependencies" :key="dep" class="dep-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9 18 15 12 9 6" />
                </svg>
                <span>{{ dep }}</span>
              </div>
            </div>
          </div>

          <!-- Verification Status -->
          <div v-if="isInstalled && verification" class="section">
            <h3>完整性验证</h3>
            <div class="verification-status">
              <div class="status-item" :class="{ success: verification.skill_md_present }">
                <svg v-if="verification.skill_md_present" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
                <span>SKILL.md 存在</span>
              </div>
              <div class="status-item" :class="{ success: verification.sha256_verified }">
                <svg v-if="verification.sha256_verified" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
                <span>SHA256 验证</span>
              </div>
              <div class="status-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                </svg>
                <span>{{ verification.files_downloaded }} 个文件</span>
              </div>
              <div class="status-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                </svg>
                <span>{{ formatSize(verification.total_size) }}</span>
              </div>
            </div>
            <div v-if="verification.errors.length > 0" class="verification-errors">
              <p>警告:</p>
              <ul>
                <li v-for="(error, index) in verification.errors" :key="index">{{ error }}</li>
              </ul>
            </div>
          </div>

          <!-- Actions -->
          <div class="dialog-actions">
            <button
              class="btn btn-secondary"
              @click.stop="openExternalUrl(`${skill.repository}/tree/main/${skill.subdirectory}`)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
                <polyline points="15 3 21 3 21 9" />
                <line x1="10" y1="14" x2="21" y2="3" />
              </svg>
              在 GitHub 查看
            </button>
            <button
              v-if="!isInstalled"
              class="btn btn-primary"
              @click="handleInstall"
              :disabled="isInstalling"
            >
              <svg v-if="isInstalling" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56" />
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              {{ isInstalling ? '安装中...' : '安装' }}
            </button>
            <button
              v-else
              class="btn btn-danger"
              @click="handleUninstall"
              :disabled="isUninstalling"
            >
              <svg v-if="isUninstalling" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56" />
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
              </svg>
              {{ isUninstalling ? '卸载中...' : '卸载' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { AnthropicSkill, InstallVerification } from '@/types';
import { open as openExternal } from '@tauri-apps/plugin-shell';

const props = defineProps<{
  skill: AnthropicSkill;
  isInstalled: boolean;
}>();

const emit = defineEmits<{
  close: [];
  install: [skill: AnthropicSkill];
  uninstall: [skill: AnthropicSkill];
}>();

const isInstalling = ref(false);
const isUninstalling = ref(false);
const verification = ref<InstallVerification | null>(null);

function handleClose(): void {
  emit('close');
}

function handleInstall(): void {
  emit('install', props.skill);
}

function handleUninstall(): void {
  emit('uninstall', props.skill);
}

function getIconStyle(name: string): Record<string, string> {
  const colors = [
    { bg: 'rgba(102, 126, 234, 0.15)', border: 'rgba(102, 126, 234, 0.3)' },
    { bg: 'rgba(245, 147, 251, 0.15)', border: 'rgba(245, 147, 251, 0.3)' },
    { bg: 'rgba(79, 172, 254, 0.15)', border: 'rgba(79, 172, 254, 0.3)' },
    { bg: 'rgba(67, 233, 123, 0.15)', border: 'rgba(67, 233, 123, 0.3)' },
    { bg: 'rgba(250, 112, 154, 0.15)', border: 'rgba(250, 112, 154, 0.3)' },
    { bg: 'rgba(161, 140, 209, 0.15)', border: 'rgba(161, 140, 209, 0.3)' },
    { bg: 'rgba(255, 154, 158, 0.15)', border: 'rgba(255, 154, 158, 0.3)' },
    { bg: 'rgba(255, 236, 210, 0.15)', border: 'rgba(255, 236, 210, 0.3)' },
  ];

  const hash = name.split('').reduce((acc, char) => {
    return char.charCodeAt(0) + ((acc << 5) - acc);
  }, 0);

  const index = Math.abs(hash) % colors.length;
  return { background: colors[index].bg, borderColor: colors[index].border };
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
}

async function openExternalUrl(url: string) {
  try {
    await openExternal(url)
  } catch (e) {
    console.warn('shell.open failed, falling back to window.open:', e)
    try {
      window.open(url, '_blank', 'noopener,noreferrer')
    } catch (err) {
      console.error('Failed to open URL:', err)
    }
  }
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: 20px;
}

.dialog-container {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 16px;
  width: 100%;
  max-width: 560px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-secondary);
}

.header-content {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.skill-icon {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  color: white;
  font-size: 24px;
  font-weight: 600;
  flex-shrink: 0;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.header-info h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.header-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 6px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.version-badge {
  font-family: monospace;
  background: var(--accent-bg);
  color: var(--accent);
}

.installed-badge {
  background: var(--success-bg);
  color: var(--success);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  background: none;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

.dialog-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.section {
  margin-bottom: 20px;
}

.section:last-child {
  margin-bottom: 0;
}

.section h3 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 10px;
}

.description {
  font-size: 14px;
  line-height: 1.6;
  color: var(--fg);
}

.metadata-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.metadata-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metadata-item .label {
  font-size: 11px;
  color: var(--fg-muted);
}

.metadata-item .value {
  font-size: 13px;
  color: var(--fg);
  word-break: break-all;
}

.metadata-item .value.link {
  color: var(--accent);
  text-decoration: none;
}

.metadata-item .value.link:hover {
  text-decoration: underline;
}

.metadata-item .value.path {
  font-family: monospace;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  padding: 4px 10px;
  font-size: 12px;
  background: var(--accent-bg);
  color: var(--accent);
  border-radius: 6px;
}

.deps-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dep-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--fg);
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.dep-item svg {
  color: var(--fg-muted);
  flex-shrink: 0;
}

.verification-status {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.status-item.success {
  color: var(--success);
}

.status-item.success svg {
  color: var(--success);
}

.verification-errors {
  margin-top: 12px;
  padding: 12px;
  background: var(--warn-bg);
  border: 1px solid var(--warn);
  border-radius: 8px;
  font-size: 12px;
}

.verification-errors p {
  font-weight: 600;
  margin-bottom: 6px;
  color: var(--warn);
}

.verification-errors ul {
  margin: 0;
  padding-left: 20px;
  color: var(--fg-muted);
}

.verification-errors li {
  margin-bottom: 4px;
}

.dialog-actions {
  display: flex;
  gap: 12px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
  margin-top: 20px;
}

.dialog-actions .btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.dialog-actions .btn.link {
  text-decoration: none;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>

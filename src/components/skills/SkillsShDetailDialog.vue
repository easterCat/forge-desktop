<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <div class="header-info">
          <h3>{{ skill.name }}</h3>
          <span class="skill-source">{{ skill.source }}</span>
        </div>
        <button class="close-btn" aria-label="关闭" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="dialog-content">
        <!-- Stats -->
        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-value">{{ formatNumber(skill.installs) }}</span>
            <span class="stat-label">总安装</span>
          </div>
          <div v-if="detail?.installs" class="stat-item">
            <span class="stat-value">{{ formatNumber(detail.installs) }}</span>
            <span class="stat-label">当前安装</span>
          </div>
          <div v-if="detail?.hash" class="stat-item">
            <span class="stat-value hash">{{ detail.hash.slice(0, 8) }}</span>
            <span class="stat-label">Hash</span>
          </div>
        </div>

        <!-- Tabs -->
        <div class="tabs">
          <button
            class="tab"
            :class="{ active: activeTab === 'files' }"
            @click="activeTab = 'files'"
          >
            文件
          </button>
          <button
            class="tab"
            :class="{ active: activeTab === 'audit' }"
            @click="loadAudit"
          >
            安全审计
          </button>
        </div>

        <!-- Files Tab -->
        <div v-if="activeTab === 'files'" class="tab-content">
          <div v-if="isLoadingDetail" class="loading-state">
            <div class="spinner"></div>
            <p>加载文件列表...</p>
          </div>
          <div v-else-if="detail?.files && detail.files.length > 0" class="file-tree">
            <div
              v-for="file in detail.files"
              :key="file.path"
              class="file-item"
              @click="toggleFile(file)"
            >
              <div class="file-header">
                <svg v-if="isMarkdown(file.path)" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M13 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V9z"/>
                  <polyline points="13 2 13 9 20 9"/>
                </svg>
                <span class="file-path">{{ file.path }}</span>
              </div>
              <div v-if="expandedFiles.has(file.path)" class="file-content">
                <pre v-if="file.contents"><code>{{ file.contents }}</code></pre>
                <p v-else class="no-preview">无预览内容</p>
              </div>
            </div>
          </div>
          <div v-else class="empty-state">
            <p>暂无文件列表</p>
            <button class="btn btn-secondary btn-sm" @click="loadDetail">
              加载详情
            </button>
          </div>
        </div>

        <!-- Audit Tab -->
        <div v-if="activeTab === 'audit'" class="tab-content">
          <div v-if="isLoadingAudit" class="loading-state">
            <div class="spinner"></div>
            <p>加载安全审计...</p>
          </div>
          <div v-else-if="audit" class="audit-list">
            <div
              v-for="entry in audit.audits"
              :key="entry.provider"
              class="audit-item"
              :class="entry.status"
            >
              <div class="audit-header">
                <span class="audit-provider">{{ entry.provider }}</span>
                <span class="audit-status" :class="entry.status">
                  {{ statusLabel(entry.status) }}
                </span>
              </div>
              <p class="audit-summary">{{ entry.summary }}</p>
              <div class="audit-meta">
                <span v-if="entry.riskLevel" class="risk-level" :class="entry.riskLevel.toLowerCase()">
                  {{ entry.riskLevel }}
                </span>
                <span class="audit-date">{{ formatDate(entry.auditedAt) }}</span>
              </div>
            </div>
          </div>
          <div v-else-if="auditError" class="error-state">
            <p>{{ auditError }}</p>
            <button class="btn btn-secondary btn-sm" @click="loadAudit">
              重试
            </button>
          </div>
          <div v-else class="empty-state">
            <p>暂无安全审计数据</p>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <div class="install-command">
          <span class="command-label">安装命令:</span>
          <code class="command-code">{{ installCommand }}</code>
          <button class="btn-icon" title="复制" @click="copyCommand">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
            </svg>
          </button>
        </div>
        <div class="footer-actions">
          <button
            class="btn btn-secondary btn-sm"
            @click.stop="openExternalUrl(skill.url)"
          >
            在浏览器中打开
          </button>
          <button
            class="btn btn-primary"
            :disabled="isInstalling || !skill.installUrl"
            @click="handleInstall"
          >
            <svg v-if="isInstalling" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 11-6.219-8.56"/>
            </svg>
            {{ isInstalling ? '安装中...' : '立即安装' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { SkillsShSkill, SkillsShSkillDetail, SkillsShAuditResponse } from '@/types';
import { useSkillsShStore } from '@/stores/skills-sh';
import { open as openExternal } from '@tauri-apps/plugin-shell';

const props = defineProps<{
  skill: SkillsShSkill;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'installed'): void;
}>();

const store = useSkillsShStore();

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

const activeTab = ref<'files' | 'audit'>('files');
const detail = ref<SkillsShSkillDetail | null>(null);
const audit = ref<SkillsShAuditResponse | null>(null);
const isLoadingDetail = ref(false);
const isLoadingAudit = ref(false);
const auditError = ref<string | null>(null);
const isInstalling = ref(false);
const expandedFiles = ref<Set<string>>(new Set());

const installCommand = computed(() => {
  if (props.skill.installUrl) {
    return `npx skills add ${props.skill.installUrl} --skill ${props.skill.slug}`;
  }
  return `npx skills add ${props.skill.source} --skill ${props.skill.slug}`;
});

async function loadDetail() {
  if (detail.value) return;
  isLoadingDetail.value = true;
  try {
    detail.value = await store.fetchDetail(props.skill.source, props.skill.slug);
  } finally {
    isLoadingDetail.value = false;
  }
}

async function loadAudit() {
  activeTab.value = 'audit';
  if (audit.value || auditError.value) return;
  isLoadingAudit.value = true;
  auditError.value = null;
  try {
    audit.value = await store.fetchAudit(props.skill.source, props.skill.slug);
  } catch (e) {
    auditError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoadingAudit.value = false;
  }
}

function toggleFile(file: { path: string; contents: string | null }) {
  if (expandedFiles.value.has(file.path)) {
    expandedFiles.value.delete(file.path);
  } else {
    expandedFiles.value.add(file.path);
    if (!file.contents) {
      loadDetail();
    }
  }
}

function isMarkdown(path: string): boolean {
  return /\.(md|mdx)$/i.test(path);
}

async function handleInstall() {
  if (!props.skill.installUrl) return;
  isInstalling.value = true;
  try {
    const result = await store.installSkill(props.skill.installUrl, props.skill.slug);
    if (result.success) {
      emit('installed');
    }
  } finally {
    isInstalling.value = false;
  }
}

async function copyCommand() {
  try {
    await navigator.clipboard.writeText(installCommand.value);
  } catch (e) {
    console.error('Failed to copy:', e);
  }
}

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
  }
  return num.toString();
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    pass: '通过',
    warn: '警告',
    fail: '失败',
  };
  return map[status] || status;
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString('zh-CN');
  } catch {
    return iso;
  }
}

// Load detail on mount
loadDetail();
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
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
  width: 90%;
  max-width: 720px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
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

.header-info h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 4px;
}

.skill-source {
  font-size: 12px;
  color: var(--fg-muted);
  font-family: var(--font-mono);
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

.stats-row {
  display: flex;
  gap: 24px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 10px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent);
  font-family: var(--font-mono);
}

.stat-value.hash {
  font-size: 14px;
  color: var(--fg-muted);
}

.stat-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.tab {
  flex: 1;
  padding: 8px 16px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.tab:hover {
  color: var(--fg);
}

.tab.active {
  background: var(--accent);
  color: white;
}

.tab-content {
  min-height: 200px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--fg-muted);
}

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--fg-muted);
  gap: 12px;
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: var(--error);
  gap: 12px;
  text-align: center;
}

.file-tree {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.file-item {
  background: var(--bg-secondary);
  border-radius: 8px;
  overflow: hidden;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.file-header:hover {
  background: var(--bg-tertiary);
}

.file-path {
  font-size: 13px;
  font-family: var(--font-mono);
  color: var(--fg);
}

.file-content {
  padding: 12px 14px;
  background: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.file-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-size: 12px;
  line-height: 1.6;
  color: var(--fg);
}

.file-content code {
  font-family: var(--font-mono);
}

.no-preview {
  font-size: 12px;
  color: var(--fg-muted);
  font-style: italic;
}

.audit-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.audit-item {
  padding: 14px;
  background: var(--bg-secondary);
  border-radius: 10px;
  border-left: 3px solid var(--border);
}

.audit-item.pass {
  border-left-color: var(--success);
}

.audit-item.warn {
  border-left-color: var(--warn);
}

.audit-item.fail {
  border-left-color: var(--error);
}

.audit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.audit-provider {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg);
}

.audit-status {
  padding: 3px 10px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  text-transform: uppercase;
}

.audit-status.pass {
  background: var(--success-bg);
  color: var(--success);
}

.audit-status.warn {
  background: var(--warn-bg);
  color: var(--warn);
}

.audit-status.fail {
  background: var(--error-bg);
  color: var(--error);
}

.audit-summary {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  margin-bottom: 8px;
}

.audit-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.risk-level {
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  text-transform: uppercase;
}

.risk-level.none {
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.risk-level.low {
  background: var(--success-bg);
  color: var(--success);
}

.risk-level.medium {
  background: var(--warn-bg);
  color: var(--warn);
}

.risk-level.high,
.risk-level.critical {
  background: var(--error-bg);
  color: var(--error);
}

.audit-date {
  font-size: 11px;
  color: var(--fg-muted);
}

.dialog-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.install-command {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 12px;
}

.command-label {
  font-size: 12px;
  color: var(--fg-muted);
  flex-shrink: 0;
}

.command-code {
  flex: 1;
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--fg);
  background: none;
  border: none;
  outline: none;
}

.footer-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-icon {
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
  transition: all 0.15s;
  flex-shrink: 0;
}

.btn-icon:hover {
  background: var(--accent);
  color: white;
}

.spin {
  animation: spin 1s linear infinite;
}
</style>

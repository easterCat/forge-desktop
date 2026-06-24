<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal agent-detail-modal">
      <div class="modal-header">
        <div class="modal-title-row">
          <span class="agent-emoji">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2M7.5 13A1.5 1.5 0 1 0 6 14.5 1.5 1.5 0 0 0 7.5 13m9 0a1.5 1.5 0 1 0 1.5 1.5 1.5 1.5 0 0 0-1.5-1.5M12 18a1 1 0 1 0 0 2 1 1 0 0 0 0-2"/>
            </svg>
          </span>
          <div>
            <h3>{{ agent.name }}</h3>
            <span class="dept-label">{{ departmentLabel }}</span>
          </div>
        </div>
        <button class="btn-icon" aria-label="关闭" @click="$emit('close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div class="detail-meta">
          <div class="meta-item">
            <span class="meta-label">来源</span>
            <span class="meta-value">{{ agent.source === 'builtin' ? '内置' : agent.source }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">已安装到</span>
            <div class="installed-targets">
              <span v-if="installedTargets.length === 0" class="no-targets">未安装</span>
              <span v-for="t in installedTargets" :key="t" class="target-badge">{{ targetLabels[t] || t }}</span>
            </div>
          </div>
        </div>

        <div class="detail-description">
          <strong>简介：</strong>{{ agent.description }}
        </div>

        <div class="detail-content">
          <div class="content-header">
            <strong>Agent 内容</strong>
          </div>
          <pre class="content-body">{{ agent.content }}</pre>
        </div>
      </div>

      <div class="modal-footer">
        <div class="install-buttons">
          <button
            class="btn btn-sm"
            :class="isInstalledTo('claude-code') ? 'btn-success' : 'btn-primary'"
            @click="handleInstall('claude-code')"
          >
            {{ isInstalledTo('claude-code') ? '✓ Claude Code' : '安装到 Claude Code' }}
          </button>
          <button
            class="btn btn-sm"
            :class="isInstalledTo('cursor') ? 'btn-success' : 'btn-primary'"
            @click="handleInstall('cursor')"
          >
            {{ isInstalledTo('cursor') ? '✓ Cursor' : '安装到 Cursor' }}
          </button>
          <button
            class="btn btn-sm"
            :class="isInstalledTo('copilot') ? 'btn-success' : 'btn-secondary'"
            @click="handleInstall('copilot')"
          >
            {{ isInstalledTo('copilot') ? '✓ Copilot' : '安装到 Copilot' }}
          </button>
        </div>
        <button v-if="agent.isCustom" class="btn btn-danger btn-sm" @click="handleDelete">
          删除
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject } from 'vue';
import { useAgentStore } from '@/stores/agent';
import type { Agent } from '@/types/agent';
import { AGENT_DEPARTMENTS } from '@/types/agent';

const props = defineProps<{
  agent: Agent;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const agentStore = useAgentStore();
const showNotification = inject<(msg: string, type?: string) => void>('showNotification');

const departmentLabel = computed(() => {
  const dept = AGENT_DEPARTMENTS.find(d => d.id === props.agent.department);
  return dept ? `${dept.emoji} ${dept.name}` : props.agent.department;
});

const installedTargets = computed(() => agentStore.getInstalledTargets(props.agent));

const targetLabels: Record<string, string> = {
  'claude-code': 'Claude Code',
  'cursor': 'Cursor',
  'copilot': 'Copilot',
};

function isInstalledTo(target: string): boolean {
  return agentStore.isInstalledTo(props.agent, target);
}

async function handleInstall(target: string) {
  if (isInstalledTo(target)) {
    // Uninstall
    try {
      await agentStore.uninstallAgent(props.agent.id, target);
      showNotification?.(`已从 ${targetLabels[target] || target} 卸载`, 'info');
    } catch {
      showNotification?.('卸载失败', 'error');
    }
  } else {
    // Install
    try {
      const path = await agentStore.installAgent(props.agent.id, target);
      showNotification?.(`已安装到 ${path}`, 'success');
    } catch {
      showNotification?.('安装失败', 'error');
    }
  }
}

async function handleDelete() {
  try {
    await agentStore.deleteAgent(props.agent.id);
    showNotification?.('已删除', 'success');
    emit('close');
  } catch {
    showNotification?.('删除失败', 'error');
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

.agent-detail-modal {
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  width: 90%;
  max-width: 720px;
  max-height: 85vh;
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

.modal-title-row {
  display: flex;
  align-items: center;
  gap: 14px;
}

.agent-emoji {
  font-size: 32px;
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--fg);
}

.dept-label {
  font-size: 12px;
  color: var(--fg-muted);
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
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-meta {
  display: flex;
  gap: 24px;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  font-size: 11px;
  color: var(--fg-ghost);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.meta-value {
  font-size: 13px;
  color: var(--fg);
}

.installed-targets {
  display: flex;
  gap: 4px;
}

.no-targets {
  font-size: 13px;
  color: var(--fg-ghost);
}

.target-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--success-bg, var(--bg-tertiary));
  color: var(--success, var(--fg-muted));
}

.detail-description {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
}

.detail-content {
  flex: 1;
  min-height: 0;
}

.content-header {
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--fg);
}

.content-body {
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  padding: 16px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 360px;
  overflow-y: auto;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-top: 1px solid var(--border);
}

.install-buttons {
  display: flex;
  gap: 8px;
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

.btn-success {
  background: var(--success-bg, var(--bg-tertiary));
  color: var(--success);
  border-color: var(--success);
}

.btn-danger {
  background: var(--error-bg, var(--bg-tertiary));
  color: var(--error);
  border-color: var(--error);
}

.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
}
</style>

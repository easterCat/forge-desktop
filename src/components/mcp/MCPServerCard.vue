<template>
  <BaseCard variant="glass" padding="md" class="mcp-server-card" :class="{ installed: isInstalled }">
    <!-- Header with icon and badges -->
    <div class="card-header">
      <div class="server-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
          <line x1="6" y1="6" x2="6.01" y2="6"/>
          <line x1="6" y1="18" x2="6.01" y2="18"/>
        </svg>
      </div>
      <div class="card-badges">
        <!-- Health Status Badge -->
        <MCPHealthBadge
          v-if="hasHealthStatus"
          :status="healthStatus"
          :show-label="false"
          size="sm"
        />
        <span v-if="isInstalled" class="badge installed-badge">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          Installed
        </span>
        <span class="badge protocol-badge" :class="server.protocol">
          {{ protocolLabel }}
        </span>
      </div>
    </div>

    <!-- Quick Actions Menu -->
    <div class="quick-actions">
      <button class="action-trigger" @click.stop="toggleMenu" title="Actions">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="1"/>
          <circle cx="12" cy="5" r="1"/>
          <circle cx="12" cy="19" r="1"/>
        </svg>
      </button>
      <div v-if="menuOpen" class="action-menu" v-click-outside="closeMenu">
        <button class="menu-item" @click="handleEdit">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
          Edit
        </button>
        <button class="menu-item" @click="handleRefresh" :disabled="isRefreshing">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: isRefreshing }">
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          {{ isRefreshing ? 'Checking...' : 'Check Health' }}
        </button>
        <button class="menu-item danger" @click="handleDelete">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Delete
        </button>
      </div>
    </div>

    <!-- Title and description -->
    <div class="card-content">
      <h3 class="server-name">{{ server.name }}</h3>
      <p class="server-desc">{{ server.description }}</p>
    </div>

    <!-- Categories and tags -->
    <div class="card-categories">
      <span
        v-for="cat in server.categories.slice(0, 2)"
        :key="cat"
        class="category-tag"
      >
        {{ cat }}
      </span>
      <span
        v-for="tag in server.tags.slice(0, 3)"
        :key="tag"
        class="category-tag tag"
      >
        {{ tag }}
      </span>
    </div>

    <!-- Required environment variables -->
    <div v-if="server.requiredEnvVars?.length" class="env-vars">
      <span class="env-label">Required env:</span>
      <span
        v-for="env in server.requiredEnvVars.slice(0, 2)"
        :key="env.name"
        class="env-var"
        :title="env.description"
      >
        {{ env.name }}
      </span>
      <span v-if="server.requiredEnvVars.length > 2" class="env-more">
        +{{ server.requiredEnvVars.length - 2 }} more
      </span>
    </div>

    <!-- Stats -->
    <div class="card-stats">
      <div v-if="server.stars" class="stat">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
        {{ formatNumber(server.stars) }}
      </div>
      <div v-if="server.downloads" class="stat">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        {{ formatNumber(server.downloads) }}
      </div>
      <div v-if="server.version" class="stat">
        v{{ server.version }}
      </div>
      <div v-if="server.npmPackage" class="stat npm">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
        </svg>
        npm
      </div>
    </div>

    <!-- Install command preview -->
    <div v-if="server.installCommand" class="install-command">
      <code>{{ truncatedCommand }}</code>
    </div>

    <!-- Progress bar -->
    <div v-if="progress" class="install-progress">
      <div class="progress-info">
        <span class="progress-stage">{{ getStageLabel(progress.stage) }}</span>
        <span class="progress-percent">{{ progress.progress }}%</span>
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :class="progress.stage"
          :style="{ width: `${progress.progress}%` }"
        ></div>
      </div>
      <p v-if="progress.message" class="progress-message">{{ progress.message }}</p>
    </div>

    <!-- Actions -->
    <div class="card-actions">
      <button
        class="btn btn-detail btn-sm"
        @click="$emit('view-details', server)"
      >
        Details
      </button>
      <button
        v-if="!isInstalled"
        class="btn btn-primary btn-sm"
        :disabled="!!progress"
        @click="$emit('install', server)"
      >
        <svg v-if="!progress" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        <span v-if="progress">Installing...</span>
        <span v-else>Install</span>
      </button>
      <button
        v-else
        class="btn btn-secondary btn-sm"
        @click="$emit('sync', server)"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
        Sync
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import MCPHealthBadge from './MCPHealthBadge.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import type { MCPServer, MCPInstallProgress, MCPService } from '@/types';
import { confirm } from '@/utils/dialog';

interface Props {
  server: MCPServer | MCPService;
  isInstalled?: boolean;
  progress?: MCPInstallProgress;
  showHealthBadge?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isInstalled: false,
  showHealthBadge: true,
});

const emit = defineEmits<{
  (e: 'install', server: MCPServer | MCPService): void;
  (e: 'sync', server: MCPServer | MCPService): void;
  (e: 'view-details', server: MCPServer | MCPService): void;
  (e: 'edit', server: MCPService): void;
  (e: 'refresh', server: MCPService): void;
  (e: 'delete', server: MCPService): void;
}>();

// Check if server has health status (MCPService vs MCPServer)
const hasHealthStatus = computed(() => {
  return 'isHealthy' in props.server;
});

const healthStatus = computed<'online' | 'offline' | 'error' | 'checking'>(() => {
  if (!hasHealthStatus.value) return 'offline';
  const service = props.server as MCPService;
  return service.isHealthy ? 'online' : 'offline';
});

// Quick actions menu
const menuOpen = ref(false);
const isRefreshing = ref(false);

function toggleMenu() {
  menuOpen.value = !menuOpen.value;
}

function closeMenu() {
  menuOpen.value = false;
}

function handleEdit() {
  closeMenu();
  if (hasHealthStatus.value) {
    emit('edit', props.server as MCPService);
  }
}

async function handleRefresh() {
  closeMenu();
  if (hasHealthStatus.value) {
    isRefreshing.value = true;
    emit('refresh', props.server as MCPService);
    // Simulate refresh duration
    setTimeout(() => {
      isRefreshing.value = false;
    }, 2000);
  }
}

async function handleDelete() {
  closeMenu();
  if (hasHealthStatus.value) {
    if (await confirm(`Delete server "${props.server.name}"? This cannot be undone.`)) {
      emit('delete', props.server as MCPService);
    }
  }
}

const protocolLabel = computed(() => {
  const labels: Record<string, string> = {
    stdio: 'STDIO',
    sse: 'SSE',
    http: 'HTTP',
  };
  return labels[props.server.protocol] || props.server.protocol.toUpperCase();
});

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num.toString();
}

function getStageLabel(stage: string): string {
  const labels: Record<string, string> = {
    pending: 'Pending',
    downloading: 'Downloading',
    installing: 'Installing',
    extracting: 'Extracting',
    syncing: 'Syncing',
    success: 'Complete',
    failed: 'Failed',
    conflict: 'Conflict',
  };
  return labels[stage] || stage;
}
</script>

<style scoped>
.mcp-server-card {
  position: relative;
  display: flex;
  flex-direction: column;
}

.mcp-server-card.installed {
  border-color: var(--success);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.server-icon {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--fg-muted);
}

.card-badges {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

/* Quick Actions Menu */
.quick-actions {
  position: absolute;
  top: 12px;
  right: 12px;
}

.action-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid rgba(255, 255, 255, 0.40);
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--fg-ghost);
  transition: all var(--t-fast);
  opacity: 0;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.mcp-server-card:hover .action-trigger {
  opacity: 1;
}

.action-trigger:hover {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
}

.action-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  min-width: 160px;
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  overflow: hidden;
  z-index: 10;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  background: none;
  border: none;
  font-size: 13px;
  color: var(--fg);
  cursor: pointer;
  transition: background 0.15s;
  text-align: left;
}

.menu-item:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.40);
}

.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-item.danger {
  color: var(--error);
}

.menu-item.danger:hover {
  background: rgba(239, 68, 68, 0.1);
}

.spinning {
  animation: spin 1s linear infinite;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.installed-badge {
  background: var(--success-bg);
  color: var(--success);
}

.protocol-badge {
  font-family: monospace;
  font-weight: 600;
}

.protocol-badge.stdio {
  background: var(--success-bg);
  color: var(--success);
}

.protocol-badge.sse {
  background: var(--warn-bg);
  color: var(--warn);
}

.protocol-badge.http {
  background: var(--info-bg);
  color: var(--info);
}

.card-content {
  flex: 1;
  margin-bottom: 12px;
}

.server-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  color: var(--fg);
}

.server-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
}

.card-categories {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.category-tag {
  padding: 3px 8px;
  font-size: 11px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  color: var(--fg-muted);
}

.category-tag.tag {
  background: var(--accent-bg);
  color: var(--accent);
}

.env-vars {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 11px;
}

.env-label {
  color: var(--fg-muted);
}

.env-var {
  padding: 2px 6px;
  background: var(--warn-bg);
  color: var(--warn);
  border-radius: 3px;
  font-family: monospace;
}

.env-more {
  color: var(--fg-muted);
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  margin-bottom: 12px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--fg-muted);
}

.stat svg {
  color: var(--warn);
}

.stat.npm {
  color: var(--error);
}

.stat.npm svg {
  color: inherit;
}

.install-command {
  margin-bottom: 12px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  overflow: hidden;
}

.install-command code {
  font-size: 11px;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  color: var(--fg-muted);
  word-break: break-all;
}

.install-progress {
  margin-bottom: 12px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 12px;
}

.progress-stage {
  color: var(--fg-muted);
}

.progress-percent {
  font-weight: 500;
  color: var(--fg);
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

.progress-fill.success {
  background: var(--success);
}

.progress-fill.failed {
  background: var(--error);
}

.progress-fill.syncing {
  background: var(--warn);
}

.progress-fill.extracting {
  background: var(--warn);
}

.progress-message {
  margin-top: 4px;
  font-size: 11px;
  color: var(--fg-muted);
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-actions .btn {
  flex: 0 0 auto;
}

.card-actions .btn-primary,
.card-actions .btn-secondary {
  margin-left: auto;
}
</style>

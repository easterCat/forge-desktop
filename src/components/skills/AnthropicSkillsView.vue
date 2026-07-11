<template>
  <div class="anthropic-skills-view">
    <!-- Header -->
    <div class="view-header">
      <div class="header-left">
        <h2>Anthropic Skills</h2>
        <span class="skill-count">{{ store.stats.total }} 个技能 · {{ store.activeSource?.label || 'Anthropic' }}</span>
        <span
          v-if="cacheBadge"
          class="cache-badge"
          :class="`cache-badge--${cacheBadge.tone}`"
          :title="cacheBadge.tooltip"
        >
          <span class="cache-badge__dot" aria-hidden="true"></span>
          {{ cacheBadge.label }}
        </span>
      </div>

    <div class="header-actions">
        <button
          class="btn btn-secondary btn-sm"
          :disabled="store.isRefreshing"
          @click="handleRefresh"
        >
          <svg
            class="sync-icon"
            :class="{ spinning: store.isRefreshing }"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
          </svg>
          刷新
        </button>
      </div>
    </div>

    <!-- Search and Filter -->
    <div class="filter-section">
      <div class="search-box">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchKeyword"
          type="text"
          placeholder="搜索技能名称或描述..."
          @input="handleSearch"
        />
        <button v-if="searchKeyword" class="clear-btn" @click="clearSearch">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <div class="filter-group">
        <select v-model="filterStatus" class="filter-select" @change="handleFilterChange">
          <option value="all">全部</option>
          <option value="installed">已安装</option>
          <option value="not_installed">未安装</option>
        </select>

        <select v-model="sortOrder" class="filter-select" @change="handleSortChange">
          <option value="name">按名称排序</option>
          <option value="version">按版本排序</option>
          <option value="install_time">按安装时间排序</option>
        </select>
      </div>

      <div class="stats-bar">
        <div class="stat-item">
          <span class="stat-value">{{ store.stats.total }}</span>
          <span class="stat-label">总计</span>
        </div>
        <div class="stat-item">
          <span class="stat-value installed">{{ store.stats.installed }}</span>
          <span class="stat-label">已安装</span>
        </div>
        <div class="stat-item">
          <span class="stat-value available">{{ store.stats.available }}</span>
          <span class="stat-label">可安装</span>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="store.isLoading" class="loading-state">
      <div class="spinner"></div>
      <p>正在加载 Anthropic Skills...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="store.error" class="error-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <p>{{ store.error }}</p>
      <button class="btn btn-primary btn-sm" @click="handleRetry">重试</button>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="store.filteredSkills.length === 0"
      class="empty-state"
    >
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <p>未找到匹配的技能</p>
      <span v-if="searchKeyword">尝试其他搜索关键词</span>
      <span v-else>点击刷新按钮获取最新列表</span>
    </div>

    <!-- Skills Grid -->
    <div v-else class="skills-grid">
      <div
        v-for="skill in store.filteredSkills"
        :key="skill.id"
        class="skill-card"
        :class="{ installed: skill.installed }"
      >
        <!-- Card Header -->
        <div class="card-header">
          <div class="skill-icon" :style="getIconStyle(skill.name)">
            {{ skill.name.charAt(0).toUpperCase() }}
          </div>
          <div class="card-badges">
            <span v-if="skill.installed" class="badge installed-badge">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              已安装
            </span>
            <span v-if="skill.version" class="badge version-badge">
              v{{ skill.version }}
            </span>
          </div>
        </div>

        <!-- Card Content -->
        <div class="card-content">
          <h3 class="skill-name">{{ skill.name }}</h3>
          <p class="skill-desc">{{ skill.description || '暂无描述' }}</p>
        </div>

        <!-- Tags -->
        <div v-if="skill.tags.length > 0" class="card-tags">
          <span v-for="tag in skill.tags.slice(0, 3)" :key="tag" class="tag">
            {{ tag }}
          </span>
        </div>

        <!-- Dependencies -->
        <div v-if="skill.dependencies.length > 0" class="card-deps">
          <span class="deps-label">依赖:</span>
          <span class="deps-list">{{ skill.dependencies.slice(0, 2).join(', ') }}</span>
          <span v-if="skill.dependencies.length > 2" class="deps-more">
            +{{ skill.dependencies.length - 2 }}
          </span>
        </div>

        <!-- Author -->
        <div v-if="skill.author" class="card-author">
          <span>作者: {{ skill.author }}</span>
        </div>

        <!-- Install Progress -->
        <div v-if="store.isInstalling(skill.id)" class="install-progress">
          <div class="progress-info">
            <span>{{ getProgressStageLabel(store.getInstallProgress(skill.id)?.stage || '') }}</span>
            <span>{{ store.getInstallProgress(skill.id)?.progress }}%</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill"
              :class="store.getInstallProgress(skill.id)?.stage"
              :style="{ width: store.getInstallProgress(skill.id)?.progress + '%' }"
            ></div>
          </div>
          <p class="progress-message">{{ store.getInstallProgress(skill.id)?.message }}</p>
        </div>

        <!-- Card Actions -->
        <div class="card-actions">
          <button
            class="btn btn-detail btn-sm"
            @click="openDetails(skill)"
          >
            详情
          </button>
          <button
            v-if="!skill.installed && !store.isInstalling(skill.id)"
            class="btn btn-primary btn-sm"
            @click="handleInstall(skill)"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            安装
          </button>
          <button
            v-else-if="skill.installed && !store.isInstalling(skill.id)"
            class="btn btn-danger btn-sm"
            @click="handleUninstall(skill)"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
            </svg>
            卸载
          </button>
        </div>
      </div>
    </div>

    <!-- Details Dialog -->
    <AnthropicSkillDetailsDialog
      v-if="showDetails && selectedSkill"
      :skill="selectedSkill"
      :is-installed="selectedSkill.installed"
      @close="showDetails = false"
      @install="handleInstall"
      @uninstall="handleUninstall"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useAnthropicSkillsStore } from '@/stores/anthropic-skills';
import AnthropicSkillDetailsDialog from './AnthropicSkillDetailsDialog.vue';
import type { AnthropicSkill, SkillFilter, SkillSort } from '@/types';
import { useSkillImportStore } from '@/stores/skill-import';

const props = withDefaults(
  defineProps<{
    /** Source id to load on mount. Defaults to 'anthropic-official'. */
    defaultSourceId?: string;
  }>(),
  { defaultSourceId: 'anthropic-official' }
);

const store = useAnthropicSkillsStore();
const skillImportStore = useSkillImportStore();

// Local state
const searchKeyword = ref('');
const filterStatus = ref<SkillFilter>('all');
const sortOrder = ref<SkillSort>('name');
const showDetails = ref(false);
const selectedSkill = ref<AnthropicSkill | null>(null);

// Header cache badge — turns `cacheStatus` + `cachedAt` into a tiny
// human-readable label (e.g. "Cached · 3m ago", "Stale · click to refresh",
// "No cache — click refresh"). Hidden while loading or when there's an
// error so we don't distract the user.
const cacheBadge = computed<{
  label: string;
  tone: 'fresh' | 'stale' | 'missing' | 'error';
  tooltip: string;
} | null>(() => {
  switch (store.cacheStatus) {
    case 'fresh': {
      const ago = formatAgo(store.cachedAt);
      return {
        label: ago ? `已缓存 · ${ago}` : '已缓存',
        tone: 'fresh',
        tooltip: store.cachedAt
          ? `缓存于 ${new Date(store.cachedAt * 1000).toLocaleString()}`
          : '已从缓存加载',
      };
    }
    case 'stale': {
      const ago = formatAgo(store.cachedAt);
      return {
        label: ago ? `缓存过期 · ${ago}` : '缓存过期',
        tone: 'stale',
        tooltip: '点击刷新获取最新数据',
      };
    }
    case 'missing':
      return {
        label: '尚未缓存 · 点击刷新',
        tone: 'missing',
        tooltip: '首次加载需要联网获取数据',
      };
    case 'error':
      return {
        label: '加载失败 · 点击刷新',
        tone: 'error',
        tooltip: '读取缓存失败',
      };
    case 'loading':
    default:
      return null;
  }
});

function formatAgo(unixSecs: number | null): string {
  if (!unixSecs) return '';
  const diff = Math.max(0, Math.floor(Date.now() / 1000) - unixSecs);
  if (diff < 60) return `${diff}s 前`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m 前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h 前`;
  return `${Math.floor(diff / 86400)}d 前`;
}

// Lifecycle
onMounted(async () => {
  await store.listenProgress();

  // Make sure the source list is loaded so `activeSource` can be resolved.
  if (store.sources.length === 0) {
    await store.fetchSources();
  }

  // Apply the source selected by the parent (Marketplace source-tab)
  // before reading the cache.
  if (store.activeSourceId !== props.defaultSourceId) {
    await store.switchSource(props.defaultSourceId);
    // `switchSource` already calls `loadCachedOnly` internally, so we
    // only need to refresh the local-skill list here.
    await loadLocalSkillsOnly();
    return;
  }

  await loadData(false);
});

onUnmounted(() => {
  store.stopListening();
});

// Methods
async function loadData(skipFetchSources: boolean): Promise<void> {
  if (!skipFetchSources) {
    // Read from on-disk cache only — no GitHub network request. The user
    // must press the Refresh button in the header to actually hit the
    // network.
    await store.loadCachedOnly();
  }

  await loadLocalSkillsOnly();
}

async function loadLocalSkillsOnly(): Promise<void> {
  const targetDir = await skillImportStore.initDefaultSkillsDir();
  if (targetDir) {
    await store.fetchLocalSkills(targetDir);
  }
}

async function handleRefresh(): Promise<void> {
  // Explicit user action — this is the *only* code path that talks to
  // GitHub. After it succeeds the on-disk cache is updated and the
  // `cacheStatus` returns to `fresh`.
  await store.refreshList();
}

async function handleRetry(): Promise<void> {
  store.clearError();
  await loadData(false);
}

function handleSearch(): void {
  store.setSearchKeyword(searchKeyword.value);
}

function clearSearch(): void {
  searchKeyword.value = '';
  store.setSearchKeyword('');
}

function handleFilterChange(): void {
  store.setFilter(filterStatus.value);
}

function handleSortChange(): void {
  store.setSortBy(sortOrder.value);
}

async function handleInstall(skill: AnthropicSkill): Promise<void> {
  const targetDir = await skillImportStore.initDefaultSkillsDir();
  if (targetDir) {
    await store.install(skill.id, targetDir);
  }
}

async function handleUninstall(skill: AnthropicSkill): Promise<void> {
  const targetDir = await skillImportStore.initDefaultSkillsDir();
  if (targetDir) {
    await store.uninstall(skill.id, targetDir);
  }
}

function openDetails(skill: AnthropicSkill): void {
  selectedSkill.value = skill;
  showDetails.value = true;
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

function getProgressStageLabel(stage: string): string {
  const labels: Record<string, string> = {
    listing: '获取文件列表',
    downloading: '下载中',
    verifying: '验证中',
    complete: '完成',
    error: '错误',
  };
  return labels[stage] || stage;
}
</script>

<style scoped>
.anthropic-skills-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.header-left h2 {
  font-size: 18px;
  font-weight: 600;
}

.skill-count {
  font-size: 13px;
  color: var(--fg-muted);
  padding: 2px 8px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.cache-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 500;
  padding: 2px 10px;
  border-radius: 12px;
  border: 1px solid transparent;
  white-space: nowrap;
  user-select: none;
}

.cache-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}

.cache-badge--fresh {
  color: oklch(45% 0.12 145);
  background: oklch(95% 0.05 145 / 0.18);
  border-color: oklch(80% 0.08 145 / 0.5);
}
.cache-badge--fresh .cache-badge__dot {
  background: oklch(60% 0.15 145);
  box-shadow: 0 0 0 2px oklch(95% 0.05 145 / 0.4);
}

.cache-badge--stale {
  color: oklch(45% 0.10 55);
  background: oklch(95% 0.05 55 / 0.18);
  border-color: oklch(80% 0.08 55 / 0.5);
}
.cache-badge--stale .cache-badge__dot {
  background: oklch(70% 0.13 55);
}

.cache-badge--missing {
  color: var(--fg-muted);
  background: var(--bg-secondary);
  border-color: var(--border);
}
.cache-badge--missing .cache-badge__dot {
  background: var(--fg-ghost);
}

.cache-badge--error {
  color: oklch(50% 0.18 25);
  background: oklch(95% 0.05 25 / 0.18);
  border-color: oklch(80% 0.10 25 / 0.5);
}
.cache-badge--error .cache-badge__dot {
  background: oklch(60% 0.20 25);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.filter-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  height: 34px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  min-width: 240px;
  flex: 1;
  max-width: 360px;
}

.search-box svg {
  color: var(--fg-muted);
  flex-shrink: 0;
}

.search-box input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 13px;
  color: var(--fg);
  line-height: 1;
}

.clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--fg-muted);
  border-radius: 4px;
}

.clear-btn:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

.filter-group {
  display: flex;
  gap: 8px;
}

.filter-select {
  height: 34px;
  padding: 0 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 13px;
  line-height: 1;
  color: var(--fg);
  cursor: pointer;
}

.stats-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-left: auto;
  padding-left: 16px;
  border-left: 1px solid var(--border);
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.stat-value {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg);
  line-height: 1;
}

.stat-value.installed {
  color: var(--success);
}

.stat-value.available {
  color: var(--accent);
}

.stat-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 48px;
  color: var(--fg-muted);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 48px;
  color: var(--error);
  text-align: center;
}

.error-state svg {
  margin-bottom: 16px;
  opacity: 0.7;
}

.error-state p {
  margin-bottom: 16px;
  color: var(--fg-muted);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 64px;
  color: var(--fg-muted);
  text-align: center;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 4px;
  color: var(--fg);
}

.empty-state span {
  font-size: 14px;
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  overflow-y: auto;
  flex: 1;
  padding-bottom: 24px;
}

.skill-card {
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  transition: all 0.2s;
}

.skill-card:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-md);
}

.skill-card.installed {
  border-color: var(--success);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.skill-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.card-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 500;
  border-radius: 4px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.installed-badge {
  background: var(--success-bg);
  color: var(--success);
}

.version-badge {
  font-family: monospace;
}

.card-content {
  flex: 1;
  margin-bottom: 12px;
}

.skill-name {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 4px;
}

.skill-desc {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
}

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}

.tag {
  padding: 2px 6px;
  font-size: 10px;
  background: var(--accent-bg);
  color: var(--accent);
  border-radius: 4px;
}

.card-deps {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--fg-muted);
  margin-bottom: 8px;
}

.deps-label {
  font-weight: 500;
}

.deps-list {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.deps-more {
  color: var(--accent);
}

.card-author {
  font-size: 11px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

.install-progress {
  margin-bottom: 12px;
  padding: 10px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
  font-size: 11px;
  color: var(--fg-muted);
}

.progress-bar {
  height: 4px;
  background: var(--bg-secondary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-fill.complete {
  background: var(--success);
}

.progress-fill.error {
  background: var(--error);
}

.progress-message {
  margin-top: 6px;
  font-size: 10px;
  color: var(--fg-muted);
}

.card-actions {
  display: flex;
  gap: 8px;
  margin-top: auto;
}

.card-actions .btn {
  flex: 0 0 auto;
}

.card-actions .btn-primary,
.card-actions .btn-danger {
  margin-left: auto;
}

.sync-icon.spinning {
  animation: spin 1s linear infinite;
}
</style>

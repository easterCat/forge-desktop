<template>
  <div class="discover-view">
    <!-- Header -->
    <div class="view-header">
      <h2>发现技能</h2>
    </div>

    <!-- Sub-Tabs -->
    <div class="sub-tabs">
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'leaderboard' }"
        @click="switchTab('leaderboard')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 20V10M18 20V4M6 20v-4"/>
        </svg>
        Leaderboard
      </button>
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'search' }"
        @click="switchTab('search')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        Search
      </button>
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'curated' }"
        @click="switchTab('curated')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
        Curated
      </button>
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'browse' }"
        @click="switchTab('browse')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        Browse by Source
      </button>
    </div>

    <!-- Leaderboard Tab -->
    <div v-if="activeSubTab === 'leaderboard'" class="tab-content">
      <!-- View Toggle -->
      <div class="view-controls">
        <div class="view-toggles">
          <button
            class="toggle-btn"
            :class="{ active: store.view === 'all-time' }"
            @click="store.setView('all-time')"
          >
            All-time
          </button>
          <button
            class="toggle-btn"
            :class="{ active: store.view === 'trending' }"
            @click="store.setView('trending')"
          >
            Trending
          </button>
          <button
            class="toggle-btn"
            :class="{ active: store.view === 'hot' }"
            @click="store.setView('hot')"
          >
            Hot
          </button>
        </div>
      </div>

      <!-- Error State -->
      <div v-if="store.error" class="error-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <p>{{ store.error }}</p>
        <div class="error-actions">
          <button class="btn btn-secondary btn-sm" @click="store.fetchLeaderboard()">
            重试
          </button>
          <button
            class="btn btn-primary btn-sm"
            @click.stop="openExternalUrl('https://www.skills.sh/')"
          >
            在浏览器中打开
          </button>
        </div>
      </div>

      <!-- Loading State -->
      <div v-else-if="store.isLoading" class="loading-state">
        <div class="spinner"></div>
        <p>加载排行榜...</p>
      </div>

      <!-- Skills Grid -->
      <div v-else class="skills-grid">
        <SkillsShSkillCard
          v-for="skill in store.skills"
          :key="skill.id"
          :skill="skill"
          @click="openDetail"
          @detail="openDetail"
          @copy="copyInstallCommand"
          @install="installSkill"
        />
      </div>

      <!-- Load More -->
      <div v-if="store.pagination.hasMore && !store.isLoading" class="load-more">
        <button class="btn btn-secondary" :disabled="store.isLoadingMore" @click="store.loadMore()">
          {{ store.isLoadingMore ? '加载中...' : '加载更多' }}
        </button>
      </div>
    </div>

    <!-- Search Tab -->
    <div v-if="activeSubTab === 'search'" class="tab-content">
      <div class="search-section">
        <div class="search-box">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索 skills..."
            @input="handleSearch"
          />
          <button v-if="searchQuery" class="clear-btn" @click="clearSearch">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Search Error -->
      <div v-if="store.searchError" class="error-state">
        <p>{{ store.searchError }}</p>
        <button class="btn btn-primary btn-sm" @click.stop="openExternalUrl('https://www.skills.sh/')">
          在浏览器中打开
        </button>
      </div>

      <!-- Searching -->
      <div v-else-if="store.isSearching" class="loading-state">
        <div class="spinner"></div>
        <p>搜索中...</p>
      </div>

      <!-- Search Results -->
      <div v-else-if="searchQuery.length >= 2" class="skills-grid">
        <SkillsShSkillCard
          v-for="skill in store.searchResults"
          :key="skill.id"
          :skill="skill"
          @click="openDetail"
          @detail="openDetail"
          @copy="copyInstallCommand"
          @install="installSkill"
        />
        <div v-if="store.searchResults.length === 0" class="empty-state">
          <p>未找到匹配的技能</p>
          <span>尝试其他关键词</span>
        </div>
      </div>

      <div v-else class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <p>输入关键词搜索 skills</p>
        <span>至少输入 2 个字符</span>
      </div>
    </div>

    <!-- Curated Tab -->
    <div v-if="activeSubTab === 'curated'" class="tab-content">
      <!-- Loading -->
      <div v-if="store.isCuratedLoading" class="loading-state">
        <div class="spinner"></div>
        <p>加载官方集合...</p>
      </div>

      <!-- Error -->
      <div v-else-if="store.curatedError" class="error-state">
        <p>{{ store.curatedError }}</p>
        <div class="error-actions">
          <button class="btn btn-secondary btn-sm" @click="store.fetchCurated()">
            重试
          </button>
          <button class="btn btn-primary btn-sm" @click.stop="openExternalUrl('https://www.skills.sh/')">
            在浏览器中打开
          </button>
        </div>
      </div>

      <!-- Curated List -->
      <div v-else-if="store.curated" class="curated-list">
        <div class="curated-header">
          <span class="stat">{{ store.curated.totalOwners }} owners</span>
          <span class="stat">{{ formatNumber(store.curated.totalSkills) }} skills</span>
          <span class="generated-at">生成于 {{ formatDate(store.curated.generatedAt) }}</span>
        </div>
        <SkillsShSourceGroupView
          v-for="(group, index) in store.curated.data"
          :key="group.owner"
          :group="group"
          :default-expanded="index < 3"
          @skill-click="openDetail"
          @detail="openDetail"
        />
      </div>
    </div>

    <!-- Browse by Source Tab -->
    <div v-if="activeSubTab === 'browse'" class="tab-content">
      <!-- Loading -->
      <div v-if="store.isLoading" class="loading-state">
        <div class="spinner"></div>
        <p>加载来源...</p>
      </div>

      <!-- Error -->
      <div v-else-if="store.error" class="error-state">
        <p>{{ store.error }}</p>
        <button class="btn btn-primary btn-sm" @click.stop="openExternalUrl('https://www.skills.sh/')">
          在浏览器中打开
        </button>
      </div>

      <!-- Source Groups -->
      <div v-else class="source-list">
        <SkillsShSourceGroupView
          v-for="(group, index) in store.allSources"
          :key="group.source"
          :group="group"
          :default-expanded="index < 3"
          @skill-click="openDetail"
          @detail="openDetail"
        />
      </div>
    </div>

    <!-- Detail Dialog -->
    <SkillsShDetailDialog
      v-if="selectedSkill"
      :skill="selectedSkill"
      @close="selectedSkill = null"
      @installed="handleInstalled"
    />

    <!-- Success/Error Messages -->
    <Transition name="fade">
      <div v-if="successMessage" class="notification success-notification">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
          <polyline points="22 4 12 14.01 9 11.01"/>
        </svg>
        {{ successMessage }}
      </div>
    </Transition>
    <Transition name="fade">
      <div v-if="errorMessage" class="notification error-notification">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        {{ errorMessage }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSkillsShStore } from '@/stores/skills-sh';
import SkillsShSkillCard from './SkillsShSkillCard.vue';
import SkillsShSourceGroupView from './SkillsShSourceGroup.vue';
import SkillsShDetailDialog from './SkillsShDetailDialog.vue';
import type { SkillsShSkill } from '@/types';
import { open as openExternal } from '@tauri-apps/plugin-shell';

type SubTab = 'leaderboard' | 'search' | 'curated' | 'browse';

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

const activeSubTab = ref<SubTab>('leaderboard');
const searchQuery = ref('');
const selectedSkill = ref<SkillsShSkill | null>(null);
const successMessage = ref('');
const errorMessage = ref('');

let searchTimeout: number | null = null;

function switchTab(tab: SubTab) {
  activeSubTab.value = tab;

  if (tab === 'leaderboard' && store.skills.length === 0 && !store.isLoading) {
    store.fetchLeaderboard();
  } else if (tab === 'curated' && !store.curated && !store.isCuratedLoading) {
    store.fetchCurated();
  } else if (tab === 'browse' && store.allSources.length === 0 && !store.isLoading) {
    if (store.skills.length === 0) {
      store.fetchLeaderboard();
    }
  }
}

function handleSearch() {
  if (searchTimeout) clearTimeout(searchTimeout);
  searchTimeout = window.setTimeout(() => {
    if (searchQuery.value.length >= 2) {
      store.search(searchQuery.value);
    }
  }, 300);
}

function clearSearch() {
  searchQuery.value = '';
  store.clearSearch();
}

function openDetail(skill: SkillsShSkill) {
  selectedSkill.value = skill;
}

async function installSkill(skill: SkillsShSkill) {
  if (!skill.installUrl) {
    errorMessage.value = '此技能没有安装 URL';
    setTimeout(() => { errorMessage.value = ''; }, 3000);
    return;
  }

  const result = await store.installSkill(skill.installUrl, skill.slug);
  if (result.success) {
    successMessage.value = result.message;
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } else {
    errorMessage.value = result.message;
    setTimeout(() => { errorMessage.value = ''; }, 5000);
  }
}

async function copyInstallCommand(skill: SkillsShSkill) {
  const cmd = skill.installUrl
    ? `npx skills add ${skill.installUrl} --skill ${skill.slug}`
    : `npx skills add ${skill.source} --skill ${skill.slug}`;

  try {
    await navigator.clipboard.writeText(cmd);
    successMessage.value = '安装命令已复制到剪贴板';
    setTimeout(() => { successMessage.value = ''; }, 2000);
  } catch {
    errorMessage.value = '复制失败';
    setTimeout(() => { errorMessage.value = ''; }, 2000);
  }
}

function handleInstalled() {
  successMessage.value = '技能安装成功！';
  setTimeout(() => { successMessage.value = ''; }, 3000);
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

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString('zh-CN');
  } catch {
    return iso;
  }
}

// Load leaderboard on mount
onMounted(() => {
  if (store.skills.length === 0) {
    store.fetchLeaderboard();
  }
});
</script>

<style scoped>
.discover-view {
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
}

.view-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.sub-tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.sub-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.sub-tab:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

.sub-tab.active {
  background: var(--accent);
  color: white;
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}

.view-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.view-toggles {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.toggle-btn {
  padding: 6px 14px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.toggle-btn:hover {
  color: var(--fg);
}

.toggle-btn.active {
  background: var(--accent);
  color: white;
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.load-more {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}

.search-section {
  margin-bottom: 16px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 40px;
  padding: 0 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  max-width: 400px;
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
  font-size: 14px;
  color: var(--fg);
}

.clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: var(--fg-muted);
}

.clear-btn:hover {
  color: var(--fg);
}

.curated-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 12px;
}

.curated-header .stat {
  font-size: 13px;
  font-weight: 600;
  color: var(--fg);
}

.curated-header .generated-at {
  font-size: 11px;
  color: var(--fg-muted);
  margin-left: auto;
}

.source-list,
.curated-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
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
  to { transform: rotate(360deg); }
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
  color: var(--fg-muted);
  text-align: center;
}

.error-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.error-state p {
  font-size: 14px;
  margin-bottom: 16px;
  max-width: 400px;
}

.error-actions {
  display: flex;
  gap: 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
  color: var(--fg-muted);
  text-align: center;
  grid-column: 1 / -1;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 15px;
  font-weight: 500;
  color: var(--fg);
  margin-bottom: 4px;
}

.empty-state span {
  font-size: 13px;
}

/* Notifications */
.notification {
  position: fixed;
  top: 20px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
  z-index: var(--z-toast);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.success-notification {
  background: var(--success-bg, #10B98120);
  color: var(--success, #10B981);
  border: 1px solid var(--success, #10B981);
}

.error-notification {
  background: var(--error-bg, #EF444420);
  color: var(--error, #EF4444);
  border: 1px solid var(--error, #EF4444);
}

.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>

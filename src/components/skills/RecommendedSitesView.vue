<template>
  <div class="recommended-sites-view">
    <!-- Header -->
    <div class="view-header">
      <div class="header-left">
        <h2>推荐技能网站</h2>
        <span class="site-count">{{ store.count }} 个网站</span>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="onResetDefaults" title="恢复为内置推荐列表">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          恢复默认
        </button>
        <button class="btn btn-primary btn-sm" @click="openCreate">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增网站
        </button>
      </div>
    </div>

    <!-- Filter bar -->
    <div class="filter-section">
      <div class="search-box">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          v-model="searchKeyword"
          type="text"
          placeholder="搜索网站名称或介绍..."
        />
        <button v-if="searchKeyword" class="clear-btn" @click="searchKeyword = ''" aria-label="清除搜索">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      <div class="region-filter">
        <button
          v-for="opt in regionFilters"
          :key="opt.value"
          type="button"
          class="region-chip"
          :class="{ active: activeRegion === opt.value }"
          @click="activeRegion = opt.value"
        >
          {{ opt.label }}
          <span v-if="opt.value !== 'all'" class="chip-count">
            {{ countByRegion[opt.value] || 0 }}
          </span>
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="filteredSites.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p v-if="store.count === 0">还没有推荐网站</p>
      <p v-else>未找到匹配的网站</p>
      <span v-if="store.count === 0">点击右上角"新增网站"添加第一个</span>
      <span v-else>尝试调整搜索关键词或区域筛选</span>
    </div>

    <!-- Site grid -->
    <div v-else class="sites-grid">
      <article
        v-for="site in filteredSites"
        :key="site.id"
        class="site-card"
      >
        <div class="card-header">
          <div class="site-icon" :style="iconStyle(site.name)">
            {{ site.name.charAt(0).toUpperCase() }}
          </div>
          <span
            class="region-badge"
            :style="{ color: store.regionColor(site.region), borderColor: store.regionColor(site.region) + '40' }"
          >
            {{ store.regionLabel(site.region) }}
          </span>
        </div>

        <div class="card-content">
          <h3 class="site-name">{{ site.name }}</h3>
          <p class="site-desc">{{ site.description }}</p>
        </div>

        <div class="card-footer">
          <button
            class="host"
            :title="site.url"
            @click.stop="onJump(site)"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
            </svg>
            <span>{{ prettyHost(site.url) }}</span>
          </button>
          <div class="card-actions">
            <button class="btn btn-secondary btn-sm" @click="onJump(site)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                <polyline points="15 3 21 3 21 9"/>
                <line x1="10" y1="14" x2="21" y2="3"/>
              </svg>
              跳转
            </button>
            <button
              class="btn-icon btn-sm"
              title="编辑"
              aria-label="编辑"
              @click="openEdit(site)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            </button>
            <button
              class="btn-icon btn-sm btn-icon-danger"
              title="删除"
              aria-label="删除"
              @click="askDelete(site)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      </article>
    </div>

    <!-- Create / edit dialog -->
    <RecommendedSiteDialog
      v-if="dialogState !== 'closed'"
      :site="editingSite"
      @close="closeDialog"
      @submit="onDialogSubmit"
    />

    <!-- Delete confirmation -->
    <Teleport to="body">
      <div
        v-if="deletingSite"
        class="dialog-overlay"
        role="dialog"
        aria-modal="true"
        @click.self="cancelDelete"
      >
        <div class="confirm-dialog">
          <h3>删除推荐网站？</h3>
          <p>确定要从列表中移除"<strong>{{ deletingSite.name }}</strong>"吗？此操作无法撤销。</p>
          <div class="dialog-actions">
            <button class="btn btn-secondary" @click="cancelDelete">取消</button>
            <button class="btn btn-danger" @click="confirmDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, ref } from 'vue';
import { open as openExternal } from '@tauri-apps/plugin-shell';
import { useRecommendedSitesStore, type RecommendedSite, type SiteRegion } from '@/stores/recommended-sites';
import RecommendedSiteDialog from './RecommendedSiteDialog.vue';

const store = useRecommendedSitesStore();
const showNotification = inject<(message: string, type?: string) => void>('showNotification', () => {});

// Filter state
const searchKeyword = ref('');
const activeRegion = ref<SiteRegion | 'all'>('all');

const regionFilters: { value: SiteRegion | 'all'; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'international', label: '国际' },
  { value: 'china', label: '国内' },
  { value: 'github', label: 'GitHub' },
  { value: 'other', label: '其他' },
];

const countByRegion = computed(() => {
  const map: Partial<Record<SiteRegion, number>> = {};
  for (const s of store.sites) {
    map[s.region] = (map[s.region] || 0) + 1;
  }
  return map;
});

const filteredSites = computed(() => {
  const kw = searchKeyword.value.trim().toLowerCase();
  return store.sites.filter(site => {
    if (activeRegion.value !== 'all' && site.region !== activeRegion.value) {
      return false;
    }
    if (!kw) return true;
    return (
      site.name.toLowerCase().includes(kw) ||
      site.description.toLowerCase().includes(kw) ||
      site.url.toLowerCase().includes(kw)
    );
  });
});

// Dialog state: 'closed' | 'create' | 'edit'
const dialogState = ref<'closed' | 'create' | 'edit'>('closed');
const editingSite = ref<RecommendedSite | null>(null);
const deletingSite = ref<RecommendedSite | null>(null);

function openCreate(): void {
  editingSite.value = null;
  dialogState.value = 'create';
}

function openEdit(site: RecommendedSite): void {
  editingSite.value = site;
  dialogState.value = 'edit';
}

function closeDialog(): void {
  dialogState.value = 'closed';
  editingSite.value = null;
}

function onDialogSubmit(payload: { name: string; description: string; url: string; region: SiteRegion }): void {
  if (dialogState.value === 'edit' && editingSite.value) {
    const ok = store.updateSite(editingSite.value.id, payload);
    if (ok) showNotification(`已更新「${payload.name}」`, 'success');
  } else {
    const site = store.addSite(payload);
    if (site) showNotification(`已添加「${site.name}」`, 'success');
  }
  closeDialog();
}

function askDelete(site: RecommendedSite): void {
  deletingSite.value = site;
}

function cancelDelete(): void {
  deletingSite.value = null;
}

function confirmDelete(): void {
  if (!deletingSite.value) return;
  const target = deletingSite.value;
  const ok = store.removeSite(target.id);
  if (ok) showNotification(`已删除「${target.name}」`, 'success');
  deletingSite.value = null;
}

function onResetDefaults(): void {
  store.resetToDefaults();
  showNotification('已恢复为内置推荐列表', 'success');
}

// Open the URL in the system browser via the Tauri shell plugin when
// running inside the desktop app; fall back to a normal anchor click
// when running in plain `vite` dev mode (e.g. for browser-based UI work).
async function onJump(site: RecommendedSite): Promise<void> {
  try {
    await openExternal(site.url);
  } catch (e) {
    console.warn('shell.open failed, falling back to window.open:', e);
    try {
      window.open(site.url, '_blank', 'noopener,noreferrer');
    } catch (err) {
      console.error('Failed to open URL:', err);
      showNotification('无法打开链接', 'error');
    }
  }
}

function prettyHost(url: string): string {
  try {
    return new URL(url).host.replace(/^www\./, '');
  } catch {
    return url;
  }
}

const iconPalettes: [string, string][] = [
  ['#F59E0B', '#FBBF24'], // amber
  ['#06B6D4', '#0EA5E9'], // cyan
  ['#10B981', '#34D399'], // emerald
  ['#8B5CF6', '#A78BFA'], // violet
  ['#EC4899', '#F472B6'], // pink
  ['#3B82F6', '#60A5FA'], // blue
  ['#F97316', '#FB923C'], // orange
  ['#14B8A6', '#2DD4BF'], // teal
];

function iconStyle(name: string): Record<string, string> {
  const hash = name.split('').reduce((acc, ch) => {
    return ch.charCodeAt(0) + ((acc << 5) - acc);
  }, 0);
  const [from, to] = iconPalettes[Math.abs(hash) % iconPalettes.length];
  return {
    background: `linear-gradient(135deg, ${from} 0%, ${to} 100%)`,
  };
}
</script>

<style scoped>
.recommended-sites-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
  min-height: 0;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.header-left h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--fg);
}

.site-count {
  font-size: 12px;
  color: var(--fg-muted);
  padding: 2px 8px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.filter-section {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 12px;
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

.region-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.region-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  font-size: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.region-chip:hover {
  border-color: var(--border-hover);
  color: var(--fg);
}

.region-chip.active {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.chip-count {
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.12);
}

.region-chip.active .chip-count {
  background: rgba(255, 255, 255, 0.18);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 64px 16px;
  color: var(--fg-muted);
  text-align: center;
  gap: 4px;
}

.empty-state svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 15px;
  font-weight: 500;
  color: var(--fg);
  margin-bottom: 2px;
}

.empty-state span {
  font-size: 13px;
}

.sites-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  overflow-y: auto;
  flex: 1;
  padding-bottom: 16px;
  align-content: start;
}

.site-card {
  display: flex;
  flex-direction: column;
  padding: 18px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
}

.site-card:hover {
  border-color: var(--accent);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
  gap: 12px;
}

.site-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  color: white;
  font-size: 18px;
  font-weight: 600;
  flex-shrink: 0;
}

.region-badge {
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 500;
  border: 1px solid;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.card-content {
  flex: 1;
  margin-bottom: 14px;
}

.site-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 4px;
  word-break: break-word;
}

.site-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.55;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
  flex-wrap: wrap;
}

.host {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-muted);
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
}

.host:hover {
  color: var(--accent);
}

.card-actions {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-left: auto;
}

.card-actions .btn {
  flex: 0 0 auto;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-icon:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--bg-primary);
}

.btn-icon.btn-icon-danger:hover {
  border-color: var(--error);
  color: var(--error);
}

.confirm-dialog {
  position: relative;
  width: 100%;
  max-width: 420px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 20px;
  z-index: 1;
}

.confirm-dialog h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--fg);
}

.confirm-dialog p {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  margin-bottom: 16px;
}

.confirm-dialog .dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn.btn-danger {
  background: var(--error);
  color: white;
  border: 1px solid var(--error);
}

.btn.btn-danger:hover {
  background: var(--error);
  filter: brightness(1.1);
}
</style>

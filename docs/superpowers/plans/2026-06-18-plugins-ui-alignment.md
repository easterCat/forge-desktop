# Plugins UI Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align Sidebar and Plugins page (Installed / Marketplace / Sources) Vue components with the HTML prototype's structure and visual styling.

**Architecture:** Structural refactoring + visual alignment — preserve existing Pinia store logic and TypeScript types while restructuring DOM and CSS to match `design/theme/forge-cross-platform-glass.html`. New SourceCard component for Sources tab.

**Tech Stack:** Vue 3 + TypeScript + Pinia + Tauri (Rust backend)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src/components/layout/Sidebar.vue` | Logo, search, nav items, footer alignment |
| Modify | `src/views/PluginsView.vue` | Tab structure, Installed cards, Sources tab |
| Modify | `src/components/common/MarketplaceCard.vue` | Add Details button, author prefix |
| Create | `src/components/plugins/SourceCard.vue` | Source card component for Sources tab |
| Modify | `src/stores/plugin-marketplace.ts` | (No changes needed — all source methods exist) |
| Modify | `src/types/plugin-marketplace.ts` | (No changes needed — PluginSource type exists) |

---

## Task 1: Sidebar — Brand Logo

**Files:**
- Modify: `src/components/layout/Sidebar.vue:3-15`

- [ ] **Step 1: Replace brand logo SVG**

Replace the current gradient logo SVG with the prototype's solid rectangle design:

```vue
<template>
  <aside class="sidebar">
    <div class="sidebar-brand">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
        <rect x="2" y="2" width="20" height="20" rx="5" fill="#2D2D2D"/>
        <rect x="6" y="6" width="12" height="12" rx="2" fill="rgba(255,255,255,0.40)"/>
        <rect x="8" y="8" width="8" height="8" rx="1.5" fill="#F5F3F0"/>
        <rect x="10" y="10" width="4" height="4" rx="1" fill="#B8944A"/>
      </svg>
      <span>Forge</span>
    </div>
```

- [ ] **Step 2: Verify logo renders correctly**

Run: `pnpm dev` and check the sidebar brand area.
Expected: Dark rectangle with nested white/amber squares, "Forge" text next to it.

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "fix(sidebar): align brand logo with prototype design"
```

---

## Task 2: Sidebar — Search Input Styling

**Files:**
- Modify: `src/components/layout/Sidebar.vue:18-31` (template)
- Modify: `src/components/layout/Sidebar.vue` (style section)

- [ ] **Step 1: Update search input template**

Replace the current search input with glass-styled version:

```vue
    <div class="sidebar-search">
      <div class="sidebar-search-wrap">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          type="text"
          placeholder="Search…"
          v-model="searchQuery"
          @input="handleSearch"
        />
      </div>
    </div>
```

- [ ] **Step 2: Update search input CSS**

Replace the `.sidebar-search` and `.sidebar-search-wrap` styles:

```css
.sidebar-search {
  padding: 0 12px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.sidebar-search-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.sidebar-search-wrap svg {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.sidebar-search-wrap input {
  width: 100%;
  padding: 8px 10px 8px 34px;
  font-size: 13px;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.32);
  font-family: inherit;
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.sidebar-search-wrap input:focus {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.40);
}

.sidebar-search-wrap input::placeholder {
  color: var(--fg-ghost);
}
```

- [ ] **Step 3: Verify search input styling**

Run: `pnpm dev` and check the sidebar search input.
Expected: Glass background, subtle border, focus state with brighter background.

- [ ] **Step 4: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "fix(sidebar): align search input glass styling with prototype"
```

---

## Task 3: Sidebar — Nav Items and Labels

**Files:**
- Modify: `src/components/layout/Sidebar.vue:33-185` (template)
- Modify: `src/components/layout/Sidebar.vue` (style section)

- [ ] **Step 1: Fix nav item padding**

Update `.nav-item` CSS:

```css
.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-left: 3px solid transparent;
  color: var(--fg-muted);
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
  transition: all var(--t-fast);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.30);
  color: var(--fg);
}

.nav-item.active {
  background: rgba(255, 255, 255, 0.30);
  color: var(--fg-title);
  border-left-color: var(--accent);
}
```

- [ ] **Step 2: Fix section title letter-spacing**

Update `.nav-section-title` CSS:

```css
.nav-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--fg-ghost);
  padding: 16px 12px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.32);
}
```

- [ ] **Step 3: Fix "Mcps" label to "MCP Servers"**

In the template, change the MCP nav item label:

```vue
        <router-link
          to="/mcp"
          class="nav-item"
          :class="{ active: isActive('/mcp') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/>
          </svg>
          MCP Servers
          <span v-if="mcpCount > 0" class="nav-badge">{{ mcpCount }}</span>
        </router-link>
```

- [ ] **Step 4: Remove "Prompts" menu item**

Delete the Prompts `router-link` block (lines ~145-155 in the template).

- [ ] **Step 5: Verify nav items**

Run: `pnpm dev` and check sidebar navigation.
Expected: "MCP Servers" label, no "Prompts" item, correct padding and hover states.

- [ ] **Step 6: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "fix(sidebar): align nav items with prototype — padding, labels, remove Prompts"
```

---

## Task 4: Sidebar — Footer Border

**Files:**
- Modify: `src/components/layout/Sidebar.vue` (style section)

- [ ] **Step 1: Add footer border-top**

Update `.sidebar-footer` CSS:

```css
.sidebar-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px;
  margin-top: auto;
  border-top: 1px solid rgba(255, 255, 255, 0.30);
  font-size: 12px;
  color: var(--fg-ghost);
}
```

- [ ] **Step 2: Update footer text styles**

```css
.user-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
}

.user-status {
  font-size: 10px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}
```

- [ ] **Step 3: Verify footer**

Run: `pnpm dev` and check sidebar footer.
Expected: Border line above footer, correct font sizes.

- [ ] **Step 4: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "fix(sidebar): add footer border-top and align text styles"
```

---

## Task 5: PluginsView — Tab Structure

**Files:**
- Modify: `src/views/PluginsView.vue:1-15` (template header)
- Modify: `src/views/PluginsView.vue:346-352` (tabItems computed)

- [ ] **Step 1: Update section header**

Replace the current section header with prototype style:

```vue
    <!-- Section Header -->
    <div class="section-header">
      <h2>Plugins</h2>
      <span class="count">{{ installedPlugins.length }} installed</span>
    </div>
```

- [ ] **Step 2: Replace Updates tab with Sources**

Update the `tabItems` computed property:

```typescript
const tabItems = computed(() => [
  { id: 'installed', label: 'Installed', count: installedPlugins.value.length || undefined },
  { id: 'marketplace', label: 'Marketplace', count: totalMarketplacePlugins.value || undefined },
  { id: 'sources', label: 'Sources', count: sources.value.length || undefined },
])
```

- [ ] **Step 3: Add Sources tab content section**

Add the Sources tab section after the Marketplace section in the template:

```vue
    <!-- ─── Tab 3: Sources ─── -->
    <div v-show="activeTab === 'sources'" class="tab-content">
      <!-- Filter bar -->
      <div class="filter-bar">
        <SearchInput
          v-model="sourcesSearch"
          placeholder="Search sources…"
          class="sources-search"
        />
        <select class="filter-select" v-model="sourcesStatusFilter">
          <option value="all">All Status</option>
          <option value="installed">Installed</option>
          <option value="pending">Pending</option>
        </select>
        <button class="btn btn-primary btn-sm" @click="showAddSourceDialog = true">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          <span>Add Source</span>
        </button>
      </div>

      <!-- Source cards grid -->
      <div class="sources-grid">
        <SourceCard
          v-for="source in filteredSources"
          :key="source.id"
          :source="source"
          :status="getSourceStatus(source.id)"
          :notes="getSourceNotes(source.id)"
          @install="handleInstallSource(source)"
          @sync="handleSyncSource(source)"
        />
      </div>
    </div>
```

- [ ] **Step 4: Remove Updates tab section**

Delete the entire `v-show="activeTab === 'updates'"` section (lines ~240-309).

- [ ] **Step 5: Verify tab switching**

Run: `pnpm dev` and navigate to Plugins page.
Expected: 3 tabs (Installed, Marketplace, Sources), tab switching works.

- [ ] **Step 6: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "feat(plugins): replace Updates tab with Sources tab"
```

---

## Task 6: PluginsView — Sources Tab Logic

**Files:**
- Modify: `src/views/PluginsView.vue` (script section)

- [ ] **Step 1: Add sources state and imports**

Add to the script section:

```typescript
import SourceCard from '@/components/plugins/SourceCard.vue'
import AddRepoSourceDialog from '@/components/plugins/AddRepoSourceDialog.vue'

// ── Tab 3: Sources ───────────────────────────────────────────────────────────
const sourcesSearch = ref('')
const sourcesStatusFilter = ref<'all' | 'installed' | 'pending'>('all')
const showAddSourceDialog = ref(false)

const filteredSources = computed(() => {
  const q = sourcesSearch.value.toLowerCase().trim()
  return marketplaceStore.sources.filter(source => {
    const status = marketplaceStore.getSourceStatus(source.id)
    const isInstalled = status?.isInstalled ?? false

    // Status filter
    if (sourcesStatusFilter.value === 'installed' && !isInstalled) return false
    if (sourcesStatusFilter.value === 'pending' && isInstalled) return false

    // Search filter
    if (q) {
      const nameMatch = source.name.toLowerCase().includes(q)
      const urlMatch = source.command?.toLowerCase().includes(q) ?? false
      if (!nameMatch && !urlMatch) return false
    }

    return true
  })
})

function getSourceStatus(sourceId: string) {
  return marketplaceStore.getSourceStatus(sourceId)
}

function getSourceNotes(sourceId: string): string {
  return marketplaceStore.sourceNotes[sourceId] ?? ''
}

async function handleInstallSource(source: PluginSource) {
  await marketplaceStore.installSource(source.id)
}

async function handleSyncSource(source: PluginSource) {
  // Sync re-installs the source to refresh it
  await marketplaceStore.installSource(source.id)
}
```

- [ ] **Step 2: Load source status on mount**

Update the `onMounted` hook to also load source status:

```typescript
onMounted(async () => {
  isLoadingInstalled.value = true
  isLoadingMarketplace.value = true
  try {
    await Promise.all([
      marketplaceStore.fetchInstalledPlugins(),
      marketplaceStore.loadMarketplaceManifest(),
      marketplaceStore.loadSourceStatus(),
    ])
    if (currentSource.value) {
      await marketplaceStore.fetchPluginsBySource(currentSource.value.id)
    }
  } finally {
    isLoadingInstalled.value = false
    isLoadingMarketplace.value = false
  }
})
```

- [ ] **Step 3: Add AddRepoSourceDialog to template**

Add at the end of the template, before the closing `</div>`:

```vue
    <!-- Add Source Dialog -->
    <AddRepoSourceDialog
      v-if="showAddSourceDialog"
      @close="showAddSourceDialog = false"
    />
```

- [ ] **Step 4: Verify sources tab functionality**

Run: `pnpm dev`, navigate to Plugins → Sources.
Expected: Source cards displayed, search/filter works, Add Source dialog opens.

- [ ] **Step 5: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "feat(plugins): wire up Sources tab with store methods"
```

---

## Task 7: PluginsView — Installed Card Restructure

**Files:**
- Modify: `src/views/PluginsView.vue:64-135` (Installed tab template)

- [ ] **Step 1: Restructure installed plugin cards**

Replace the installed plugin card template with prototype structure:

```vue
      <!-- Plugin cards grid -->
      <div v-else class="plugin-list">
        <div
          v-for="plugin in filteredInstalled"
          :key="plugin.id"
          class="card plugin-card"
        >
          <div class="plugin-card-head">
            <div style="display:flex;align-items:center;gap:14px">
              <div class="plugin-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <div>
                <div class="plugin-name-row">
                  <span class="plugin-name">{{ plugin.name }}</span>
                  <span class="plugin-version">v{{ plugin.version || '—' }}</span>
                  <span v-if="getPluginSyncCount(plugin) > 0" class="sync-count">
                    {{ getPluginSyncCount(plugin) }} synced
                  </span>
                </div>
                <div class="plugin-meta">
                  {{ plugin.description || 'No description' }}
                  <template v-if="plugin.author"> · {{ plugin.author }}</template>
                  <template v-if="plugin.software"> · {{ plugin.software }}</template>
                </div>
              </div>
            </div>

            <div class="btn-group">
              <div
                class="toggle"
                :class="{ on: !plugin.disabled }"
                @click="handleToggle(plugin)"
                :title="plugin.disabled ? 'Enable plugin' : 'Disable plugin'"
              ></div>
              <button
                v-if="plugin.hasUpdate"
                class="btn btn-secondary btn-sm"
                @click="handleUpdate(plugin)"
              >
                Update
              </button>
              <button
                class="btn-icon btn-sm"
                @click="openPluginMenu(plugin)"
                title="More options"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- CLI Tools sync chips -->
          <div v-if="plugin.cliToolKeys && plugin.cliToolKeys.length > 0" class="plugin-cli-row">
            <span class="plugin-cli-label">CLI Tools</span>
            <CliSyncChip
              v-for="toolKey in plugin.cliToolKeys"
              :key="toolKey"
              :tool-key="toolKey"
              :tool-name="getCliMeta(toolKey).name"
              :tool-icon="getCliMeta(toolKey).icon"
              :tool-color="getCliMeta(toolKey).color"
              :state="getPluginSyncState(plugin, toolKey)"
              @click="handleCliSync(plugin, $event as any)"
            />
          </div>
        </div>
      </div>
```

- [ ] **Step 2: Add sync count helper**

Add to the script section:

```typescript
function getPluginSyncCount(plugin: MarketplacePlugin): number {
  if (!plugin.cliToolKeys) return 0
  return plugin.cliToolKeys.filter(key => {
    const syncKey = `${plugin.sourceId}::${plugin.name}::${key}`
    return syncState.value[syncKey] === 'synced'
  }).length
}
```

- [ ] **Step 3: Update installed card CSS**

Replace the `.card.plugin-card` and related styles:

```css
.card.plugin-card {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 20px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  overflow: hidden;
  position: relative;
}

.card.plugin-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.plugin-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.plugin-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.plugin-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.plugin-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
}

.plugin-version {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-ghost);
}

.sync-count {
  font-size: 10px;
  font-weight: 700;
  font-family: var(--font-mono);
  padding: 2px 8px;
  border-radius: 99px;
  background: rgba(90, 138, 100, 0.10);
  color: var(--success);
  border: 1px solid rgba(90, 138, 100, 0.20);
}

.plugin-meta {
  font-size: 12px;
  color: var(--fg-ghost);
  line-height: 1.5;
  margin-top: 2px;
}

.plugin-cli-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 12px;
  margin-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.10);
}

.plugin-cli-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-ghost);
  margin-right: 2px;
  white-space: nowrap;
}
```

- [ ] **Step 4: Verify installed cards**

Run: `pnpm dev`, navigate to Plugins → Installed.
Expected: Cards match prototype structure with icon, name, version, meta, toggle, CLI sync chips.

- [ ] **Step 5: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "fix(plugins): restructure Installed cards to match prototype"
```

---

## Task 8: MarketplaceCard — Visual Alignment

**Files:**
- Modify: `src/components/common/MarketplaceCard.vue`

- [ ] **Step 1: Add "by " prefix to author**

Update the author section in the template:

```vue
        <div class="marketplace-card-author">
          <template v-if="author">by {{ author }}</template>
          <span v-if="sourceBadge" class="source-badge">{{ sourceBadge }}</span>
        </div>
```

- [ ] **Step 2: Add Details button to footer**

Update the footer section:

```vue
    <div class="marketplace-card-footer">
      <span v-if="version" class="version">v{{ version }}</span>
      <div class="btn-group">
        <button
          class="btn btn-icon"
          @click.stop="emit('click')"
          title="Details"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
        </button>
        <button
          v-if="!installed"
          class="btn btn-primary btn-sm"
          @click.stop="emit('install')"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <span>Install</span>
        </button>
        <button v-else class="btn btn-secondary btn-sm">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          <span>Installed</span>
        </button>
      </div>
    </div>
```

- [ ] **Step 3: Update footer CSS for btn-icon**

Add to the style section:

```css
.marketplace-card-footer .btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.marketplace-card-footer .btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
```

- [ ] **Step 4: Verify marketplace cards**

Run: `pnpm dev`, navigate to Plugins → Marketplace.
Expected: Cards show "by author", Details button, Install/Installed buttons with icons.

- [ ] **Step 5: Commit**

```bash
git add src/components/common/MarketplaceCard.vue
git commit -m "fix(marketplace): align card footer with prototype — Details button, author prefix"
```

---

## Task 9: SourceCard — New Component

**Files:**
- Create: `src/components/plugins/SourceCard.vue`

- [ ] **Step 1: Create SourceCard component**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import type { PluginSource, SourceStatus } from '@/types'

interface Props {
  source: PluginSource
  status?: SourceStatus
  notes?: string
}

const props = withDefaults(defineProps<Props>(), {
  status: undefined,
  notes: '',
})

const emit = defineEmits<{
  install: []
  sync: []
}>()

const isInstalled = computed(() => props.status?.isInstalled ?? false)

const repoTypeLabel = computed(() => {
  switch (props.source.repoType) {
    case 'market': return 'Market'
    case 'res': return 'Resource'
    default: return props.source.repoType ?? 'Unknown'
  }
})

const sourceUrl = computed(() => {
  return props.source.command ?? ''
})

const installedPath = computed(() => {
  return props.status?.installedPath ?? props.status?.installedPaths?.[0] ?? '—'
})
</script>

<template>
  <div class="card source-card">
    <div class="source-card-head">
      <div class="source-card-icon">
        <svg v-if="source.repoType === 'market'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
          <polyline points="9 22 9 12 15 12 15 22"/>
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
      </div>
      <div style="flex:1;min-width:0">
        <div class="source-card-title">
          <span class="source-name-text">{{ source.name }}</span>
          <span v-if="isInstalled" class="badge success">Installed</span>
          <span v-else class="badge warn">Pending</span>
        </div>
        <div class="source-card-subtitle">
          {{ repoTypeLabel }} · {{ source.pluginCount ?? 0 }} plugins
        </div>
      </div>
    </div>

    <div v-if="notes" class="source-card-notes" :title="notes">{{ notes }}</div>

    <div v-if="sourceUrl" class="source-card-url" :title="sourceUrl">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
      </svg>
      <span class="url-text">{{ sourceUrl }}</span>
      <a class="url-link" :href="sourceUrl" target="_blank" title="Open in browser">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
          <polyline points="15 3 21 3 21 9"/>
          <line x1="10" y1="14" x2="21" y2="3"/>
        </svg>
      </a>
    </div>

    <div class="source-card-path">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <span class="path-text">{{ installedPath }}</span>
    </div>

    <div class="source-card-footer">
      <div class="btn-group">
        <button class="btn-icon btn-sm" title="View details">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
        </button>
        <button class="btn-icon btn-sm" title="Edit notes">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </button>
        <button class="btn-icon btn-sm" title="More">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
          </svg>
        </button>
      </div>
      <button
        v-if="isInstalled"
        class="btn btn-secondary btn-sm"
        @click="emit('sync')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
        <span>Sync</span>
      </button>
      <button
        v-else
        class="btn btn-primary btn-sm"
        @click="emit('install')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        <span>Install</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.source-card {
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 14px;
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  overflow: hidden;
  position: relative;
}

.source-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.source-card-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.source-card-icon {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.source-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.source-name-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-subtitle {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
}

.source-card-notes {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-url {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.source-card-url svg {
  flex-shrink: 0;
  opacity: 0.4;
}

.source-card-url .url-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-url .url-link {
  flex-shrink: 0;
  color: var(--fg-ghost);
  opacity: 0.4;
  transition: all var(--t-fast);
  cursor: pointer;
  display: flex;
  align-items: center;
}

.source-card-url .url-link:hover {
  opacity: 1;
  color: var(--accent);
}

.source-card-path {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.source-card-path svg {
  flex-shrink: 0;
  opacity: 0.4;
}

.source-card-path .path-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid var(--border);
  padding-top: 14px;
}

.source-card-footer .btn-group {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.source-card-footer .btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.source-card-footer .btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.01em;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.badge.success {
  background: rgba(90, 138, 100, 0.15);
  border: 1px solid rgba(90, 138, 100, 0.20);
  color: var(--success);
}

.badge.success::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--success);
}

.badge.warn {
  background: rgba(184, 148, 74, 0.15);
  border: 1px solid rgba(184, 148, 74, 0.20);
  color: var(--warn);
}

.badge.warn::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--warn);
}
</style>
```

- [ ] **Step 2: Verify SourceCard renders**

Run: `pnpm dev`, navigate to Plugins → Sources.
Expected: Source cards with icon, name, status badge, URL, path, action buttons.

- [ ] **Step 3: Commit**

```bash
git add src/components/plugins/SourceCard.vue
git commit -m "feat(plugins): add SourceCard component for Sources tab"
```

---

## Task 10: PluginsView — Sources Grid CSS

**Files:**
- Modify: `src/views/PluginsView.vue` (style section)

- [ ] **Step 1: Add sources grid styles**

Add to the `<style scoped>` section:

```css
/* ── Sources ──────────────────────────────────────────────────────────────── */
.sources-search {
  flex: 1;
  min-width: 200px;
  max-width: 360px;
}

.sources-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}
```

- [ ] **Step 2: Verify sources grid layout**

Run: `pnpm dev`, navigate to Plugins → Sources.
Expected: Responsive grid of source cards, 2+ columns on wide screens.

- [ ] **Step 3: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "style(plugins): add sources grid layout"
```

---

## Task 11: Integration — Toggle and Section Header Count

**Files:**
- Modify: `src/views/PluginsView.vue`

- [ ] **Step 1: Add toggle CSS**

Add to the style section (if not already present from Task 7):

```css
/* ── Toggle ─────────────────────────────────────────────────────────────── */
.toggle {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.22);
  border: 1px solid rgba(255, 255, 255, 0.32);
  position: relative;
  cursor: pointer;
  transition: background var(--t-base);
  flex-shrink: 0;
}

.toggle.on {
  background: var(--accent);
}

.toggle::after {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: white;
  top: 2px;
  left: 2px;
  transition: transform var(--t-base);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.toggle.on::after {
  transform: translateX(16px);
}
```

- [ ] **Step 2: Verify toggle interaction**

Run: `pnpm dev`, navigate to Plugins → Installed.
Expected: Toggle switches on/off, persists state via store.

- [ ] **Step 3: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "fix(plugins): add toggle CSS for installed cards"
```

---

## Task 12: Final Verification

- [ ] **Step 1: Run full dev server**

```bash
pnpm dev
```

- [ ] **Step 2: Verify all 5 components**

1. **Sidebar**: Logo, search glass style, nav items, "MCP Servers" label, no "Prompts", footer border
2. **PluginsView tabs**: Installed / Marketplace / Sources (no Updates)
3. **Installed cards**: Icon, name, version, meta, toggle, CLI sync chips
4. **Marketplace cards**: "by author", Details button, Install/Installed with icons
5. **Sources tab**: Search, filter, Add Source button, source cards grid

- [ ] **Step 3: Run type check**

```bash
pnpm type-check
```

Expected: No TypeScript errors.

- [ ] **Step 4: Run lint**

```bash
pnpm lint
```

Expected: No lint errors (or only pre-existing ones).

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete Plugins UI alignment with prototype

- Sidebar: logo, search glass, nav items, footer
- PluginsView: replace Updates with Sources tab
- Installed: restructured cards with CLI sync
- Marketplace: Details button, author prefix
- Sources: new SourceCard component with full layout"
```

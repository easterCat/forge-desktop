# Sidebar Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Completely rewrite `Sidebar.vue` to precisely match the design reference `forge-cross-platform-glass.html`, including adding the Import/Export route.

**Architecture:** Single-file Vue 3 component with scoped CSS using CSS variables. All styling values are taken directly from the design reference's CSS. Navigation uses `router-link` with Vue Router.

**Tech Stack:** Vue 3, TypeScript, Vue Router, CSS Variables (from `src/assets/theme.css` and `src/assets/tokens/`)

---

## File Structure

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/views/ImportExportView.vue` | Placeholder view for Import/Export page |
| Modify | `src/router/index.ts:59` | Add `/import-export` route |
| Rewrite | `src/components/layout/Sidebar.vue` | Complete rewrite matching design reference |

---

### Task 1: Add Import/Export Route

**Files:**
- Create: `src/views/ImportExportView.vue`
- Modify: `src/router/index.ts:59`

- [ ] **Step 1: Create ImportExportView.vue placeholder**

Create `src/views/ImportExportView.vue`:

```vue
<template>
  <div class="section-header">
    <h2>Import / Export</h2>
  </div>
  <div class="settings-grid">
    <div class="setting-group">
      <h4>Export Configuration</h4>
      <p style="font-size:13px;color:var(--fg-muted);margin-bottom:12px">
        Package your configs as a portable .forge file
      </p>
      <button class="btn btn-primary">Export as .forge</button>
    </div>
    <div class="setting-group">
      <h4>Import Configuration</h4>
      <p style="font-size:13px;color:var(--fg-muted);margin-bottom:12px">
        Import configs from a .forge package or Git repo
      </p>
      <button class="btn btn-secondary">Import from Git Repository</button>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
```

- [ ] **Step 2: Add route to router/index.ts**

Open `src/router/index.ts` and add the following route entry after the `/backup` route (after line 42):

```typescript
  {
    path: '/import-export',
    name: 'ImportExport',
    component: () => import('@/views/ImportExportView.vue'),
  },
```

The routes array should now have `/import-export` between `/backup` and `/settings`.

- [ ] **Step 3: Verify dev server starts without errors**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite --host 2>&1 | head -20`
Expected: Server starts, no route errors.

- [ ] **Step 4: Commit**

```bash
git add src/views/ImportExportView.vue src/router/index.ts
git commit -m "feat(router): add Import/Export route and placeholder view"
```

---

### Task 2: Rewrite Sidebar.vue Template

**Files:**
- Rewrite: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: Replace the entire template section**

Replace the entire `<template>` section of `src/components/layout/Sidebar.vue` with:

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

    <nav class="sidebar-nav">
      <!-- Overview Section -->
      <div class="nav-section">
        <div class="nav-section-title">Overview</div>
        <router-link
          to="/"
          class="nav-item"
          :class="{ active: isActive('/') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <rect x="3" y="3" width="7" height="7" rx="1"/>
            <rect x="14" y="3" width="7" height="7" rx="1"/>
            <rect x="3" y="14" width="7" height="7" rx="1"/>
            <rect x="14" y="14" width="7" height="7" rx="1"/>
          </svg>
          Dashboard
        </router-link>
      </div>

      <!-- Manage Section -->
      <div class="nav-section">
        <div class="nav-section-title">Manage</div>
        <router-link
          to="/cli-tools"
          class="nav-item"
          :class="{ active: isActive('/cli-tools') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="4 17 10 11 4 5"/>
            <line x1="12" y1="19" x2="20" y2="19"/>
          </svg>
          CLI Tools
          <span v-if="cliToolsCount > 0" class="nav-badge">{{ cliToolsCount }}</span>
        </router-link>

        <router-link
          to="/software"
          class="nav-item"
          :class="{ active: isActive('/software') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          Software
          <span v-if="softwareCount > 0" class="nav-badge">{{ softwareCount }}</span>
        </router-link>

        <router-link
          to="/plugins"
          class="nav-item"
          :class="{ active: isActive('/plugins') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
          </svg>
          Plugins
          <span v-if="pluginsCount > 0" class="nav-badge">{{ pluginsCount }}</span>
        </router-link>

        <router-link
          to="/skills"
          class="nav-item"
          :class="{ active: isActive('/skills') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
          Skills
          <span v-if="skillsCount > 0" class="nav-badge">{{ skillsCount }}</span>
        </router-link>

        <router-link
          to="/agents"
          class="nav-item"
          :class="{ active: isActive('/agents') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
          Agents
          <span v-if="agentsCount > 0" class="nav-badge">{{ agentsCount }}</span>
        </router-link>

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

        <router-link
          to="/rules"
          class="nav-item"
          :class="{ active: isActive('/rules') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
          </svg>
          Rules
          <span v-if="rulesCount > 0" class="nav-badge">{{ rulesCount }}</span>
        </router-link>
      </div>

      <!-- Data Section -->
      <div class="nav-section">
        <div class="nav-section-title">Data</div>
        <router-link
          to="/backup"
          class="nav-item"
          :class="{ active: isActive('/backup') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          Backup & Restore
        </router-link>

        <router-link
          to="/import-export"
          class="nav-item"
          :class="{ active: isActive('/import-export') }"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="16 3 21 3 21 8"/>
            <line x1="4" y1="20" x2="21" y2="3"/>
            <polyline points="21 16 21 21 16 21"/>
            <line x1="15" y1="15" x2="21" y2="21"/>
          </svg>
          Import / Export
        </router-link>
      </div>
    </nav>

    <div class="sidebar-footer">
      <div class="avatar">R</div>
      <div>
        <div class="user-name">rhino</div>
        <div class="user-status">Local only · v{{ appVersion }}</div>
      </div>
    </div>
  </aside>
</template>
```

- [ ] **Step 2: Verify template compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No template errors (store type errors are acceptable).

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "refactor(sidebar): rewrite template to match design reference"
```

---

### Task 3: Rewrite Sidebar.vue Script

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: Replace the entire script section**

Replace the entire `<script setup lang="ts">` section of `Sidebar.vue` with:

```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoute } from 'vue-router';
import { useSoftwareStore } from '@/stores/software';
import { usePluginStore } from '@/stores/plugin';
import { useSkillStore } from '@/stores/skill';
import { useMCPStore } from '@/stores/mcp';
import { useRuleStore } from '@/stores/rule';
import { useAgentStore } from '@/stores/agent';

const softwareStore = useSoftwareStore();
const pluginStore = usePluginStore();
const skillStore = useSkillStore();
const mcpStore = useMCPStore();
const ruleStore = useRuleStore();
const agentStore = useAgentStore();

const route = useRoute();

const searchQuery = ref('');
const appVersion = '0.1.0';

const isActive = (path: string) => {
  return route.path === path;
};

const cliToolsCount = computed(() => softwareStore.cliTools.length);
const softwareCount = computed(() => softwareStore.softwareList.length);
const pluginsCount = computed(() => pluginStore.plugins.length);
const skillsCount = computed(() => skillStore.skills.length);
const mcpCount = computed(() => mcpStore.services.length);
const rulesCount = computed(() => ruleStore.rules.length);
const agentsCount = computed(() => agentStore.agents.length);

const handleSearch = () => {
  console.log('Search:', searchQuery.value);
};
</script>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "refactor(sidebar): rewrite script with appVersion and clean imports"
```

---

### Task 4: Rewrite Sidebar.vue Styles

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: Replace the entire style section**

Replace the entire `<style scoped>` section of `Sidebar.vue` with:

```vue
<style scoped>
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.40);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid rgba(255, 255, 255, 0.22);
  z-index: 10;
}

.sidebar-brand {
  height: var(--topbar-h);
  min-height: var(--topbar-h);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.30);
}

.sidebar-brand svg {
  flex-shrink: 0;
}

.sidebar-brand span {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--fg-title);
}

.sidebar-search {
  padding: 0 12px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.sidebar-search-wrap {
  position: relative;
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

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px;
}

.nav-section {
  margin-bottom: 4px;
}

.nav-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--fg-ghost);
  padding: 16px 12px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.32);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  cursor: pointer;
  border-left: 3px solid transparent;
  text-decoration: none;
  transition: all var(--t-fast);
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

.nav-item svg {
  flex-shrink: 0;
  opacity: 0.5;
}

.nav-item.active svg {
  opacity: 1;
}

.nav-badge {
  margin-left: auto;
  font-size: 11px;
  font-weight: 600;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.sidebar-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.30);
  font-size: 12px;
  color: var(--fg-ghost);
}

.sidebar-footer .avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid rgba(255, 255, 255, 0.32);
  color: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 11px;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.sidebar-footer .user-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
}

.sidebar-footer .user-status {
  font-size: 10px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

/* Responsive: hide sidebar on mobile */
@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
}
</style>
```

- [ ] **Step 2: Verify no CSS syntax errors**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -10`
Expected: Build succeeds (or only unrelated errors).

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "style(sidebar): rewrite styles to exactly match design reference"
```

---

### Task 5: Update AppFrame Page Titles

**Files:**
- Modify: `src/components/layout/AppFrame.vue:59-71`

- [ ] **Step 1: Add Import/Export to page titles**

In `src/components/layout/AppFrame.vue`, find the `pageTitles` object (around line 59) and add the Import/Export entry:

```typescript
const pageTitles: Record<string, string> = {
  '/': 'Dashboard',
  '/cli-tools': 'CLI Tools',
  '/software': 'Software',
  '/plugins': 'Plugins',
  '/skills': 'Skills',
  '/mcp': 'MCP Servers',
  '/rules': 'Rules',
  '/backup': 'Backup & Restore',
  '/import-export': 'Import / Export',
  '/settings': 'Settings',
  '/prompts': 'Prompt Manager',
  '/agents': 'Agents',
};
```

- [ ] **Step 2: Commit**

```bash
git add src/components/layout/AppFrame.vue
git commit -m "feat(layout): add Import/Export to page titles"
```

---

### Task 6: Final Verification

- [ ] **Step 1: Run TypeScript check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | tail -20`
Expected: No new errors introduced by sidebar changes.

- [ ] **Step 2: Run dev server and visually verify**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite --host 2>&1 | head -10`
Expected: Dev server starts, sidebar renders correctly with all nav items.

- [ ] **Step 3: Verify all routes work**

Click each sidebar nav item and verify:
- Dashboard (`/`) loads
- CLI Tools (`/cli-tools`) loads with badge
- Software (`/software`) loads with badge
- Plugins (`/plugins`) loads with badge
- Skills (`/skills`) loads with badge
- Agents (`/agents`) loads with badge
- MCP Servers (`/mcp`) loads with badge
- Rules (`/rules`) loads with badge
- Backup & Restore (`/backup`) loads
- Import / Export (`/import-export`) loads

- [ ] **Step 4: Verify responsive behavior**

Resize browser to ≤768px width. Sidebar should hide. MobileTabbar should appear.

- [ ] **Step 5: Final commit with all changes**

```bash
git add -A
git commit -m "feat(sidebar): complete rewrite matching design reference

- Rewrite Sidebar.vue template, script, and styles
- Add Import/Export route and placeholder view
- Update AppFrame page titles
- All CSS values match forge-cross-platform-glass.html exactly"
```

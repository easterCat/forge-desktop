<template>
  <aside class="sidebar" :class="{ collapsed }">
    <div class="sidebar-brand">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
        <rect x="2" y="2" width="20" height="20" rx="5" fill="#2D2D2D"/>
        <rect x="6" y="6" width="12" height="12" rx="2" fill="rgba(255,255,255,0.40)"/>
        <rect x="8" y="8" width="8" height="8" rx="1.5" fill="#F5F3F0"/>
        <rect x="10" y="10" width="4" height="4" rx="1" fill="#B8944A"/>
      </svg>
      <span>Forge</span>
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
          <span class="nav-label">Dashboard</span>
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
          <span class="nav-label">CLI Tools</span>
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
          <span class="nav-label">Software</span>
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
          <span class="nav-label">Plugins</span>
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
          <span class="nav-label">Skills</span>
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
          <span class="nav-label">Agents</span>
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
          <span class="nav-label">MCP Servers</span>
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
          <span class="nav-label">Rules</span>
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
          <span class="nav-label">Backup & Restore</span>
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
          <span class="nav-label">Import / Export</span>
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

<script setup lang="ts">
defineProps<{
  collapsed?: boolean
}>();

import { computed } from 'vue';
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
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--glass-sidebar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid rgba(255, 255, 255, 0.22);
  z-index: 10;
  flex-shrink: 0;
  transition: width var(--t-base), min-width var(--t-base);
  position: relative;
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
  transition: opacity var(--t-fast);
  white-space: nowrap;
  overflow: hidden;
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
  padding: 16px 8px 6px 8px;
  transition: opacity var(--t-fast);
  white-space: nowrap;
  overflow: hidden;
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
  transition: all var(--t-fast), padding var(--t-base), justify-content var(--t-base);
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
  transition: opacity var(--t-fast);
}

.nav-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: opacity var(--t-fast);
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
  transition: opacity var(--t-fast);
}

.sidebar-footer .user-status {
  font-size: 10px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
  transition: opacity var(--t-fast);
}

/* Sidebar collapse state — matches prototype */
.sidebar.collapsed {
  width: 60px;
  min-width: 60px;
  overflow: hidden;
}

.sidebar.collapsed .sidebar-brand span {
  display: none;
}

.sidebar.collapsed .sidebar-nav {
  padding: 0 6px;
}

.sidebar.collapsed .nav-section-title {
  display: none;
}

.sidebar.collapsed .nav-item {
  justify-content: center;
  padding: 10px 0;
  border-left: none;
}

.sidebar.collapsed .nav-label {
  display: none;
}

.sidebar.collapsed .nav-item svg {
  margin-right: 0;
}

.sidebar.collapsed .nav-badge {
  display: none;
}

.sidebar.collapsed .sidebar-footer {
  padding: 12px 0;
  justify-content: center;
}

.sidebar.collapsed .sidebar-footer > div:not(.avatar) {
  display: none;
}

/* Responsive: at tablet width, auto-collapse */
@media (max-width: 1024px) {
  .sidebar {
    width: 60px;
    min-width: 60px;
    overflow: hidden;
  }

  .sidebar .sidebar-brand span {
    display: none;
  }

  .sidebar .sidebar-nav {
    padding: 0 6px;
  }

  .sidebar .nav-section-title,
  .sidebar .nav-label {
    display: none;
  }

  .sidebar .nav-item {
    justify-content: center;
    padding: 10px 0;
    border-left: none;
  }

  .sidebar .nav-item svg {
    margin-right: 0;
  }

  .sidebar .nav-badge {
    display: none;
  }

  .sidebar .sidebar-footer {
    padding: 12px 0;
    justify-content: center;
  }

  .sidebar .sidebar-footer > div:not(.avatar) {
    display: none;
  }

  /* Fully hide when collapsed at tablet */
  .sidebar.collapsed {
    width: 0;
    min-width: 0;
    padding: 0;
    border: none;
  }
}

/* Responsive: hide sidebar on mobile */
@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
}
</style>

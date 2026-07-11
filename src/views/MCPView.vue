<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>MCP Servers</h2>
      <span class="count">{{ servers.length }} servers</span>
    </div>

    <!-- Tab Bar -->
    <TabBar v-model="activeTab" :tabs="tabItems" />

    <!-- Services Tab -->
    <div v-show="activeTab === 'services'" style="display: contents">
      <!-- Filter Bar -->
      <div class="filter-bar">
        <div class="search-input">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search MCP servers…"
            @input="handleSearch"
          />
          <button v-if="searchQuery" class="clear-btn" @click="searchQuery = ''">
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="#9A9A9A" stroke-width="1.5" stroke-linecap="round">
              <line x1="2" y1="2" x2="8" y2="8"/>
              <line x1="8" y1="2" x2="2" y2="8"/>
            </svg>
          </button>
        </div>
        <select v-model="healthFilter" class="filter-select" @change="handleFilter">
          <option value="all">All Status</option>
          <option value="healthy">Healthy</option>
          <option value="unhealthy">Unreachable</option>
        </select>
        <select v-model="authFilter" class="filter-select" @change="handleFilter">
          <option value="all">All Auth</option>
          <option value="none">No Auth</option>
          <option value="oauth">OAuth</option>
          <option value="token">Token</option>
        </select>
        <div class="btn-group">
          <button class="btn btn-secondary btn-sm" @click="handleDiscover">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/>
            </svg>
            Discover
          </button>
          <button class="btn btn-primary btn-sm" @click="handleAddServer">+ Add Server</button>
        </div>
      </div>

      <!-- Server Card Grid -->
      <div v-if="filteredServers.length > 0" class="card-grid">
        <div
          v-for="server in filteredServers"
          :key="server.name"
          class="card mcp-card"
        >
          <div class="card-head">
            <div
              class="card-icon"
              :style="{
                background: server.healthy ? 'rgba(90,138,100,0.10)' : 'rgba(184,90,66,0.10)',
                color: server.healthy ? 'var(--success)' : 'var(--error)'
              }"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/>
              </svg>
            </div>
            <div style="flex: 1; min-width: 0">
              <div class="card-title">
                {{ server.name }}
                <span class="badge" :class="server.healthy ? 'success' : 'error'">
                  {{ server.healthy ? 'Healthy' : 'Unreachable' }}
                </span>
              </div>
              <div class="card-subtitle">{{ server.endpoint }}</div>
            </div>
          </div>
          <div class="card-meta">
            <div class="card-meta-item"><span class="label">Auth</span><span class="value">{{ server.auth }}</span></div>
            <div class="card-meta-item"><span class="label">Tools</span><span class="value">{{ server.tools }}</span></div>
            <div class="card-meta-item"><span class="label">Last Check</span><span class="value">{{ server.lastChecked }}</span></div>
            <div class="card-meta-item"><span class="label">Group</span><span class="value">{{ server.group || 'Default' }}</span></div>
          </div>
          <div class="card-divider"></div>
          <div class="card-footer">
            <div class="card-footer-left">
              <span class="badge" :class="server.healthy ? 'success' : 'error'">
                {{ server.healthy ? 'Connected' : 'Disconnected' }}
              </span>
            </div>
            <div class="card-footer-right">
              <button class="btn btn-secondary btn-sm" @click="handleCheckHealth(server)">Check Health</button>
              <DropdownMenu :model-value="openDropdown === server.name" :min-width="160" @update:model-value="(v: boolean) => openDropdown = v ? server.name : null">
                <template #trigger>
                  <button class="btn-icon btn-sm" aria-label="More actions" @click.stop="toggleDropdown(server.name)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                    </svg>
                  </button>
                </template>
                <button class="dropdown-item" @click.stop="handleViewLogs(server)">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                  View Logs
                </button>
                <button class="dropdown-item" @click.stop="handleRestart(server)">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
                  Restart
                </button>
                <div class="dropdown-divider"></div>
                <button class="dropdown-item danger" @click.stop="handleDisconnect(server)">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  Disconnect
                </button>
              </DropdownMenu>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
          <circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/>
        </svg>
        <h3>No MCP servers found</h3>
        <p>Try adjusting your search or filter criteria.</p>
      </div>
    </div>

    <!-- Groups Tab -->
    <div v-show="activeTab === 'groups'" style="display: contents">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
        <p style="font-size: 13px; color: var(--fg-muted)">Organize MCP servers into logical groups for easier management.</p>
        <button class="btn btn-primary btn-sm" @click="showNotification?.('Create group dialog', 'info')">+ Create Group</button>
      </div>
      <div class="group-chips">
        <div class="group-chip active">All <span class="count">3</span></div>
        <div class="group-chip">Development <span class="count">2</span></div>
        <div class="group-chip">CI/CD <span class="count">1</span></div>
      </div>
      <div class="card-grid">
        <div v-for="server in servers.slice(0, 3)" :key="server.name" class="card mcp-card">
          <div class="card-head">
            <div class="card-icon" style="background: rgba(90,138,100,0.10); color: var(--success)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/>
              </svg>
            </div>
            <div style="flex: 1; min-width: 0">
              <div class="card-title">{{ server.name }}</div>
              <div class="card-subtitle">{{ server.endpoint }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Audit Log Tab -->
    <div v-show="activeTab === 'audit'" style="display: contents">
      <div class="filter-bar">
        <div class="search-input">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
            <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <input type="text" placeholder="Search audit log…" />
        </div>
        <select class="filter-select">
          <option value="all">All Actions</option>
          <option value="install">Install</option>
          <option value="uninstall">Uninstall</option>
          <option value="health_check">Health Check</option>
          <option value="sync">Sync</option>
        </select>
      </div>
      <div class="card" style="padding: 0; overflow: hidden">
        <table class="audit-table">
          <thead><tr><th>Time</th><th>Actor</th><th>Action</th><th>Service</th><th>Status</th><th>Details</th></tr></thead>
          <tbody>
            <tr><td style="font-family: var(--font-mono)">2026-06-18 14:32</td><td>user</td><td><span class="badge info">health_check</span></td><td>git</td><td><span class="badge success">OK</span></td><td style="font-family: var(--font-mono)">latency: 12ms</td></tr>
            <tr><td style="font-family: var(--font-mono)">2026-06-18 14:30</td><td>user</td><td><span class="badge warn">install</span></td><td>node_repl</td><td><span class="badge success">OK</span></td><td>stdio://node-repl-mcp</td></tr>
            <tr><td style="font-family: var(--font-mono)">2026-06-18 14:28</td><td>system</td><td><span class="badge error">health_check</span></td><td>gitlab</td><td><span class="badge error">FAIL</span></td><td>connection timeout</td></tr>
            <tr><td style="font-family: var(--font-mono)">2026-06-18 13:15</td><td>user</td><td><span class="badge info">sync</span></td><td>git</td><td><span class="badge success">OK</span></td><td>synced to cursor</td></tr>
            <tr><td style="font-family: var(--font-mono)">2026-06-18 12:00</td><td>system</td><td><span class="badge info">health_check</span></td><td>all</td><td><span class="badge success">2/3</span></td><td>periodic check</td></tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue';
import { useMCPStore } from '@/stores/mcp';
import DropdownMenu from '@/components/common/DropdownMenu.vue';
import TabBar from '@/components/common/TabBar.vue';
import { confirm } from '@/utils/dialog';

const mcpStore = useMCPStore();
const showNotification = inject<(message: string, type?: string) => void>('showNotification');

// Mock data for first-run preview. The store is the source of truth at
// runtime — `effectiveServers` falls back to this only when `mcpStore.services`
// is empty, so first-time users see a populated list instead of an empty
// page. Once any service is registered, the real data wins. Keep this list
// tiny (3 entries) since it's just a placeholder, not a fixture.
const mockServers: Array<{
  name: string
  endpoint: string
  auth: string
  healthy: boolean
  tools: number
  lastChecked: string
  group?: string
}> = [
  {
    name: 'git',
    endpoint: 'stdio://git-mcp',
    auth: 'none',
    healthy: true,
    tools: 8,
    lastChecked: '1 min ago',
    group: 'version-control',
  },
  {
    name: 'node_repl',
    endpoint: 'stdio://node-repl-mcp',
    auth: 'none',
    healthy: true,
    tools: 6,
    lastChecked: '1 min ago',
  },
  {
    name: 'gitlab',
    endpoint: 'https://gitlab.com/api/v4/mcp',
    auth: 'oauth',
    healthy: false,
    tools: 12,
    lastChecked: '5 min ago',
  },
];

// State — use store data with mock fallback
const activeTab = ref('services');
const searchQuery = ref('');
const healthFilter = ref('all');
const authFilter = ref('all');

// Use store data when available; fall back to the first-run preview list
// only when the user has not registered any services yet.
const effectiveServers = computed(() => {
  return mcpStore.services.length > 0
    ? mcpStore.services.map(s => ({
        name: s.name,
        endpoint: s.endpoint,
        auth: s.authType,
        healthy: s.healthStatus === 'online',
        tools: 0,
        lastChecked: new Date(s.updatedAt).toLocaleString(),
        group: s.groupId || undefined,
      }))
    : mockServers;
});

// Template-level alias: `servers` was a `ref(mockServers)` that didn't
// reflect the store. Route every read through `effectiveServers` so the
// header count and the groups preview stay in sync with the store.
const servers = effectiveServers;

const tabItems = computed(() => [
  { id: 'services', label: 'Services', count: effectiveServers.value.length },
  { id: 'groups', label: 'Groups' },
  { id: 'audit', label: 'Audit Log' },
]);

// Computed
const filteredServers = computed(() => {
  return effectiveServers.value.filter((s) => {
    const matchesSearch =
      !searchQuery.value ||
      s.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      s.endpoint.toLowerCase().includes(searchQuery.value.toLowerCase());
    const matchesHealth =
      healthFilter.value === 'all' ||
      (healthFilter.value === 'healthy' && s.healthy) ||
      (healthFilter.value === 'unhealthy' && !s.healthy);
    const matchesAuth =
      authFilter.value === 'all' || s.auth === authFilter.value;
    return matchesSearch && matchesHealth && matchesAuth;
  });
});

// Handlers
function handleDiscover() {
  if (showNotification) showNotification('Discovering MCP services...', 'info');
}

function handleAddServer() {
  if (showNotification) showNotification('Add Server dialog coming soon', 'info');
}

function handleCheckHealth(server: (typeof mockServers)[number]) {
  if (showNotification) showNotification(`Health check: ${server.name} ${server.healthy ? 'OK' : 'FAIL'}`, server.healthy ? 'success' : 'error');
}

// More options dropdown
const openDropdown = ref<string | null>(null)

function toggleDropdown(key: string) {
  openDropdown.value = openDropdown.value === key ? null : key
}

function handleViewLogs(server: (typeof mockServers)[number]) {
  openDropdown.value = null
  if (showNotification) showNotification(`View logs: ${server.name}`, 'info')
}

function handleRestart(server: (typeof mockServers)[number]) {
  openDropdown.value = null
  if (showNotification) showNotification(`Restarting ${server.name}...`, 'info')
}

async function handleDisconnect(server: (typeof mockServers)[number]) {
  openDropdown.value = null
  if (await confirm(`确认断开 ${server.name}？`)) {
    if (showNotification) showNotification(`已断开 ${server.name}`, 'info')
  }
}

function handleSearch() {
  // Search is reactive via v-model, filtering happens in computed
}

function handleFilter() {
  // Filter is reactive via v-model, filtering happens in computed
}
</script>

<style scoped>
/* === Filter Bar === */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1 1 180px;
  min-width: 0;
  position: relative;
}

.search-input input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.32);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.30);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.search-input input:focus {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.40);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

.search-input svg {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.search-input .clear-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.32);
  cursor: pointer;
  border: none;
  padding: 0;
}

.filter-select {
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.32) url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%239A9A9A' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E") no-repeat right 12px center;
  border: 1px solid rgba(255, 255, 255, 0.30);
  appearance: none;
  cursor: pointer;
  font-family: inherit;
  color: var(--fg);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  flex: 0 1 auto;
  min-width: 0;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.filter-select:focus {
  border-color: rgba(255, 255, 255, 0.30);
}

.btn-group {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-left: auto;
}

/* === Card Grid === */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  align-items: stretch;
}

.card-head {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.card-icon {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border);
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.card-subtitle {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  row-gap: 4px;
  word-break: break-word;
}

.card-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.card-meta-item .value {
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
  max-width: 160px;
}

.card-divider {
  height: 1px;
  background: var(--border);
  margin-top: auto;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 2px;
  margin-top: auto;
  width: 100%;
}

.card-footer-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.card-footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  width: fit-content;
}

/* === Group Chips === */
.group-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.group-chip {
  padding: 4px 12px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.30);
  background: rgba(255, 255, 255, 0.30);
  color: var(--fg-muted);
  transition: all var(--t-fast);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
}

.group-chip:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.group-chip.active {
  background: rgba(45, 45, 45, 0.10);
  border-color: var(--accent);
  color: var(--accent);
}

.group-chip .count {
  margin-left: 4px;
  font-family: var(--font-mono);
  opacity: 0.6;
}

/* === Audit Table === */
.audit-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.audit-table th {
  text-align: left;
  padding: 8px 12px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  border-bottom: 1px solid rgba(255, 255, 255, 0.32);
  background: rgba(255, 255, 255, 0.32);
}

.audit-table td {
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  vertical-align: middle;
  color: var(--fg-muted);
}

.audit-table tr:hover td {
  background: rgba(255, 255, 255, 0.32);
}

/* === Empty State === */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  text-align: center;
  color: var(--fg-ghost);
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.4;
}

.empty-state h3 {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 4px;
  color: var(--fg-muted);
}

.empty-state p {
  font-size: 13px;
  margin: 0;
}

</style>

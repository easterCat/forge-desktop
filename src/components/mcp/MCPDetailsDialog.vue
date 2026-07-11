<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>MCP Server Details</h3>
        <button class="close-btn" aria-label="关闭" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Tabs Navigation -->
      <div class="tabs-nav">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="tab-btn"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          <component :is="tab.icon" />
          {{ tab.label }}
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Tab: Overview -->
        <div v-if="activeTab === 'overview'" class="tab-content">
        <!-- Server Overview -->
        <div class="server-header">
          <div class="server-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
              <line x1="6" y1="6" x2="6.01" y2="6"/>
              <line x1="6" y1="18" x2="6.01" y2="18"/>
            </svg>
          </div>
          <div class="server-title">
            <h2>{{ server.name }}</h2>
            <div class="server-meta">
              <span v-if="server.author" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                  <circle cx="12" cy="7" r="4"/>
                </svg>
                {{ server.author }}
              </span>
              <span v-if="server.version" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
                </svg>
                v{{ server.version }}
              </span>
              <span v-if="server.license" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                {{ server.license }}
              </span>
            </div>
          </div>
          <div class="server-badges">
            <!-- Health Badge for MCPService -->
            <MCPHealthBadge
              v-if="hasHealthStatus"
              :status="healthStatus"
              :show-label="true"
              size="sm"
            />
            <span class="badge protocol-badge" :class="server.protocol">
              {{ server.protocol.toUpperCase() }}
            </span>
            <span v-if="server.sourceId" class="badge source-badge">{{ server.sourceId }}</span>
            <span v-if="isInstalled" class="badge installed-badge">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
              Installed
            </span>
          </div>
        </div>

        <!-- Description -->
        <div class="section">
          <h4>Description</h4>
          <p class="description">{{ server.description }}</p>
          <p v-if="server.longDescription" class="long-description">
            {{ server.longDescription }}
          </p>
        </div>

        <!-- Required Environment Variables -->
        <div v-if="server.requiredEnvVars?.length" class="section">
          <h4>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z"/>
              <path d="M2 17l10 5 10-5"/>
              <path d="M2 12l10 5 10-5"/>
            </svg>
            Required Environment Variables
          </h4>
          <div class="env-vars-grid">
            <div
              v-for="env in server.requiredEnvVars"
              :key="env.name"
              class="env-var-card"
            >
              <div class="env-header">
                <code class="env-name">{{ env.name }}</code>
                <span v-if="env.required" class="required-badge">Required</span>
                <span v-else class="optional-badge">Optional</span>
              </div>
              <p v-if="env.description" class="env-desc">{{ env.description }}</p>
              <div v-if="env.defaultValue" class="env-default">
                <span>Default:</span>
                <code>{{ env.defaultValue }}</code>
              </div>
              <div v-if="env.example" class="env-example">
                <span>Example:</span>
                <code>{{ env.example }}</code>
              </div>
            </div>
          </div>
        </div>

        <!-- Categories and Tags -->
        <div class="section">
          <h4>Categories</h4>
          <div class="tags-list">
            <span
              v-for="cat in server.categories"
              :key="cat"
              class="category-tag"
            >
              {{ cat }}
            </span>
          </div>
        </div>

        <div v-if="server.tags.length > 0" class="section">
          <h4>Tags</h4>
          <div class="tags-list">
            <span
              v-for="tag in server.tags"
              :key="tag"
              class="tag"
            >
              {{ tag }}
            </span>
          </div>
        </div>

        <!-- Required Permissions -->
        <div v-if="server.requiredPermissions?.length" class="section">
          <h4>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
            Required Permissions
          </h4>
          <div class="permissions-list">
            <span
              v-for="perm in server.requiredPermissions"
              :key="perm"
              class="permission-tag"
            >
              {{ perm }}
            </span>
          </div>
        </div>

        <!-- Stats -->
        <div class="section stats-section">
          <div v-if="server.stars" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
            <div class="stat-value">{{ formatNumber(server.stars) }}</div>
            <div class="stat-label">Stars</div>
          </div>
          <div v-if="server.downloads" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            <div class="stat-value">{{ formatNumber(server.downloads) }}</div>
            <div class="stat-label">Downloads</div>
          </div>
          <div v-if="server.lastUpdated" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
            <div class="stat-value">{{ formatDate(server.lastUpdated) }}</div>
            <div class="stat-label">Last Updated</div>
          </div>
        </div>

        <!-- NPM Package -->
        <div v-if="server.npmPackage" class="section">
          <h4>NPM Package</h4>
          <div class="npm-command">
            <code>npm install {{ server.npmPackage }}</code>
            <button class="copy-btn" @click="copyNpmCommand">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Links -->
        <div v-if="server.repository || server.homepage" class="section">
          <h4>Links</h4>
          <div class="links-list">
            <button
              v-if="server.repository"
              class="link-item"
              @click.stop="openExternalUrl(server.repository)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
              </svg>
              Repository
            </button>
            <button
              v-if="server.homepage"
              class="link-item"
              @click.stop="openExternalUrl(server.homepage)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="2" y1="12" x2="22" y2="12"/>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
              </svg>
              Homepage
            </button>
          </div>
        </div>

        <!-- Install Command -->
        <div v-if="server.installCommand" class="section">
          <h4>Install Command</h4>
          <div class="command-box">
            <code>{{ server.installCommand }}</code>
            <button class="copy-btn" @click="copyCommand">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
          </div>
        </div>
        </div>

        <!-- Tab: Discovery (Tools/Resources/Prompts) -->
        <div v-if="activeTab === 'discovery'" class="tab-content">
          <div class="section">
            <div class="section-header">
              <h4>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
                </svg>
                Available Tools
              </h4>
              <button class="btn btn-sm btn-secondary" :disabled="isDiscovering" @click="discoverService">
                <svg v-if="isDiscovering" class="spinner" width="14" height="14" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
                </svg>
                {{ isDiscovering ? 'Discovering...' : 'Refresh Discovery' }}
              </button>
            </div>

            <div v-if="discoveryCache?.tools?.length" class="tools-list">
              <div
                v-for="tool in discoveryCache.tools"
                :key="tool.name"
                class="tool-item"
              >
                <div class="tool-header">
                  <code class="tool-name">{{ tool.name }}</code>
                  <button class="btn btn-sm btn-primary" @click="openInvocationDialog(tool)">
                    Try It
                  </button>
                </div>
                <p v-if="tool.description" class="tool-description">{{ tool.description }}</p>
              </div>
            </div>
            <div v-else class="empty-tab">
              <p>No tools discovered yet.</p>
              <button class="btn btn-sm btn-secondary" @click="discoverService">
                Discover Tools
              </button>
            </div>
          </div>

          <div class="section">
            <h4>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              Resources
            </h4>
            <div v-if="discoveryCache?.resources?.length" class="resources-list">
              <div
                v-for="resource in discoveryCache.resources"
                :key="resource.uri"
                class="resource-item"
              >
                <code class="resource-uri">{{ resource.uri }}</code>
                <span v-if="resource.name" class="resource-name">{{ resource.name }}</span>
              </div>
            </div>
            <div v-else class="empty-tab">
              <p>No resources available.</p>
            </div>
          </div>

          <div class="section">
            <h4>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              Prompts
            </h4>
            <div v-if="discoveryCache?.prompts?.length" class="prompts-list">
              <div
                v-for="prompt in discoveryCache.prompts"
                :key="prompt.name"
                class="prompt-item"
              >
                <span class="prompt-name">{{ prompt.name }}</span>
                <span v-if="prompt.description" class="prompt-desc">{{ prompt.description }}</span>
              </div>
            </div>
            <div v-else class="empty-tab">
              <p>No prompts available.</p>
            </div>
          </div>
        </div>

        <!-- Tab: Health History -->
        <div v-if="activeTab === 'health'" class="tab-content">
          <div class="section">
            <div class="section-header">
              <h4>Health Check History</h4>
              <button class="btn btn-sm btn-secondary" :disabled="isLoadingHealth" @click="loadHealthHistory">
                <svg v-if="isLoadingHealth" class="spinner" width="14" height="14" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
                </svg>
                {{ isLoadingHealth ? 'Loading...' : 'Refresh' }}
              </button>
            </div>

            <div v-if="healthHistory.length > 0" class="health-history">
              <div
                v-for="record in healthHistory"
                :key="record.id"
                class="health-record"
                :class="record.status"
              >
                <div class="record-status">
                  <span class="status-dot"></span>
                  <span class="status-label">{{ record.status }}</span>
                </div>
                <div class="record-time">
                  {{ formatTimestamp(record.checkedAt) }}
                </div>
                <div v-if="record.latencyMs" class="record-latency">
                  {{ record.latencyMs }}ms
                </div>
                <div v-if="record.errorMessage" class="record-error">
                  {{ record.errorMessage }}
                </div>
              </div>
            </div>
            <div v-else class="empty-tab">
              <p>No health check history available.</p>
              <button class="btn btn-sm btn-secondary" @click="loadHealthHistory">
                Load History
              </button>
            </div>
          </div>
        </div>

        <!-- Tab: Activity -->
        <div v-if="activeTab === 'activity'" class="tab-content">
          <div class="section">
            <div class="section-header">
              <h4>Recent Activity</h4>
              <button class="btn btn-sm btn-secondary" :disabled="isLoadingActivity" @click="loadActivity">
                <svg v-if="isLoadingActivity" class="spinner" width="14" height="14" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="31.4 31.4"/>
                </svg>
                {{ isLoadingActivity ? 'Loading...' : 'Refresh' }}
              </button>
            </div>

            <div v-if="recentActivity.length > 0" class="activity-list">
              <div
                v-for="entry in recentActivity"
                :key="entry.id"
                class="activity-entry"
              >
                <div class="activity-icon" :class="entry.action">
                  <svg v-if="entry.action === 'create'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                  </svg>
                  <svg v-else-if="entry.action === 'update'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                  <svg v-else-if="entry.action === 'delete'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                  <svg v-else-if="entry.action === 'invoke'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polygon points="5 3 19 12 5 21 5 3"/>
                  </svg>
                  <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <polyline points="12 6 12 12 16 14"/>
                  </svg>
                </div>
                <div class="activity-content">
                  <div class="activity-header">
                    <span class="activity-action">{{ formatAction(entry.action) }}</span>
                    <span v-if="entry.actor" class="activity-actor">by {{ entry.actor }}</span>
                  </div>
                  <span class="activity-time">{{ formatTimestamp(entry.createdAt) }}</span>
                </div>
                <span class="activity-status" :class="entry.status">
                  {{ entry.status }}
                </span>
              </div>
            </div>
            <div v-else class="empty-tab">
              <p>No recent activity.</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          Close
        </button>
        <button
          v-if="hasHealthStatus"
          class="btn btn-secondary"
          @click="handleSync"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          Sync
        </button>
        <button
          v-else-if="!isInstalled"
          class="btn btn-primary"
          @click="handleInstall"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Install Server
        </button>
        <button
          v-else
          class="btn btn-secondary"
          @click="handleSync"
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
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import MCPHealthBadge from './MCPHealthBadge.vue';
import type {
  MCPService,
  MCPTool,
  MCPHealthRecord,
  MCPAuditEntry,
  MCPDiscoveryCache,
  MCPServerUnion,
} from '@/types';
import { open as openExternal } from '@tauri-apps/plugin-shell';

interface Props {
  server: MCPServerUnion;
  isInstalled: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'install', server: MCPServerUnion): void;
  (e: 'sync', server: MCPServerUnion): void;
  (e: 'invoke-tool', service: MCPService, tool: MCPTool): void;
  (e: 'discover', serviceId: string): void;
}>();

// Check if server has health status (MCPService vs MCPServer)
const hasHealthStatus = computed(() => {
  return 'isHealthy' in props.server;
});

const healthStatus = computed<'online' | 'offline' | 'error'>(() => {
  if (!hasHealthStatus.value) return 'offline';
  // `hasHealthStatus` proves `isHealthy` is present; cast via `unknown` so
  // the TS conversion guard accepts the downcast on `MCPServerUnion`.
  const service = props.server as unknown as MCPService;
  return service.isHealthy ? 'online' : 'offline';
});

// Tabs
type TabId = 'overview' | 'discovery' | 'health' | 'activity';
const activeTab = ref<TabId>('overview');

const tabs: Array<{ id: TabId; label: string; icon: string }> = [
  { id: 'overview', label: 'Overview', icon: 'span' },
  { id: 'discovery', label: 'Discovery', icon: 'span' },
  { id: 'health', label: 'Health', icon: 'span' },
  { id: 'activity', label: 'Activity', icon: 'span' },
];

// Discovery data
const discoveryCache = ref<MCPDiscoveryCache | null>(null);
const isDiscovering = ref(false);

// Health history
const healthHistory = ref<MCPHealthRecord[]>([]);
const isLoadingHealth = ref(false);

// Recent activity
const recentActivity = ref<MCPAuditEntry[]>([]);
const isLoadingActivity = ref(false);

onMounted(async () => {
  if (hasHealthStatus.value) {
    const service = props.server as unknown as MCPService;
    if (service.discoveryCache) {
      discoveryCache.value = service.discoveryCache;
    }
    loadHealthHistory();
    loadActivity();
  }
});

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

async function discoverService() {
  if (!hasHealthStatus.value) return;
  isDiscovering.value = true;
  emit('discover', (props.server as unknown as MCPService).id);
  // Reset after timeout (actual result comes from parent)
  setTimeout(() => {
    isDiscovering.value = false;
  }, 3000);
}

async function loadHealthHistory() {
  if (!hasHealthStatus.value) return;
  isLoadingHealth.value = true;
  // In real implementation, this would fetch from store
  // For now, show sample data
  healthHistory.value = [];
  isLoadingHealth.value = false;
}

async function loadActivity() {
  if (!hasHealthStatus.value) return;
  isLoadingActivity.value = true;
  // In real implementation, this would fetch from store
  recentActivity.value = [];
  isLoadingActivity.value = false;
}

function openInvocationDialog(tool: MCPTool) {
  if (hasHealthStatus.value) {
    emit('invoke-tool', props.server as unknown as MCPService, tool);
  }
}

function formatAction(action: string): string {
  const labels: Record<string, string> = {
    create: 'Created',
    update: 'Updated',
    delete: 'Deleted',
    health_check: 'Health Check',
    invoke: 'Invoked',
  };
  return labels[action] || action;
}

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num.toString();
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateStr;
  }
}

function formatTimestamp(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return dateStr;
  }
}

function copyCommand() {
  if (props.server.installCommand) {
    navigator.clipboard.writeText(props.server.installCommand);
  }
}

function copyNpmCommand() {
  if (props.server.npmPackage) {
    navigator.clipboard.writeText(`npm install ${props.server.npmPackage}`);
  }
}

function handleInstall() {
  emit('install', props.server);
}

function handleSync() {
  emit('sync', props.server);
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.dialog {
  width: 100%;
  max-width: 720px;
  max-height: 90vh;
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  display: flex;
  flex-direction: column;
}

/* Tabs Navigation */
.tabs-nav {
  display: flex;
  gap: 4px;
  padding: 0 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-secondary);
  border-radius: 12px 12px 0 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: var(--fg);
}

.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
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

.server-header {
  display: flex;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 20px;
}

.server-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent);
  border-radius: 14px;
  color: white;
  flex-shrink: 0;
}

.server-title {
  flex: 1;
}

.server-title h2 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 8px;
}

.server-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--fg-muted);
}

.server-badges {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
}

.badge {
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
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

.source-badge {
  background: var(--bg-tertiary);
  color: var(--fg-muted);
  text-transform: capitalize;
}

.installed-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--success-bg);
  color: var(--success);
}

.section {
  margin-bottom: 20px;
}

.section h4 {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 10px;
}

.section h4 svg {
  color: var(--fg-muted);
}

.description {
  font-size: 14px;
  line-height: 1.6;
  color: var(--fg);
  margin-bottom: 8px;
}

.long-description {
  font-size: 13px;
  line-height: 1.6;
  color: var(--fg-muted);
}

.env-vars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.env-var-card {
  padding: 12px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.env-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.env-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
}

.required-badge {
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 500;
  background: var(--error-bg);
  color: var(--error);
  border-radius: 3px;
}

.optional-badge {
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 500;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
  border-radius: 3px;
}

.env-desc {
  font-size: 12px;
  color: var(--fg-muted);
  margin-bottom: 6px;
}

.env-default, .env-example {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--fg-muted);
  margin-top: 4px;
}

.env-default code, .env-example code {
  padding: 2px 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: monospace;
  font-size: 11px;
  color: var(--fg);
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.category-tag {
  padding: 4px 10px;
  font-size: 12px;
  background: var(--accent-bg);
  color: var(--accent);
  border-radius: 4px;
}

.tag {
  padding: 4px 10px;
  font-size: 12px;
  background: var(--bg-secondary);
  color: var(--fg-muted);
  border-radius: 4px;
}

.permissions-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  padding: 4px 10px;
  font-size: 12px;
  background: var(--warn-bg);
  color: var(--warn);
  border-radius: 4px;
}

.stats-section {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  text-align: center;
}

.stat-card svg {
  color: var(--warn);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  margin-bottom: 2px;
}

.stat-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.npm-command {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  background: #cb383720;
  border: 1px solid #cb383750;
  border-radius: 8px;
}

.npm-command code {
  flex: 1;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: #cb3837;
  word-break: break-all;
}

.links-list {
  display: flex;
  gap: 12px;
}

.link-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: var(--radius);
  font-size: 13px;
  color: var(--fg);
  text-decoration: none;
  transition: all 0.2s;
}

.link-item:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

.command-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.command-box code {
  flex: 1;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 12px;
  color: var(--accent);
  word-break: break-all;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--bg-primary);
  color: var(--fg);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

/* Tab Content */
.tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-header h4 {
  margin-bottom: 0;
}

/* Discovery Tab */
.tools-list,
.resources-list,
.prompts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-item {
  padding: 12px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.tool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.tool-name {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--accent);
}

.tool-description {
  font-size: 12px;
  color: var(--fg-muted);
  margin: 0;
  line-height: 1.5;
}

.resource-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.resource-uri {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--fg);
}

.resource-name {
  font-size: 12px;
  color: var(--fg-muted);
}

.prompt-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.prompt-name {
  font-size: 13px;
  font-weight: 500;
}

.prompt-desc {
  font-size: 12px;
  color: var(--fg-muted);
}

/* Health History Tab */
.health-history {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.health-record {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.health-record.online .status-dot {
  background: var(--success);
}

.health-record.offline .status-dot,
.health-record.error .status-dot {
  background: var(--error);
}

.record-status {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 100px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-label {
  font-size: 12px;
  font-weight: 500;
  text-transform: capitalize;
}

.record-time {
  flex: 1;
  font-size: 12px;
  color: var(--fg-muted);
  font-family: monospace;
}

.record-latency {
  font-size: 12px;
  color: var(--accent);
  font-family: monospace;
}

.record-error {
  font-size: 11px;
  color: var(--error);
  max-width: 200px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Activity Tab */
.activity-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.activity-entry {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.activity-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  flex-shrink: 0;
}

.activity-icon.create {
  background: var(--success-bg);
  color: var(--success);
}

.activity-icon.update {
  background: var(--info-bg);
  color: var(--info);
}

.activity-icon.delete {
  background: var(--error-bg);
  color: var(--error);
}

.activity-icon.invoke {
  background: rgba(139, 92, 246, 0.12);
  color: var(--accent);
}

.activity-icon.health_check {
  background: var(--warn-bg);
  color: var(--warn);
}

.activity-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.activity-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.activity-action {
  font-size: 13px;
  font-weight: 500;
}

.activity-actor {
  font-size: 12px;
  color: var(--fg-muted);
}

.activity-time {
  font-size: 11px;
  color: var(--fg-muted);
}

.activity-status {
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.activity-status.success {
  background: rgba(16, 185, 129, 0.15);
  color: #10B981;
}

.activity-status.failure {
  background: rgba(239, 68, 68, 0.15);
  color: #EF4444;
}

/* Empty Tab State */
.empty-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px;
  text-align: center;
}

.empty-tab p {
  font-size: 13px;
  color: var(--fg-muted);
  margin: 0;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-sm {
  padding: 5px 10px;
  font-size: 11px;
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover, var(--accent));
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.btn-secondary:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>

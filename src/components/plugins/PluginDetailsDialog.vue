<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="$emit('close')">
      <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>Plugin Details</h3>
        <button class="close-btn" aria-label="关闭" title="Close" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">

        <!-- Loading State -->
        <div v-if="isLoading" class="loading-state">
          <div class="loading-spinner"></div>
          <p>Loading plugin capabilities...</p>
        </div>

        <!-- Error State -->
        <div v-else-if="loadError" class="error-state">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <h4>Failed to load plugin details</h4>
          <p>{{ loadError }}</p>
          <button class="btn btn-secondary btn-sm" @click="loadCapabilities">Retry</button>
        </div>

        <!-- Plugin Content -->
        <template v-else-if="capabilities">
          <!-- Plugin Overview Header -->
          <div class="plugin-header">
            <div class="plugin-icon">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
            <div class="plugin-title">
              <h2>{{ capabilities.name }}</h2>
              <div class="plugin-meta">
                <span v-if="capabilities.author" class="meta-item">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                    <circle cx="12" cy="7" r="4"/>
                  </svg>
                  {{ capabilities.author }}
                </span>
                <span v-if="capabilities.version" class="meta-item">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
                  </svg>
                  v{{ capabilities.version }}
                </span>
                <span v-if="capabilities.license" class="meta-item">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                    <polyline points="14 2 14 8 20 8"/>
                  </svg>
                  {{ capabilities.license }}
                </span>
              </div>
            </div>
            <div class="plugin-badges">
              <span v-if="capabilities.source === 'local'" class="badge installed-badge">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
                Local
              </span>
              <span v-else class="badge remote-badge">
                Remote
              </span>
            </div>
          </div>

          <!-- Description -->
          <div v-if="capabilities.description" class="section">
            <h4>Description</h4>
            <p class="description">{{ capabilities.description }}</p>
          </div>

          <!-- Meta Grid: Installed Path / Repository -->
          <div class="meta-grid">
            <div v-if="capabilities.installedPath" class="meta-cell">
              <span class="meta-label">Installed Path</span>
              <code class="meta-value path-value" :title="capabilities.installedPath">
                {{ capabilities.installedPath }}
              </code>
            </div>
            <div v-if="capabilities.repository" class="meta-cell">
              <span class="meta-label">Repository</span>
              <button
                class="meta-link"
                @click.stop="openExternalUrl(capabilities.repository)"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                  <polyline points="15 3 21 3 21 9"/>
                  <line x1="10" y1="14" x2="21" y2="3"/>
                </svg>
                Open
              </button>
            </div>
            <div v-if="capabilities.dependencies.length > 0" class="meta-cell">
              <span class="meta-label">Dependencies</span>
              <div class="deps-list">
                <span v-for="dep in capabilities.dependencies" :key="dep" class="dep-tag">{{ dep }}</span>
              </div>
            </div>
          </div>

          <!-- Capability Count Cards -->
          <div class="capability-counts">
            <div
              class="cap-card"
              :class="{ active: capabilities.capabilities.skills > 0 }"
              @click="activeTab = 'skills'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
              </svg>
              <div class="cap-value">{{ capabilities.capabilities.skills }}</div>
              <div class="cap-label">Skills</div>
            </div>
            <div
              class="cap-card"
              :class="{ active: capabilities.capabilities.hooks > 0 }"
              @click="activeTab = 'hooks'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
              <div class="cap-value">{{ capabilities.capabilities.hooks }}</div>
              <div class="cap-label">Hooks</div>
            </div>
            <div
              class="cap-card"
              :class="{ active: capabilities.capabilities.commands > 0 }"
              @click="activeTab = 'commands'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="4 17 10 11 4 5"/>
                <line x1="12" y1="19" x2="20" y2="19"/>
              </svg>
              <div class="cap-value">{{ capabilities.capabilities.commands }}</div>
              <div class="cap-label">Commands</div>
            </div>
            <div
              class="cap-card"
              :class="{ active: capabilities.capabilities.mcpServers > 0 }"
              @click="activeTab = 'mcp'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
                <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
                <line x1="6" y1="6" x2="6.01" y2="6"/>
                <line x1="6" y1="18" x2="6.01" y2="18"/>
              </svg>
              <div class="cap-value">{{ capabilities.capabilities.mcpServers }}</div>
              <div class="cap-label">MCP</div>
            </div>
            <div
              class="cap-card"
              :class="{ active: capabilities.capabilities.lspServers > 0 }"
              @click="activeTab = 'lsp'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
              </svg>
              <div class="cap-value">{{ capabilities.capabilities.lspServers }}</div>
              <div class="cap-label">LSP</div>
            </div>
          </div>

          <!-- Tabs -->
          <div class="tabs-section">
            <div class="tabs-header">
              <button
                v-for="tab in visibleTabs"
                :key="tab.id"
                class="tab-btn"
                :class="{ active: activeTab === tab.id }"
                @click="activeTab = tab.id"
              >
                {{ tab.label }}
                <span class="tab-count">{{ tab.count }}</span>
              </button>
            </div>

            <div class="tab-content">
              <!-- Overview Tab -->
              <div v-if="activeTab === 'overview'" class="tab-pane">
                <div v-if="capabilities.manifestFiles.length > 0" class="section">
                  <h4>Manifest Files</h4>
                  <div class="manifest-files-list">
                    <span v-for="f in capabilities.manifestFiles" :key="f" class="manifest-file">
                      {{ f }}
                    </span>
                  </div>
                </div>
                <div v-else class="empty-tab">
                  <p>No manifest files declared.</p>
                </div>
              </div>

              <!-- Skills Tab -->
              <div v-else-if="activeTab === 'skills'" class="tab-pane">
                <div v-if="capabilities.skills.length === 0" class="empty-tab">
                  <p>No skills found in this plugin.</p>
                </div>
                <div v-else class="cap-list">
                  <div v-for="skill in capabilities.skills" :key="skill.name" class="cap-item">
                    <div class="cap-item-header">
                      <div class="cap-item-name">{{ skill.name }}</div>
                      <div class="cap-item-badges">
                        <span v-if="skill.hasScripts" class="cap-badge scripts-badge">Scripts</span>
                        <span v-if="skill.hasReferences" class="cap-badge refs-badge">Refs</span>
                      </div>
                    </div>
                    <p class="cap-item-desc">{{ skill.description || 'No description' }}</p>
                    <code class="cap-item-path">{{ skill.path }}</code>
                  </div>
                </div>
              </div>

              <!-- Commands Tab -->
              <div v-else-if="activeTab === 'commands'" class="tab-pane">
                <div v-if="capabilities.commands.length === 0" class="empty-tab">
                  <p>No commands found in this plugin.</p>
                </div>
                <div v-else class="cap-list">
                  <div v-for="cmd in capabilities.commands" :key="cmd.name" class="cap-item">
                    <div class="cap-item-header">
                      <div class="cap-item-name">/{{ cmd.name }}</div>
                      <div class="cap-item-badges">
                        <span
                          v-for="tool in cmd.allowedTools"
                          :key="tool"
                          class="cap-badge tool-badge"
                        >
                          {{ tool }}
                        </span>
                      </div>
                    </div>
                    <p class="cap-item-desc">{{ cmd.description || 'No description' }}</p>
                    <code class="cap-item-path">{{ cmd.path }}</code>
                  </div>
                </div>
              </div>

              <!-- Hooks Tab -->
              <div v-else-if="activeTab === 'hooks'" class="tab-pane">
                <div v-if="capabilities.hooks.length === 0" class="empty-tab">
                  <p>No hooks found in this plugin.</p>
                </div>
                <div v-else class="cap-list">
                  <div v-for="hook in capabilities.hooks" :key="hook.event + hook.matcher" class="cap-item">
                    <div class="cap-item-header">
                      <div class="cap-item-name">{{ hook.event }}</div>
                      <div style="display:flex;gap:6px;align-items:center">
                        <span
                          class="cap-badge"
                          :class="hook.scriptExists ? 'installed-badge' : 'warn-badge'"
                        >
                          <svg v-if="hook.scriptExists" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="20 6 9 17 4 12"/>
                          </svg>
                          {{ hook.scriptExists ? 'Script exists' : 'Script missing' }}
                        </span>
                        <button
                          class="btn btn-secondary btn-sm"
                          :disabled="executingHook === hook.event + (hook.matcher || '')"
                          @click="runHookTest(hook)"
                        >
                          {{ executingHook === hook.event + (hook.matcher || '') ? 'Running...' : '▶ Run' }}
                        </button>
                      </div>
                    </div>
                    <div v-if="hook.matcher" class="cap-item-matcher">
                      <span class="matcher-label">Matcher:</span>
                      <code class="matcher-re">{{ hook.matcher }}</code>
                    </div>
                    <code class="cap-item-command">{{ hook.command }}</code>
                  </div>
                </div>
              </div>

              <!-- MCP Tab -->
              <div v-else-if="activeTab === 'mcp'" class="tab-pane">
                <div v-if="capabilities.mcpServers.length === 0" class="empty-tab">
                  <p>No MCP servers declared in this plugin.</p>
                </div>
                <div v-else class="cap-list">
                  <div v-for="mcp in capabilities.mcpServers" :key="mcp.name" class="cap-item">
                    <div class="cap-item-header">
                      <div class="cap-item-name">{{ mcp.name }}</div>
                      <button
                        class="btn btn-secondary btn-sm"
                        :disabled="probeState[mcp.name] === 'running'"
                        @click="runMcpTest(mcp.name)"
                      >
                        {{ probeState[mcp.name] === 'running' ? 'Testing...' : 'Test Connection' }}
                      </button>
                    </div>
                    <div v-if="probeState[mcp.name] === 'success'" class="probe-result success">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                      Reachable · {{ probeResults[mcp.name]?.durationMs }}ms
                      <span v-if="probeResults[mcp.name]?.serverInfo" class="probe-info">
                        · {{ (probeResults[mcp.name]?.serverInfo as any)?.name || 'connected' }}
                      </span>
                    </div>
                    <div v-else-if="probeState[mcp.name] === 'error'" class="probe-result error">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                      {{ probeResults[mcp.name]?.error || 'Connection failed' }}
                    </div>
                    <div class="cap-item-detail">
                      <code class="mcp-command">{{ mcp.command }}</code>
                      <span v-if="mcp.args.length > 0" class="mcp-args">
                        {{ mcp.args.join(' ') }}
                      </span>
                    </div>
                    <div v-if="mcp.env" class="mcp-env">
                      <span class="env-label">Env vars:</span>
                      <span v-for="(val, key) in mcp.env" :key="key" class="env-pair">
                        <code>{{ key }}</code>={{ val }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- LSP Tab -->
              <div v-else-if="activeTab === 'lsp'" class="tab-pane">
                <div v-if="capabilities.lspServers.length === 0" class="empty-tab">
                  <p>No LSP servers declared in this plugin.</p>
                </div>
                <div v-else class="cap-list">
                  <div v-for="lsp in capabilities.lspServers" :key="lsp.name" class="cap-item">
                    <div class="cap-item-header">
                      <div class="cap-item-name">{{ lsp.name }}</div>
                    </div>
                    <div class="cap-item-detail">
                      <code class="mcp-command">{{ lsp.command }}</code>
                      <span v-if="lsp.args.length > 0" class="mcp-args">
                        {{ lsp.args.join(' ') }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- Hook Execution Result Dialog -->
      <div v-if="hookResult" class="dialog-overlay" @click.self="hookResult = null">
        <div class="dialog hook-result-dialog">
          <div class="dialog-header">
            <div>
              <h3>Hook Result: {{ hookResult.event }}</h3>
              <span class="dialog-subtitle">Duration: {{ hookResult.durationMs }}ms · Exit: {{ hookResult.exitCode ?? 'N/A' }}</span>
            </div>
            <button class="close-btn" aria-label="关闭" @click="hookResult = null">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <div class="dialog-content">
            <div class="result-grid">
              <div class="result-section">
                <h4>stdout</h4>
                <pre class="result-output success-output">{{ hookResult.stdout || '(empty)' }}</pre>
              </div>
              <div v-if="hookResult.stderr" class="result-section">
                <h4>stderr</h4>
                <pre class="result-output error-output">{{ hookResult.stderr }}</pre>
              </div>
              <div v-if="hookResult.parsedJson" class="result-section">
                <h4>Parsed JSON</h4>
                <pre class="result-output">{{ JSON.stringify(hookResult.parsedJson, null, 2) }}</pre>
              </div>
            </div>
            <div v-if="hookResult.logPath" class="log-path-row">
              <span class="log-path-label">Log:</span>
              <code class="log-path">{{ hookResult.logPath }}</code>
            </div>
          </div>
          <div class="dialog-actions">
            <button class="btn btn-secondary" @click="hookResult = null">Close</button>
          </div>
        </div>
      </div>

      <!-- Validation Report Dialog -->
      <div v-if="validationReport" class="dialog-overlay" @click.self="validationReport = null">
        <div class="dialog validation-dialog">
          <div class="dialog-header">
            <div>
              <h3>Validation Report</h3>
              <span class="dialog-subtitle">
                <span class="validation-badge" :class="validationReport.valid ? 'valid' : 'invalid'">
                  {{ validationReport.valid ? '✓ Valid' : '✕ Invalid' }}
                </span>
                · {{ validationReport.errors.length }} errors · {{ validationReport.warnings.length }} warnings
              </span>
            </div>
            <button class="close-btn" aria-label="关闭" @click="validationReport = null">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
          <div class="dialog-content">
            <div v-if="validationReport.errors.length > 0" class="result-section">
              <h4 class="error-heading">Errors</h4>
              <div v-for="issue in validationReport.errors" :key="issue.code" class="issue-item error-issue">
                <span class="issue-code">{{ issue.code }}</span>
                <span class="issue-msg">{{ issue.message }}</span>
                <code v-if="issue.path" class="issue-path">{{ issue.path }}</code>
              </div>
            </div>
            <div v-if="validationReport.warnings.length > 0" class="result-section">
              <h4 class="warn-heading">Warnings</h4>
              <div v-for="issue in validationReport.warnings" :key="issue.code" class="issue-item warn-issue">
                <span class="issue-code">{{ issue.code }}</span>
                <span class="issue-msg">{{ issue.message }}</span>
                <code v-if="issue.path" class="issue-path">{{ issue.path }}</code>
              </div>
            </div>
            <div class="cap-summary">
              <h4>Capabilities</h4>
              <div class="cap-grid">
                <span>Skills: {{ validationReport.capabilities.skills }}</span>
                <span>Hooks: {{ validationReport.capabilities.hooks }}</span>
                <span>Commands: {{ validationReport.capabilities.commands }}</span>
                <span>Mcps: {{ validationReport.capabilities.mcpServers }}</span>
                <span>LSP Servers: {{ validationReport.capabilities.lspServers }}</span>
              </div>
            </div>
          </div>
          <div class="dialog-actions">
            <button class="btn btn-secondary" @click="validationReport = null">Close</button>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="handleValidate">Validate</button>
        <button class="btn btn-secondary" @click="$emit('close')">Close</button>
      </div>
    </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePluginMarketplaceStore } from '@/stores/plugin-marketplace';
import { resolvePluginVersion } from '@/utils/plugin-version';
import { open as openExternal } from '@tauri-apps/plugin-shell';
import type {
  MarketplacePlugin,
  PluginCapabilities,
  HookExecutionResult,
  ValidationReport,
  McpProbeResult,
} from '@/types';

interface Props {
  plugin: MarketplacePlugin;
}

// `defineProps` registers the type for template auto-destructure AND
// exposes a `props` binding we read at runtime. Even when we only use
// the runtime binding from setup, ESLint doesn't see the macro and
// reports the binding as "unused" — suppress inline.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const props = defineProps<Props>();

defineEmits<{
  (e: 'close'): void;
}>();

const marketplaceStore = usePluginMarketplaceStore();

const isLoading = ref(true);
const loadError = ref<string | null>(null);
const capabilities = ref<PluginCapabilities | null>(null);

async function loadCapabilities() {
  isLoading.value = true;
  loadError.value = null;
  try {
    const caps = await marketplaceStore.fetchPluginCapabilities(props.plugin);
    // If capabilities version is missing, resolve from filesystem
    if (!caps.version && caps.installedPath) {
      const resolved = await resolvePluginVersion(caps.installedPath);
      caps.version = resolved !== 'unknown' ? resolved : caps.version;
    }
    capabilities.value = caps;
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  loadCapabilities();
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

// Tab state — default to 'overview'
const activeTab = ref<string>('overview');

interface TabDef {
  id: string;
  label: string;
  count: number;
}

const visibleTabs = computed<TabDef[]>(() => {
  if (!capabilities.value) return [];
  const c = capabilities.value.capabilities;
  const tabs: TabDef[] = [
    { id: 'overview', label: 'Overview', count: 0 },
  ];
  if (c.skills > 0) tabs.push({ id: 'skills', label: 'Skills', count: c.skills });
  if (c.commands > 0) tabs.push({ id: 'commands', label: 'Commands', count: c.commands });
  if (c.hooks > 0) tabs.push({ id: 'hooks', label: 'Hooks', count: c.hooks });
  if (c.mcpServers > 0) tabs.push({ id: 'mcp', label: 'MCP', count: c.mcpServers });
  if (c.lspServers > 0) tabs.push({ id: 'lsp', label: 'LSP', count: c.lspServers });
  return tabs;
});

// Feature 2: Hook execution
const executingHook = ref<string | null>(null);
const hookResult = ref<HookExecutionResult | null>(null);

async function runHookTest(hook: { event: string; matcher?: string }) {
  if (!props.plugin.sourceId || !props.plugin.name) return;
  const key = hook.event + (hook.matcher || '');
  executingHook.value = key;
  try {
    const result = await invoke<HookExecutionResult>('execute_plugin_hook', {
      sourceId: props.plugin.sourceId,
      pluginName: props.plugin.name,
      event: hook.event,
      matcher: hook.matcher || null,
    });
    hookResult.value = result;
  } catch (e) {
    hookResult.value = {
      event: hook.event,
      matcher: hook.matcher,
      command: '',
      exitCode: -1,
      stdout: '',
      stderr: String(e),
      startedAt: new Date().toISOString(),
      durationMs: 0,
      logPath: '',
    };
  } finally {
    executingHook.value = null;
  }
}

// Feature 4: Validation
const validationReport = ref<ValidationReport | null>(null);

async function handleValidate() {
  if (!props.plugin.installPath) return;
  try {
    const report = await invoke<ValidationReport>('validate_plugin_path', {
      path: props.plugin.installPath,
    });
    validationReport.value = report;
  } catch (e) {
    console.error('Validation failed:', e);
  }
}

// Feature 3: MCP probe
const probeState = ref<Record<string, 'idle' | 'running' | 'success' | 'error'>>({});
const probeResults = ref<Record<string, McpProbeResult>>({});

async function runMcpTest(serverName: string) {
  if (!props.plugin.sourceId || !props.plugin.name) return;
  probeState.value[serverName] = 'running';
  try {
    const result = await invoke<McpProbeResult>('probe_plugin_mcp', {
      sourceId: props.plugin.sourceId,
      pluginName: props.plugin.name,
      serverName,
    });
    probeResults.value[serverName] = result;
    probeState.value[serverName] = result.reachable ? 'success' : 'error';
  } catch (e) {
    probeResults.value[serverName] = {
      reachable: false,
      error: String(e),
      durationMs: 0,
    };
    probeState.value[serverName] = 'error';
  }
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
  position: relative;
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
  z-index: 1;
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
  color: var(--fg);
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

/* Loading / Error States */
.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  gap: 16px;
  text-align: center;
}

.loading-state p,
.error-state p {
  color: var(--fg-muted);
  font-size: 14px;
}

.error-state h4 {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
}

.error-state svg {
  color: var(--error, #ef4444);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Plugin Header */
.plugin-header {
  display: flex;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 20px;
}

.plugin-icon {
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

.plugin-title {
  flex: 1;
  min-width: 0;
}

.plugin-title h2 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 8px;
  color: var(--fg);
}

.plugin-meta {
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

.plugin-badges {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
  flex-shrink: 0;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
}

.installed-badge {
  background: var(--success-bg, rgba(16, 185, 129, 0.1));
  color: var(--success, #10b981);
}

.remote-badge {
  background: var(--accent-bg);
  color: var(--accent);
}

/* Sections */
.section {
  margin-bottom: 20px;
}

.section h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin-bottom: 10px;
}

.description {
  font-size: 14px;
  line-height: 1.6;
  color: var(--fg);
}

/* Meta Grid */
.meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.meta-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
}

.meta-value {
  font-size: 13px;
  color: var(--fg);
}

.path-value {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  word-break: break-all;
  color: var(--fg-muted);
}

.meta-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--accent);
  text-decoration: none;
}

.meta-link:hover {
  text-decoration: underline;
}

.deps-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.dep-tag {
  padding: 2px 6px;
  font-size: 11px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
  border-radius: 4px;
}

/* Capability Count Cards */
.capability-counts {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 10px;
  margin-bottom: 20px;
}

.cap-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 8px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
  gap: 6px;
}

.cap-card svg {
  color: var(--fg-ghost);
  transition: color 0.2s;
}

.cap-card.active {
  border-color: var(--accent);
  background: rgba(245, 158, 11, 0.05);
}

.cap-card.active svg {
  color: var(--accent);
}

.cap-card:hover {
  border-color: var(--accent);
}

.cap-value {
  font-size: 22px;
  font-weight: 700;
  color: var(--fg);
  line-height: 1;
}

.cap-label {
  font-size: 11px;
  color: var(--fg-muted);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

/* Tabs */
.tabs-section {
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  overflow: hidden;
  box-shadow: var(--shadow-md);
}

.tabs-header {
  display: flex;
  border-bottom: 1px solid var(--border);
  background: var(--bg-secondary);
  overflow-x: auto;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: var(--fg);
  background: var(--bg-tertiary);
}

.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  font-size: 10px;
  font-weight: 600;
  font-family: var(--font-mono, monospace);
  background: var(--bg-tertiary);
  color: var(--fg-muted);
  border-radius: 9px;
}

.tab-btn.active .tab-count {
  background: rgba(245, 158, 11, 0.15);
  color: var(--accent);
}

.tab-content {
  padding: 16px;
  max-height: 360px;
  overflow-y: auto;
}

.tab-pane {
  min-height: 80px;
}

.empty-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: var(--fg-muted);
  font-size: 14px;
}

/* Capability List */
.cap-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.cap-item {
  padding: 12px 14px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.cap-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.cap-item-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg);
  font-family: var(--font-mono, monospace);
}

.cap-item-badges {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.cap-badge {
  padding: 2px 7px;
  font-size: 10px;
  font-weight: 500;
  border-radius: 4px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.scripts-badge {
  background: rgba(245, 158, 11, 0.1);
  color: var(--accent);
}

.refs-badge {
  background: var(--accent-bg);
  color: var(--accent);
}

.tool-badge {
  background: var(--success-bg);
  color: var(--success);
}

.warn-badge {
  background: var(--error-bg);
  color: var(--error);
}

.cap-item-desc {
  font-size: 13px;
  line-height: 1.5;
  color: var(--fg-muted);
  margin-bottom: 6px;
}

.cap-item-path {
  display: block;
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--fg-ghost);
  word-break: break-all;
}

.cap-item-matcher {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.matcher-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.matcher-re {
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--accent);
  padding: 1px 5px;
  background: var(--accent-bg);
  border-radius: 3px;
}

.cap-item-command {
  display: block;
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--fg-muted);
  word-break: break-all;
}

/* MCP / LSP detail */
.cap-item-detail {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}

.mcp-command {
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  color: var(--accent);
  background: var(--accent-bg);
  padding: 2px 6px;
  border-radius: 4px;
}

.mcp-args {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: var(--font-mono, monospace);
}

.mcp-env {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 4px;
}

.env-label {
  font-size: 11px;
  color: var(--fg-muted);
}

.env-pair {
  font-size: 11px;
  color: var(--fg-muted);
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: 3px;
}

.env-pair code {
  color: var(--accent);
}

/* Manifest files */
.manifest-files-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.manifest-file {
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  color: var(--fg-muted);
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
}

/* Actions */
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

/* Hook Result & Validation dialogs */
.hook-result-dialog,
.validation-dialog {
  max-width: 720px;
}

.dialog-subtitle {
  font-size: 12px;
  color: var(--fg-muted);
  display: flex;
  align-items: center;
  gap: 6px;
}

.result-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.result-section h4 {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin: 0 0 8px;
}

.result-output {
  width: 100%;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 12px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  line-height: 1.6;
  color: var(--fg);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
  margin: 0;
}

.success-output { border-left: 3px solid var(--success); }
.error-output { border-left: 3px solid var(--error); }

.log-path-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}
.log-path-label { font-size: 11px; color: var(--fg-muted); }
.log-path { font-size: 11px; font-family: var(--font-mono, monospace); color: var(--fg-ghost); word-break: break-all; }

.validation-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}
.validation-badge.valid { background: var(--success-bg); color: var(--success); }
.validation-badge.invalid { background: var(--error-bg); color: var(--error); }

.issue-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 8px;
}
.error-issue { background: var(--error-bg); border: 1px solid rgba(239, 68, 68, 0.2); }
.warn-issue { background: var(--warn-bg); border: 1px solid rgba(245, 158, 11, 0.2); }

.issue-code {
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono, monospace);
  color: var(--fg);
}
.error-issue .issue-code { color: var(--error); }
.warn-issue .issue-code { color: var(--accent); }

.issue-msg { font-size: 12px; color: var(--fg-muted); line-height: 1.4; }
.issue-path { font-size: 10px; font-family: var(--font-mono, monospace); color: var(--fg-ghost); word-break: break-all; }

.cap-summary { margin-top: 8px; }
.cap-summary h4 {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin: 0 0 8px;
}
.cap-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 6px;
  font-size: 12px;
  color: var(--fg-muted);
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 12px;
  box-shadow: var(--shadow-md);
}
.cap-grid span { color: var(--fg); }

.error-heading { color: var(--error) !important; }
.warn-heading { color: var(--accent) !important; }

/* MCP probe result */
.probe-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  margin-bottom: 6px;
}
.probe-result.success {
  background: var(--success-bg);
  color: var(--success);
  border: 1px solid rgba(16, 185, 129, 0.2);
}
.probe-result.error {
  background: var(--error-bg);
  color: var(--error);
  border: 1px solid rgba(239, 68, 68, 0.2);
}
.probe-info { opacity: 0.8; }
</style>

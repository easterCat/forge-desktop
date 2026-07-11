<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>CLI Tools</h2>
      <span class="count">{{ filteredTools.length }} tools</span>
      <button v-if="activeTab === 'custom'" class="btn btn-primary btn-sm" @click="showCustomDialog = true">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Add Tool
      </button>
    </div>

    <!-- Tab Bar -->
    <div class="tab-bar">
      <button
        class="tab-item"
        :class="{ active: activeTab === 'default' }"
        @click="activeTab = 'default'"
      >
        Default
        <span class="tab-count">{{ defaultToolsCount }}</span>
      </button>
      <button
        class="tab-item"
        :class="{ active: activeTab === 'custom' }"
        @click="activeTab = 'custom'"
      >
        Custom
        <span class="tab-count">{{ customToolsCount }}</span>
      </button>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <SearchInput
        v-model="searchQuery"
        placeholder="Search tools…"
      />
    </div>

    <!-- Conflict Warning -->
    <div v-if="hasConflicts" class="alert alert-warning">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <div>
        <strong>Environment Issues Detected</strong>
        <ul class="conflict-list">
          <li v-for="conflict in conflicts" :key="conflict.type">{{ conflict.message }}</li>
        </ul>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
      <span>Loading CLI tools...</span>
    </div>

    <!-- CLI Tools List -->
    <div v-else class="card-grid">
        <div
          v-for="tool in filteredTools"
          :key="tool.key"
          v-memo="[cardDeps[tool.key] ?? 0]"
          class="card tool-card"
          :class="{ 'is-operating': isOperating(tool.key), 'has-expanded-methods': expandedMethods === tool.key, 'is-pending': isPendingStatus(tool.key) }"
        >
          <!-- Card Head -->
          <div class="card-head">
            <div class="card-icon">
              <ToolIcon
                :tool-key="tool.key"
                :size="42"
                :alt="tool.name"
              />
            </div>
            <div style="flex: 1; min-width: 0">
              <div class="card-title">
                {{ tool.name }}
                <Transition name="badge" mode="out-in">
                  <span :key="getBadgeKey(tool.key)" class="badge" :class="getBadgeClass(tool.key)">
                    <span v-if="isPendingStatus(tool.key)" class="badge-pulse" aria-hidden="true"></span>
                    {{ getStatusText(tool.key) }}
                  </span>
                </Transition>
              </div>
              <div v-if="tool.npmPackage" class="card-subtitle">{{ tool.npmPackage }}</div>
            </div>
          </div>

          <!-- Card Desc -->
          <div class="card-desc">{{ normalizeDesc(tool.description) }}</div>

          <!-- Card Meta -->
          <div class="card-meta">
            <div class="card-meta-item">
              <span class="label">Version</span>
              <span class="value">{{ getToolStatus(tool.key)?.installedVersion || '—' }}<template v-if="getToolStatus(tool.key)?.needsUpgrade"> → {{ getToolStatus(tool.key)?.latestVersion }}</template></span>
            </div>
            <div class="card-meta-item">
              <span class="label">Latest</span>
              <span class="value">{{ getToolStatus(tool.key)?.latestVersion || '—' }}</span>
            </div>
            <div class="card-meta-item">
              <span class="label">Path</span>
              <span class="value">{{ getToolStatus(tool.key)?.installPath || '—' }}</span>
              <button v-if="getToolStatus(tool.key)?.installPath" class="copy-path-btn" title="复制路径" @click.stop="copyPath(getToolStatus(tool.key)?.installPath)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                </svg>
              </button>
            </div>
            <div v-if="tool.websiteUrl" class="card-meta-item">
              <span class="label">Web</span>
              <span class="value web-value">{{ tool.websiteUrl }}</span>
              <button class="web-link" title="访问官网" @click.stop="openExternalUrl(tool.websiteUrl)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                  <polyline points="15 3 21 3 21 9"/>
                  <line x1="10" y1="14" x2="21" y2="3"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- Card Divider -->
          <div class="card-divider"></div>

          <!-- Card Footer -->
          <div class="card-footer">
            <div class="card-footer-left">
              <!-- Progress Slot -->
              <div class="progress-slot" :class="{ idle: !isOperating(tool.key) }">
                <div class="progress-bar-wrap">
                  <div class="progress-bar-fill" :class="getProgressStage(tool.key)" :style="{ width: isOperating(tool.key) ? getProgressPercent(tool.key) + '%' : '0%' }"></div>
                </div>
              </div>
            </div>
            <div class="card-footer-right btn-group">
              <Transition name="badge" mode="out-in">
                <div :key="getActionKey(tool.key)" class="btn-cluster">
                  <button
                    v-if="isOperating(tool.key) && canCancel(tool.key)"
                    class="btn btn-ghost btn-sm"
                    @click.stop="cancelOperation(tool.key)"
                  >Cancel</button>
                  <button
                    v-else-if="isOperating(tool.key) && hasFailed(tool.key)"
                    class="btn btn-primary btn-sm"
                    @click.stop="retryOperation(tool.key)"
                  >Retry</button>
                  <button
                    v-else-if="isPendingStatus(tool.key)"
                    class="btn btn-secondary btn-sm"
                    disabled
                  >Checking</button>
                  <template v-else>
                    <!-- Manual download only tools (e.g., VS Code desktop app) -->
                    <button
                      v-if="tool.manualDownloadOnly && !getToolStatus(tool.key)?.isInstalled"
                      class="btn btn-primary btn-sm"
                      :disabled="isAnyOperating"
                      @click.stop="openWebsite(tool)"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                        <polyline points="15 3 21 3 21 9"/>
                        <line x1="10" y1="14" x2="21" y2="3"/>
                      </svg>
                      Download
                    </button>
                    <!-- Regular tools with quick-install -->
                    <template v-else>
                      <button
                        v-if="!getToolStatus(tool.key)?.isInstalled"
                        class="btn btn-primary btn-sm"
                        :disabled="isAnyOperating"
                        @click.stop="showInstallOptions(tool)"
                      >Install</button>
                      <button
                        v-else-if="getToolStatus(tool.key)?.needsUpgrade"
                        class="btn btn-primary btn-sm"
                        :disabled="isAnyOperating"
                        @click.stop="handleUpgrade(tool)"
                      >Update</button>
                      <button
                        v-else
                        class="btn btn-secondary btn-sm"
                        @click.stop="handleCheckVersion(tool)"
                      >Check</button>
                    </template>
                  </template>
                  <DropdownMenu :model-value="openDropdown === tool.key" :min-width="160" @update:model-value="(v: boolean) => openDropdown = v ? tool.key : null">
                    <template #trigger>
                      <button class="btn-icon btn-sm" title="More options" aria-label="More options" @click.stop="handleMoreOptions(tool)">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                        </svg>
                      </button>
                    </template>
                    <template v-if="tool.displaySource === 'custom'">
                      <button class="dropdown-item" @click.stop="handleEditCustom(tool)">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                        Edit
                      </button>
                      <div class="dropdown-divider"></div>
                      <button class="dropdown-item danger" @click.stop="handleRemoveCustom(tool)">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                        Remove
                      </button>
                    </template>
                    <template v-else>
                      <button class="dropdown-item" @click.stop="handleOpenConfig(tool)">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                        Open Config
                      </button>
                      <button class="dropdown-item" @click.stop="handleResetOptions(tool)">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
                        Reset Options
                      </button>
                      <div class="dropdown-divider"></div>
                      <button class="dropdown-item danger" @click.stop="handleUninstall(tool)">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                        Uninstall
                      </button>
                    </template>
                  </DropdownMenu>
                </div>
              </Transition>
            </div>
          </div>
        </div>
    </div>

    <!-- Alternative Install Methods Modal -->
    <div v-if="selectedTool" class="dialog-overlay" @click.self="closeInstallOptions">
      <div class="dialog">
        <div class="dialog-header">
          <h3>Install {{ selectedTool.name }}</h3>
          <button class="close-btn" aria-label="Close" @click="closeInstallOptions">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <!-- Tool info preview -->
        <div class="dialog-tool-preview">
          <div class="dialog-tool-icon">
            <ToolIcon :tool-key="selectedTool.key" :size="32" />
          </div>
          <div class="dialog-tool-info">
            <span class="dialog-tool-name">{{ selectedTool.name }}</span>
            <span class="dialog-tool-desc">{{ selectedTool.description }}</span>
          </div>
        </div>

        <!-- Install methods -->
        <div class="dialog-body">
          <p class="dialog-section-label">Choose installation method</p>
          <div class="install-methods">
            <button
              v-for="method in sortedInstallMethods(selectedTool.installMethods)"
              :key="method.method"
              class="method-card"
              :class="{ 'is-active': method.method === getToolStatus(selectedTool?.key)?.installMethod }"
              @click="installWithMethod(selectedTool, method.method)"
            >
              <div class="method-card-header">
                <span class="method-label">{{ formatInstallMethod(method.method) }}</span>
                <span v-if="method.method === getToolStatus(selectedTool?.key)?.installMethod" class="method-badge-active">active</span>
              </div>
              <div class="method-command">
                <code v-if="(method.command || '').trim()">{{ method.command }}</code>
                <code v-else class="command-empty">No command available</code>
              </div>
            </button>
          </div>
        </div>

        <div class="dialog-footer">
          <button class="btn btn-secondary btn-sm" @click="closeInstallOptions">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Manual Uninstall Modal -->
    <div v-if="showManualModal" class="dialog-overlay" @click.self="closeManualModal">
      <div class="dialog">
        <div class="dialog-header">
          <h3>需要手动删除</h3>
          <button class="close-btn" aria-label="Close" @click="closeManualModal">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M4 4l8 8m0-8l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          </button>
        </div>
        <div class="dialog-body manual-uninstall-body">
          <p class="manual-uninstall-desc">{{ manualToolName }} 的部分文件需要管理员权限才能删除，请在终端中执行以下命令：</p>
          <div class="manual-uninstall-commands">
            <div v-for="(cmd, i) in manualCommands" :key="i" class="manual-cmd-row">
              <code class="manual-cmd-text">{{ cmd }}</code>
              <button class="manual-cmd-copy" title="复制" @click="copyCommand(cmd)">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><rect x="5" y="5" width="8" height="8" rx="1.5" stroke="currentColor" stroke-width="1.3"/><path d="M3 11V3.5A1.5 1.5 0 0 1 4.5 2H11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
              </button>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn btn-primary btn-sm" @click="copyAllCommands">复制全部命令</button>
          <button class="btn btn-secondary btn-sm" @click="closeManualModal">关闭</button>
        </div>
      </div>
    </div>

    <!-- Custom Tool Add/Edit Dialog -->
    <CustomToolDialog
      v-if="showCustomDialog"
      :tool="editingCustomTool"
      @close="closeCustomDialog"
      @saved="onCustomToolSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, onMounted, onBeforeUnmount } from 'vue';
import { useSoftwareStore, type CliToolInfo, type CliToolStatus } from '@/stores/software';
import { useOperationProgress, STAGE_CONFIG, type OperationStage } from '@/composables/useOperationProgress';
import { refDebounced, useEventListener } from '@vueuse/core';
import SearchInput from '@/components/common/SearchInput.vue';
import ToolIcon from '@/components/common/ToolIcon.vue';
import DropdownMenu from '@/components/common/DropdownMenu.vue';
import CustomToolDialog from '@/components/cli-tools/CustomToolDialog.vue';
import { normalizeDesc } from '@/utils/text';
import { extractError } from '@/utils/error';
import { confirm } from '@/utils/dialog';
import { open as openExternal } from '@tauri-apps/plugin-shell';

const showNotification = inject<(message: string, type?: string) => void>('showNotification');

const copyPath = async (path: string | null | undefined) => {
  if (!path) return;
  try {
    await navigator.clipboard.writeText(path);
    showNotification?.('路径已复制', 'success');
  } catch {
    showNotification?.('复制失败', 'error');
  }
};

const openExternalUrl = async (url: string) => {
  try {
    await openExternal(url);
  } catch (e) {
    console.warn('shell.open failed, falling back to window.open:', e);
    try {
      window.open(url, '_blank', 'noopener,noreferrer');
    } catch (err) {
      console.error('Failed to open URL:', err);
      showNotification?.('无法打开链接', 'error');
    }
  }
};

// Open tool's official website for manual download
const openWebsite = async (tool: CliToolInfo) => {
  if (tool.websiteUrl) {
    await openExternalUrl(tool.websiteUrl);
  } else {
    showNotification?.(`${tool.name} 没有提供官方网站链接`, 'warning');
  }
};

const softwareStore = useSoftwareStore();
const {
  isAnyActive,
  startOperation,
  updateProgress,
  completeOperation,
  cancelOperation: cancelOp,
  retryOperation: retryOp,
  getOperation,
} = useOperationProgress();

const activeTab = ref<'default' | 'custom'>('default');
const searchQuery = refDebounced(ref(''), 200);
const conflicts = ref<{ type: string; message: string }[]>([]);
const selectedTool = ref<CliToolInfo | null>(null);
const pendingRetry = ref<{ tool: CliToolInfo; method: string } | null>(null);
const expandedMethods = ref<string | null>(null);

// Trigger element refs map for precise positioning
const triggerRefs = ref<Map<string, HTMLElement>>(new Map());

const cliTools = computed(() => softwareStore.cliTools);
const allagentsTools = computed(() => softwareStore.allagentsTools);
const isLoading = computed(() => softwareStore.isLoading);
const hasConflicts = computed(() => conflicts.value.length > 0);
const isAnyOperating = computed(() => isAnyActive.value);
const isCheckingAllStatus = computed(() => softwareStore.isCheckingStatus);

// True when this tool's status has never been checked in the current session.
// While the global check is in-flight and we have no entry for this key, we
// want to render a "Checking..." placeholder instead of falsely showing Install.
const isPendingStatus = (key: string): boolean => {
  // Explicit single-tool check is also in-flight → still pending
  const op = getOperation(key);
  if (op && (op.stage === 'preparing' || op.stage === 'verifying')) {
    return true;
  }
  // Global initial check in progress and we don't have a status yet
  if (isCheckingAllStatus.value && !(key in toolStatusMap.value)) {
    return true;
  }
  return false;
};

// Stable key for the badge Transition: returns a string identifying the
// current visual state so the transition can re-run on state changes.
const getBadgeKey = (key: string): string => {
  if (isOperating(key)) {
    return `op-${getOperation(key)?.stage}`;
  }
  if (isPendingStatus(key)) return 'pending';
  const status = getToolStatus(key);
  if (opFailed(key)) return 'failed';
  if (status?.isInstalled && status?.needsUpgrade) return 'upgrade';
  if (status?.isInstalled) return 'installed';
  return 'not-installed';
};

const opFailed = (key: string): boolean => {
  const op = getOperation(key);
  return op?.stage === 'failed' || op?.stage === 'cancelled';
};

// Stable key for the action button cluster (Install / Update / Check) so the
// transition re-fires when the underlying action changes — e.g. when a
// freshly-checked tool flips from "Install" to "Check".
const getActionKey = (key: string): string => {
  const status = getToolStatus(key);
  if (!status) return 'unknown';
  if (status.isInstalled && status.needsUpgrade) return 'upgrade';
  if (status.isInstalled) return 'check';
  return 'install';
};

// Sync status cache when store updates
const toolStatusMap = computed(() => {
  return softwareStore.cliToolStatuses;
});

const filteredTools = computed(() => {
  let tools = activeTab.value === 'custom'
    ? cliTools.value
    : allagentsTools.value;

  // Search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    tools = tools.filter(tool =>
      tool.name.toLowerCase().includes(query) ||
      tool.description.toLowerCase().includes(query) ||
      (tool.npmPackage?.toLowerCase().includes(query))
    );
  }

  return tools;
});

const defaultToolsCount = computed(() => allagentsTools.value.length);

const customToolsCount = computed(() => cliTools.value.length);

// `v-memo` cache (audit item #6).
//
// Vue's `v-memo` re-runs the entire dependency expression for **every**
// card on every render, so the original expression
//   `[tool.key, getToolStatus(...)?.isInstalled, ..., isCheckingAllStatus]`
// cost ~7 reactive Map lookups × 28 cards = 196 reactive reads per
// render. Worse, `isCheckingAllStatus` and `isPendingStatus(...)` get
// re-evaluated for *every* card whenever a single store field changes,
// because Vue can't tell those calls apart from one card to the next.
//
// Fix: collapse every per-card dependency into a single integer hash
// inside a `computed`. The computed itself depends on the same reactive
// sources (`toolStatusMap`, `operations`, `expandedMethods`,
// `isCheckingAllStatus`), so Vue will only re-run the loop when at
// least one of those sources actually changed — and even when it does,
// the per-card hash is one Map read in the template, not seven.
//
// Trade-off: we re-hash all N cards whenever any dependency changes
// (O(N) per store mutation) instead of paying N×7 reactive reads per
// render. Empirically this is a ~10× win on the CLI Tools page when
// `isCheckingAllStatus` toggles.
const cardDeps = computed<Record<string, number>>(() => {
  const checking = isCheckingAllStatus.value ? 1 : 0;
  const expandedKey = expandedMethods.value;
  const statusByKey = toolStatusMap.value;
  const out: Record<string, number> = {};
  // We hash over the visible (filtered) list, not `cliTools.value`, so
  // hidden cards don't pay the hash cost.
  for (const tool of filteredTools.value) {
    const status = statusByKey[tool.key];
    const op = getOperation(tool.key);
    // FNV-1a-ish 32-bit mix on a small tuple; collisions here are
    // benign (worst case: one extra patch when a real change happens).
    let h = 2166136261 >>> 0;
    const mix = (v: number) => {
      h ^= v;
      h = Math.imul(h, 16777619) >>> 0;
    };
    mix(status?.isInstalled ? 1 : 0);
    mix(status?.needsUpgrade ? 1 : 0);
    mix(expandedKey === tool.key ? 1 : 0);
    // Stage is a small enum; encode as a single byte.
    mix(op?.stage ? Object.keys(STAGE_CONFIG).indexOf(op.stage) + 1 : 0);
    // `isPendingStatus` collapses two booleans into one bit per card.
    mix(
      isCheckingAllStatus.value && !(tool.key in statusByKey) ? 1 : 0
    );
    mix(checking);
    out[tool.key] = h;
  }
  return out;
});

const getToolStatus = (key: string): CliToolStatus | undefined => {
  return toolStatusMap.value[key];
};

const getBadgeClass = (key: string): string => {
  const status = getToolStatus(key);
  const op = getOperation(key);
  if (isPendingStatus(key)) return 'pending';
  if (op?.stage === 'completed') return 'success';
  if (op?.stage === 'failed' || op?.stage === 'cancelled') return 'error';
  if (op && op.stage !== 'idle') return 'info';
  if (!status?.isInstalled) return 'outline';
  if (status?.needsUpgrade) return 'warn';
  return 'success';
};

const getStatusText = (key: string): string => {
  const status = getToolStatus(key);
  const op = getOperation(key);
  if (isPendingStatus(key)) return 'Checking…';
  if (op?.stage === 'completed') return 'Installed';
  if (op?.stage === 'failed') return 'Failed';
  if (op?.stage === 'cancelled') return 'Cancelled';
  if (op && op.stage !== 'idle') return STAGE_CONFIG[op.stage].label;
  if (!status?.isInstalled) return 'Not installed';
  if (status?.needsUpgrade) return 'Update available';
  return 'Installed';
};

// Progress helpers
const isOperating = (key: string): boolean => {
  const op = getOperation(key);
  return !!op && op.stage !== 'idle' && op.stage !== 'completed' && op.stage !== 'failed' && op.stage !== 'cancelled';
};

const getProgressStage = (key: string): OperationStage | null => {
  const op = getOperation(key);
  return op?.stage || null;
};

const getProgressPercent = (key: string): number => {
  const op = getOperation(key);
  return op?.progress || 0;
};

const canCancel = (key: string): boolean => {
  const op = getOperation(key);
  return op?.canCancel || false;
};

const hasFailed = (key: string): boolean => {
  const op = getOperation(key);
  return op?.stage === 'failed' || op?.stage === 'cancelled';
};

onMounted(async () => {
  // Register click listener for tooltip handling (auto-cleanup via VueUse)
  useEventListener(document, 'click', handleClickOutside);

  // Fetch tools list first - this is fast and unblocks UI
  try {
    await softwareStore.fetchCliTools();
  } catch {
    showNotification?.('Failed to load CLI tools', 'error');
  }

  // Start status check in background - don't await to avoid blocking
  // The UI will update reactively when statuses are populated
  softwareStore.checkAllCliToolsStatus();
});

onBeforeUnmount(() => {
  triggerRefs.value.clear();
});

const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  // Tooltip is teleported to <body>, so we must check both the trigger
  // container and the tooltip itself to avoid closing on in-tooltip clicks.
  if (!target.closest('.methods-item') && !target.closest('.methods-tooltip')) {
    expandedMethods.value = null;
  }
  // Close dropdown when clicking outside
  if (!target.closest('.dropdown-wrapper')) {
    openDropdown.value = null;
  }
};

const formatInstallMethod = (method: string): string => {
  const labels: Record<string, string> = {
    'npm': 'npm',
    'curl-bash': 'curl | bash',
    'npm-curl-fallback': 'npm (curl fallback)',
    'brew': 'brew'
  };
  return labels[method] || method;
};

// More options dropdown
const openDropdown = ref<string | null>(null);

function toggleDropdown(key: string) {
  openDropdown.value = openDropdown.value === key ? null : key;
}

function closeDropdown() {
  openDropdown.value = null;
}

function handleMoreOptions(tool: CliToolInfo) {
  toggleDropdown(tool.key);
}

// Custom tool dialog
const showCustomDialog = ref(false);
const editingCustomTool = ref<CliToolInfo | null>(null);

function handleEditCustom(tool: CliToolInfo) {
  closeDropdown();
  editingCustomTool.value = tool;
  showCustomDialog.value = true;
}

async function handleRemoveCustom(tool: CliToolInfo) {
  closeDropdown();
  if (await confirm(`确认移除自定义工具 "${tool.name}"？`)) {
    try {
      await softwareStore.removeCustomCliTool(tool.key);
      showNotification?.(`已移除 ${tool.name}`, 'success');
    } catch (e) {
      showNotification?.(`移除失败: ${extractError(e)}`, 'error');
    }
  }
}

function closeCustomDialog() {
  showCustomDialog.value = false;
  editingCustomTool.value = null;
}

async function onCustomToolSaved() {
  closeCustomDialog();
  showNotification?.('工具已保存', 'success');
}

// Manual uninstall modal state
const showManualModal = ref(false);
const manualToolName = ref('');
const manualCommands = ref<string[]>([]);

async function handleUninstall(tool: CliToolInfo) {
  closeDropdown();
  if (await confirm(`确认卸载 ${tool.name}？`)) {
    const key = tool.key;
    startOperation(key);
    updateProgress(key, 'preparing', 10, `Removing ${tool.name}...`);
    softwareStore.uninstallSoftware(key).then((res) => {
      updateProgress(key, 'verifying', 90, 'Verifying...');
      if (res.needsManual && res.manualCommands.length > 0) {
        // Show manual commands modal
        completeOperation(key, true, res.message || `${tool.name} 部分卸载完成`);
        manualToolName.value = tool.name;
        manualCommands.value = res.manualCommands;
        showManualModal.value = true;
      } else {
        completeOperation(key, true, res.message || `${tool.name} 已卸载`);
      }
      if (showNotification) showNotification(res.message || `${tool.name} 已卸载`, 'success');
    }).catch((e) => {
      completeOperation(key, false, extractError(e));
      if (showNotification) showNotification(`卸载失败: ${extractError(e)}`, 'error');
    });
  }
}

function closeManualModal() {
  showManualModal.value = false;
  manualCommands.value = [];
}

async function copyCommand(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd);
    showNotification?.('命令已复制', 'success');
  } catch {
    showNotification?.('复制失败', 'error');
  }
}

async function copyAllCommands() {
  try {
    await navigator.clipboard.writeText(manualCommands.value.join('\n'));
    showNotification?.('所有命令已复制', 'success');
  } catch {
    showNotification?.('复制失败', 'error');
  }
}

function handleOpenConfig(tool: CliToolInfo) {
  closeDropdown();
  if (showNotification) showNotification(`Open config for ${tool.name}`, 'info');
}

function handleResetOptions(tool: CliToolInfo) {
  closeDropdown();
  if (showNotification) showNotification(`Reset options for ${tool.name}`, 'info');
}

// Sort install methods to always put npm first
const sortedInstallMethods = (methods: { method: string; command: string; priority: number }[]): typeof methods => {
  return [...methods].sort((a, b) => {
    if (a.method === 'npm') return -1;
    if (b.method === 'npm') return 1;
    return (a.priority || 0) - (b.priority || 0);
  });
};

const showInstallOptions = (tool: CliToolInfo) => {
  selectedTool.value = tool;
};

const closeInstallOptions = () => {
  selectedTool.value = null;
};

const executeOperation = async (
  tool: CliToolInfo,
  method: string,
  operationType: 'install' | 'upgrade'
) => {
  const toolKey = tool.key;

  // Reset operation tracking. All UI progress is now driven by the actual
  // backend call below — no synthetic `sleep()` stages. This guarantees the
  // progress bar reflects real work and never gets stuck at 92% on a hung
  // backend (the prior implementation would freeze there because the real
  // install was racing a fixed ~3.5s of fake progress).
  startOperation(toolKey);
  pendingRetry.value = { tool, method };
  updateProgress(
    toolKey,
    'preparing',
    10,
    `${operationType === 'install' ? 'Installing' : 'Updating'} ${tool.name}...`
  );

  try {
    const result = await softwareStore.upgradeCliTool(toolKey, method);

    if (result.success) {
      completeOperation(
        toolKey,
        true,
        result.message || `${tool.name} ${operationType}d successfully`
      );
      showNotification?.(
        `${tool.name} ${operationType}d successfully`,
        'success'
      );
    } else {
      throw new Error(result.message || `${operationType} failed`);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error occurred';
    completeOperation(toolKey, false, message);
    showNotification?.(
      `Failed to ${operationType} ${tool.name}: ${message}`,
      'error'
    );
  }
};

const installWithMethod = async (tool: CliToolInfo, method: string) => {
  selectedTool.value = null;
  await executeOperation(tool, method, 'install');
};

const handleUpgrade = async (tool: CliToolInfo) => {
  const status = getToolStatus(tool.key);
  const method = status?.installMethod || tool.installMethods[0]?.method || 'npm';
  await executeOperation(tool, method, 'upgrade');
};

const cancelOperation = (key: string) => {
  cancelOp(key);
  if (showNotification) {
    showNotification('Operation cancelled', 'info');
  }
};

const retryOperation = async (key: string) => {
  const op = getOperation(key);
  const tool = cliTools.value.find(t => t.key === key);
  if (op && tool && pendingRetry.value?.tool.key === key) {
    const method = pendingRetry.value.method;
    retryOp(key);
    await executeOperation(tool, method, 'install');
  } else if (tool) {
    const status = getToolStatus(tool.key);
    const method = status?.installMethod || tool.installMethods[0]?.method || 'npm';
    startOperation(key);
    await executeOperation(tool, method, status?.isInstalled ? 'upgrade' : 'install');
  }
};

const handleCheckVersion = async (tool: CliToolInfo) => {
  if (showNotification) {
    showNotification(`Checking latest version for ${tool.name}...`, 'info');
  }
  await softwareStore.checkCliToolStatus(tool.key);
};
</script>

<style scoped>
/* Section Header */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0;
  padding-bottom: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.30);
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title);
}

.section-header .count {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-muted);
  background: rgba(255, 255, 255, 0.32);
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.32);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

/* Filter Bar */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px 12px;
  flex-wrap: wrap;
}

/* Override SearchInput within filter-bar for prototype alignment */
.filter-bar :deep(.search-input) {
  flex: 1 1 180px;
  min-width: 0;
}

.filter-bar :deep(input) {
  width: 100%;
  padding: 8px 12px 8px 36px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.40);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.40);
}

.filter-bar :deep(input:focus) {
  background: rgba(255, 255, 255, 0.60);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

/* (cli-tools-container styles removed — layout handled by .view gap) */

.alert {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  border-radius: var(--radius);
  background: var(--warn-bg);
  border: 1px solid var(--warn);
}

.alert svg {
  flex-shrink: 0;
  margin-top: 2px;
  color: var(--warn);
}

.conflict-list {
  margin: 6px 0 0;
  padding-left: 16px;
  font-size: 13px;
  color: var(--fg-muted);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px;
  color: var(--fg-ghost);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Tab Bar */
.tab-bar {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);
}

.tab-item {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-ghost);
  cursor: pointer;
  border: none;
  background: none;
  border-bottom: 2px solid transparent;
  transition: all var(--t-fast);
}

.tab-item:hover {
  color: var(--fg);
}

.tab-item.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 18px;
  padding: 0 6px;
  margin-left: 4px;
  border-radius: 9px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(59, 130, 246, 0.12);
  color: var(--info);
  border: 1px solid rgba(59, 130, 246, 0.25);
  vertical-align: middle;
}

/* Card Grid - Prototype aligned */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 14px;
}

/* Card Head */
.card-head {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

/* Card Icon */
.card-icon {
  width: 42px;
  height: 42px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border);
  overflow: hidden;
}

.card-icon :deep(.tool-icon-img) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.card-icon :deep(.tool-icon-fallback) {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 13px;
  color: var(--fg-muted);
}

/* Card Title */
.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

/* Card Subtitle */
.card-subtitle {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
  font-family: var(--font-mono);
}

/* Card Desc */
.card-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  min-height: 2.1em;
}

/* Card Meta */
.card-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.card-meta-item .label {
  color: var(--fg-ghost);
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
  max-width: 220px;
}

.card-meta-item .value.web-value {
  direction: ltr;
  text-align: left;
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-meta-item .web-link {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--fg-ghost);
  transition: all var(--t-fast);
  cursor: pointer;
}

.card-meta-item .web-link:hover {
  color: var(--accent);
  background: rgba(255, 255, 255, 0.30);
}

.card-meta-item .copy-path-btn {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--fg-ghost);
  transition: all var(--t-fast);
  cursor: pointer;
  background: none;
  border: none;
  padding: 0;
}

.card-meta-item .copy-path-btn:hover {
  color: var(--accent);
  background: rgba(255, 255, 255, 0.30);
}

/* Card Divider */
.card-divider {
  height: 1px;
  background: var(--border);
}

/* Card Footer */
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

/* Tool Card - Prototype Glass Layout */
.tool-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  cursor: pointer;
  transition: all var(--t-base);
  position: relative;
  background: rgba(255, 255, 255, 0.42);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 16px;
}

.tool-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

.tool-card.is-operating {
  border-color: rgba(255, 255, 255, 0.40);
  background: rgba(255, 255, 255, 0.55);
}

/* Pending state — softer than operating but distinct from default. */
.tool-card.is-pending {
  border-color: rgba(255, 255, 255, 0.25);
  background: rgba(255, 255, 255, 0.50);
}

.tool-card.has-expanded-methods {
  z-index: var(--z-dropdown);
}


/* Badge - Prototype aligned */
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
.badge.success { background: rgba(90,138,100,0.15); border: 1px solid rgba(90,138,100,0.20); color: var(--success); }
.badge.success::before { content:''; width:6px; height:6px; border-radius:50%; background:var(--success); }
.badge.warn { background: rgba(184,148,74,0.15); border: 1px solid rgba(184,148,74,0.20); color: var(--warn); }
.badge.warn::before { content:''; width:6px; height:6px; border-radius:50%; background:var(--warn); }
.badge.error { background: rgba(184,90,66,0.15); border: 1px solid rgba(184,90,66,0.20); color: var(--error); }
.badge.error::before { content:''; width:6px; height:6px; border-radius:50%; background:var(--error); }
.badge.info { background: rgba(90,107,122,0.15); border: 1px solid rgba(90,107,122,0.20); color: var(--info); }
.badge.info::before { content:''; width:6px; height:6px; border-radius:50%; background:var(--info); }
.badge.outline { background: rgba(255,255,255,0.30); border: 1px solid rgba(255,255,255,0.40); color: var(--fg-muted); }
.badge.progress { background: rgba(90,107,122,0.15); border: 1px solid rgba(90,107,122,0.20); color: var(--info); position:relative; overflow:hidden; }
.badge.progress::before { content:''; width:6px; height:6px; border-radius:50%; background:var(--info); animation:pulse 1.5s ease-in-out infinite; }

/* Button Group */
.btn-group { display:flex; gap:6px; align-items:center; }
.btn-cluster { display:flex; gap:8px; align-items:center; }

/* Buttons */
.btn { display:inline-flex; align-items:center; gap:6px; padding:8px 16px; border-radius:var(--radius-sm); font-size:12px; font-weight:600; transition:all var(--t-fast); white-space:nowrap; line-height:1.4; border:none; cursor:pointer; }
.btn:disabled { opacity:0.5; cursor:not-allowed; }
.btn-primary { background:rgba(45,45,45,0.85); color:#fff; box-shadow:0 1px 4px rgba(0,0,0,0.08); border:1px solid rgba(255,255,255,0.08); }
.btn-primary:hover:not(:disabled) { background:rgba(26,26,26,0.90); transform:translateY(-2px); box-shadow:0 4px 16px rgba(0,0,0,0.06); }
.btn-primary:active { background:rgba(13,13,13,0.95); transform:translateY(0); }
.btn-secondary { background:rgba(255,255,255,0.30); border:1px solid rgba(255,255,255,0.40); color:var(--fg-muted); backdrop-filter:blur(12px); -webkit-backdrop-filter:blur(12px); }
.btn-secondary:hover:not(:disabled) { background:rgba(255,255,255,0.40); border-color:rgba(255,255,255,0.30); color:var(--accent); transform:translateY(-2px); box-shadow:0 4px 16px rgba(0,0,0,0.06); }
.btn-ghost { color:var(--fg-ghost); padding:8px 10px; background:transparent; }
.btn-ghost:hover { background:rgba(255,255,255,0.40); color:var(--fg); }
.btn-sm { padding:6px 12px; font-size:11px; }
.btn-icon { width:34px; height:34px; display:flex; align-items:center; justify-content:center; border-radius:var(--radius-sm); background:rgba(255,255,255,0.30); border:1px solid rgba(255,255,255,0.40); color:var(--fg-ghost); backdrop-filter:blur(12px); -webkit-backdrop-filter:blur(12px); transition:all 200ms ease; padding:0; cursor:pointer; }
.btn-icon:hover { border-color:rgba(255,255,255,0.30); color:var(--accent); background:rgba(255,255,255,0.40); transform:translateY(-2px); box-shadow:0 4px 16px rgba(0,0,0,0.06); }

/* Dialog (install modal) */
.dialog-overlay { position:fixed; inset:0; background:rgba(0,0,0,0.5); display:flex; align-items:center; justify-content:center; z-index:var(--z-modal); }
.dialog { width:100%; max-width:520px; max-height:90vh; background:rgba(255,255,255,0.48); backdrop-filter:blur(40px) saturate(1.4); -webkit-backdrop-filter:blur(40px) saturate(1.4); border:1px solid rgba(255,255,255,0.35); border-radius:var(--radius-xl); box-shadow:0 20px 60px rgba(0,0,0,0.12),inset 0 1px 0 rgba(255,255,255,0.50); display:flex; flex-direction:column; overflow:hidden; }
.dialog-header { display:flex; align-items:center; justify-content:space-between; padding:16px 20px; border-bottom:1px solid var(--border); }
.dialog-header h3 { font-size:16px; font-weight:600; color:var(--fg-title); }
.close-btn { display:flex; align-items:center; justify-content:center; width:32px; height:32px; background:none; border:none; border-radius:6px; cursor:pointer; color:var(--fg-muted); transition:all 200ms ease; }
.close-btn:hover { background:rgba(255,255,255,0.40); color:var(--fg); }
.dialog-tool-preview { display:flex; align-items:center; gap:12px; padding:14px 20px; background:rgba(255,255,255,0.18); border-bottom:1px solid var(--border); }
.dialog-tool-icon { width:42px; height:42px; display:flex; align-items:center; justify-content:center; background:rgba(45,45,45,0.06); border:1px solid var(--border); border-radius:10px; flex-shrink:0; }
.dialog-tool-info { display:flex; flex-direction:column; gap:2px; min-width:0; }
.dialog-tool-name { font-size:14px; font-weight:600; color:var(--fg-title); }
.dialog-tool-desc { font-size:12px; color:var(--fg-ghost); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.dialog-body { padding:16px 20px; overflow-y:auto; flex:1; }
.dialog-section-label { font-size:12px; font-weight:500; color:var(--fg-muted); text-transform:uppercase; letter-spacing:0.04em; margin-bottom:12px; }
.install-methods { display:flex; flex-direction:column; gap:8px; }
.method-card { display:flex; flex-direction:column; gap:8px; padding:12px 14px; background:rgba(255,255,255,0.40); border:1px solid rgba(255,255,255,0.35); border-radius:var(--radius); cursor:pointer; transition:all var(--t-fast); text-align:left; width:100%; }
.method-card:hover { background:rgba(255,255,255,0.58); border-color:rgba(255,255,255,0.50); transform:translateY(-1px); box-shadow:0 4px 16px rgba(0,0,0,0.06); }
.method-card.is-active { border-color:var(--accent); background:rgba(255,255,255,0.50); }
.method-card-header { display:flex; align-items:center; gap:8px; }
.method-label { font-size:13px; font-weight:600; color:var(--fg-title); font-family:var(--font-mono); }
.method-badge-active { font-size:10px; padding:2px 6px; background:rgba(90,138,100,0.15); color:var(--success); border-radius:4px; font-weight:600; }
.method-command { background:rgba(255,255,255,0.32); border:1px solid rgba(255,255,255,0.30); border-radius:var(--radius-sm); padding:8px 10px; overflow-x:auto; }
.method-command code { font-family:var(--font-mono); font-size:11px; color:var(--fg-muted); white-space:nowrap; }
.command-empty { font-style:italic; opacity:0.6; }
.dialog-footer { display:flex; justify-content:flex-end; gap:8px; padding:12px 20px; border-top:1px solid var(--border); }

/* Manual Uninstall Modal */
.manual-uninstall-body { display:flex; flex-direction:column; gap:14px; }
.manual-uninstall-desc { font-size:13px; color:var(--fg-muted); line-height:1.5; }
.manual-uninstall-commands { display:flex; flex-direction:column; gap:8px; }
.manual-cmd-row { display:flex; align-items:center; gap:8px; background:rgba(0,0,0,0.06); border:1px solid rgba(0,0,0,0.08); border-radius:var(--radius-sm); padding:10px 12px; }
.manual-cmd-text { flex:1; font-family:ui-monospace, SFMono-Regular, Menlo, monospace; font-size:12px; color:var(--fg-title); word-break:break-all; user-select:all; }
.manual-cmd-copy { width:28px; height:28px; display:flex; align-items:center; justify-content:center; border:none; background:rgba(255,255,255,0.40); border-radius:6px; color:var(--fg-muted); cursor:pointer; transition:all 150ms ease; flex-shrink:0; }
.manual-cmd-copy:hover { background:rgba(255,255,255,0.60); color:var(--accent); }

/* Tooltip methods */
.methods-tooltip { position:fixed; z-index:var(--z-dropdown); width:340px; background:rgba(255,255,255,0.48); backdrop-filter:blur(40px) saturate(1.4); -webkit-backdrop-filter:blur(40px) saturate(1.4); border:1px solid rgba(255,255,255,0.35); border-radius:var(--radius-xl); box-shadow:0 20px 60px rgba(0,0,0,0.12),inset 0 1px 0 rgba(255,255,255,0.50); overflow:hidden; }
.tooltip-header { display:flex; justify-content:space-between; align-items:center; padding:12px 14px; border-bottom:1px solid rgba(255,255,255,0.32); }
.tooltip-title { font-size:12px; font-weight:600; color:var(--fg-title); }
.tooltip-close { width:24px; height:24px; display:flex; align-items:center; justify-content:center; border:none; background:transparent; color:var(--fg-ghost); cursor:pointer; border-radius:6px; transition:all 150ms ease; }
.tooltip-close:hover { background:rgba(255,255,255,0.40); color:var(--fg); }
.method-list { padding:8px; display:flex; flex-direction:column; gap:6px; max-height:280px; overflow-y:auto; }
.method-entry { padding:10px 12px; background:rgba(255,255,255,0.42); border:1px solid rgba(255,255,255,0.35); border-radius:var(--radius); transition:all var(--t-fast); }
.method-entry:hover { background:rgba(255,255,255,0.58); border-color:rgba(255,255,255,0.50); }
.method-header { display:flex; align-items:center; gap:8px; margin-bottom:6px; }
.method-name { font-size:12px; font-weight:600; color:var(--fg-title); font-family:var(--font-mono); }
.method-active-inline { font-size:10px; padding:2px 6px; background:rgba(90,138,100,0.15); color:var(--success); border-radius:4px; font-weight:600; }
.method-command { background:rgba(255,255,255,0.32); border:1px solid rgba(255,255,255,0.30); border-radius:var(--radius-sm); padding:8px 10px; overflow-x:auto; }
.method-command code { font-family:var(--font-mono); font-size:11px; color:var(--fg-muted); white-space:nowrap; }

/* Tooltip Transition */
.tooltip-enter-active, .tooltip-leave-active { transition:opacity 200ms ease; }
.tooltip-enter-from, .tooltip-leave-to { opacity:0; }
</style>

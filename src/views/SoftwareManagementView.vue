<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSoftwareStore } from '@/stores/software'
import FilterBar from '@/components/common/FilterBar.vue'
import SearchInput from '@/components/common/SearchInput.vue'
import Badge from '@/components/common/Badge.vue'
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import TabBar from '@/components/common/TabBar.vue'
import ProgressSlot from '@/components/common/ProgressSlot.vue'
import VersionManagerModal from '@/components/VersionManagerModal.vue'
import { extractError } from '@/utils/error'
import { useOperationProgress } from '@/composables/useOperationProgress'
import { confirm } from '@/utils/dialog'
import { open as openExternal } from '@tauri-apps/plugin-shell'

const { startOperation, updateProgress, completeOperation, getOperation } = useOperationProgress()

// Manual uninstall modal state
const showManualModal = ref(false)
const manualToolName = ref('')
const manualCommands = ref<string[]>([])
const copiedIndex = ref<number | null>(null)
const copiedAll = ref(false)

// Version manager modal state
const showVersionManager = ref(false)
const versionManagerKey = ref('')
const versionManagerName = ref('')

function closeManualModal() {
  showManualModal.value = false
  manualCommands.value = []
  copiedIndex.value = null
  copiedAll.value = false
}

async function copyCommand(cmd: string, index: number) {
  try {
    await navigator.clipboard.writeText(cmd)
    copiedIndex.value = index
    setTimeout(() => { copiedIndex.value = null }, 2000)
  } catch {
    console.error('Failed to copy command')
  }
}

async function copyAllCommands() {
  try {
    await navigator.clipboard.writeText(manualCommands.value.join('\n'))
    copiedAll.value = true
    setTimeout(() => { copiedAll.value = false }, 2000)
  } catch {
    console.error('Failed to copy commands')
  }
}

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

// Software interface
interface Software {
  key: string
  name: string
  version: string
  configPath: string
  installed: boolean
  tier: string
  platform: string
  desc?: string
  web?: string
  color?: string
  icon?: string
  lastChecked?: string
}

const softwareStore = useSoftwareStore()

const searchQuery = ref('')
const selectedTier = ref('environment')  // 默认选中环境分组
const selectedPlatform = ref('all')
const selectedStatus = ref('all')

// Homebrew installation status
const isHomebrewInstalled = computed(() => {
  const homebrew = softwareList.value.find(sw => sw.key === 'homebrew')
  return homebrew?.installed ?? false
})

// Check if software requires homebrew (nvm, pyenv)
function requiresHomebrew(key: string): boolean {
  return ['nvm', 'pyenv'].includes(key)
}

// Tier options - 不显示全部，只显示相关分组
const tierOptions = [
  { value: 'environment', label: '环境工具' },
  { value: 'ai-tools', label: 'AI 工具' },
  { value: 'development', label: '开发工具' },
  { value: 'runtime', label: '运行时' },
  { value: 'debug', label: '调试工具' },
  { value: 'productivity', label: '效率工具' },
]

// Platform options
const platformOptions = [
  { value: 'all', label: 'All Platforms' },
  { value: 'macos', label: 'macOS' },
  { value: 'windows', label: 'Windows' },
  { value: 'linux', label: 'Linux' },
  { value: 'cross', label: 'Cross-platform' },
]

// Status options
const statusOptions = [
  { value: 'all', label: 'All Status' },
  { value: 'detected', label: 'Detected' },
  { value: 'not-found', label: 'Not Found' },
]

const tabItems = [
  { id: 'all', label: 'All' },
  { id: 'detected', label: 'Detected' },
  { id: 'not-found', label: 'Not Found' },
]

// Mock data matching prototype (PENDING: use useSoftwareStore().softwareList)
const mockSoftware: Software[] = [
  { key: 'cursor', name: 'Cursor', version: '0.47.9', configPath: '~/.cursor/', installed: true, tier: 't1', platform: 'cross', desc: 'AI 原生代码编辑器，集成多模型智能补全与跨文件重构能力。', web: 'https://cursor.com', lastChecked: '2024-01-01' },
  { key: 'claude-desktop', name: 'Claude Desktop', version: '0.9.2', configPath: '~/.claude/', installed: true, tier: 't1', platform: 'cross', desc: 'Anthropic 官方桌面应用，提供 Claude AI 对话与文件分析功能。', web: 'https://www.anthropic.com', lastChecked: '2024-01-01' },
  { key: 'windsurf', name: 'Windsurf', version: '1.6.4', configPath: '~/.windsurf/', installed: true, tier: 't2', platform: 'cross', desc: 'AI 原生编程 IDE，支持智能补全、多文件编辑与自主代理模式。', web: 'https://windsurf.com', lastChecked: '2024-01-01' },
  { key: 'continue', name: 'Continue', version: '0.9.21', configPath: '~/.continue/', installed: false, tier: 't3', platform: 'cross', desc: '开源 AI 编码助手，支持多模型接入与 IDE 内智能对话。', web: 'https://www.continue.dev', lastChecked: '2024-01-01' },
  { key: 'cody', name: 'Cody', version: '1.2.0', configPath: '~/.cody/', installed: false, tier: 't3', platform: 'cross', desc: 'Sourcegraph AI 助手，基于全代码库索引提供上下文感知补全。', web: 'https://sourcegraph.com/cody', lastChecked: '2024-01-01' },
  { key: 'copilot', name: 'Copilot', version: '1.3.0', configPath: '~/.copilot/', installed: true, tier: 't2', platform: 'cross', desc: 'GitHub AI 结对编程工具，提供实时代码补全与智能建议。', web: 'https://github.com/features/copilot', lastChecked: '2024-01-01' },
]

// Use store data if available, otherwise mock
const softwareList = computed(() => {
  console.log('Computing softwareList, store length:', softwareStore.softwareList.length)
  if (softwareStore.softwareList.length > 0) {
    console.log('Using store data')
    // Map store data to Software interface
    return softwareStore.softwareList.map(s => {
      console.log('Software:', s.key, 'isInstalled:', s.isInstalled, 'version:', s.version)
      return {
        key: s.key,
        name: s.name,
        version: s.version || '',
        configPath: s.configPath || '',
        installed: s.isInstalled,
        tier: getTierByKey(s.key),
        platform: s.platform || 'cross',
        desc: getDescByKey(s.key),
        web: s.websiteUrl || '',
        color: getColorByKey(s.key),
        icon: getIconByKey(s.key),
        lastChecked: s.lastChecked || '',
      }
    })
  }
  console.log('Using mock data')
  return mockSoftware
})

// Get tier by software key - 新的分组逻辑
function getTierByKey(key: string): string {
  const tierMap: Record<string, string> = {
    // 环境工具
    'homebrew': 'environment', 'nvm': 'environment', 'pyenv': 'environment',
    'node': 'environment', 'python': 'environment',
    // AI 工具
    'cursor': 'ai-tools', 'windsurf': 'ai-tools', 'claude-desktop': 'ai-tools',
    'continue': 'ai-tools', 'cody': 'ai-tools', 'copilot': 'ai-tools',
    // 开发工具
    'vscode': 'development', 'oh-my-posh': 'development', 'chocolatey': 'development',
    'scoop': 'development', 'ssh': 'development', 'windows-terminal': 'development',
    'iterm2': 'development',
    // 运行时
    'docker': 'runtime', 'docker-compose': 'runtime', 'ffmpeg': 'runtime',
    'goenv': 'runtime', 'jenv': 'runtime', 'asdf': 'runtime',
    // 调试工具
    'apifox': 'debug', 'postman': 'debug', 'charles': 'debug',
    'cyberduck': 'debug', 'filezilla': 'debug',
    // 效率工具
    'snipaste': 'productivity', 'obsidian': 'productivity', 'excalidraw': 'productivity',
  }
  return tierMap[key] || 'environment'
}

// Get description by software key
function getDescByKey(key: string): string {
  const descMap: Record<string, string> = {
    'homebrew': 'macOS 包管理器，用于安装和管理软件包。',
    'nvm': 'Node.js 版本管理器，支持安装和切换多个 Node.js 版本。',
    'pyenv': 'Python 版本管理器，支持安装和切换多个 Python 版本。',
    'vscode': '微软开发的轻量级代码编辑器。',
    'docker': '容器化平台，用于构建和运行应用程序。',
    'node': 'JavaScript 运行时环境。',
    'python': '通用编程语言。',
  }
  return descMap[key] || `${key} 可通过 Forge 进行一键安装与版本管理，支持跨平台同步和状态检测。`
}

// Get color by software key
function getColorByKey(key: string): string {
  const colorMap: Record<string, string> = {
    'homebrew': '#FBB040',
    'nvm': '#68A063',
    'pyenv': '#3776AB',
    'vscode': '#007ACC',
    'docker': '#2496ED',
    'cursor': '#5C5C5C',
    'windsurf': '#5C5C5C',
    'claude-desktop': '#D97757',
  }
  return colorMap[key] || '#5C5C5C'
}

// Get icon by software key
function getIconByKey(key: string): string {
  const iconMap: Record<string, string> = {
    'homebrew': '🍺',
    'nvm': '📦',
    'pyenv': '🐍',
    'vscode': '💻',
    'docker': '🐳',
  }
  return iconMap[key] || key.charAt(0).toUpperCase()
}

// Platform filter map for mock data
const platformFilterMap: Record<string, string[]> = {
  macos: ['Homebrew', 'Cursor', 'Claude Desktop'],
  windows: ['Copilot', 'Chocolatey', 'Scoop'],
  linux: [],
  cross: ['NVM', 'pyenv', 'Cursor', 'Claude Desktop', 'Windsurf', 'Continue', 'Cody', 'Copilot'],
}

// Filtered software list
const filteredSoftware = computed(() => {
  let result = softwareList.value

  // Search filter
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(sw =>
      sw.name.toLowerCase().includes(query) ||
      sw.configPath.toLowerCase().includes(query)
    )
  }

  // Tier filter - 默认选中环境工具
  if (selectedTier.value) {
    result = result.filter(sw => sw.tier === selectedTier.value)
  }

  // Platform filter
  if (selectedPlatform.value !== 'all') {
    const allowed = platformFilterMap[selectedPlatform.value]
    if (allowed) {
      result = result.filter(sw => allowed.includes(sw.name))
    }
  }

  // Status filter
  if (selectedStatus.value === 'detected') {
    result = result.filter(sw => sw.installed)
  } else if (selectedStatus.value === 'not-found') {
    result = result.filter(sw => !sw.installed)
  }

  return result
})

// Software count
const softwareCount = computed(() => filteredSoftware.value.length)

onMounted(async () => {
  console.log('SoftwareManagementView mounted, detecting software...')
  try {
    await softwareStore.detectSoftware()
    console.log('Software detected:', softwareStore.softwareList.length, 'items')
    console.log('Software list:', softwareStore.softwareList)
  } catch (e) {
    console.error('Failed to detect software:', e)
  }
})

function onSearchUpdate(val: string) {
  searchQuery.value = val
}

function onTierChange(val: string) {
  selectedTier.value = val
}

function onPlatformChange(val: string) {
  selectedPlatform.value = val
}

function onStatusChange(val: string) {
  selectedStatus.value = val
}

function tierColor(tier: string): string {
  const colors: Record<string, string> = {
    'environment': 'var(--accent)',
    'ai-tools': 'var(--info)',
    'development': 'var(--success)',
    'runtime': 'var(--warn)',
    'debug': 'var(--fg-ghost)',
    'productivity': 'var(--fg-ghost)',
  }
  return colors[tier] || 'var(--fg-ghost)'
}

function tierLabel(tier: string): string {
  const labels: Record<string, string> = {
    'environment': '环境',
    'ai-tools': 'AI',
    'development': '开发',
    'runtime': '运行时',
    'debug': '调试',
    'productivity': '效率',
  }
  return labels[tier] || tier
}

// More options dropdown
const openDropdown = ref<string | null>(null)

function toggleDropdown(key: string) {
  openDropdown.value = openDropdown.value === key ? null : key
}

function handleViewLogs(sw: Software) {
  openDropdown.value = null
  console.log('View logs:', sw.name)
}

function handleCheckUpdate(sw: Software) {
  openDropdown.value = null
  console.log('Check update:', sw.name)
}

function handleEnvironmentManage(sw: Software) {
  openDropdown.value = null
  versionManagerKey.value = sw.key
  versionManagerName.value = sw.name
  showVersionManager.value = true
}

function closeVersionManager() {
  showVersionManager.value = false
  versionManagerKey.value = ''
  versionManagerName.value = ''
}

async function handleInstall(sw: Software) {
  openDropdown.value = null
  const key = `sw-${sw.key}`
  startOperation(key)
  updateProgress(key, 'preparing', 10, `Installing ${sw.name}...`)

  try {
    const result = await softwareStore.installSoftware(sw.key)
    updateProgress(key, 'verifying', 90, 'Verifying...')
    if (result.success) {
      completeOperation(key, true, result.message || `${sw.name} installed successfully`)
      console.log(result.message || `${sw.name} installed successfully`)
    } else {
      completeOperation(key, false, result.message || `Failed to install ${sw.name}`)
      console.error(result.message || `Failed to install ${sw.name}`)
    }
  } catch (e) {
    completeOperation(key, false, extractError(e))
    console.error('Installation failed:', extractError(e))
  }
}

async function handleUninstall(sw: Software) {
  openDropdown.value = null
  if (await confirm(`确认卸载 ${sw.name}？`)) {
    const key = `sw-${sw.key}`
    startOperation(key)
    updateProgress(key, 'preparing', 10, `Removing ${sw.name}...`)
    softwareStore.uninstallSoftware(sw.key).then((res) => {
      updateProgress(key, 'verifying', 90, 'Verifying...')
      if (res.needsManual && res.manualCommands.length > 0) {
        completeOperation(key, true, res.message || `${sw.name} 部分卸载完成`)
        manualToolName.value = sw.name
        manualCommands.value = res.manualCommands
        showManualModal.value = true
      } else {
        completeOperation(key, true, res.message || `${sw.name} 已卸载`)
      }
      console.log(res.message || `${sw.name} 已卸载`)
    }).catch((e) => {
      completeOperation(key, false, extractError(e))
      console.error('卸载失败:', extractError(e))
    })
  }
}
</script>

<template>
  <div class="view active">
    <!-- Section Header -->
    <div class="section-header">
      <h2>Software Detection</h2>
      <span class="count">{{ softwareCount }} apps</span>
    </div>

    <!-- Tab Bar -->
    <TabBar v-model="selectedStatus" :tabs="tabItems" />

    <!-- Filter Bar -->
    <FilterBar
      :search-model-value="searchQuery"
      search-placeholder="Search software…"
      @update:search-model-value="onSearchUpdate"
    >
      <select
        class="filter-select"
        :value="selectedTier"
        @change="onTierChange(($event.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in tierOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
      <select
        class="filter-select"
        :value="selectedPlatform"
        @change="onPlatformChange(($event.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in platformOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
      <select
        class="filter-select"
        :value="selectedStatus"
        @change="onStatusChange(($event.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in statusOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </FilterBar>

    <!-- Software Grid -->
    <div class="card-grid" v-if="filteredSoftware.length > 0">
      <div
        v-for="sw in filteredSoftware"
        :key="sw.key"
        class="card sw-card"
      >
        <div class="card-head">
          <div
            class="card-icon"
            :style="{
              background: (sw.color || '#5C5C5C') + '12',
              color: sw.color || '#5C5C5C',
              borderColor: (sw.color || '#5C5C5C') + '25',
              fontSize: '14px',
              fontWeight: '700',
              fontFamily: 'var(--font-mono)'
            }"
          >{{ sw.icon || sw.name.charAt(0) }}</div>
          <div style="flex: 1; min-width: 0">
            <div class="card-title">
              {{ sw.name }}
              <span class="badge" :class="sw.installed ? 'success' : 'outline'">
                {{ sw.installed ? 'Detected' : 'Not found' }}
              </span>
            </div>
            <div class="card-subtitle">v{{ sw.version }} · {{ sw.platform }} · <span :style="{ color: tierColor(sw.tier), textTransform: 'uppercase', fontWeight: '600', fontSize: '10px', letterSpacing: '0.04em' }">{{ tierLabel(sw.tier) }}</span></div>
          </div>
        </div>
        <div class="card-desc">{{ sw.desc }}</div>
        <div class="card-meta">
          <div v-if="sw.key !== 'homebrew'" class="card-meta-item"><span class="label">Config</span><span class="value" style="font-family: var(--font-mono); font-size: 11px">{{ sw.configPath }}</span></div>
          <div v-if="sw.web" class="card-meta-item">
            <span class="label">Web</span>
            <span class="value web-value">{{ sw.web }}</span>
            <button class="web-link" title="访问官网" @click.stop="openExternalUrl(sw.web)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                <polyline points="15 3 21 3 21 9"/>
                <line x1="10" y1="14" x2="21" y2="3"/>
              </svg>
            </button>
          </div>
          <div class="card-meta-item"><span class="label">Checked</span><span class="value">{{ sw.lastChecked }}</span></div>
        </div>
        <div class="card-footer">
          <div class="card-footer-left">
            <ProgressSlot
              :stage="getOperation(`sw-${sw.key}`)?.stage || 'idle'"
              :progress="getOperation(`sw-${sw.key}`)?.progress || 0"
            />
          </div>
          <div class="card-footer-right">
            <!-- Homebrew dependency warning for nvm/pyenv -->
            <div v-if="requiresHomebrew(sw.key) && !isHomebrewInstalled && !sw.installed" class="homebrew-warning">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <span>需要先安装 Homebrew</span>
            </div>
            <!-- Environment Management button for nvm/pyenv -->
            <button
              v-if="['nvm', 'pyenv'].includes(sw.key) && sw.installed"
              class="btn btn-secondary btn-sm"
              @click="handleEnvironmentManage(sw)"
            >环境管理</button>
            <button
              v-if="sw.installed"
              class="btn btn-secondary btn-sm"
              @click="$emit('open-config', sw)"
            >Open Config</button>
            <button
              v-else
              class="btn btn-primary btn-sm"
              :disabled="requiresHomebrew(sw.key) && !isHomebrewInstalled"
              @click="handleInstall(sw)"
            >Install</button>
            <DropdownMenu :model-value="openDropdown === sw.key" @update:model-value="(v: boolean) => openDropdown = v ? sw.key : null" :min-width="160">
              <template #trigger>
                <button class="btn-icon btn-sm" @click.stop="toggleDropdown(sw.key)" title="More actions" aria-label="More actions">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
                  </svg>
                </button>
              </template>
              <button class="dropdown-item" @click.stop="handleViewLogs(sw)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                View Logs
              </button>
              <button class="dropdown-item" @click.stop="handleCheckUpdate(sw)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
                Check Update
              </button>
              <div class="dropdown-divider"></div>
              <button class="dropdown-item danger" @click.stop="handleUninstall(sw)">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                Uninstall
              </button>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </div>
    <!-- Empty State -->
    <div v-else class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
        <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
      </svg>
      <h3>No software found</h3>
      <p>Try adjusting your search or filter criteria.</p>
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
        <div class="dialog-body">
          <p style="font-size:13px;color:var(--fg-muted);line-height:1.5;margin-bottom:14px">{{ manualToolName }} 的部分文件需要管理员权限才能删除，请在终端中执行以下命令：</p>
          <div style="display:flex;flex-direction:column;gap:8px">
            <div v-for="(cmd, i) in manualCommands" :key="i" style="display:flex;align-items:center;gap:8px;background:rgba(0,0,0,0.06);border:1px solid rgba(0,0,0,0.08);border-radius:6px;padding:10px 12px">
              <code style="flex:1;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;color:var(--fg-title);word-break:break-all;user-select:all">{{ cmd }}</code>
              <button style="width:28px;height:28px;display:flex;align-items:center;justify-content:center;border:none;background:rgba(255,255,255,0.40);border-radius:6px;color:var(--fg-muted);cursor:pointer;flex-shrink:0" @click="copyCommand(cmd, i)">
                <svg v-if="copiedIndex !== i" width="14" height="14" viewBox="0 0 16 16" fill="none"><rect x="5" y="5" width="8" height="8" rx="1.5" stroke="currentColor" stroke-width="1.3"/><path d="M3 11V3.5A1.5 1.5 0 0 1 4.5 2H11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
                <svg v-else width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M4 8l3 3 5-5" stroke="#22c55e" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
              </button>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn btn-primary btn-sm" @click="copyAllCommands">
            <svg v-if="!copiedAll" width="14" height="14" viewBox="0 0 16 16" fill="none"><rect x="5" y="5" width="8" height="8" rx="1.5" stroke="currentColor" stroke-width="1.3"/><path d="M3 11V3.5A1.5 1.5 0 0 1 4.5 2H11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
            <svg v-else width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M4 8l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            {{ copiedAll ? '已复制' : '复制全部命令' }}
          </button>
          <button class="btn btn-secondary btn-sm" @click="closeManualModal">关闭</button>
        </div>
      </div>
    </div>

    <!-- Version Manager Modal -->
    <VersionManagerModal
      :show="showVersionManager"
      :software-key="versionManagerKey"
      :software-name="versionManagerName"
      @close="closeVersionManager"
    />
  </div>
</template>

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
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--fg-title);
  letter-spacing: -0.01em;
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

/* Filter Bar overrides */
:deep(.filter-bar) {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px 12px;
}

:deep(.filter-select) {
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius-sm);
  color: var(--fg);
  cursor: pointer;
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%239A9A9A' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
}

:deep(.filter-select:hover) {
  border-color: rgba(255, 255, 255, 0.40);
}

:deep(.filter-select:focus) {
  border-color: rgba(255, 255, 255, 0.40);
  background-color: rgba(255, 255, 255, 0.40);
}

/* Card Grid */
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

.card-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  min-height: 2.1em;
}

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

.card-meta-item .value {
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
  max-width: 260px;
}

.card-meta-item .value.web-value {
  direction: ltr;
  text-align: left;
  max-width: 260px;
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

.homebrew-warning {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--warn);
  background: rgba(255, 193, 7, 0.1);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 193, 7, 0.2);
}

.homebrew-warning svg {
  flex-shrink: 0;
}

/* Software Actions */
.software-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.software-actions .btn {
  min-width: 70px;
}

.text-danger {
  color: var(--error);
}

.text-danger:hover {
  background: var(--error-bg);
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px 24px;
  text-align: center;
}

.empty-state svg {
  opacity: 0.4;
}

.empty-state h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-muted);
}

.empty-state p {
  margin: 0;
  font-size: 13px;
  color: var(--fg-ghost);
}

/* Dialog */
.dialog-overlay { position:fixed; inset:0; background:rgba(0,0,0,0.5); display:flex; align-items:center; justify-content:center; z-index:var(--z-modal); }
.dialog { width:100%; max-width:520px; max-height:90vh; background:rgba(255,255,255,0.48); backdrop-filter:blur(40px) saturate(1.4); -webkit-backdrop-filter:blur(40px) saturate(1.4); border:1px solid rgba(255,255,255,0.35); border-radius:var(--radius-xl); box-shadow:0 20px 60px rgba(0,0,0,0.12),inset 0 1px 0 rgba(255,255,255,0.50); display:flex; flex-direction:column; overflow:hidden; }
.dialog-header { display:flex; align-items:center; justify-content:space-between; padding:16px 20px; border-bottom:1px solid var(--border); }
.dialog-header h3 { font-size:16px; font-weight:600; color:var(--fg-title); }
.close-btn { display:flex; align-items:center; justify-content:center; width:32px; height:32px; background:none; border:none; border-radius:6px; cursor:pointer; color:var(--fg-muted); transition:all 200ms ease; }
.close-btn:hover { background:rgba(255,255,255,0.40); color:var(--fg); }
.dialog-body { padding:16px 20px; overflow-y:auto; flex:1; }
.dialog-footer { display:flex; justify-content:flex-end; gap:8px; padding:12px 20px; border-top:1px solid var(--border); }
</style>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { extractError } from '@/utils/error'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  show: boolean
  softwareKey: string
  softwareName: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

// Version management state
const versions = ref<{ version: string; isInstalled: boolean; isCurrent: boolean; isGlobal: boolean }[]>([])
const currentVersion = ref<string | null>(null)
const globalVersion = ref<string | null>(null)
const isLoadingVersions = ref(false)
const isInstallingVersion = ref(false)
const isRemovingVersion = ref(false)
const newVersionInput = ref('')
const error = ref<string | null>(null)
const operationMessage = ref<string | null>(null)

// Available versions state
const availableVersions = ref<{ version: string; lts: string | null }[]>([])
const isLoadingAvailable = ref(false)
const showVersionPicker = ref(false)

// Computed - sorted installed versions
const sortedVersions = computed(() => {
  return [...versions.value].sort((a, b) => {
    const aParts = a.version.replace(/^[vV]/, '').split('.').map(Number)
    const bParts = b.version.replace(/^[vV]/, '').split('.').map(Number)
    for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
      const aNum = aParts[i] || 0
      const bNum = bParts[i] || 0
      if (aNum !== bNum) return bNum - aNum
    }
    return 0
  })
})

// Computed - grouped available versions (each major version shows only the latest minor)
const groupedVersions = computed(() => {
  if (availableVersions.value.length === 0) return []

  const majorMap = new Map<string, { version: string; lts: string | null; patch: number[] }>()

  for (const v of availableVersions.value) {
    const versionStr = v.version.replace(/^[vV]/, '')
    const parts = versionStr.split('.')
    if (parts.length < 2) continue

    const major = parts[0]

    const existing = majorMap.get(major)
    if (!existing) {
      majorMap.set(major, {
        version: v.version,
        lts: v.lts,
        patch: parts.slice(1).map(Number)
      })
    } else {
      // Compare versions to keep the latest
      const existingParts = existing.patch
      const newParts = parts.slice(1).map(Number)
      for (let i = 0; i < Math.max(existingParts.length, newParts.length); i++) {
        const eNum = existingParts[i] || 0
        const nNum = newParts[i] || 0
        if (nNum > eNum) {
          majorMap.set(major, {
            version: v.version,
            lts: v.lts,
            patch: newParts
          })
          break
        } else if (nNum < eNum) {
          break
        }
      }
    }
  }

  // Sort by major version descending
  return Array.from(majorMap.values())
    .sort((a, b) => {
      const aMajor = parseInt(a.version.replace(/^[vV]/, '').split('.')[0], 10)
      const bMajor = parseInt(b.version.replace(/^[vV]/, '').split('.')[0], 10)
      return bMajor - aMajor
    })
})

// Filtered versions based on input
const filteredVersions = computed(() => {
  if (!newVersionInput.value.trim()) {
    return groupedVersions.value
  }
  const query = newVersionInput.value.trim().toLowerCase()
  return groupedVersions.value.filter(v =>
    v.version.toLowerCase().includes(query)
  )
})

// Load versions
async function loadVersions() {
  if (!props.softwareKey) return

  isLoadingVersions.value = true
  error.value = null
  operationMessage.value = null

  try {
    const result = await invoke<{
      success: boolean
      versions: { version: string; isInstalled: boolean; isCurrent: boolean; isGlobal: boolean }[]
      currentVersion: string | null
      globalVersion: string | null
      message: string
    }>('get_version_list', { softwareKey: props.softwareKey })

    if (result.success) {
      versions.value = result.versions
      currentVersion.value = result.currentVersion
      globalVersion.value = result.globalVersion
    } else {
      error.value = result.message
    }
  } catch (e) {
    error.value = extractError(e)
    console.error('Failed to load versions:', e)
  } finally {
    isLoadingVersions.value = false
  }
}

// Load available versions
async function loadAvailableVersions() {
  if (!props.softwareKey || props.softwareKey === 'homebrew') return

  isLoadingAvailable.value = true

  try {
    const result = await invoke<{ version: string; lts: string | null }[]>('get_available_versions', {
      softwareKey: props.softwareKey
    })
    availableVersions.value = result
  } catch (e) {
    console.error('Failed to load available versions:', e)
    availableVersions.value = []
  } finally {
    isLoadingAvailable.value = false
  }
}

// Toggle version picker
function toggleVersionPicker() {
  showVersionPicker.value = !showVersionPicker.value
  if (showVersionPicker.value && availableVersions.value.length === 0 && !isLoadingAvailable.value) {
    loadAvailableVersions()
  }
}

// Select version from picker
function selectVersion(version: string) {
  newVersionInput.value = version
  showVersionPicker.value = false
}

// Install new version
async function installVersion() {
  if (!newVersionInput.value.trim()) return

  isInstallingVersion.value = true
  error.value = null
  operationMessage.value = `正在安装 ${newVersionInput.value}...`
  showVersionPicker.value = false

  try {
    const result = await invoke<{
      success: boolean
      message: string
      newVersion: string | null
    }>('install_version', {
      softwareKey: props.softwareKey,
      version: newVersionInput.value.trim()
    })

    if (result.success) {
      operationMessage.value = result.message
      newVersionInput.value = ''
      await loadVersions()
      setTimeout(() => { operationMessage.value = null }, 3000)
    } else {
      error.value = result.message
      operationMessage.value = null
    }
  } catch (e) {
    error.value = extractError(e)
    console.error('Failed to install version:', e)
    operationMessage.value = null
  } finally {
    isInstallingVersion.value = false
  }
}

// Set global version
async function setGlobalVersion(version: string) {
  if (!version || version === globalVersion.value) return

  operationMessage.value = `正在设置全局版本为 ${version}...`

  try {
    const result = await invoke<{
      success: boolean
      message: string
      newVersion: string | null
    }>('set_global_version', {
      softwareKey: props.softwareKey,
      version
    })

    if (result.success) {
      globalVersion.value = version
      versions.value = versions.value.map(v => ({
        ...v,
        isGlobal: v.version === version
      }))
      operationMessage.value = result.message
      setTimeout(() => { operationMessage.value = null }, 3000)
    } else {
      error.value = result.message
      operationMessage.value = null
    }
  } catch (e) {
    error.value = extractError(e)
    console.error('Failed to set global version:', e)
    operationMessage.value = null
  }
}

// Remove version
async function removeVersion(version: string) {
  if (!version || version === currentVersion.value) {
    error.value = 'Cannot remove current version'
    return
  }

  isRemovingVersion.value = true
  error.value = null
  operationMessage.value = `正在删除 ${version}...`

  try {
    const result = await invoke<{
      success: boolean
      message: string
      newVersion: string | null
    }>('remove_version', {
      softwareKey: props.softwareKey,
      version
    })

    if (result.success) {
      operationMessage.value = result.message
      await loadVersions()
      setTimeout(() => { operationMessage.value = null }, 3000)
    } else {
      error.value = result.message
      operationMessage.value = null
    }
  } catch (e) {
    error.value = extractError(e)
    console.error('Failed to remove version:', e)
    operationMessage.value = null
  } finally {
    isRemovingVersion.value = false
  }
}

// Clear error
function clearError() {
  error.value = null
}

// Watch for show prop changes
watch(() => props.show, (newShow) => {
  if (newShow) {
    loadVersions()
  } else {
    // Reset state when closing
    error.value = null
    operationMessage.value = null
    newVersionInput.value = ''
    showVersionPicker.value = false
  }
})

onMounted(() => {
  if (props.show) {
    loadVersions()
  }
})
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog version-manager-dialog">
      <!-- Header -->
      <div class="dialog-header">
        <div class="header-content">
          <h3>{{ softwareName }}</h3>
          <span class="header-subtitle">版本管理</span>
        </div>
        <button class="close-btn" aria-label="Close" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8m0-8l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="dialog-body">
        <!-- Loading State -->
        <div v-if="isLoadingVersions" class="state-container">
          <div class="loading-spinner"></div>
          <span class="state-text">正在加载版本列表...</span>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="state-container error">
          <svg class="error-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span class="state-text">{{ error }}</span>
          <button class="btn btn-ghost btn-sm" @click="clearError">关闭</button>
        </div>

        <!-- Version Content -->
        <div v-else class="version-content">
          <!-- Current Version Card -->
          <div class="current-version-card">
            <div v-if="globalVersion && softwareKey !== 'homebrew'" class="version-card-header">
              <span class="version-label">全局默认</span>
              <span class="version-value global">{{ globalVersion }}</span>
            </div>
            <div v-else class="version-card-header">
              <span class="version-label">状态</span>
              <span class="version-value">{{ currentVersion || '未安装' }}</span>
            </div>
          </div>

          <!-- Install Section -->
          <div class="install-section">
            <div class="install-header">
              <label class="section-label">安装新版本</label>
              <button
                v-if="softwareKey !== 'homebrew'"
                class="btn btn-ghost btn-xs toggle-picker-btn"
                @click="toggleVersionPicker"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path v-if="!showVersionPicker" d="M6 9l6 6 6-6"/>
                  <path v-else d="M18 15l-6-6-6 6"/>
                </svg>
                {{ showVersionPicker ? '收起' : '选择版本' }}
              </button>
            </div>

            <div class="install-input-group">
              <input
                v-model="newVersionInput"
                type="text"
                class="input"
                :placeholder="softwareKey === 'nvm' ? 'v20.19.0' : '3.12.10'"
                @keyup.enter="installVersion"
              />
              <button
                class="btn btn-primary"
                :disabled="!newVersionInput.trim() || isInstallingVersion"
                @click="installVersion"
              >
                <svg v-if="isInstallingVersion" class="spinner-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
                </svg>
                <span>{{ isInstallingVersion ? '安装中' : '安装' }}</span>
              </button>
            </div>

            <!-- Version Picker Panel -->
            <div v-if="showVersionPicker" class="version-picker">
              <div v-if="isLoadingAvailable" class="picker-loading">
                <div class="loading-spinner-sm"></div>
                <span>加载可用版本...</span>
              </div>
              <div v-else-if="filteredVersions.length === 0" class="picker-empty">
                <span>暂无可用版本</span>
              </div>
              <div v-else class="picker-content">
                <div class="picker-search">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="8"/>
                    <path d="M21 21l-4.35-4.35"/>
                  </svg>
                  <input
                    v-model="newVersionInput"
                    type="text"
                    class="picker-search-input"
                    placeholder="输入版本号筛选..."
                  />
                </div>
                <div class="picker-list">
                  <div
                    v-for="v in filteredVersions"
                    :key="v.version"
                    class="picker-item"
                    @click="selectVersion(v.version)"
                  >
                    <span class="picker-item-version">{{ v.version }}</span>
                    <span v-if="v.lts" class="picker-item-lts">{{ v.lts }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Operation Message -->
          <div v-if="operationMessage" class="operation-message">
            <svg class="spinner-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
            </svg>
            <span>{{ operationMessage }}</span>
          </div>

          <!-- Version List -->
          <div class="versions-section">
            <div class="section-header">
              <label class="section-label">已安装版本</label>
              <span class="version-count">{{ sortedVersions.length }} 个</span>
            </div>

            <div v-if="sortedVersions.length === 0" class="empty-state">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round">
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
                <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
                <line x1="12" y1="22.08" x2="12" y2="12"/>
              </svg>
              <span>暂无已安装版本</span>
            </div>

            <div v-else class="version-list">
              <div
                v-for="versionInfo in sortedVersions"
                :key="versionInfo.version"
                class="version-item"
                :class="{
                  'active': versionInfo.isCurrent,
                  'global': versionInfo.isGlobal
                }"
              >
                <div class="version-main">
                  <span class="version-number">{{ versionInfo.version }}</span>
                  <div class="version-badges">
                    <span v-if="versionInfo.isGlobal" class="badge global">全局</span>
                  </div>
                </div>
                <div class="version-actions">
                  <button
                    v-if="!versionInfo.isGlobal && softwareKey !== 'homebrew'"
                    class="btn btn-ghost btn-xs"
                    @click="setGlobalVersion(versionInfo.version)"
                  >
                    全局
                  </button>
                  <button
                    v-if="!versionInfo.isCurrent"
                    class="btn btn-ghost btn-xs danger"
                    :disabled="isRemovingVersion"
                    @click="removeVersion(versionInfo.version)"
                  >
                    删除
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Dialog Overlay */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

/* Dialog Container */
.dialog {
  width: 100%;
  max-width: 520px;
  max-height: 85vh;
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Dialog Header */
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.header-content h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-title);
}

.header-subtitle {
  font-size: 12px;
  color: var(--fg-ghost);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 150ms ease;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg);
}

/* Dialog Body */
.dialog-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

/* State Containers */
.state-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 16px;
  text-align: center;
}

.state-container.error {
  color: var(--error);
}

.state-text {
  font-size: 13px;
  color: var(--fg-muted);
}

.state-container.error .state-text {
  color: var(--error);
}

.error-icon {
  opacity: 0.8;
}

/* Loading Spinner */
.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.spinner-icon {
  animation: spin 1s linear infinite;
}

/* Version Content */
.version-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* Current Version Card */
.current-version-card {
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.version-card-header,
.version-card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
}

.version-card-footer {
  border-top: 1px solid var(--border);
}

.version-label {
  font-size: 12px;
  color: var(--fg-ghost);
}

.version-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--fg-title);
  font-family: var(--font-mono);
}

.version-value.global {
  color: var(--success);
}

/* Install Section */
.install-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.install-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.toggle-picker-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--accent);
}

.install-input-group {
  display: flex;
  gap: 8px;
}

.install-input-group .input {
  flex: 1;
  padding: 8px 12px;
  font-size: 13px;
  font-family: var(--font-mono);
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius-sm);
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast);
}

.install-input-group .input:focus {
  border-color: var(--accent);
}

.install-input-group .input::placeholder {
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

/* Version Picker Panel */
.version-picker {
  background: rgba(255, 255, 255, 0.24);
  border: 1px solid rgba(255, 255, 255, 0.20);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.picker-loading,
.picker-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  color: var(--fg-ghost);
  font-size: 12px;
}

.loading-spinner-sm {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.picker-content {
  display: flex;
  flex-direction: column;
}

.picker-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  color: var(--fg-ghost);
}

.picker-search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 12px;
  color: var(--fg);
  font-family: var(--font-mono);
}

.picker-search-input::placeholder {
  color: var(--fg-ghost);
  font-family: inherit;
}

.picker-list {
  max-height: 200px;
  overflow-y: auto;
  padding: 4px;
}

.picker-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--t-fast);
}

.picker-item:hover {
  background: rgba(var(--accent-rgb, 59, 130, 246), 0.1);
}

.picker-item-version {
  font-size: 13px;
  font-family: var(--font-mono);
  color: var(--fg);
}

.picker-item-lts {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  background: rgba(var(--success-rgb, 34, 197, 94), 0.15);
  color: var(--success);
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

/* Operation Message */
.operation-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(var(--accent-rgb, 59, 130, 246), 0.1);
  border: 1px solid rgba(var(--accent-rgb, 59, 130, 246), 0.2);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--accent);
}

/* Versions Section */
.versions-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.version-count {
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: var(--fg-ghost);
  font-size: 13px;
}

/* Version List */
.version-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.32);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius-sm);
  transition: all var(--t-fast);
}

.version-item:hover {
  background: rgba(255, 255, 255, 0.40);
}

.version-item.active {
  border-color: var(--accent);
  background: rgba(var(--accent-rgb, 59, 130, 246), 0.08);
}

.version-item.global {
  border-color: var(--success);
  background: rgba(var(--success-rgb, 34, 197, 94), 0.08);
}

.version-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.version-number {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-title);
  font-family: var(--font-mono);
}

.version-badges {
  display: flex;
  gap: 4px;
}

.badge {
  font-size: 9px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.badge.current {
  background: rgba(var(--accent-rgb, 59, 130, 246), 0.15);
  color: var(--accent);
}

.badge.global {
  background: rgba(var(--success-rgb, 34, 197, 94), 0.15);
  color: var(--success);
}

.version-actions {
  display: flex;
  gap: 4px;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--t-fast);
  white-space: nowrap;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--accent);
  color: white;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 90%, black);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.32);
  color: var(--fg);
  border: 1px solid rgba(255, 255, 255, 0.30);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.40);
}

.btn-ghost {
  background: none;
  color: var(--fg-muted);
  border: none;
  padding: 6px 10px;
}

.btn-ghost:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.30);
  color: var(--fg);
}

.btn-ghost.danger {
  color: var(--error);
}

.btn-ghost.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: var(--error);
}

.btn-xs {
  padding: 4px 8px;
  font-size: 11px;
}

/* Dialog Footer */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.16);
  flex-shrink: 0;
}
</style>

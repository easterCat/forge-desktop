<template>
  <div class="view settings-view">
    <!-- Page Header -->
    <div class="section-header">
      <h2>Settings</h2>
    </div>

    <div class="settings-grid">
      <!-- GitHub Token -->
      <div class="setting-group">
        <h4>GitHub Token</h4>
        <p class="group-desc">Required for private repository access</p>

        <div class="setting-row">
          <span class="label">Token status</span>
          <span v-if="settingsStore.isConfigured" class="badge success">Configured</span>
          <span v-else class="badge warn">Not set</span>
        </div>

        <div v-if="settingsStore.tokenPreview" class="setting-row">
          <span class="label">Preview</span>
          <span class="mono-value">{{ settingsStore.tokenPreview }}</span>
        </div>

        <div class="setting-actions">
          <button class="btn btn-primary btn-sm" @click="showTokenModal = true">Update Token</button>
          <button
            class="btn btn-ghost btn-sm btn-danger"
            :disabled="!settingsStore.isConfigured"
            @click="handleClearToken"
          >Clear</button>
        </div>
      </div>

      <!-- Application -->
      <div class="setting-group">
        <h4>Application</h4>

        <!-- Theme Grid -->
        <div class="theme-grid">
          <div
            v-for="theme in availableThemes"
            :key="theme.id"
            class="theme-card"
            :class="{ active: themeStore.activeThemeId === theme.id }"
            :title="theme.name"
            @click="selectTheme(theme.id)"
          >
            <div class="theme-preview">
              <span
                v-for="(color, i) in theme.previewColors"
                :key="i"
                :style="{ background: color }"
              />
            </div>
            <div class="theme-info">
              <span class="theme-name">{{ theme.name }}</span>
            </div>
          </div>
        </div>

        <div class="setting-row">
          <span class="label">Auto-scan on launch</span>
          <ToggleSwitch v-model="autoScan" />
        </div>

        <div class="setting-row">
          <span class="label">Daily backup</span>
          <ToggleSwitch v-model="dailyBackup" />
        </div>

        <div class="setting-row">
          <span class="label">Notification sounds</span>
          <ToggleSwitch v-model="notifSounds" />
        </div>
      </div>

      <!-- Performance -->
      <div class="setting-group">
        <h4>Performance</h4>

        <div class="setting-row">
          <span class="label">CLI parallel detection</span>
          <span class="mono-value">rayon · 60s timeout</span>
        </div>

        <div class="setting-row">
          <span class="label">SQLite cache</span>
          <span class="mono-value">bundled rusqlite</span>
        </div>

        <div class="setting-row">
          <span class="label">MCP Discovery Cache</span>
          <span class="mono-value">100 entries · 5m TTL</span>
        </div>
      </div>

      <!-- Data Paths -->
      <div class="setting-group">
        <h4>Data Paths</h4>

        <div class="setting-row">
          <span class="label">App data</span>
          <span class="mono-value subtle">~/.local/share/forge/</span>
        </div>

        <div class="setting-row">
          <span class="label">Database</span>
          <span class="mono-value subtle">forge.db (SQLite)</span>
        </div>

        <div class="setting-row">
          <span class="label">Logs</span>
          <span class="mono-value subtle">~/.local/share/forge/logs/</span>
        </div>
      </div>
    </div>

    <!-- Update Token Modal -->
    <Modal v-model="showTokenModal" title="Update GitHub Token" width="480px">
      <div class="token-modal-body">
        <label for="token-input">Personal Access Token</label>
        <input
          id="token-input"
          v-model="newToken"
          type="password"
          placeholder="ghp_xxxxxxxxxxxx"
          autocomplete="off"
          @keydown.enter="handleSaveToken"
        />
        <p class="token-hint">
          Generate a token at
          <button class="link-button" @click.stop="openExternalUrl('https://github.com/settings/tokens')">github.com/settings/tokens</button>
          with <code>repo</code> scope.
        </p>
        <p v-if="settingsStore.error" class="token-error">{{ settingsStore.error }}</p>
      </div>

      <template #footer>
        <button class="btn btn-ghost" @click="showTokenModal = false">Cancel</button>
        <button
          class="btn btn-primary"
          :disabled="!newToken.trim() || settingsStore.isLoading"
          @click="handleSaveToken"
        >
          {{ settingsStore.isLoading ? 'Saving...' : 'Save Token' }}
        </button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, inject } from 'vue'
import ToggleSwitch from '@/components/common/ToggleSwitch.vue'
import Modal from '@/components/common/Modal.vue'
import { useThemeStore } from '@/stores/theme'
import { useSettingsStore } from '@/stores/settings'
import type { ThemeId } from '@/stores/theme'
import { open as openExternal } from '@tauri-apps/plugin-shell'

const themeStore = useThemeStore()
const settingsStore = useSettingsStore()
const showNotification = inject<(msg: string, type?: string) => void>('showNotification')

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

// App toggles — persisted to localStorage
const autoScan = ref(loadToggle('forge-auto-scan', true))
const dailyBackup = ref(loadToggle('forge-daily-backup', true))
const notifSounds = ref(loadToggle('forge-notif-sounds', false))

// Token modal state
const showTokenModal = ref(false)
const newToken = ref('')

function loadToggle(key: string, fallback: boolean): boolean {
  const stored = localStorage.getItem(key)
  return stored !== null ? stored === 'true' : fallback
}

watch(autoScan, (v) => localStorage.setItem('forge-auto-scan', String(v)))
watch(dailyBackup, (v) => localStorage.setItem('forge-daily-backup', String(v)))
watch(notifSounds, (v) => localStorage.setItem('forge-notif-sounds', String(v)))

onMounted(() => {
  settingsStore.refresh()
})

// Available themes — 20 themes matching prototype
const availableThemes = [
  { id: 'warm' as ThemeId, name: 'Warm Glass', previewColors: ['#F5F3F0', '#F0EDE8', '#D2CAB8', '#2D2D2D', '#5A8A64', '#B8944A'] },
  { id: 'cool-mist' as ThemeId, name: 'Cool Mist', previewColors: ['#EEF1F5', '#DDE2EA', '#B8C4D4', '#1E2A3A', '#4A8A6A', '#6B7FAA'] },
  { id: 'midnight' as ThemeId, name: 'Midnight', previewColors: ['#1A1D24', '#242830', '#2E3340', '#E0E4EC', '#5A9A7A', '#7A8AAA'] },
  { id: 'sakura' as ThemeId, name: 'Sakura', previewColors: ['#F8F0F2', '#F0DDE2', '#E4B8C4', '#3A2028', '#8A5A6A', '#C47A8A'] },
  { id: 'sage' as ThemeId, name: 'Sage', previewColors: ['#EFF3EE', '#DAE4D8', '#B8CCB4', '#1E2A1E', '#5A8A5A', '#8AAA6A'] },
  { id: 'lavender' as ThemeId, name: 'Lavender', previewColors: ['#F2EFF8', '#E0DAF0', '#C4B8E0', '#2A2038', '#7A5AAA', '#A070C0'] },
  { id: 'ocean' as ThemeId, name: 'Ocean', previewColors: ['#ECF4F4', '#D4E8E8', '#A8D4D4', '#0A2828', '#2A8A8A', '#4A9AAA'] },
  { id: 'ember' as ThemeId, name: 'Ember', previewColors: ['#F8F2EC', '#F0E0CC', '#E0C8A0', '#382010', '#B87A3A', '#D49A4A'] },
  { id: 'slate' as ThemeId, name: 'Slate', previewColors: ['#F0F0EE', '#E0E0DC', '#C8C8C0', '#2A2A28', '#6A7A6A', '#8A8A7A'] },
  { id: 'aurora' as ThemeId, name: 'Aurora', previewColors: ['#F0F4F8', '#DAE4F0', '#A8C4E0', '#14202C', '#4A8ACC', '#8A6ACC'] },
  { id: 'cream' as ThemeId, name: 'Cream', previewColors: ['#FAF8F5', '#F2EDE6', '#E8DFD0', '#2C2820', '#7A8A5A', '#AA9A60'] },
  { id: 'arctic' as ThemeId, name: 'Arctic', previewColors: ['#F4F8FA', '#E4EEF4', '#C8DAE8', '#142430', '#3A7AAA', '#5A9ACC'] },
  { id: 'rose-gold' as ThemeId, name: 'Rose Gold', previewColors: ['#F8F2F4', '#F0DDE2', '#E0C0C8', '#301820', '#B85A70', '#D48A6A'] },
  { id: 'cyberpunk' as ThemeId, name: 'Cyberpunk', previewColors: ['#0E0E18', '#1A1A2E', '#2A2A40', '#E0E0F0', '#FF2E63', '#08D9D6'] },
  { id: 'forest' as ThemeId, name: 'Forest', previewColors: ['#F0F4EE', '#D8E4D0', '#B0C8A0', '#14200E', '#3A6A2A', '#5A8A4A'] },
  { id: 'desert' as ThemeId, name: 'Desert Sand', previewColors: ['#F5EEE4', '#E8D8C4', '#D0BC98', '#2C2014', '#A07848', '#C49A58'] },
  { id: 'cotton-candy' as ThemeId, name: 'Cotton Candy', previewColors: ['#F8F0F8', '#F0DAF0', '#E0C0E8', '#282030', '#C060A0', '#60A0C0'] },
  { id: 'charcoal' as ThemeId, name: 'Charcoal', previewColors: ['#181818', '#252525', '#333333', '#E8E8E8', '#6A8A6A', '#8A6A6A'] },
  { id: 'peach' as ThemeId, name: 'Peach Fuzz', previewColors: ['#FBF0E8', '#F5DCC8', '#E8C0A0', '#302018', '#D48A58', '#E8A070'] },
  { id: 'nordic' as ThemeId, name: 'Nordic', previewColors: ['#F4F6F8', '#E2E8EE', '#C8D4E0', '#1C2430', '#5A7A9A', '#8A6A5A'] },
]

function selectTheme(themeId: ThemeId) {
  themeStore.setTheme(themeId)
}

async function handleSaveToken() {
  try {
    await settingsStore.saveToken(newToken.value)
    newToken.value = ''
    showTokenModal.value = false
    showNotification?.('GitHub token updated', 'success')
  } catch {
    // error is already set in the store
  }
}

async function handleClearToken() {
  try {
    await settingsStore.clearToken()
    showNotification?.('GitHub token cleared', 'success')
  } catch {
    // error is already set in the store
  }
}

// Reset form when modal closes
watch(showTokenModal, (open) => {
  if (!open) {
    newToken.value = ''
    settingsStore.error = null
  }
})
</script>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* Section header */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title);
}

/* 2-column settings grid */
.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

@media (max-width: 768px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}

/* Setting group card */
.setting-group {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.setting-group h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  margin-bottom: 4px;
}

.group-desc {
  font-size: 13px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

/* Setting rows */
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-row .label {
  font-size: 13px;
  color: var(--fg-muted);
}

.mono-value {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-muted);
}

.mono-value.subtle {
  font-size: 11px;
  color: var(--fg-ghost);
}

/* Setting actions row */
.setting-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

/* Badge — uses global .badge.success / .badge.warn from theme.css */

/* Button styles */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  transition: all var(--t-fast);
  font-family: inherit;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover {
  filter: brightness(1.1);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-ghost {
  background: transparent;
  color: var(--fg-muted);
  border: 1px solid var(--border);
}

.btn-ghost:hover {
  background: var(--glass-input);
  color: var(--fg);
}

.btn-ghost:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-danger {
  color: var(--error) !important;
}

.btn-danger:hover {
  background: rgba(180, 60, 60, 0.08);
}

/* Theme grid */
.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 5px;
  padding: 8px 0;
  max-height: 240px;
  overflow-y: auto;
  margin-bottom: 8px;
}

.theme-card {
  position: relative;
  cursor: pointer;
  border-radius: var(--radius-sm);
  overflow: hidden;
  border: 1.5px solid rgba(255, 255, 255, 0.40);
  transition: all var(--t-base);
  background: rgba(255, 255, 255, 0.10);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
}

.theme-card:hover {
  border-color: rgba(255, 255, 255, 0.35);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.theme-card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1.5px var(--accent-glow);
}

.theme-card.active::after {
  content: '';
  position: absolute;
  top: 4px;
  right: 4px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='20 6 9 17 4 12'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: center;
  background-size: 10px;
}

.theme-preview {
  display: flex;
  height: 20px;
}

.theme-preview span {
  flex: 1;
}

.theme-info {
  padding: 5px 7px 6px;
}

.theme-name {
  font-size: 10px;
  font-weight: 600;
  color: var(--fg-title);
  line-height: 1.2;
}

/* Token modal */
.token-modal-body label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  margin-bottom: 8px;
}

.token-modal-body input {
  width: 100%;
  padding: 10px 14px;
  font-size: 14px;
  font-family: var(--font-mono);
  background: var(--glass-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.token-modal-body input:focus {
  border-color: var(--accent);
  background: var(--glass-input-focus);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

.token-hint {
  font-size: 12px;
  color: var(--fg-ghost);
  margin-top: 10px;
  line-height: 1.5;
}

.token-hint a {
  color: var(--accent);
  text-decoration: none;
}

.token-hint a:hover {
  text-decoration: underline;
}

.token-hint .link-button {
  color: var(--accent);
  text-decoration: none;
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  cursor: pointer;
}

.token-hint .link-button:hover {
  text-decoration: underline;
}

.token-hint code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: rgba(255, 255, 255, 0.15);
  padding: 1px 5px;
  border-radius: 3px;
}

.token-error {
  font-size: 12px;
  color: var(--error);
  margin-top: 8px;
}
</style>

<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog">
      <!-- Header -->
      <div class="dialog-header">
        <h3>Skill Details</h3>
        <button class="close-btn" aria-label="关闭" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="dialog-content">
        <!-- Skill Overview -->
        <div class="skill-header">
          <div class="skill-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
          </div>
          <div class="skill-title">
            <h2>{{ skill.name }}</h2>
            <div class="skill-meta">
              <span v-if="skill.author" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                  <circle cx="12" cy="7" r="4"/>
                </svg>
                {{ skill.author }}
              </span>
              <span v-if="skill.version" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
                </svg>
                v{{ skill.version }}
              </span>
              <span v-if="skill.license" class="meta-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                {{ skill.license }}
              </span>
            </div>
          </div>
          <div class="skill-badges">
            <span class="badge source-badge">{{ skill.sourceId }}</span>
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
          <p class="description">{{ skill.description }}</p>
          <p v-if="skill.longDescription" class="long-description">
            {{ skill.longDescription }}
          </p>
        </div>

        <!-- Categories and Tags -->
        <div class="section">
          <h4>Categories</h4>
          <div class="tags-list">
            <span
              v-for="cat in skill.categories"
              :key="cat"
              class="category-tag"
            >
              {{ cat }}
            </span>
          </div>
        </div>

        <div v-if="skill.tags.length > 0" class="section">
          <h4>Tags</h4>
          <div class="tags-list">
            <span
              v-for="tag in skill.tags"
              :key="tag"
              class="tag"
            >
              {{ tag }}
            </span>
          </div>
        </div>

        <!-- Stats -->
        <div class="section stats-section">
          <div v-if="skill.stars" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
            <div class="stat-value">{{ formatNumber(skill.stars) }}</div>
            <div class="stat-label">Stars</div>
          </div>
          <div v-if="skill.downloads" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            <div class="stat-value">{{ formatNumber(skill.downloads) }}</div>
            <div class="stat-label">Downloads</div>
          </div>
          <div v-if="skill.lastUpdated" class="stat-card">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
            <div class="stat-value">{{ formatDate(skill.lastUpdated) }}</div>
            <div class="stat-label">Last Updated</div>
          </div>
        </div>

        <!-- Links -->
        <div v-if="skill.repository || skill.homepage" class="section">
          <h4>Links</h4>
          <div class="links-list">
            <button
              v-if="skill.repository"
              class="link-item"
              @click.stop="openExternalUrl(skill.repository)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
              </svg>
              Repository
            </button>
            <button
              v-if="skill.homepage"
              class="link-item"
              @click.stop="openExternalUrl(skill.homepage)"
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
        <div v-if="skill.installCommand" class="section">
          <h4>Install Command</h4>
          <div class="command-box">
            <code>{{ skill.installCommand }}</code>
            <button class="copy-btn" @click="copyCommand">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">
          Close
        </button>
        <button
          v-if="!isInstalled"
          class="btn btn-primary"
          @click="handleInstall"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Install Skill
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
import type { MarketplaceSkill } from '@/types';
import { open as openExternal } from '@tauri-apps/plugin-shell';

interface Props {
  skill: MarketplaceSkill;
  isInstalled: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'install', skill: MarketplaceSkill): void;
}>();

function formatNumber(num: number): string {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num.toString();
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

function copyCommand() {
  if (props.skill.installCommand) {
    navigator.clipboard.writeText(props.skill.installCommand);
  }
}

function handleInstall() {
  emit('install', props.skill);
}

function handleSync() {
  // Emit sync event
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
  max-width: 600px;
  max-height: 85vh;
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

.skill-header {
  display: flex;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 20px;
}

.skill-icon {
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

.skill-title {
  flex: 1;
}

.skill-title h2 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 8px;
}

.skill-meta {
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

.skill-badges {
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
  text-transform: capitalize;
}

.source-badge {
  background: var(--bg-tertiary);
  color: var(--fg-muted);
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
  margin-bottom: 8px;
}

.long-description {
  font-size: 13px;
  line-height: 1.6;
  color: var(--fg-muted);
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
</style>

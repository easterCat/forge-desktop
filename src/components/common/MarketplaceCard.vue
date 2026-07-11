<script setup lang="ts">
import BaseCard from './BaseCard.vue'

interface Props {
  name: string
  author?: string
  description?: string
  version?: string
  downloads?: number
  stars?: number
  installed?: boolean
  icon?: string
  iconColor?: string
  tags?: string[]
}

withDefaults(defineProps<Props>(), {
  author: '',
  description: '',
  version: '',
  downloads: 0,
  stars: 0,
  installed: false,
  icon: 'PL',
  iconColor: '#B8944A',
  tags: () => [],
})

const emit = defineEmits<{
  click: []
  install: []
}>()

function formatDownloads(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}
</script>

<template>
  <BaseCard variant="glass" padding="lg" class="marketplace-card" @click="emit('click')">
    <div class="marketplace-card-head">
      <div
        class="marketplace-card-icon"
        :style="{
          background: `${iconColor}15`,
          borderColor: `${iconColor}40`,
          color: iconColor,
        }"
      >
        {{ icon }}
      </div>
      <div class="marketplace-card-info">
        <div class="marketplace-card-name">
          {{ name }}
          <span v-if="installed" class="installed-dot" />
        </div>
        <div class="marketplace-card-author">
          <template v-if="author">by {{ author }}</template>
        </div>
      </div>
    </div>

    <p v-if="description" class="marketplace-card-desc">{{ description }}</p>

    <div v-if="tags.length > 0" class="marketplace-card-tags">
      <span v-for="tag in tags" :key="tag" class="tag">{{ tag }}</span>
    </div>

    <div class="marketplace-card-meta">
      <span v-if="downloads > 0">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        {{ formatDownloads(downloads) }}
      </span>
      <span v-if="stars > 0">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
        </svg>
        {{ formatDownloads(stars) }}
      </span>
    </div>

    <div class="marketplace-card-footer">
      <span v-if="version" class="version">v{{ version }}</span>
      <span v-else class="version version-empty" />
      <div class="btn-group">
        <button
          class="btn btn-icon"
          title="Details"
          @click.stop="emit('click')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
        </button>
        <button
          v-if="!installed"
          class="btn btn-primary btn-sm"
          @click.stop="emit('install')"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <span>Install</span>
        </button>
        <button v-else class="btn btn-secondary btn-sm">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          <span>Installed</span>
        </button>
      </div>
    </div>
  </BaseCard>
</template>

<style scoped>
.marketplace-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3, 12px);
  overflow: hidden;
  position: relative;
}

.marketplace-card-head {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.marketplace-card-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  border: 1px solid;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
}

.marketplace-card-info {
  flex: 1;
  min-width: 0;
}

.marketplace-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.installed-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--success);
  flex-shrink: 0;
}

.marketplace-card-author {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
}

.marketplace-card-desc {
  font-size: 12px;
  color: var(--fg-muted);
  line-height: 1.5;
  min-height: 2.1em;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.marketplace-card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 99px;
  background: rgba(255, 255, 255, 0.40);
  border: 1px solid rgba(255, 255, 255, 0.32);
  color: var(--fg-ghost);
  font-weight: 500;
}

.marketplace-card-meta {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 11px;
  color: var(--fg-ghost);
}

.marketplace-card-meta svg {
  flex-shrink: 0;
  opacity: 0.5;
}

.marketplace-card-meta span {
  display: flex;
  align-items: center;
  gap: 4px;
}

.marketplace-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid var(--border);
  padding-top: 12px;
  margin-top: auto;
}

.marketplace-card-footer .version {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-ghost);
}

.marketplace-card-footer .version-empty {
  min-width: 24px;
}

.marketplace-card-footer .btn-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.marketplace-card-footer .btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.marketplace-card-footer .btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
</style>

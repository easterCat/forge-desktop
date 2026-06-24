<template>
  <div class="skill-card" :class="{ installed: isInstalled }">
    <!-- Header with icon and badges -->
    <div class="card-header">
      <div class="skill-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
      </div>
      <div class="card-badges">
        <span v-if="isInstalled" class="badge installed-badge">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          Installed
        </span>
        <span class="badge source-badge">{{ skill.sourceId }}</span>
      </div>
    </div>

    <!-- Title and description -->
    <div class="card-content">
      <h3 class="skill-name">{{ skill.name }}</h3>
      <p class="skill-desc">{{ skill.description }}</p>
    </div>

    <!-- Categories -->
    <div class="card-categories">
      <span
        v-for="cat in skill.categories.slice(0, 3)"
        :key="cat"
        class="category-tag"
      >
        {{ cat }}
      </span>
      <span
        v-for="tag in skill.tags.slice(0, 2)"
        :key="tag"
        class="category-tag tag"
      >
        {{ tag }}
      </span>
    </div>

    <!-- Stats -->
    <div class="card-stats">
      <div v-if="skill.stars" class="stat">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
        {{ formatNumber(skill.stars) }}
      </div>
      <div v-if="skill.downloads" class="stat">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        {{ formatNumber(skill.downloads) }}
      </div>
      <div v-if="skill.version" class="stat">
        v{{ skill.version }}
      </div>
      <div v-if="skill.author" class="stat author">
        by {{ skill.author }}
      </div>
    </div>

    <!-- Progress bar -->
    <div v-if="progress" class="install-progress">
      <div class="progress-info">
        <span class="progress-stage">{{ getStageLabel(progress.stage) }}</span>
        <span class="progress-percent">{{ progress.progress }}%</span>
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :class="progress.stage"
          :style="{ width: `${progress.progress}%` }"
        ></div>
      </div>
      <p v-if="progress.message" class="progress-message">{{ progress.message }}</p>
    </div>

    <!-- Actions -->
    <div class="card-actions">
      <button
        class="btn btn-detail btn-sm"
        @click="$emit('view-details', skill)"
      >
        Details
      </button>
      <button
        v-if="!isInstalled"
        class="btn btn-primary btn-sm"
        :disabled="!!progress"
        @click="$emit('install', skill)"
      >
        <svg v-if="!progress" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        <span v-if="progress">Installing...</span>
        <span v-else>Install</span>
      </button>
      <button
        v-else
        class="btn btn-secondary btn-sm"
        @click="$emit('sync', skill)"
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
</template>

<script setup lang="ts">
import type { MarketplaceSkill, InstallProgress } from '@/types';

interface Props {
  skill: MarketplaceSkill;
  isInstalled?: boolean;
  progress?: InstallProgress;
}

defineProps<Props>();

defineEmits<{
  (e: 'install', skill: MarketplaceSkill): void;
  (e: 'sync', skill: MarketplaceSkill): void;
  (e: 'view-details', skill: MarketplaceSkill): void;
}>();

function formatNumber(num: number): string {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num.toString();
}

function getStageLabel(stage: string): string {
  const labels: Record<string, string> = {
    pending: 'Pending',
    downloading: 'Downloading',
    installing: 'Installing',
    syncing: 'Syncing',
    success: 'Complete',
    failed: 'Failed',
    conflict: 'Conflict',
  };
  return labels[stage] || stage;
}
</script>

<style scoped>
.skill-card {
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  transition: all 0.2s;
}

.skill-card:hover {
  background: var(--bg-card-hover);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

.skill-card.installed {
  border-color: var(--success);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.skill-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent);
  border-radius: 10px;
  color: white;
}

.card-badges {
  display: flex;
  gap: 6px;
}

.badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.installed-badge {
  background: var(--success-bg);
  color: var(--success);
}

.source-badge {
  text-transform: capitalize;
}

.card-content {
  flex: 1;
  margin-bottom: 12px;
}

.skill-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  color: var(--fg);
}

.skill-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.5;
  min-height: 2.1em;
}

.card-categories {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.category-tag {
  padding: 3px 8px;
  font-size: 11px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  color: var(--fg-muted);
}

.category-tag.tag {
  background: var(--accent-bg);
  color: var(--accent);
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  margin-bottom: 12px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--fg-muted);
}

.stat svg {
  color: var(--warn);
}

.stat.author {
  margin-left: auto;
  font-style: italic;
}

.install-progress {
  margin-bottom: 12px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 12px;
}

.progress-stage {
  color: var(--fg-muted);
}

.progress-percent {
  font-weight: 500;
  color: var(--fg);
}

.progress-bar {
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-fill.success {
  background: var(--success);
}

.progress-fill.failed {
  background: var(--error);
}

.progress-fill.syncing {
  background: var(--warn);
}

.progress-message {
  margin-top: 4px;
  font-size: 11px;
  color: var(--fg-muted);
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-actions .btn {
  flex: 0 0 auto;
}

.card-actions .btn-primary,
.card-actions .btn-secondary {
  margin-left: auto;
}
</style>

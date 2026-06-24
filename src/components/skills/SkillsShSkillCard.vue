<template>
  <div class="skill-card" @click="$emit('click', skill)">
    <div class="card-header">
      <div class="skill-icon">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
      </div>
      <div class="card-badges">
        <span class="badge source-badge">{{ formatSource(skill.source) }}</span>
        <span v-if="skill.sourceType === 'github'" class="badge github-badge">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          GitHub
        </span>
      </div>
    </div>

    <div class="card-content">
      <h3 class="skill-name">{{ skill.name }}</h3>
      <p class="skill-slug">{{ skill.slug }}</p>
    </div>

    <div class="card-stats">
      <div class="install-count">
        <span class="count-value">{{ formatNumber(skill.installs) }}</span>
        <span class="count-label">安装</span>
      </div>
      <div v-if="skill.installsYesterday" class="hot-change">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
          <polyline points="17 6 23 6 23 12"/>
        </svg>
        <span>+{{ formatNumber(skill.installsYesterday) }}/天</span>
      </div>
    </div>

    <div class="card-actions">
      <button class="btn btn-detail btn-sm" @click.stop="$emit('detail', skill)">
        详情
      </button>
      <button class="btn btn-secondary btn-sm" @click.stop="$emit('copy', skill)" title="复制安装命令">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
          <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
        </svg>
      </button>
      <button
        class="btn btn-primary btn-sm"
        @click.stop="$emit('install', skill)"
        :disabled="!skill.installUrl"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        安装
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SkillsShSkill } from '@/types';

const props = defineProps<{
  skill: SkillsShSkill;
}>();

defineEmits<{
  (e: 'click', skill: SkillsShSkill): void;
  (e: 'detail', skill: SkillsShSkill): void;
  (e: 'copy', skill: SkillsShSkill): void;
  (e: 'install', skill: SkillsShSkill): void;
}>();

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
  }
  return num.toString();
}

function formatSource(source: string): string {
  const parts = source.split('/');
  if (parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return source;
}
</script>

<style scoped>
.skill-card {
  display: flex;
  flex-direction: column;
  padding: 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.skill-card:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-md);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.skill-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent);
  border-radius: 6px;
  color: var(--fg-white);
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
  font-size: 10px;
  font-weight: 500;
  border-radius: 4px;
  background: var(--bg-tertiary);
  color: var(--fg-muted);
}

.source-badge {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.github-badge {
  color: #e4e4e7;
}

.card-content {
  margin-bottom: 10px;
}

.skill-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 2px;
}

.skill-slug {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

.card-stats {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  margin-bottom: 10px;
}

.install-count {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.count-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--accent);
  font-family: var(--font-mono);
}

.count-label {
  font-size: 10px;
  color: var(--fg-muted);
}

.hot-change {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--success);
}

.card-actions {
  display: flex;
  gap: 6px;
}

.card-actions .btn {
  flex: 0 0 auto;
  justify-content: center;
}

.card-actions .btn-primary {
  margin-left: auto;
}
</style>

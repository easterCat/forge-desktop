<template>
  <div class="source-group">
    <div class="group-header" @click="toggleExpanded">
      <div class="source-info">
        <div class="source-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
          </svg>
        </div>
        <div class="source-details">
          <span class="source-name">{{ groupSource }}</span>
          <span class="source-meta">{{ group.skills?.length || 0 }} 个技能</span>
        </div>
      </div>
      <div class="source-stats">
        <span class="install-count">{{ formatNumber(group.totalInstalls) }}</span>
        <svg
          class="expand-icon"
          :class="{ expanded: isExpanded }"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </div>
    </div>

    <Transition name="expand">
      <div v-if="isExpanded" class="group-content">
        <div class="featured-skill" v-if="featuredSkill">
          <span class="featured-label">精选</span>
          <span class="featured-name">{{ featuredSkill.name }}</span>
        </div>

        <div class="skills-list">
          <div
            v-for="skill in group.skills"
            :key="skill.id"
            class="skill-row"
            @click="$emit('skillClick', skill)"
          >
            <div class="skill-info">
              <span class="skill-name">{{ skill.name }}</span>
              <span class="skill-slug">{{ skill.slug }}</span>
            </div>
            <div class="skill-actions">
              <span class="install-stat">{{ formatNumber(skill.installs) }}</span>
              <button class="btn-icon" @click.stop="$emit('detail', skill)" title="查看详情">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8"/>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { SkillsShSourceGroup, SkillsShCuratedOwnerGroup } from '@/types';

const props = defineProps<{
  group: SkillsShSourceGroup | SkillsShCuratedOwnerGroup;
  defaultExpanded?: boolean;
}>();

defineEmits<{
  (e: 'skillClick', skill: any): void;
  (e: 'detail', skill: any): void;
}>();

const isExpanded = ref(props.defaultExpanded ?? false);

// CuratedOwnerGroup has 'owner', SkillsShSourceGroup has 'source'
const groupSource = computed(() => {
  return 'owner' in props.group
    ? (props.group as SkillsShCuratedOwnerGroup).owner
    : (props.group as SkillsShSourceGroup).source;
});

const featuredSkill = computed(() => {
  const skills = props.group.skills;
  if (skills.length === 0) return null;
  return skills.reduce((prev, curr) =>
    prev.installs > curr.installs ? prev : curr
  );
});

function toggleExpanded() {
  isExpanded.value = !isExpanded.value;
}

function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
  }
  return num.toString();
}
</script>

<style scoped>
.source-group {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.group-header:hover {
  background: var(--bg-tertiary);
}

.source-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.source-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-bg);
  border-radius: 8px;
  color: var(--accent);
}

.source-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.source-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg);
}

.source-meta {
  font-size: 11px;
  color: var(--fg-muted);
}

.source-stats {
  display: flex;
  align-items: center;
  gap: 12px;
}

.install-count {
  font-size: 15px;
  font-weight: 700;
  color: var(--accent);
  font-family: var(--font-mono);
}

.expand-icon {
  color: var(--fg-muted);
  transition: transform 0.2s;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.group-content {
  border-top: 1px solid var(--border);
  padding: 12px 16px;
}

.featured-skill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--accent-bg);
  border-radius: 6px;
  margin-bottom: 12px;
}

.featured-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.featured-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skill-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-primary);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.skill-row:hover {
  background: var(--bg-tertiary);
}

.skill-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.skill-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
}

.skill-slug {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

.skill-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.install-stat {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.15s;
}

.btn-icon:hover {
  background: var(--accent);
  color: white;
}

/* Expand transition */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 500px;
}
</style>

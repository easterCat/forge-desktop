<script setup lang="ts">
import BaseCard from './BaseCard.vue'

export type StatTint = 'warm' | 'cool' | 'soft' | 'amber'

interface Props {
  label: string
  value: string | number
  sub?: string
  tint?: StatTint
  accent?: boolean
  valueColor?: string
}

withDefaults(defineProps<Props>(), {
  tint: 'warm',
  accent: false,
})
</script>

<template>
  <BaseCard variant="elevated" padding="lg" class="stat-card" :class="`tint-${tint}`">
    <div class="stat-label">{{ label }}</div>
    <div
      class="stat-value"
      :class="{ accent }"
      :style="valueColor ? { color: valueColor } : {}"
    >
      {{ value }}
    </div>
    <div v-if="sub" class="stat-sub">{{ sub }}</div>
    <slot />
  </BaseCard>
</template>

<style scoped>
.stat-card {
  --tint-color: rgba(195, 178, 155, 0.12);
  overflow: hidden;
}

.stat-label {
  font-size: var(--text-xs, 11px);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-bottom: var(--space-2, 8px);
  font-weight: 600;
}

.stat-value {
  font-family: var(--font-mono);
  font-size: var(--text-3xl, 32px);
  font-weight: 700;
  color: var(--fg-title);
  line-height: var(--leading-tight, 1.2);
}

.stat-value.accent {
  color: var(--accent);
}

.stat-sub {
  font-size: var(--text-xs, 11px);
  color: var(--fg-ghost);
  margin-top: var(--space-1-5, 6px);
  font-family: var(--font-mono);
}

/* Tint Variants — 简化版，单一渐变 */
.stat-card.tint-warm { --tint-color: rgba(195, 178, 155, 0.15); }
.stat-card.tint-cool { --tint-color: rgba(155, 178, 195, 0.15); }
.stat-card.tint-soft { --tint-color: rgba(178, 155, 195, 0.15); }
.stat-card.tint-amber { --tint-color: rgba(195, 178, 155, 0.18); }

.stat-card::before {
  content: '';
  position: absolute;
  top: -50%;
  right: -30%;
  width: 120%;
  height: 120%;
  background: radial-gradient(
    ellipse at center,
    var(--tint-color) 0%,
    transparent 70%
  );
  pointer-events: none;
  opacity: 0.8;
  transition: opacity 0.3s, transform 0.3s;
}

.stat-card:hover::before {
  opacity: 1;
  transform: scale(1.1);
}
</style>

<script setup lang="ts">
import { computed } from 'vue'
import type { OperationStage } from '@/composables/useOperationProgress'

interface Props {
  stage: OperationStage
  progress: number
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
})

const progressColor = computed(() => {
  if (props.stage === 'idle') return 'var(--fg-ghost)'
  switch (props.stage) {
    case 'completed': return 'var(--success)'
    case 'failed': return 'var(--error)'
    case 'cancelled': return 'var(--fg-ghost)'
    default: return 'var(--accent)'
  }
})

const barWidth = computed(() => {
  if (props.stage === 'idle') return '100%'
  return props.progress + '%'
})
</script>

<template>
  <div class="progress-slot" :class="{ idle: stage === 'idle' }">
    <div class="progress-bar-wrap">
      <div
        class="progress-bar-fill"
        :class="stage"
        :style="{ width: barWidth, background: progressColor }"
      />
    </div>
  </div>
</template>

<style scoped>
.progress-slot {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  width: 100%;
}

.progress-bar-wrap {
  flex: 1;
  min-width: 0;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-slot.idle .progress-bar-wrap {
  background: transparent;
}

.progress-slot.idle .progress-bar-fill {
  background: transparent;
}
</style>

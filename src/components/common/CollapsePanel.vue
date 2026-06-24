<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  title: string
  expanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  expanded: true,
})

const emit = defineEmits<{
  'update:expanded': [value: boolean]
}>()

const isExpanded = ref(props.expanded)

watch(() => props.expanded, (val) => {
  isExpanded.value = val
})

function toggle() {
  isExpanded.value = !isExpanded.value
  emit('update:expanded', isExpanded.value)
}
</script>

<template>
  <div class="collapse-panel">
    <div
      class="panel-header"
      role="button"
      tabindex="0"
      :aria-expanded="isExpanded"
      @click="toggle"
      @keydown.enter.prevent="toggle"
      @keydown.space.prevent="toggle"
    >
      <slot name="header">
        <span class="panel-title">{{ title }}</span>
      </slot>
      <svg
        class="chevron"
        :class="{ rotated: !isExpanded }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </div>
    <Transition name="collapse">
      <div v-show="isExpanded" class="panel-body">
        <slot />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.collapse-panel {
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: pointer;
  transition: background var(--t-fast);
  user-select: none;
}

.panel-header:hover {
  background: rgba(255, 255, 255, 0.20);
}

.panel-header:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
  border-radius: var(--radius-sm);
}

.panel-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
}

.chevron {
  color: var(--fg-muted);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.chevron.rotated {
  transform: rotate(-90deg);
}

.panel-body {
  padding: 0 16px 12px;
}

/* Collapse transition */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  max-height: 500px;
}
</style>

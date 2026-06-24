<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  modelValue?: string
  placeholder?: string
  debounceMs?: number
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  placeholder: 'Search…',
  debounceMs: 0,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const localValue = ref(props.modelValue)
const showClear = ref(false)

watch(() => props.modelValue, (val) => {
  localValue.value = val
  showClear.value = val.length > 0
})

watch(localValue, (val) => {
  showClear.value = val.length > 0
  emit('update:modelValue', val)
})

function clear() {
  localValue.value = ''
  showClear.value = false
  emit('update:modelValue', '')
}
</script>

<template>
  <div class="search-input">
    <svg
      class="search-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="var(--fg-ghost)"
      stroke-width="2"
      stroke-linecap="round"
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    <input
      v-model="localValue"
      type="text"
      :placeholder="placeholder"
    />
    <button
      v-show="showClear"
      class="clear-btn visible"
      type="button"
      aria-label="Clear search"
      @click="clear"
    >
      <svg
        width="10"
        height="10"
        viewBox="0 0 10 10"
        fill="none"
        stroke="var(--fg-ghost)"
        stroke-width="1.5"
        stroke-linecap="round"
      >
        <line x1="2" y1="2" x2="8" y2="8" />
        <line x1="8" y1="2" x2="2" y2="8" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.search-input {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 100%;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  z-index: 1;
}

input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.32);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.30);
  font-family: inherit;
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast), box-shadow var(--t-fast);
}

input:focus {
  background: rgba(255, 255, 255, 0.40);
  border-color: rgba(255, 255, 255, 0.40);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

input::placeholder {
  color: var(--fg-ghost);
}

.clear-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.32);
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--t-fast);
  border: none;
  padding: 0;
}

input:not(:placeholder-shown) ~ .clear-btn,
.clear-btn.visible {
  opacity: 1;
}
</style>

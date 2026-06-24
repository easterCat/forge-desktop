<script setup lang="ts">
interface Props {
  modelValue: boolean
  label?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
}
</script>

<template>
  <div class="toggle-wrap" :class="{ disabled }">
    <div
      class="toggle"
      :class="{ on: modelValue }"
      role="switch"
      tabindex="0"
      :aria-checked="modelValue"
      :aria-disabled="disabled"
      :aria-label="label"
      @click="toggle"
      @keydown.enter.prevent="toggle"
      @keydown.space.prevent="toggle"
    />
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.toggle-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toggle-wrap.disabled {
  opacity: 0.5;
  pointer-events: none;
}

.toggle {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.22);
  border: none;
  position: relative;
  cursor: pointer;
  transition: background var(--t-base);
  padding: 0;
}

.toggle.on {
  background: var(--accent);
}

.toggle:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.toggle-wrap .toggle::after {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: white;
  top: 50%;
  left: 2px;
  transform: translateY(-50%);
  transition: transform var(--t-base);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.toggle-wrap .toggle.on::after {
  transform: translate(16px, -50%);
}

.toggle-label {
  font-size: 12px;
  color: var(--fg-muted);
}
</style>

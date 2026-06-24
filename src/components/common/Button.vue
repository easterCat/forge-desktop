<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'icon'
  size?: 'sm' | 'md'
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  type: 'button',
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

function handleClick(event: MouseEvent) {
  if (!props.disabled) {
    emit('click', event)
  }
}
</script>

<template>
  <button
    :type="type"
    :class="[
      'btn',
      `btn-${variant}`,
      size === 'sm' ? 'btn-sm' : '',
    ]"
    :disabled="disabled"
    @click="handleClick"
  >
    <slot />
  </button>
</template>

<style>
/* Button uses global .btn classes from theme.css */
/* Variant-specific styles are also in theme.css (.btn-primary, .btn-secondary, .btn-ghost, .btn-icon) */
</style>

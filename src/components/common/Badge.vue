<script setup lang="ts">
export type BadgeType = 'success' | 'warn' | 'error' | 'info' | 'outline' | 'progress'

interface Props {
  type?: BadgeType
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  disabled: false,
})
</script>

<template>
  <span :class="['badge', type]" :disabled="disabled">
    <slot />
  </span>
</template>

<style>
/* Badge uses global .badge classes from theme.css */
</style>

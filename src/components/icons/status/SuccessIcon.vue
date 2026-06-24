<template>
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    viewBox="0 0 24 24" 
    :width="size" 
    :height="size" 
    fill="none" 
    stroke="currentColor" 
    :stroke-width="strokeWidth"
    stroke-linecap="round" 
    stroke-linejoin="round"
    :class="$attrs.class"
  >
    <polyline points="20 6 9 17 4 12"/>
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  size?: number | string
  strokeWidth?: number
}>(), {
  size: 24,
  strokeWidth: 1.5,
})
</script>

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
    <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
    <polyline points="13 2 13 9 20 9"/>
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

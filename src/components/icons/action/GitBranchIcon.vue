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
    <line x1="6" y1="3" x2="6" y2="15"/>
    <circle cx="18" cy="6" r="3"/>
    <circle cx="6" cy="18" r="3"/>
    <path d="M18 9a9 9 0 0 1-9 9"/>
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

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
    <circle cx="12" cy="12" r="3"/>
    <path d="M12 1v4"/>
    <path d="M12 19v4"/>
    <path d="M4.22 4.22l2.83 2.83"/>
    <path d="M16.95 16.95l2.83 2.83"/>
    <path d="M1 12h4"/>
    <path d="M19 12h4"/>
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

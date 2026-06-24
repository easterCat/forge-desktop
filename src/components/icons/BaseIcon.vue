<!--
  Forge - SVG Icons Component
  Usage: Import individual icons or use as a sprite
  
  <script setup>
  import { DashboardIcon, SearchIcon, SettingsIcon } from '@/components/icons'
  </script>
  
  <template>
    <DashboardIcon class="icon" />
    <SearchIcon class="icon icon-sm" />
    <SettingsIcon class="icon" />
  </template>
-->

<template>
  <svg 
    :width="size" 
    :height="size" 
    viewBox="0 0 24 24" 
    fill="none" 
    :stroke="color" 
    :stroke-width="strokeWidth"
    stroke-linecap="round" 
    stroke-linejoin="round"
    :class="$attrs.class"
  >
    <slot />
  </svg>
</template>

<script setup lang="ts">
defineOptions({ name: 'BaseIcon' })

withDefaults(defineProps<{
  size?: number | string
  color?: string
  strokeWidth?: number
}>(), {
  size: 24,
  color: 'currentColor',
  strokeWidth: 1.5,
})
</script>

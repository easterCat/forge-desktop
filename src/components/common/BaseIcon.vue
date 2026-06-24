<script setup lang="ts">
interface Props {
  /** 图标名称 */
  name?: string
  /** 图标尺寸 */
  size?: 'xs' | 'sm' | 'md' | 'lg'
  /** 图标颜色 */
  color?: string
}

withDefaults(defineProps<Props>(), {
  size: 'md',
  color: 'currentColor',
})
</script>

<template>
  <svg
    :class="['base-icon', `size-${size}`]"
    :style="{ color }"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <slot />
  </svg>
</template>

<style scoped>
/* ================================================
   BASE ICON — 统一图标尺寸
   ================================================ */
.base-icon {
  flex-shrink: 0;
  transition: color 0.2s;
}

/* Sizes */
.size-xs {
  width: 12px;
  height: 12px;
}

.size-sm {
  width: 14px;
  height: 14px;
}

.size-md {
  width: 18px;
  height: 18px;
}

.size-lg {
  width: 24px;
  height: 24px;
}
</style>

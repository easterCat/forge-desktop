<script setup lang="ts">
interface Props {
  /** Single character or initials to display */
  label: string
  /** Background color for the avatar circle */
  bg?: string
  /** Text color */
  color?: string
  /** Size in pixels (default 28) */
  size?: number
}

withDefaults(defineProps<Props>(), {
  bg: 'rgba(255, 255, 255, 0.30)',
  color: 'var(--accent)',
  size: 28,
})
</script>

<template>
  <div
    class="avatar"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      background: bg,
      color: color,
      fontSize: `${Math.round(size * 0.4)}px`,
    }"
    :aria-label="label"
    role="img"
  >
    {{ label }}
  </div>
</template>

<style scoped>
.avatar {
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.30);
  border: 1px solid rgba(255, 255, 255, 0.32);
  color: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  flex-shrink: 0;
  user-select: none;
  transition: all var(--t-fast);
}

.avatar:hover {
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg);
}
</style>

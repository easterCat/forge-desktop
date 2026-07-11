<script setup lang="ts">
interface Props {
  lines?: number
  height?: string
  animated?: boolean
}

withDefaults(defineProps<Props>(), {
  lines: 3,
  height: '14px',
  animated: true,
})
</script>

<template>
  <div class="skeleton-wrapper" :class="{ 'no-animation': !animated }">
    <div
      v-for="i in lines"
      :key="i"
      class="skeleton skeleton-line"
      :class="{
        short: i === 1 && lines > 1,
        medium: i === 2 && lines > 2,
      }"
      :style="{ height }"
    />
  </div>
</template>

<style scoped>
.skeleton-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton {
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0.40) 25%,
    rgba(255, 255, 255, 0.40) 50%,
    rgba(255, 255, 255, 0.40) 75%
  );
  background-size: 200% 100%;
  border-radius: var(--radius-sm);
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-line.short {
  width: 60%;
}

.skeleton-line.medium {
  width: 80%;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.no-animation .skeleton {
  animation: none;
  background: rgba(255, 255, 255, 0.30);
}

@media (prefers-reduced-motion: reduce) {
  .skeleton {
    animation: none;
    background: rgba(255, 255, 255, 0.30);
  }
}
</style>

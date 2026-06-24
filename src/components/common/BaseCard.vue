<script setup lang="ts">
interface Props {
  /** 卡片变体 */
  variant?: 'default' | 'glass' | 'elevated'
  /** 是否可交互 */
  interactive?: boolean
  /** 内部间距 */
  padding?: 'sm' | 'md' | 'lg'
}

withDefaults(defineProps<Props>(), {
  variant: 'glass',
  interactive: false,
  padding: 'md',
})
</script>

<template>
  <div
    :class="[
      'base-card',
      `variant-${variant}`,
      `pad-${padding}`,
      { interactive, 'liquid-glass': variant !== 'default' }
    ]"
  >
    <slot />
  </div>
</template>

<style scoped>
/* ================================================
   BASE CARD — 核心容器
   ================================================ */
.base-card {
  position: relative;
  border-radius: var(--radius-lg, 16px);
  transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

/* --- Variants --- */
.variant-default {
  background: var(--glass-bg);
  border: 1px solid var(--border);
}

.variant-glass {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-bg-blur, 20px)) saturate(var(--glass-bg-saturation, 1.2));
  -webkit-backdrop-filter: blur(var(--glass-bg-blur, 20px)) saturate(var(--glass-bg-saturation, 1.2));
  border: 1px solid var(--border);
  box-shadow:
    0 2px 16px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 var(--glass-inner-glow);
}

.variant-elevated {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-bg-blur, 20px)) saturate(var(--glass-bg-saturation, 1.2));
  -webkit-backdrop-filter: blur(var(--glass-bg-blur, 20px)) saturate(var(--glass-bg-saturation, 1.2));
  border: 1px solid var(--border);
  box-shadow:
    0 4px 24px rgba(0, 0, 0, 0.08),
    0 0 0 1px rgba(255, 255, 255, 0.3),
    inset 0 1px 0 var(--glass-inner-glow);
}

/* --- Padding --- */
.pad-sm { padding: var(--space-3, 12px); }
.pad-md { padding: var(--space-4, 16px); }
.pad-lg { padding: var(--space-5, 20px); }

/* --- Interactive States --- */
.interactive {
  cursor: pointer;
}

.interactive:hover {
  background: var(--glass-bg-hover);
  border-color: var(--border-hover);
  transform: translateY(-2px);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.12),
    0 0 0 1px rgba(255, 255, 255, 0.4),
    0 0 20px var(--accent-glow, rgba(45, 45, 45, 0.08)),
    inset 0 1px 0 var(--glass-inner-glow);
}

.interactive:active {
  transform: translateY(0) scale(0.99);
  transition-duration: 0.1s;
  box-shadow:
    0 2px 8px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 var(--glass-inner-glow);
}

/* --- Focus Visible (Accessibility) --- */
.interactive:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* --- Reduced Motion --- */
@media (prefers-reduced-motion: reduce) {
  .base-card {
    transition: none;
  }
  .interactive:hover {
    transform: none;
  }
}
</style>

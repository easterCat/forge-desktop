<script setup lang="ts">
interface Props {
  /** 按钮变体 */
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  /** 按钮尺寸 */
  size?: 'sm' | 'md' | 'lg'
  /** 是否禁用 */
  disabled?: boolean
  /** 是否加载中 */
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
})
</script>

<template>
  <button
    :class="[
      'base-btn',
      `variant-${variant}`,
      `size-${size}`,
      { disabled, loading }
    ]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="btn-spinner" />
    <slot />
  </button>
</template>

<style scoped>
/* ================================================
   BASE BUTTON — 统一按钮样式
   ================================================ */
.base-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2, 8px);
  font-weight: 500;
  border-radius: var(--radius-md, 10px);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  white-space: nowrap;
  user-select: none;
}

.base-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.base-btn:active:not(.disabled) {
  transform: scale(0.98);
}

/* --- Sizes --- */
.size-sm {
  padding: var(--space-1-5, 6px) var(--space-3, 12px);
  font-size: var(--text-sm, 12px);
  height: 32px;
}

.size-md {
  padding: var(--space-2, 8px) var(--space-4, 16px);
  font-size: var(--text-base, 14px);
  height: 38px;
}

.size-lg {
  padding: var(--space-3, 12px) var(--space-5, 20px);
  font-size: var(--text-lg, 16px);
  height: 44px;
}

/* --- Variants --- */
.variant-primary {
  background: var(--accent);
  color: var(--fg-white);
  border-color: var(--accent);
}

.variant-primary:hover:not(.disabled) {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
  box-shadow: 0 4px 16px var(--accent-glow);
}

.variant-primary:active:not(.disabled) {
  background: var(--accent-press);
}

.variant-secondary {
  background: var(--glass-bg);
  color: var(--fg);
  border-color: var(--border);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.variant-secondary:hover:not(.disabled) {
  background: var(--glass-bg-hover);
  border-color: var(--border-hover);
  box-shadow: var(--shadow-sm);
}

.variant-ghost {
  background: transparent;
  color: var(--fg-muted);
  border-color: transparent;
}

.variant-ghost:hover:not(.disabled) {
  background: var(--glass-bg);
  color: var(--fg);
}

.variant-danger {
  background: var(--error);
  color: white;
  border-color: var(--error);
}

.variant-danger:hover:not(.disabled) {
  background: #a04a35;
  box-shadow: 0 4px 16px rgba(184, 90, 66, 0.3);
}

/* --- States --- */
.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.loading {
  position: relative;
  color: transparent;
}

/* --- Spinner --- */
.btn-spinner {
  position: absolute;
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.variant-primary .btn-spinner {
  border-top-color: var(--fg-white);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* --- Reduced Motion --- */
@media (prefers-reduced-motion: reduce) {
  .base-btn {
    transition: none;
  }
  .btn-spinner {
    animation-duration: 1.5s;
  }
}
</style>

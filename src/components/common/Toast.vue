<script setup lang="ts">
import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'info' | 'warn'

export interface ToastOptions {
  id?: string
  type?: ToastType
  message: string
  duration?: number
}

interface Props {
  toasts?: ToastOptions[]
}

const props = withDefaults(defineProps<Props>(), {
  toasts: () => [],
})

const emit = defineEmits<{
  dismiss: [id: string]
}>()

const visibleToasts = ref<ToastOptions[]>([...props.toasts])

function addToast(toast: ToastOptions) {
  const id = toast.id ?? `toast-${Date.now()}`
  visibleToasts.value.push({ ...toast, id })
  if (toast.duration !== 0) {
    setTimeout(() => removeToast(id), toast.duration ?? 4000)
  }
}

function removeToast(id: string) {
  const idx = visibleToasts.value.findIndex(t => t.id === id)
  if (idx !== -1) {
    visibleToasts.value.splice(idx, 1)
    emit('dismiss', id)
  }
}

defineExpose({ addToast })
</script>

<template>
  <Teleport to="body">
    <div class="toast-container" aria-live="polite" aria-atomic="false">
      <TransitionGroup name="toast">
        <div
          v-for="toast in visibleToasts"
          :key="toast.id"
          :class="['toast', toast.type ?? 'info']"
          role="alert"
        >
          <!-- Success icon -->
          <svg v-if="toast.type === 'success'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2.5" stroke-linecap="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <!-- Error icon -->
          <svg v-else-if="toast.type === 'error'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--error)" stroke-width="2.5" stroke-linecap="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="15" y1="9" x2="9" y2="15" />
            <line x1="9" y1="9" x2="15" y2="15" />
          </svg>
          <!-- Warn icon -->
          <svg v-else-if="toast.type === 'warn'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--warn)" stroke-width="2.5" stroke-linecap="round">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          <!-- Info icon (default) -->
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--info)" stroke-width="2.5" stroke-linecap="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
          <span class="toast-message">{{ toast.message }}</span>
          <button class="toast-dismiss" @click="removeToast(toast.id!)" aria-label="Dismiss">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 200;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  position: relative;
  background: var(--glass-bg);
  backdrop-filter: blur(30px) saturate(1.3);
  -webkit-backdrop-filter: blur(30px) saturate(1.3);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.10), inset 0 1px 0 rgba(255, 255, 255, 0.40);
  pointer-events: auto;
  max-width: 360px;
}

.toast.success {
  border-color: rgba(90, 138, 100, 0.40);
}

.toast.error {
  border-color: rgba(184, 90, 66, 0.40);
}

.toast.info {
  border-color: rgba(90, 107, 122, 0.40);
}

.toast.warn {
  border-color: rgba(184, 148, 74, 0.40);
}

.toast-message {
  flex: 1;
  color: var(--fg-muted);
  line-height: 1.4;
}

.toast-dismiss {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--glass-input);
  border: none;
  color: var(--fg-ghost);
  cursor: pointer;
  flex-shrink: 0;
  padding: 0;
  transition: all var(--t-fast);
}

.toast-dismiss:hover {
  background: var(--glass-bg-hover);
  color: var(--fg);
}

/* Vue TransitionGroup */
.toast-enter-active {
  animation: toastIn var(--t-slow) ease forwards;
}

.toast-leave-active {
  animation: toastOut var(--t-slow) ease forwards;
  position: absolute;
  right: 0;
}

.toast-move {
  transition: transform 300ms ease;
}

@keyframes toastIn {
  from {
    opacity: 0;
    transform: translateX(30px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toastOut {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(30px);
  }
}
</style>

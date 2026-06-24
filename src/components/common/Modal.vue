<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

interface Props {
  modelValue?: boolean
  title?: string
  width?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  title: '',
  width: '560px',
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  close: []
}>()

const isVisible = ref(props.modelValue)
const isLeaving = ref(false)

watch(() => props.modelValue, (val) => {
  if (val) {
    isVisible.value = true
    isLeaving.value = false
    document.body.style.overflow = 'hidden'
  } else {
    triggerExit()
  }
})

function triggerExit() {
  isLeaving.value = true
  setTimeout(() => {
    isVisible.value = false
    isLeaving.value = false
    document.body.style.overflow = ''
    emit('update:modelValue', false)
    emit('close')
  }, 300)
}

function closeOnOverlay(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    triggerExit()
  }
}

function close() {
  triggerExit()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.modelValue) {
    triggerExit()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = ''
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="isVisible"
        class="modal-overlay"
        @click="closeOnOverlay"
      >
        <div
          :class="['modal', { leaving: isLeaving }]"
          :style="{ maxWidth: width }"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="title ? 'modal-title' : undefined"
        >
          <div class="modal-header">
            <h3 v-if="title" id="modal-title">{{ title }}</h3>
            <button class="modal-close" @click="close" aria-label="Close modal">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          <div class="modal-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="modal-actions">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.22);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal {
  background: var(--glass-bg);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  padding: 32px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), 0 4px 20px rgba(0, 0, 0, 0.05), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  position: relative;
  transition: opacity 300ms ease, transform 300ms ease;
}

.modal.leaving {
  animation: modalOut 300ms ease forwards;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 20px;
}

.modal-header h3 {
  font-size: 20px;
  font-weight: 700;
  color: var(--fg-title);
  letter-spacing: -0.02em;
  line-height: 1.3;
  margin: 0;
}

.modal-close {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--glass-input);
  border: 1px solid var(--border);
  color: var(--fg-ghost);
  cursor: pointer;
  transition: all var(--t-fast);
  flex-shrink: 0;
  padding: 0;
}

.modal-close:hover {
  background: var(--glass-bg-hover);
  color: var(--fg);
}

.modal-body {
  color: var(--fg-muted);
}

.modal-body label {
  display: block;
  font-size: 13px;
  color: var(--fg-muted);
  margin-bottom: 8px;
  margin-top: 20px;
  font-weight: 500;
  letter-spacing: 0.01em;
}

.modal-body input,
.modal-body select,
.modal-body textarea {
  width: 100%;
  padding: 10px 14px;
  font-size: 14px;
  line-height: 1.5;
  background: var(--glass-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--fg);
  font-family: inherit;
  outline: none;
  transition: border-color var(--t-fast), background var(--t-fast);
}

.modal-body input:focus,
.modal-body select:focus,
.modal-body textarea:focus {
  border-color: var(--border-hover);
  background: var(--glass-input-focus);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

.modal-body textarea {
  min-height: 100px;
  resize: vertical;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 28px;
  padding-top: 20px;
  border-top: 1px solid var(--border);
}

/* Vue Transition */
.modal-enter-active {
  animation: modalIn 300ms ease forwards;
}

.modal-leave-active {
  animation: modalOut 300ms ease forwards;
}

@keyframes modalIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@keyframes modalOut {
  from {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
  to {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
  }
}
</style>

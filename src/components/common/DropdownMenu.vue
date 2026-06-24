<script setup lang="ts">
import { ref, watch, nextTick, onBeforeUnmount } from 'vue'
import { useDropdown } from '@/composables/useDropdown'

interface Props {
  modelValue: boolean
  minWidth?: number
}

const props = withDefaults(defineProps<Props>(), {
  minWidth: 160,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const wrapperRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const { positionStyle, computePosition } = useDropdown(triggerRef)

// Find the actual trigger button inside the wrapper and bind ref to it
function findTriggerButton() {
  if (!wrapperRef.value) return
  const btn = wrapperRef.value.querySelector('.btn-icon, .btn') as HTMLElement | null
  if (btn) triggerRef.value = btn
}

watch(() => props.modelValue, async (open) => {
  if (open) {
    findTriggerButton()
    await nextTick()
    const menuHeight = menuRef.value?.offsetHeight || 200
    computePosition(props.minWidth, menuHeight)
  }
}, { immediate: true })

function handleClickOutside(e: MouseEvent) {
  if (!props.modelValue) return
  const target = e.target as Node
  // Ignore clicks on the trigger button (inside wrapper) — let @click.stop handle toggle
  if (wrapperRef.value?.contains(target)) return
  if (menuRef.value && !menuRef.value.contains(target)) {
    emit('update:modelValue', false)
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.modelValue) {
    emit('update:modelValue', false)
  }
}

function handleScroll() {
  if (props.modelValue) {
    emit('update:modelValue', false)
  }
}

watch(() => props.modelValue, (open) => {
  if (open) {
    document.addEventListener('click', handleClickOutside, true)
    document.addEventListener('keydown', handleKeydown)
    window.addEventListener('scroll', handleScroll, true)
  } else {
    document.removeEventListener('click', handleClickOutside, true)
    document.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('scroll', handleScroll, true)
  }
}, { immediate: true })

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside, true)
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('scroll', handleScroll, true)
})
</script>

<template>
  <div class="dropdown-wrapper" @click.stop ref="wrapperRef">
    <slot name="trigger" />
    <Teleport to="body">
      <Transition name="dropdown">
        <div
          v-if="modelValue"
          ref="menuRef"
          class="dropdown-menu"
          :style="[positionStyle, { minWidth: minWidth + 'px' }]"
        >
          <slot />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

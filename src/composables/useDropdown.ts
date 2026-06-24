import { ref, nextTick, type Ref, type CSSProperties } from 'vue'

export function useDropdown(triggerRef: Ref<HTMLElement | null>) {
  const positionStyle = ref<CSSProperties>({})

  async function computePosition(menuWidth = 160, menuHeight = 200) {
    await nextTick()

    const trigger = triggerRef.value
    if (!trigger) {
      positionStyle.value = {}
      return
    }

    const rect = trigger.getBoundingClientRect()
    const vw = window.innerWidth
    const gap = 6

    let top: number
    let left: number

    // Try opening upward first
    if (rect.top - gap - menuHeight > 0) {
      top = rect.top - gap - menuHeight
      left = rect.right - menuWidth
    } else {
      top = rect.bottom + gap
      left = rect.right - menuWidth
    }

    // Clamp horizontal
    if (left < 8) left = 8
    if (left + menuWidth > vw - 8) left = vw - menuWidth - 8

    // Clamp vertical
    if (top < 8) top = 8

    positionStyle.value = {
      position: 'fixed',
      top: `${top}px`,
      left: `${left}px`,
    }
  }

  return { positionStyle, computePosition }
}

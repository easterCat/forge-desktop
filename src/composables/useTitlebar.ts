import { ref, onMounted } from 'vue'

export type Platform = 'macos' | 'windows' | 'linux'

const currentPlatform = ref<Platform>('macos')

export function useTitlebar() {
  async function detectPlatform() {
    try {
      const { platform } = await import('@tauri-apps/plugin-os')
      const p = await platform()
      if (p === 'macos') currentPlatform.value = 'macos'
      else if (p === 'windows') currentPlatform.value = 'windows'
      else currentPlatform.value = 'linux'
    } catch {
      const ua = navigator.userAgent.toLowerCase()
      if (ua.includes('mac')) currentPlatform.value = 'macos'
      else if (ua.includes('win')) currentPlatform.value = 'windows'
      else currentPlatform.value = 'linux'
    }
  }

  async function minimize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().minimize()
    } catch { /* dev mode */ }
  }

  async function toggleMaximize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().toggleMaximize()
    } catch { /* dev mode */ }
  }

  async function close() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch { /* dev mode */ }
  }

  onMounted(() => {
    detectPlatform()
  })

  return {
    platform: currentPlatform,
    minimize,
    toggleMaximize,
    close,
  }
}

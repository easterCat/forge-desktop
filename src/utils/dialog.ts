/**
 * Cross-platform confirm dialog utility
 * Wraps @tauri-apps/plugin-dialog for desktop, falls back to window.confirm for web
 */
export async function confirm(message: string): Promise<boolean> {
  // Tauri desktop
  if (window.__TAURI_INTERNALS__) {
    const { confirm } = await import('@tauri-apps/plugin-dialog')
    return await confirm(message)
  }
  // Web fallback
  return window.confirm(message)
}

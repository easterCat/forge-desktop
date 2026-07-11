/**
 * Cross-platform confirm dialog utility
 * Wraps @tauri-apps/plugin-dialog for desktop, falls back to window.confirm for web
 */
export async function confirm(message: string): Promise<boolean> {
  // Tauri desktop: the global is injected at runtime; TS has no type for
  // it, so feature-detect via a guard and use a typed alias inside.
  const w = window as unknown as {
    __TAURI_INTERNALS__?: unknown;
  };
  if (w.__TAURI_INTERNALS__) {
    const { confirm } = await import('@tauri-apps/plugin-dialog');
    return await confirm(message);
  }
  // Web fallback
  return window.confirm(message);
}

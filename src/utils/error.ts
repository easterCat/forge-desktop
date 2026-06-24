/**
 * Safely extract a human-readable error message from any thrown value.
 * Tauri invoke() may throw strings, plain objects, or non-standard Errors.
 */
export function extractError(e: unknown): string {
  if (e instanceof Error) return e.message
  if (typeof e === 'string') return e
  try { return JSON.stringify(e) } catch { return String(e) }
}

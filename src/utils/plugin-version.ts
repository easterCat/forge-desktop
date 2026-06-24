/**
 * Plugin version resolution utilities.
 *
 * Resolves the display version for an installed plugin by reading manifest
 * files from the plugin's `installedPath` in priority order:
 *
 * 1. `.claude-plugin/marketplace.json` → `version`
 * 2. `.claude-plugin/plugin.json` → `version`
 * 3. `package.json` → `version`
 * 4. `"unknown"` if none found
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Resolve the display version for a plugin at the given installed path.
 * Calls the Rust backend which reads manifest files in priority order.
 */
export async function resolvePluginVersion(installedPath: string): Promise<string> {
  try {
    const version = await invoke<string>('resolve_plugin_version', { installedPath })
    return version
  } catch (e) {
    console.error('Failed to resolve plugin version:', e)
    return 'unknown'
  }
}

/**
 * Format a resolved version for display. Prepends `v` prefix.
 * Returns `v unknown` when version is empty or "unknown".
 */
export function displayVersion(version?: string | null): string {
  if (!version || version === 'unknown') return 'v unknown'
  return `v${version}`
}

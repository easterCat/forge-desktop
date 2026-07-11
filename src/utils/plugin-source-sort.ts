import type { PluginSource } from '@/types';
import { PRESET_MARKETPLACE_SOURCES } from '@/types';

/**
 * Sort the marketplace source list with two stable criteria:
 *  1. Preset sources (`PRESET_MARKETPLACE_SOURCES`) come before user-added
 *     ones, so the default curated list stays at the top regardless of
 *     plugin count.
 *  2. Within the same tier, sort by `pluginCount` descending so the
 *     content-rich repositories surface first.
 *  3. Ties (or sources with no count yet) fall back to the original list
 *     order so the sort is stable across renders.
 *
 * Pure function — accepts the list and the preset set explicitly so it
 * can be unit-tested without setting up a Pinia store.
 */
export function sortMarketplaceSources<T extends Pick<PluginSource, 'id' | 'pluginCount'>>(
  sources: readonly T[],
  presetIds: ReadonlySet<string> = new Set(PRESET_MARKETPLACE_SOURCES.map(s => s.id)),
): T[] {
  const indexed = sources.map((s, i) => ({ s, i }));
  indexed.sort((a, b) => {
    // Preset sources always come first.
    const aPreset = presetIds.has(a.s.id) ? 0 : 1;
    const bPreset = presetIds.has(b.s.id) ? 0 : 1;
    if (aPreset !== bPreset) return aPreset - bPreset;
    // Within the same tier, sort by pluginCount descending.
    const diff = (b.s.pluginCount ?? 0) - (a.s.pluginCount ?? 0);
    return diff !== 0 ? diff : a.i - b.i;
  });
  return indexed.map(x => x.s);
}
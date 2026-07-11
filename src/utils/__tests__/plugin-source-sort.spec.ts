import { describe, it, expect } from 'vitest';
import { sortMarketplaceSources } from '@/utils/plugin-source-sort';

describe('sortMarketplaceSources', () => {
  it('places preset sources before user-added ones regardless of pluginCount', () => {
    const preset = { id: 'preset-1', pluginCount: 1 };
    const user = { id: 'user-1', pluginCount: 9999 };
    const result = sortMarketplaceSources([user, preset], new Set(['preset-1']));
    expect(result.map(s => s.id)).toEqual(['preset-1', 'user-1']);
  });

  it('sorts within the same tier by pluginCount descending', () => {
    const a = { id: 'a', pluginCount: 10 };
    const b = { id: 'b', pluginCount: 100 };
    const c = { id: 'c', pluginCount: 50 };
    const result = sortMarketplaceSources([a, b, c], new Set());
    expect(result.map(s => s.id)).toEqual(['b', 'c', 'a']);
  });

  it('falls back to original list order on tie (stable sort)', () => {
    const a = { id: 'a', pluginCount: 5 };
    const b = { id: 'b', pluginCount: 5 };
    const c = { id: 'c', pluginCount: 5 };
    // Reorder input — the output should reflect the input order for ties.
    const result = sortMarketplaceSources([c, a, b], new Set());
    expect(result.map(s => s.id)).toEqual(['c', 'a', 'b']);
  });

  it('treats missing pluginCount as 0', () => {
    const withCount = { id: 'a', pluginCount: 1 };
    const noCount = { id: 'b' };
    const result = sortMarketplaceSources([noCount, withCount], new Set());
    expect(result.map(s => s.id)).toEqual(['a', 'b']);
  });

  it('returns an empty array for empty input', () => {
    expect(sortMarketplaceSources([], new Set())).toEqual([]);
  });

  it('does not mutate the input array', () => {
    const a = { id: 'a', pluginCount: 1 };
    const b = { id: 'b', pluginCount: 2 };
    const input = [a, b];
    const snapshot = [...input];
    sortMarketplaceSources(input, new Set());
    expect(input).toEqual(snapshot);
  });

  it('combines preset-priority and count-desc together', () => {
    // Two presets + two user-added; preset one has a tiny count, user
    // two has the largest. Expected order:
    //   preset (highest count) > preset (smaller count) >
    //   user (highest count) > user (smallest count)
    const presetHigh = { id: 'ph', pluginCount: 100 };
    const presetLow = { id: 'pl', pluginCount: 50 };
    const userHigh = { id: 'uh', pluginCount: 9999 };
    const userLow = { id: 'ul', pluginCount: 10 };
    const result = sortMarketplaceSources(
      [userLow, presetHigh, userHigh, presetLow],
      new Set(['ph', 'pl']),
    );
    expect(result.map(s => s.id)).toEqual(['ph', 'pl', 'uh', 'ul']);
  });
});
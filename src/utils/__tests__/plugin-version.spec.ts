import { describe, it, expect, vi, beforeEach } from 'vitest';
import { displayVersion, resolvePluginVersion } from '../plugin-version';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('displayVersion', () => {
  it('returns "v unknown" for undefined', () => {
    expect(displayVersion(undefined)).toBe('v unknown');
  });

  it('returns "v unknown" for null', () => {
    expect(displayVersion(null)).toBe('v unknown');
  });

  it('returns "v unknown" for empty string', () => {
    expect(displayVersion('')).toBe('v unknown');
  });

  it('returns "v unknown" for "unknown"', () => {
    expect(displayVersion('unknown')).toBe('v unknown');
  });

  it('prepends v to version string', () => {
    expect(displayVersion('1.0.0')).toBe('v1.0.0');
  });

  it('handles version string with v prefix', () => {
    // The function literally does v${version}, behavior matches implementation
    expect(displayVersion('v1.0.0')).toBe('vv1.0.0');
  });
});

describe('resolvePluginVersion', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns version from backend', async () => {
    vi.mocked(invoke).mockResolvedValue('2.0.0');
    const result = await resolvePluginVersion('/some/path');
    expect(result).toBe('2.0.0');
    expect(invoke).toHaveBeenCalledWith('resolve_plugin_version', { installedPath: '/some/path' });
  });

  it('returns "unknown" when invoke throws', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('file not found'));
    const result = await resolvePluginVersion('/bad/path');
    expect(result).toBe('unknown');
  });

  it('returns empty string when backend returns empty string', async () => {
    vi.mocked(invoke).mockResolvedValue('');
    const result = await resolvePluginVersion('/some/path');
    // The function returns the invoke result directly without validation
    expect(result).toBe('');
  });
});

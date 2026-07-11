import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useTitlebar } from '../useTitlebar';

// Mock must be at top level for vi.mock to work
vi.mock('@tauri-apps/plugin-os', () => ({
  platform: vi.fn().mockResolvedValue('macos'),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
  })),
}));

describe('useTitlebar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns functions for minimize, toggleMaximize, close', async () => {
    const result = useTitlebar();
    expect(typeof result.minimize).toBe('function');
    expect(typeof result.toggleMaximize).toBe('function');
    expect(typeof result.close).toBe('function');
  });

  it('platform is reactive and defaults to macos', () => {
    const { platform } = useTitlebar();
    expect(platform.value).toBe('macos');
  });

  it('minimize does not throw in test environment', async () => {
    const { minimize } = useTitlebar();
    await expect(minimize()).resolves.not.toThrow();
  });

  it('toggleMaximize does not throw in test environment', async () => {
    const { toggleMaximize } = useTitlebar();
    await expect(toggleMaximize()).resolves.not.toThrow();
  });

  it('close does not throw in test environment', async () => {
    const { close } = useTitlebar();
    await expect(close()).resolves.not.toThrow();
  });
});

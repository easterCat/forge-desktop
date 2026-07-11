import { describe, it, expect, vi, beforeEach } from 'vitest';
import { confirm } from '../dialog';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  confirm: vi.fn(),
}));

vi.stubGlobal('window', {
  ...window,
  __TAURI_INTERNALS__: false,
  confirm: vi.fn().mockReturnValue(false),
});

describe('confirm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(window.confirm).mockReturnValue(false);
  });

  it('falls back to window.confirm when not in Tauri', async () => {
    const result = await confirm('Delete this?');
    expect(window.confirm).toHaveBeenCalledWith('Delete this?');
    expect(result).toBe(false);
  });

  it('window.confirm returns true when user confirms', async () => {
    vi.mocked(window.confirm).mockReturnValue(true);
    const result = await confirm('Proceed?');
    expect(result).toBe(true);
  });
});

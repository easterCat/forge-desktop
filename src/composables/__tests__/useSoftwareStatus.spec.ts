import { describe, it, expect } from 'vitest';
import { computeStatus, formatVersion, STATUS_LABELS, STATUS_FILTERS } from '../useSoftwareStatus';

describe('computeStatus', () => {
  it('returns backend status when present', () => {
    const sw = { isInstalled: false, version: '1.0', status: 'outdated' as const };
    expect(computeStatus(sw)).toBe('outdated');
  });

  it('returns notinstalled when isInstalled is false', () => {
    const sw = { isInstalled: false, version: null, status: undefined };
    expect(computeStatus(sw)).toBe('notinstalled');
  });

  it('returns unknown when installed but no version', () => {
    const sw = { isInstalled: true, version: '', status: undefined };
    expect(computeStatus(sw)).toBe('unknown');
  });

  it('returns installed when installed with version', () => {
    const sw = { isInstalled: true, version: '2.0', status: undefined };
    expect(computeStatus(sw)).toBe('installed');
  });
});

describe('formatVersion', () => {
  it('returns N/A for null', () => {
    expect(formatVersion(null)).toBe('N/A');
  });

  it('returns N/A for undefined', () => {
    expect(formatVersion(undefined)).toBe('N/A');
  });

  it('returns N/A for empty string', () => {
    expect(formatVersion('')).toBe('N/A');
  });

  it('returns N/A for whitespace-only string', () => {
    expect(formatVersion('   ')).toBe('N/A');
  });

  it('prepends v to version string', () => {
    expect(formatVersion('1.0.0')).toBe('v1.0.0');
  });

  it('trims whitespace', () => {
    expect(formatVersion('  1.0  ')).toBe('v1.0');
  });
});

describe('STATUS_LABELS', () => {
  it('contains all status types', () => {
    expect(STATUS_LABELS.installed).toBeTruthy();
    expect(STATUS_LABELS.notinstalled).toBeTruthy();
    expect(STATUS_LABELS.outdated).toBeTruthy();
    expect(STATUS_LABELS.unknown).toBeTruthy();
  });
});

describe('STATUS_FILTERS', () => {
  it('contains all option including all', () => {
    expect(STATUS_FILTERS[0]).toMatchObject({ value: 'all', label: 'All' });
  });

  it('contains all status types', () => {
    const values = STATUS_FILTERS.map(f => f.value);
    expect(values).toContain('installed');
    expect(values).toContain('notinstalled');
    expect(values).toContain('outdated');
    expect(values).toContain('unknown');
  });
});

import type { Software, SoftwareStatus } from '@/types/software';

/**
 * Derive a display status for a software entry.
 *
 * Trust the backend's `status` field when present (Rust computes it during
 * scan). Otherwise fall back to a local derivation so legacy/mock data
 * (e.g. the inline defaults in `SoftwareManagementView`) still get a status.
 */
export function computeStatus(software: Pick<Software, 'isInstalled' | 'version' | 'status'>): SoftwareStatus {
  if (software.status) return software.status;
  if (!software.isInstalled) return 'notinstalled';
  if (!software.version || software.version.length === 0) return 'unknown';
  return 'installed';
}

export const STATUS_LABELS: Record<SoftwareStatus, string> = {
  installed: 'Installed',
  notinstalled: 'Not Installed',
  outdated: 'Update Available',
  unknown: 'Unknown',
};

export const STATUS_FILTERS: Array<{ value: 'all' | SoftwareStatus; label: string }> = [
  { value: 'all', label: 'All' },
  { value: 'installed', label: 'Installed' },
  { value: 'notinstalled', label: 'Not Installed' },
  { value: 'outdated', label: 'Update Available' },
  { value: 'unknown', label: 'Unknown' },
];

/** Format a version string for display. Returns "N/A" for empty/null. */
export function formatVersion(version: string | null | undefined): string {
  if (!version) return 'N/A';
  const trimmed = version.trim();
  if (!trimmed) return 'N/A';
  return trimmed.startsWith('v') ? trimmed : `v${trimmed}`;
}

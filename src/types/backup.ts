export interface BackupRecord {
  id: string;
  name: string;
  path: string;
  size: number | null;
  fileCount: number | null;
  createdAt: string;
  includes: string | null;
}

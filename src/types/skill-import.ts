// Skill Import types
export interface SkillImportResult {
  success: boolean;
  skill_name: string;
  skill_path: string;
  message: string;
}

export interface LocalSkill {
  name: string;
  path: string;
  type: 'directory' | 'file';
  has_skill_md: boolean;
  description?: string;
}

export interface ImportResult {
  success: boolean;
  skill_name: string;
  method: 'copy' | 'symlink';
  target_path: string;
  message: string;
}

export type ImportMethod = 'copy' | 'symlink';

// Skill Repository types
export interface SkillRepository {
  id: string;
  name: string;
  url: string;
  type: 'github' | 'gitlab' | 'gitee' | 'custom';
  last_sync_at?: string;
  status: 'pending' | 'syncing' | 'synced' | 'error';
  error_message?: string;
  skill_count: number;
  added_at: string;
  skills: DiscoveredSkill[];
}

export interface DiscoveredSkill {
  name: string;
  path: string;
  description?: string;
  version?: string;
  repository_id: string;
  repository_name: string;
}

export interface RepositoryValidation {
  valid: boolean;
  name?: string;
  branch_count: number;
  error_message?: string;
}

export interface SyncResult {
  success: boolean;
  skill_count: number;
  message: string;
}

export interface DownloadSkillResult {
  success: boolean;
  skill_name: string;
  skill_path: string;
  message: string;
}

// Anthropic Skills Type Definitions

export interface RemoteSkillSource {
  id: string;
  label: string;
  owner: string;
  repo: string;
  branch: string;
  subdir_prefix: string;
  cache_file: string;
}

export interface AnthropicSkill {
  id: string;
  name: string;
  description: string;
  version: string | null;
  author: string | null;
  tags: string[];
  dependencies: string[];
  repository: string;
  subdirectory: string;
  default_branch: string;
  file_count: number;
  size_bytes: number;
  installed: boolean;
  installed_path: string | null;
  installed_at: string | null;
  source_id?: string;
}

export interface InstallProgress {
  skill_id: string;
  stage: 'listing' | 'downloading' | 'verifying' | 'complete' | 'error';
  progress: number;
  message: string;
  files_downloaded: number;
  files_total: number;
}

export interface InstallVerification {
  success: boolean;
  files_downloaded: number;
  total_size: number;
  skill_md_present: boolean;
  sha256_verified: boolean;
  errors: string[];
}

export type SkillFilter = 'all' | 'installed' | 'not_installed';
export type SkillSort = 'name' | 'version' | 'install_time';

/**
 * Returned by the `list_remote_skills_cached_only` Rust command. Mirrors
 * `CachedSkillsResult` on the Rust side.
 */
export interface CachedSkillsResult {
  skills: AnthropicSkill[];
  cache_exists: boolean;
  is_stale: boolean;
  cached_at: number | null;
}

/**
 * Coarse status the UI uses to render the cache-freshness hint in the
 * header.
 *
 * - `loading`  : we're inside an `loadCachedOnly` or `fetchList` call
 * - `fresh`    : cache hit, younger than TTL
 * - `stale`    : cache hit, older than TTL — user may want to refresh
 * - `missing`  : no cache file on disk yet
 * - `error`    : last call failed
 */
export type CacheStatus = 'loading' | 'fresh' | 'stale' | 'missing' | 'error';

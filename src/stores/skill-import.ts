// Skill Import & Repository Store
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  SkillImportResult,
  LocalSkill,
  ImportResult,
  ImportMethod,
  SkillRepository,
  DiscoveredSkill,
  RepositoryValidation,
  SyncResult,
  DownloadSkillResult,
} from '@/types';

export const useSkillImportStore = defineStore('skillImport', () => {
  // State - Local Import
  const localSkills = ref<LocalSkill[]>([]);
  const detectedPaths = ref<string[]>([]);
  const selectedLocalPath = ref<string>('');
  const isScanning = ref(false);

  // State - Repository
  const repositories = ref<SkillRepository[]>([]);
  const discoveredSkills = ref<DiscoveredSkill[]>([]);
  const selectedRepositoryId = ref<string | null>(null);
  const isLoadingRepositories = ref(false);
  const isSyncing = ref(false);
  const isValidating = ref(false);
  const validationResult = ref<RepositoryValidation | null>(null);

  // State - Install Progress
  const isInstalling = ref(false);
  const installProgress = ref<Map<string, { progress: number; status: string }>>(new Map());

  // Error state
  const error = ref<string | null>(null);

  // Computed
  const selectedRepository = computed(() =>
    repositories.value.find(r => r.id === selectedRepositoryId.value) || null
  );

  const filteredSkills = computed(() => {
    if (!selectedRepositoryId.value) {
      // Return all skills from all repositories
      return discoveredSkills.value;
    }
    return discoveredSkills.value.filter(
      s => s.repository_id === selectedRepositoryId.value
    );
  });

  const repositorySkillCount = computed(() => {
    const counts: Record<string, number> = {};
    for (const repo of repositories.value) {
      counts[repo.id] = repo.skills.length;
    }
    return counts;
  });

  // Actions - Local Skills
  const defaultSkillsDir = ref('');

  async function initDefaultSkillsDir(): Promise<string> {
    try {
      const dir = await invoke<string>('get_default_skills_dir');
      defaultSkillsDir.value = dir;
      return dir;
    } catch (e) {
      console.error('Failed to get default skills dir:', e);
      return '';
    }
  }

  async function detectCliSkillsPaths(): Promise<string[]> {
    try {
      const paths = await invoke<string[]>('detect_cli_skills_paths');
      detectedPaths.value = paths;
      if (paths.length > 0 && !selectedLocalPath.value) {
        selectedLocalPath.value = paths[0];
      }
      return paths;
    } catch (e) {
      console.error('Failed to detect CLI skills paths:', e);
      return [];
    }
  }

  async function scanLocalSkills(basePath: string): Promise<LocalSkill[]> {
    try {
      isScanning.value = true;
      error.value = null;
      const skills = await invoke<LocalSkill[]>('scan_local_skills', { basePath });
      localSkills.value = skills;
      return skills;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '扫描失败';
      return [];
    } finally {
      isScanning.value = false;
    }
  }

  async function importLocalSkill(
    sourcePath: string,
    targetDir: string,
    method: ImportMethod
  ): Promise<ImportResult> {
    try {
      isInstalling.value = true;
      error.value = null;
      const result = await invoke<ImportResult>('import_local_skill', {
        sourcePath,
        targetDir,
        importMethod: method,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '导入失败';
      throw e;
    } finally {
      isInstalling.value = false;
    }
  }

  async function importMultipleLocalSkills(
    sources: string[],
    targetDir: string,
    method: ImportMethod,
    onProgress?: (current: number, total: number) => void
  ): Promise<ImportResult[]> {
    const results: ImportResult[] = [];
    for (let i = 0; i < sources.length; i++) {
      const result = await importLocalSkill(sources[i], targetDir, method);
      results.push(result);
      onProgress?.(i + 1, sources.length);
    }
    return results;
  }

  // Actions - ZIP Install
  async function installFromZip(zipPath: string, targetDir: string): Promise<SkillImportResult> {
    try {
      isInstalling.value = true;
      error.value = null;
      const result = await invoke<SkillImportResult>('unzip_skill_package', {
        zipPath,
        targetDir,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '安装失败';
      throw e;
    } finally {
      isInstalling.value = false;
    }
  }

  // Actions - Repository
  async function fetchRepositories(): Promise<SkillRepository[]> {
    try {
      isLoadingRepositories.value = true;
      error.value = null;
      const repos = await invoke<SkillRepository[]>('get_repositories');
      repositories.value = repos;
      return repos;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取仓库列表失败';
      return [];
    } finally {
      isLoadingRepositories.value = false;
    }
  }

  async function addRepository(url: string, name?: string): Promise<SkillRepository> {
    try {
      error.value = null;
      const repo = await invoke<SkillRepository>('add_repository', { url, name });
      repositories.value.push(repo);
      return repo;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '添加仓库失败';
      throw e;
    }
  }

  async function removeRepository(repoId: string): Promise<void> {
    try {
      error.value = null;
      await invoke('remove_repository', { repoId });
      repositories.value = repositories.value.filter(r => r.id !== repoId);
      if (selectedRepositoryId.value === repoId) {
        selectedRepositoryId.value = null;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除仓库失败';
      throw e;
    }
  }

  async function validateRepository(url: string): Promise<RepositoryValidation> {
    try {
      isValidating.value = true;
      error.value = null;
      const result = await invoke<RepositoryValidation>('validate_repository', { url });
      validationResult.value = result;
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '验证失败';
      throw e;
    } finally {
      isValidating.value = false;
    }
  }

  async function syncRepository(repoId: string): Promise<SyncResult> {
    try {
      isSyncing.value = true;
      error.value = null;
      
      // Update local state
      const repo = repositories.value.find(r => r.id === repoId);
      if (repo) {
        repo.status = 'syncing';
      }
      
      const result = await invoke<SyncResult>('sync_repository', { repoId });
      
      // Refresh repositories
      await fetchRepositories();
      
      // Update discovered skills
      updateDiscoveredSkills();
      
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '同步失败';
      
      // Update status to error
      const repo = repositories.value.find(r => r.id === repoId);
      if (repo) {
        repo.status = 'error';
        repo.error_message = e instanceof Error ? e.message : '同步失败';
      }
      
      throw e;
    } finally {
      isSyncing.value = false;
    }
  }

  async function syncAllRepositories(): Promise<SyncResult[]> {
    try {
      isSyncing.value = true;
      error.value = null;
      
      const results = await invoke<SyncResult[]>('sync_all_repositories');
      await fetchRepositories();
      updateDiscoveredSkills();
      
      return results;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '同步失败';
      return [];
    } finally {
      isSyncing.value = false;
    }
  }

  function updateDiscoveredSkills() {
    const allSkills: DiscoveredSkill[] = [];
    for (const repo of repositories.value) {
      for (const skill of repo.skills) {
        allSkills.push({
          ...skill,
          repository_id: repo.id,
          repository_name: repo.name,
        });
      }
    }
    discoveredSkills.value = allSkills;
  }

  async function installDiscoveredSkill(
    skill: DiscoveredSkill,
    targetDir: string
  ): Promise<{ success: boolean; message: string }> {
    try {
      isInstalling.value = true;
      
      // Update progress
      const progressMap = new Map(installProgress.value);
      progressMap.set(skill.name, { progress: 0, status: 'installing' });
      installProgress.value = progressMap;
      
      // Call Rust backend to download skill from repository
      const result = await invoke<DownloadSkillResult>('download_skill_from_repository', {
        repoId: skill.repository_id,
        skillPath: skill.path,
        targetDir: targetDir,
      });
      
      // Clear progress
      const newMap = new Map(installProgress.value);
      newMap.delete(skill.name);
      installProgress.value = newMap;
      
      if (result.success) {
        return { success: true, message: result.message };
      } else {
        return { success: false, message: result.message };
      }
    } catch (e) {
      const newMap = new Map(installProgress.value);
      newMap.delete(skill.name);
      installProgress.value = newMap;
      
      return {
        success: false,
        message: e instanceof Error ? e.message : '安装失败',
      };
    } finally {
      isInstalling.value = false;
    }
  }

  async function installMultipleDiscoveredSkills(
    skills: DiscoveredSkill[],
    targetDir: string,
    onProgress?: (current: number, total: number) => void
  ): Promise<{ success: number; failed: number; errors: string[] }> {
    let success = 0;
    let failed = 0;
    const errors: string[] = [];
    
    for (let i = 0; i < skills.length; i++) {
      const result = await installDiscoveredSkill(skills[i], targetDir);
      if (result.success) {
        success++;
      } else {
        failed++;
        errors.push(`${skills[i].name}: ${result.message}`);
      }
      onProgress?.(i + 1, skills.length);
    }
    
    return { success, failed, errors };
  }

  function selectRepository(repoId: string | null) {
    selectedRepositoryId.value = repoId;
  }

  function clearError() {
    error.value = null;
  }

  function reset() {
    localSkills.value = [];
    detectedPaths.value = [];
    selectedLocalPath.value = '';
    repositories.value = [];
    discoveredSkills.value = [];
    selectedRepositoryId.value = null;
    error.value = null;
  }

  return {
    // State - Local Import
    localSkills,
    detectedPaths,
    selectedLocalPath,
    isScanning,
    defaultSkillsDir,

    // State - Repository
    repositories,
    discoveredSkills,
    selectedRepositoryId,
    isLoadingRepositories,
    isSyncing,
    isValidating,
    validationResult,

    // State - Install
    isInstalling,
    installProgress,

    // Error
    error,

    // Computed
    selectedRepository,
    filteredSkills,
    repositorySkillCount,

    // Actions - Local Skills
    detectCliSkillsPaths,
    scanLocalSkills,
    importLocalSkill,
    importMultipleLocalSkills,
    initDefaultSkillsDir,

    // Actions - ZIP Install
    installFromZip,

    // Actions - Repository
    fetchRepositories,
    addRepository,
    removeRepository,
    validateRepository,
    syncRepository,
    syncAllRepositories,
    selectRepository,

    // Actions - Install
    installDiscoveredSkill,
    installMultipleDiscoveredSkills,

    // Utils
    clearError,
    reset,
  };
});

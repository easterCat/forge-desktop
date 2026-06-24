// Skill Marketplace Store - State management for skill marketplace

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  SkillSource,
  MarketplaceSkill,
  PaginatedSkills,
  SyncTarget,
  SyncConfig,
  MarketplaceInstallProgress,
  CategoryKey,
} from '@/types';

export const useSkillMarketplaceStore = defineStore('skillMarketplace', () => {
  // State
  const sources = ref<SkillSource[]>([]);
  const currentSource = ref<SkillSource | null>(null);
  const skills = ref<MarketplaceSkill[]>([]);
  const localSkills = ref<MarketplaceSkill[]>([]);
  const syncTargets = ref<SyncTarget[]>([]);

  // Pagination state
  const currentPage = ref(1);
  const pageSize = ref(20);
  const totalSkills = ref(0);
  const totalPages = ref(0);

  // Filter state
  const selectedCategory = ref<string | null>(null);
  const searchKeyword = ref('');

  // Loading states
  const isLoadingSources = ref(false);
  const isLoadingSkills = ref(false);
  const isInstalling = ref(false);
  const isSyncing = ref(false);

  // Error state
  const error = ref<string | null>(null);

  // Install progress tracking
  const installProgress = ref<Map<string, MarketplaceInstallProgress>>(new Map<string, MarketplaceInstallProgress>());

  // Computed
  const hasNextPage = computed(() => currentPage.value < totalPages.value);
  const hasPrevPage = computed(() => currentPage.value > 1);

  // Installed skill names lookup - computed for automatic updates
  const installedSkillNames = computed(() => {
    const set = new Set<string>();
    for (const skill of localSkills.value) {
      set.add(skill.name);
    }
    return set;
  });

  const isSkillInstalled = (skillName: string): boolean => {
    return installedSkillNames.value.has(skillName);
  };
  
  // Actions
  async function fetchSources() {
    try {
      isLoadingSources.value = true;
      error.value = null;
      sources.value = await invoke<SkillSource[]>('get_skill_sources');
      
      // Select first source by default
      if (sources.value.length > 0 && !currentSource.value) {
        currentSource.value = sources.value[0];
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch sources';
      console.error('Failed to fetch sources:', e);
    } finally {
      isLoadingSources.value = false;
    }
  }
  
  async function fetchSkills() {
    if (!currentSource.value) return;
    
    try {
      isLoadingSkills.value = true;
      error.value = null;
      
      const result = await invoke<PaginatedSkills>('fetch_marketplace_skills', {
        sourceId: currentSource.value.id,
        page: currentPage.value,
        pageSize: pageSize.value,
        category: selectedCategory.value,
        keyword: searchKeyword.value || null,
      });
      
      skills.value = result.items;
      totalSkills.value = result.total;
      totalPages.value = result.totalPages;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch skills';
      console.error('Failed to fetch skills:', e);
    } finally {
      isLoadingSkills.value = false;
    }
  }
  
  async function fetchLocalSkills(projectPath: string) {
    try {
      localSkills.value = await invoke<MarketplaceSkill[]>('get_local_marketplace_skills', {
        localPath: projectPath,
      });
    } catch (e) {
      console.error('Failed to fetch local skills:', e);
    }
  }
  
  function selectSource(source: SkillSource) {
    currentSource.value = source;
    currentPage.value = 1;
    fetchSkills();
  }
  
  function setPage(page: number) {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page;
      fetchSkills();
    }
  }
  
  function setPageSize(size: number) {
    pageSize.value = size;
    currentPage.value = 1;
    fetchSkills();
  }
  
  function setCategory(category: string | null) {
    selectedCategory.value = category;
    currentPage.value = 1;
    fetchSkills();
  }
  
  function setSearchKeyword(keyword: string) {
    searchKeyword.value = keyword;
    currentPage.value = 1;
    fetchSkills();
  }
  
  async function installSkill(skill: MarketplaceSkill, projectPath: string) {
    // Create new Map for shallowRef to trigger reactivity
    const progressMap = new Map(installProgress.value);
    const progress: MarketplaceInstallProgress = {
      skillId: skill.id,
      skillName: skill.name,
      stage: 'downloading',
      progress: 0,
      message: '正在安装技能...',
      startedAt: new Date().toISOString(),
    };
    progressMap.set(skill.id, progress);
    installProgress.value = progressMap;
    isInstalling.value = true;
    
    try {
      // Update progress
      const newMap1 = new Map(installProgress.value);
      progress.progress = 30;
      progress.message = '正在下载技能文件...';
      newMap1.set(skill.id, { ...progress });
      installProgress.value = newMap1;
      
      const installPath = await invoke<string>('install_marketplace_skill', {
        skill,
        localPath: projectPath,
      });
      
      // Update progress
      const newMap2 = new Map(installProgress.value);
      progress.progress = 100;
      progress.stage = 'success';
      progress.message = `技能已安装到 ${installPath}`;
      progress.completedAt = new Date().toISOString();
      newMap2.set(skill.id, { ...progress });
      installProgress.value = newMap2;
      
      // Refresh local skills
      await fetchLocalSkills(projectPath);
      
      return { success: true, path: installPath };
    } catch (e) {
      const newMap = new Map(installProgress.value);
      progress.stage = 'failed';
      progress.error = e instanceof Error ? e.message : String(e);
      progress.message = `安装失败: ${progress.error}`;
      newMap.set(skill.id, { ...progress });
      installProgress.value = newMap;
      
      return { success: false, error: e };
    } finally {
      isInstalling.value = false;
      
      // Clear progress after delay
      setTimeout(() => {
        const newMap = new Map(installProgress.value);
        newMap.delete(skill.id);
        installProgress.value = newMap;
      }, 5000);
    }
  }
  
  async function syncSkillToTarget(
    skillName: string,
    projectPath: string,
    target: SyncTarget
  ) {
    isSyncing.value = true;
    
    try {
      const result = await invoke<{ success: boolean; error?: string }>('sync_skill_to_target', {
        skillName,
        localPath: projectPath,
        target,
      });
      
      return result;
    } catch (e) {
      return { success: false, error: e instanceof Error ? e.message : String(e) };
    } finally {
      isSyncing.value = false;
    }
  }
  
  async function fetchSyncTargets() {
    try {
      syncTargets.value = await invoke<SyncTarget[]>('get_sync_targets');
    } catch (e) {
      console.error('Failed to fetch sync targets:', e);
    }
  }
  
  async function addSyncTarget(target: SyncTarget) {
    try {
      const newTarget = await invoke<SyncTarget>('add_sync_target', { target });
      syncTargets.value.push(newTarget);
      return newTarget;
    } catch (e) {
      throw e;
    }
  }
  
  async function removeSyncTarget(targetId: string) {
    try {
      await invoke('remove_sync_target', { targetId });
      syncTargets.value = syncTargets.value.filter(t => t.id !== targetId);
    } catch (e) {
      throw e;
    }
  }
  
  function getInstallProgress(skillId: string): MarketplaceInstallProgress | undefined {
    return installProgress.value.get(skillId);
  }
  
  function clearError() {
    error.value = null;
  }
  
  return {
    // State
    sources,
    currentSource,
    skills,
    localSkills,
    syncTargets,
    currentPage,
    pageSize,
    totalSkills,
    totalPages,
    selectedCategory,
    searchKeyword,
    isLoadingSources,
    isLoadingSkills,
    isInstalling,
    isSyncing,
    error,
    installProgress,
    
    // Computed
    hasNextPage,
    hasPrevPage,
    installedSkillNames,
    
    // Actions
    fetchSources,
    fetchSkills,
    fetchLocalSkills,
    selectSource,
    setPage,
    setPageSize,
    setCategory,
    setSearchKeyword,
    installSkill,
    syncSkillToTarget,
    fetchSyncTargets,
    addSyncTarget,
    removeSyncTarget,
    getInstallProgress,
    clearError,
    isSkillInstalled,
  };
});

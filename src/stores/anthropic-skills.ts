// Anthropic Skills Store - State management for anthropics/skills and other skill sources

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { message } from '@tauri-apps/plugin-dialog';
import type {
  AnthropicSkill,
  CachedSkillsResult,
  CacheStatus,
  InstallProgress,
  InstallVerification,
  SkillFilter,
  SkillSort,
  RemoteSkillSource,
} from '@/types';

export const useAnthropicSkillsStore = defineStore('anthropicSkills', () => {
  // State
  const skills = ref<AnthropicSkill[]>([]);
  const localSkills = ref<AnthropicSkill[]>([]);
  const installingMap = ref<Map<string, InstallProgress>>(new Map());
  const isLoading = ref(false);
  const isRefreshing = ref(false);
  const error = ref<string | null>(null);
  const searchKeyword = ref('');
  const filter = ref<SkillFilter>('all');
  const sortBy = ref<SkillSort>('name');

  // Source state
  const sources = ref<RemoteSkillSource[]>([]);
  const activeSourceId = ref<string>('anthropic-official');
  const activeSource = ref<RemoteSkillSource | null>(null);

  // Cache state. `cacheStatus` reflects the *on-disk* cache for the
  // currently active source; it does not change when the view is
  // mounted/unmounted, so the UI can render a stable "fresh / stale /
  // missing" hint. `cachedAt` is the unix-seconds timestamp the cache was
  // last written.
  const cacheStatus = ref<CacheStatus>('loading');
  const cachedAt = ref<number | null>(null);

  // Event listeners
  let anthropicProgressUnlisten: UnlistenFn | null = null;
  let remoteProgressUnlisten: UnlistenFn | null = null;

  // Computed
  const installedSkillIds = computed(() => {
    const ids = new Set<string>();
    for (const skill of localSkills.value) {
      ids.add(skill.id);
    }
    return ids;
  });

  const isSkillInstalled = (skillId: string): boolean => {
    return installedSkillIds.value.has(skillId);
  };

  const isInstalling = (skillId: string): boolean => {
    return installingMap.value.has(skillId);
  };

  const getInstallProgress = (skillId: string): InstallProgress | undefined => {
    return installingMap.value.get(skillId);
  };

  const filteredSkills = computed(() => {
    let result = skills.value.map(skill => ({
      ...skill,
      installed: installedSkillIds.value.has(skill.id),
    }));

    // Apply installed status from local skills
    for (const localSkill of localSkills.value) {
      const index = result.findIndex(s => s.id === localSkill.id);
      if (index !== -1) {
        result[index] = {
          ...result[index],
          installed: true,
          installed_path: localSkill.installed_path,
          installed_at: localSkill.installed_at,
        };
      }
    }

    // Filter by search keyword
    if (searchKeyword.value) {
      const keyword = searchKeyword.value.toLowerCase();
      result = result.filter(skill =>
        skill.name.toLowerCase().includes(keyword) ||
        skill.description.toLowerCase().includes(keyword) ||
        skill.id.toLowerCase().includes(keyword)
      );
    }

    // Filter by status
    if (filter.value === 'installed') {
      result = result.filter(skill => skill.installed);
    } else if (filter.value === 'not_installed') {
      result = result.filter(skill => !skill.installed);
    }

    // Sort
    result.sort((a, b) => {
      switch (sortBy.value) {
        case 'name':
          return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
        case 'version': {
          const versionA = a.version || '0.0.0';
          const versionB = b.version || '0.0.0';
          return versionB.localeCompare(versionA);
        }
        case 'install_time': {
          const timeA = a.installed_at || '';
          const timeB = b.installed_at || '';
          return timeB.localeCompare(timeA);
        }
        default:
          return 0;
      }
    });

    return result;
  });

  const availableSkills = computed(() => {
    return filteredSkills.value.filter(skill => !skill.installed);
  });

  const installedSkills = computed(() => {
    return filteredSkills.value.filter(skill => skill.installed);
  });

  const stats = computed(() => ({
    total: skills.value.length,
    installed: installedSkills.value.length,
    available: availableSkills.value.length,
  }));

  // Actions
  async function fetchSources(): Promise<void> {
    try {
      const result = await invoke<RemoteSkillSource[]>('get_remote_skill_sources');
      sources.value = result;

      // If we don't have an active source yet, pick the one matching
      // activeSourceId (which the view may have pre-set) or fall back to the
      // first source.
      if (!activeSource.value && result.length > 0) {
        const desired = result.find(s => s.id === activeSourceId.value) || result[0];
        activeSource.value = desired;
        activeSourceId.value = desired.id;
      }
    } catch (e) {
      console.error('Failed to fetch skill sources:', e);
    }
  }

  /**
   * Set the active source id WITHOUT clearing or refetching skills. The caller
   * is responsible for calling `fetchList()` afterwards.
   *
   * Used by views that already know which source they want (e.g. the
   * Marketplace source-tab picks Anthropic or Composio).
   */
  function setActiveSourceId(sourceId: string): void {
    const source = sources.value.find(s => s.id === sourceId) || null;
    activeSource.value = source;
    activeSourceId.value = sourceId;
  }

  async function switchSource(sourceId: string): Promise<void> {
    const source = sources.value.find(s => s.id === sourceId);
    if (!source) {
      console.error('Source not found:', sourceId);
      return;
    }

    activeSource.value = source;
    activeSourceId.value = sourceId;

    // Reset cache metadata — the new source has its own cache file. The
    // view will follow up with `loadCachedOnly()` (or `loadData` which
    // itself calls `loadCachedOnly`).
    cacheStatus.value = 'loading';
    cachedAt.value = null;

    // Clear skills and reload from cache (no network)
    skills.value = [];
    await loadCachedOnly();
  }

  /**
   * Read the cached skills for the active source **without** issuing a
   * network request. This is what `AnthropicSkillsView.onMounted` calls, so
   * re-entering the page is instant. The only way to actually hit the
   * network is the explicit Refresh button → `refreshList()`.
   *
   * On success the store's `skills` array is populated from the cache and
   * `cacheStatus` is set to `fresh` / `stale` / `missing` accordingly.
   */
  async function loadCachedOnly(): Promise<void> {
    if (!activeSource.value) {
      cacheStatus.value = 'missing';
      return;
    }

    try {
      isLoading.value = true;
      error.value = null;
      cacheStatus.value = 'loading';

      const result = await invoke<CachedSkillsResult>('list_remote_skills_cached_only', {
        source: activeSource.value,
      });

      skills.value = result.skills;
      cachedAt.value = result.cached_at;

      if (!result.cache_exists) {
        cacheStatus.value = 'missing';
      } else if (result.is_stale) {
        cacheStatus.value = 'stale';
      } else {
        cacheStatus.value = 'fresh';
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      cacheStatus.value = 'error';
      console.error('Failed to read cached remote skills:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchList(refresh = false): Promise<void> {
    if (!activeSource.value) {
      error.value = 'No skill source selected';
      return;
    }

    try {
      isLoading.value = true;
      error.value = null;

      const result = await invoke<AnthropicSkill[]>('list_remote_skills', {
        source: activeSource.value,
        refresh,
      });

      skills.value = result;
      // Any successful call to the network-backed command produces a
      // fresh cache file on disk.
      cacheStatus.value = 'fresh';
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to fetch remote skills:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function refreshList(): Promise<void> {
    if (!activeSource.value) {
      return;
    }

    try {
      isRefreshing.value = true;
      error.value = null;

      const result = await invoke<AnthropicSkill[]>('list_remote_skills', {
        source: activeSource.value,
        refresh: true,
      });

      skills.value = result;
      cacheStatus.value = 'fresh';
      cachedAt.value = Math.floor(Date.now() / 1000);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to refresh remote skills:', e);
    } finally {
      isRefreshing.value = false;
    }
  }

  async function fetchLocalSkills(targetDir: string): Promise<void> {
    if (!activeSource.value) {
      return;
    }

    try {
      const result = await invoke<AnthropicSkill[]>('get_local_remote_skills', {
        source: activeSource.value,
        targetDir,
      });

      localSkills.value = result;

      // Update installed status in main skills list
      const installedIds = new Set(result.map(s => s.id));
      skills.value = skills.value.map(skill => ({
        ...skill,
        installed: installedIds.has(skill.id),
      }));
    } catch (e) {
      console.error('Failed to fetch local remote skills:', e);
    }
  }

  async function install(
    skillId: string,
    targetDir: string
  ): Promise<{ success: boolean; error?: string }> {
    if (!activeSource.value) {
      return { success: false, error: 'No active source' };
    }

    try {
      // Check if already installed
      if (isSkillInstalled(skillId)) {
        const skill = skills.value.find(s => s.id === skillId);
        const skillName = skill?.name || skillId;

        await message(
          `技能 "${skillName}" 已安装。\n\n如需重新安装，请先卸载当前版本。`,
          { title: '技能已安装', kind: 'warning' }
        );
        return { success: false, error: 'Skill already installed' };
      }

      // Ensure progress listener is active BEFORE invoking (avoids missing early events)
      await ensureProgressListeners();

      // Set initial progress
      const progress: InstallProgress = {
        skill_id: skillId,
        stage: 'listing',
        progress: 0,
        message: '正在初始化...',
        files_downloaded: 0,
        files_total: 0,
      };

      const newMap = new Map(installingMap.value);
      newMap.set(skillId, progress);
      installingMap.value = newMap;

      const result = await invoke<InstallProgress>('install_remote_skill', {
        source: activeSource.value,
        skillId,
        targetDir,
      });

      // Update progress to complete
      const completeMap = new Map(installingMap.value);
      completeMap.set(skillId, { ...result });
      installingMap.value = completeMap;

      // Refresh local skills
      await fetchLocalSkills(targetDir);

      // Clear progress after delay
      setTimeout(() => {
        const clearedMap = new Map(installingMap.value);
        clearedMap.delete(skillId);
        installingMap.value = clearedMap;
      }, 3000);

      return { success: true };
    } catch (e) {
      // Update progress to error
      const errorMap = new Map(installingMap.value);
      errorMap.set(skillId, {
        skill_id: skillId,
        stage: 'error' as const,
        progress: 0,
        message: e instanceof Error ? e.message : String(e),
        files_downloaded: 0,
        files_total: 0,
      });
      installingMap.value = errorMap;

      // Clear error after delay
      setTimeout(() => {
        const clearedMap = new Map(installingMap.value);
        clearedMap.delete(skillId);
        installingMap.value = clearedMap;
      }, 5000);

      return { success: false, error: e instanceof Error ? e.message : String(e) };
    }
  }

  async function uninstall(
    skillId: string,
    targetDir: string
  ): Promise<{ success: boolean; error?: string }> {
    if (!activeSource.value) {
      return { success: false, error: 'No active source' };
    }

    try {
      await invoke('uninstall_remote_skill', {
        source: activeSource.value,
        skillId,
        targetDir,
      });

      // Update local skills
      localSkills.value = localSkills.value.filter(s => s.id !== skillId);

      // Update installed status
      skills.value = skills.value.map(skill =>
        skill.id === skillId
          ? { ...skill, installed: false, installed_path: null, installed_at: null }
          : skill
      );

      return { success: true };
    } catch (e) {
      return { success: false, error: e instanceof Error ? e.message : String(e) };
    }
  }

  async function verify(
    skillId: string,
    targetDir: string
  ): Promise<InstallVerification | null> {
    if (!activeSource.value) {
      return null;
    }

    try {
      const result = await invoke<InstallVerification>('verify_remote_skill_install', {
        source: activeSource.value,
        skillId,
        targetDir,
      });

      return result;
    } catch (e) {
      console.error('Verification failed:', e);
      return null;
    }
  }

  async function ensureProgressListeners(): Promise<void> {
    // Anthropic official progress listener
    if (!anthropicProgressUnlisten) {
      try {
        anthropicProgressUnlisten = await listen<InstallProgress>(
          'anthropic-skill-install-progress',
          (event) => {
            const newMap = new Map(installingMap.value);
            newMap.set(event.payload.skill_id, event.payload);
            installingMap.value = newMap;
          }
        );
      } catch (e) {
        console.error('Failed to setup anthropic progress listener:', e);
      }
    }

    // Remote skill progress listener (for other sources)
    if (!remoteProgressUnlisten) {
      try {
        remoteProgressUnlisten = await listen<InstallProgress>(
          'remote-skill-install-progress',
          (event) => {
            const newMap = new Map(installingMap.value);
            newMap.set(event.payload.skill_id, event.payload);
            installingMap.value = newMap;
          }
        );
      } catch (e) {
        console.error('Failed to setup remote progress listener:', e);
      }
    }
  }

  async function listenProgress(): Promise<void> {
    await ensureProgressListeners();
  }

  function stopListening(): void {
    if (anthropicProgressUnlisten) {
      anthropicProgressUnlisten();
      anthropicProgressUnlisten = null;
    }
    if (remoteProgressUnlisten) {
      remoteProgressUnlisten();
      remoteProgressUnlisten = null;
    }
  }

  function setSearchKeyword(keyword: string): void {
    searchKeyword.value = keyword;
  }

  function setFilter(newFilter: SkillFilter): void {
    filter.value = newFilter;
  }

  function setSortBy(newSort: SkillSort): void {
    sortBy.value = newSort;
  }

  function clearError(): void {
    error.value = null;
  }

  return {
    // State
    skills,
    localSkills,
    installingMap,
    isLoading,
    isRefreshing,
    error,
    searchKeyword,
    filter,
    sortBy,
    sources,
    activeSourceId,
    activeSource,
    cacheStatus,
    cachedAt,

    // Computed
    installedSkillIds,
    filteredSkills,
    availableSkills,
    installedSkills,
    stats,

    // Actions
    fetchSources,
    setActiveSourceId,
    switchSource,
    fetchList,
    loadCachedOnly,
    refreshList,
    fetchLocalSkills,
    install,
    uninstall,
    verify,
    listenProgress,
    ensureProgressListeners,
    stopListening,
    setSearchKeyword,
    setFilter,
    setSortBy,
    clearError,

    // Helpers
    isSkillInstalled,
    isInstalling,
    getInstallProgress,
  };
});

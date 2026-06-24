// Skills.sh Pinia Store
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  SkillsShView,
  SkillsShSkill,
  SkillsShPage,
  SkillsShCuratedResponse,
  SkillsShSkillDetail,
  SkillsShAuditResponse,
  SkillsShInstallResult,
  SkillsShSourceGroup,
} from '@/types';

export const useSkillsShStore = defineStore('skillsSh', () => {
  // State
  const view = ref<SkillsShView>('all-time');
  const skills = ref<SkillsShSkill[]>([]);
  const pagination = ref({ page: 0, perPage: 100, total: 0, hasMore: false });
  const isLoading = ref(false);
  const isLoadingMore = ref(false);
  const error = ref<string | null>(null);

  // Search state
  const searchKeyword = ref('');
  const searchResults = ref<SkillsShSkill[]>([]);
  const isSearching = ref(false);
  const searchError = ref<string | null>(null);

  // Curated state
  const curated = ref<SkillsShCuratedResponse | null>(null);
  const isCuratedLoading = ref(false);
  const curatedError = ref<string | null>(null);

  // Detail cache
  const detailCache = ref<Map<string, SkillsShSkillDetail>>(new Map());
  const auditCache = ref<Map<string, SkillsShAuditResponse>>(new Map());

  // Install state
  const isInstalling = ref(false);
  const installError = ref<string | null>(null);

  // All sources (for Browse by Source)
  const allSources = ref<SkillsShSourceGroup[]>([]);

  // Computed
  const groupedBySource = computed(() => {
    const groups = new Map<string, SkillsShSourceGroup>();
    for (const skill of skills.value) {
      if (!groups.has(skill.source)) {
        groups.set(skill.source, {
          source: skill.source,
          totalInstalls: 0,
          skillCount: 0,
          skills: [],
        });
      }
      const group = groups.get(skill.source)!;
      group.totalInstalls += skill.installs;
      group.skillCount += 1;
      group.skills.push(skill);
    }
    return Array.from(groups.values()).sort((a, b) => b.totalInstalls - a.totalInstalls);
  });

  const topSkills = computed(() => skills.value.slice(0, 50));

  const hasError = computed(() => error.value !== null);
  const isApi401 = computed(() => error.value?.includes('401') ?? false);

  // Actions
  async function fetchLeaderboard(reset = true): Promise<void> {
    if (reset) {
      pagination.value = { page: 0, perPage: 100, total: 0, hasMore: false };
      skills.value = [];
    }

    try {
      isLoading.value = reset;
      error.value = null;

      const result = await invoke<SkillsShPage>('fetch_skills_sh_leaderboard', {
        view: view.value,
        page: pagination.value.page,
        perPage: pagination.value.perPage,
      });

      if (reset) {
        skills.value = result.data;
      } else {
        skills.value = [...skills.value, ...result.data];
      }
      pagination.value = result.pagination;

      // Update allSources for Browse by Source
      updateAllSources();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to fetch leaderboard:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMore(): Promise<void> {
    if (!pagination.value.hasMore || isLoadingMore.value) return;

    try {
      isLoadingMore.value = true;
      pagination.value.page += 1;

      const result = await invoke<SkillsShPage>('fetch_skills_sh_leaderboard', {
        view: view.value,
        page: pagination.value.page,
        perPage: pagination.value.perPage,
      });

      skills.value = [...skills.value, ...result.data];
      pagination.value = result.pagination;
    } catch (e) {
      pagination.value.page -= 1;
      console.error('Failed to load more:', e);
    } finally {
      isLoadingMore.value = false;
    }
  }

  async function search(query: string): Promise<void> {
    if (query.length < 2) {
      searchResults.value = [];
      return;
    }

    try {
      isSearching.value = true;
      searchError.value = null;

      const result = await invoke<SkillsShSkill[]>('search_skills_sh', {
        query,
        limit: 50,
      });

      searchResults.value = result;
    } catch (e) {
      searchError.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to search:', e);
    } finally {
      isSearching.value = false;
    }
  }

  async function fetchCurated(): Promise<void> {
    if (curated.value !== null) return;

    try {
      isCuratedLoading.value = true;
      curatedError.value = null;

      const result = await invoke<SkillsShCuratedResponse>('fetch_skills_sh_curated');
      curated.value = result;
    } catch (e) {
      curatedError.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to fetch curated:', e);
    } finally {
      isCuratedLoading.value = false;
    }
  }

  async function fetchDetail(source: string, slug: string): Promise<SkillsShSkillDetail | null> {
    const key = `${source}/${slug}`;

    if (detailCache.value.has(key)) {
      return detailCache.value.get(key)!;
    }

    try {
      const result = await invoke<SkillsShSkillDetail>('fetch_skills_sh_skill_detail', {
        source,
        slug,
      });
      detailCache.value.set(key, result);
      return result;
    } catch (e) {
      console.error('Failed to fetch detail:', e);
      return null;
    }
  }

  async function fetchAudit(source: string, slug: string): Promise<SkillsShAuditResponse | null> {
    const key = `${source}/${slug}`;

    if (auditCache.value.has(key)) {
      return auditCache.value.get(key)!;
    }

    try {
      const result = await invoke<SkillsShAuditResponse>('fetch_skills_sh_audit', {
        source,
        slug,
      });
      auditCache.value.set(key, result);
      return result;
    } catch (e) {
      console.error('Failed to fetch audit:', e);
      return null;
    }
  }

  async function installSkill(
    installUrl: string,
    slug: string,
    agent: string = 'cursor'
  ): Promise<{ success: boolean; message: string }> {
    try {
      isInstalling.value = true;
      installError.value = null;

      const result = await invoke<SkillsShInstallResult>('install_skill_via_skills_sh', {
        installUrl,
        slug,
        targetAgent: agent,
      });

      return {
        success: result.success,
        message: result.message,
      };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      installError.value = msg;
      return {
        success: false,
        message: msg,
      };
    } finally {
      isInstalling.value = false;
    }
  }

  function setView(newView: SkillsShView): void {
    if (view.value !== newView) {
      view.value = newView;
      fetchLeaderboard(true);
    }
  }

  function clearSearch(): void {
    searchKeyword.value = '';
    searchResults.value = [];
    searchError.value = null;
  }

  function clearError(): void {
    error.value = null;
  }

  function updateAllSources(): void {
    const groups = new Map<string, SkillsShSourceGroup>();
    for (const skill of skills.value) {
      if (!groups.has(skill.source)) {
        groups.set(skill.source, {
          source: skill.source,
          totalInstalls: 0,
          skillCount: 0,
          skills: [],
        });
      }
      const group = groups.get(skill.source)!;
      group.totalInstalls += skill.installs;
      group.skillCount += 1;
      group.skills.push(skill);
    }
    allSources.value = Array.from(groups.values()).sort((a, b) => b.totalInstalls - a.totalInstalls);
  }

  return {
    // State
    view,
    skills,
    pagination,
    isLoading,
    isLoadingMore,
    error,
    searchKeyword,
    searchResults,
    isSearching,
    searchError,
    curated,
    isCuratedLoading,
    curatedError,
    detailCache,
    auditCache,
    isInstalling,
    installError,
    allSources,

    // Computed
    groupedBySource,
    topSkills,
    hasError,
    isApi401,

    // Actions
    fetchLeaderboard,
    loadMore,
    search,
    fetchCurated,
    fetchDetail,
    fetchAudit,
    installSkill,
    setView,
    clearSearch,
    clearError,
  };
});

/**
 * 统一 Skill Store
 *
 * 合并现有的 5 个 Skill Store 为 1 个：
 * - skill.ts (基础 CRUD)
 * - skill-marketplace.ts (市场浏览/安装)
 * - skill-import.ts (本地导入/仓库管理)
 * - anthropic-skills.ts (Anthropic 官方/远程源)
 * - skills-sh.ts (skills.sh 排行榜)
 *
 * 通过 AllAgentsService 进行同步，保留各来源的特殊逻辑。
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  UnifiedPlugin,
  PluginSource,
  SyncTarget,
  SyncStatus,
} from '@/types/unified-plugin';

// ============================================================================
// 来源类型
// ============================================================================

/** 技能源类型 */
export type SkillSourceType =
  | 'builtin'        // 内置 DB 技能
  | 'marketplace'    // Marketplace 来源
  | 'anthropic'      // Anthropic 官方
  | 'remote'         // 远程自定义源
  | 'skills-sh'      // skills.sh 排行榜
  | 'local'          // 本地导入
  | 'repository';    // Git 仓库

/** 统一的安装进度 */
export interface SkillInstallProgress {
  stage: 'listing' | 'downloading' | 'verifying' | 'copying' | 'complete' | 'error';
  progress: number;
  message?: string;
  error?: string;
}

/** 技能来源信息 */
export interface SkillSourceInfo {
  id: string;
  name: string;
  type: SkillSourceType;
  url?: string;
  enabled: boolean;
  lastSyncedAt?: string;
}

/** 仓库信息 (skill-import) */
export interface SkillRepository {
  id: string;
  name: string;
  url: string;
  branch?: string;
  lastSyncedAt?: string;
  skillCount: number;
}

// ============================================================================
// Store 定义
// ============================================================================

export const useUnifiedSkillStore = defineStore('unified-skill', () => {
  // ==========================================================================
  // 状态
  // ==========================================================================

  /** 所有技能（统一类型） */
  const skills = ref<UnifiedPlugin[]>([]);

  /** 技能源列表 */
  const sources = ref<SkillSourceInfo[]>([]);

  /** 活动源 ID */
  const activeSourceId = ref<string>('');

  /** 仓库列表 (Git 来源) */
  const repositories = ref<SkillRepository[]>([]);

  /** 同步目标 */
  const syncTargets = ref<SyncTarget[]>([]);

  /** 搜索关键词 */
  const searchKeyword = ref('');

  /** 选中的分类 */
  const selectedCategory = ref<string | null>(null);

  /** 分页 */
  const currentPage = ref(1);
  const pageSize = ref(20);
  const totalSkills = ref(0);

  /** 加载状态 */
  const isLoading = ref(false);
  const isInstalling = ref(false);
  const isSyncing = ref(false);
  const error = ref<string | null>(null);

  /** 安装进度 */
  const installProgress = ref<Map<string, SkillInstallProgress>>(new Map());

  /** 事件监听清理 */
  let progressUnlisten: UnlistenFn | null = null;

  // ==========================================================================
  // 计算属性
  // ==========================================================================

  /** 按来源分组 */
  const skillsBySource = computed(() => {
    const grouped: Record<SkillSourceType, UnifiedPlugin[]> = {
      builtin: [],
      marketplace: [],
      anthropic: [],
      remote: [],
      'skills-sh': [],
      local: [],
      repository: [],
    };

    for (const skill of skills.value) {
      const sourceType = skill.source.type as SkillSourceType;
      if (grouped[sourceType]) {
        grouped[sourceType].push(skill);
      }
    }

    return grouped;
  });

  /** 已安装的技能 */
  const installedSkills = computed(() =>
    skills.value.filter(s => s.installed)
  );

  /** 当前源的技能 */
  const currentSourceSkills = computed(() => {
    if (!activeSourceId.value) {
      return skills.value;
    }

    const source = sources.value.find(s => s.id === activeSourceId.value);
    if (!source) {
      return skills.value;
    }

    return skills.value.filter(s => {
      if (source.type === 'marketplace') {
        return s.source.type === 'marketplace' && s.source.marketplace === source.name;
      }
      if (source.type === 'anthropic' || source.type === 'remote') {
        return s.source.type === 'github' && s.tags.includes('anthropic');
      }
      return s.source.type === source.type;
    });
  });

  /** 过滤后的技能 */
  const filteredSkills = computed(() => {
    let result = currentSourceSkills.value;

    // 搜索过滤
    if (searchKeyword.value) {
      const kw = searchKeyword.value.toLowerCase();
      result = result.filter(s =>
        s.name.toLowerCase().includes(kw) ||
        s.description?.toLowerCase().includes(kw) ||
        s.tags.some(t => t.toLowerCase().includes(kw))
      );
    }

    // 分类过滤
    if (selectedCategory.value) {
      result = result.filter(s =>
        s.categories.includes(selectedCategory.value!)
      );
    }

    return result;
  });

  /** 分页后的技能 */
  const paginatedSkills = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredSkills.value.slice(start, start + pageSize.value);
  });

  /** 总页数 */
  const totalPages = computed(() =>
    Math.ceil(filteredSkills.value.length / pageSize.value)
  );

  // ==========================================================================
  // 来源管理
  // ==========================================================================

  /** 加载来源列表 */
  async function fetchSources(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 allagents marketplace 获取来源
      const result = await invoke<{ success: boolean; data?: any; error?: string }>(
        'allagents_marketplace_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.output) {
        // 解析 allagents marketplace 列表
        sources.value = parseMarketplaceList(result.data.output);
      }

      // 添加内置来源
      sources.value.push({
        id: 'builtin',
        name: '内置技能',
        type: 'builtin',
        enabled: true,
      });

      // 添加 skills.sh 来源
      sources.value.push({
        id: 'skills-sh',
        name: 'skills.sh',
        type: 'skills-sh',
        url: 'https://skills.sh',
        enabled: true,
      });
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 切换活动源 */
  function switchSource(sourceId: string) {
    activeSourceId.value = sourceId;
    currentPage.value = 1;
    searchKeyword.value = '';
    selectedCategory.value = null;
  }

  // ==========================================================================
  // 技能加载
  // ==========================================================================

  /** 加载所有技能 */
  async function fetchSkills(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 allagents 获取技能列表
      const result = await invoke<{ success: boolean; data?: any; error?: string }>(
        'allagents_skill_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.skills) {
        skills.value = result.data.skills.map((s: any) =>
          convertAllagentsSkill(s)
        );
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  /** 加载本地已安装技能 (from DB) */
  async function fetchLocalSkills(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; data?: any }>(
        'get_skills',
        { softwareId: '' }
      );

      if (result.success && result.data) {
        const localSkills = result.data.map((s: any) =>
          convertLocalSkill(s)
        );

        // 合并到现有列表，避免重复
        const existingIds = new Set(skills.value.map(s => s.id));
        for (const skill of localSkills) {
          if (!existingIds.has(skill.id)) {
            skills.value.push(skill);
          }
        }
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  // ==========================================================================
  // 安装/卸载
  // ==========================================================================

  /** 安装技能 */
  async function installSkill(
    skill: UnifiedPlugin,
    targetDir?: string
  ): Promise<boolean> {
    isInstalling.value = true;
    error.value = null;

    try {
      const spec = skill.allagentsSpec ?? skill.name;

      // 更新进度
      installProgress.value.set(skill.id, {
        stage: 'downloading',
        progress: 0,
        message: '正在下载...',
      });

      const result = await invoke<{ success: boolean; data?: any; error?: string }>(
        'allagents_skill_add',
        {
          workspacePath: '',
          name: skill.name,
          from: skill.source.repo,
          plugin: skill.parentPlugin,
          scope: 'project',
        }
      );

      if (result.success) {
        // 更新技能状态
        const idx = skills.value.findIndex(s => s.id === skill.id);
        if (idx >= 0) {
          skills.value[idx].installed = true;
          skills.value[idx].installedAt = new Date().toISOString();
          skills.value[idx].syncStatus = 'pending';
        }

        installProgress.value.set(skill.id, {
          stage: 'complete',
          progress: 100,
        });

        return true;
      } else {
        error.value = result.error ?? '安装失败';
        installProgress.value.set(skill.id, {
          stage: 'error',
          progress: 0,
          error: result.error,
        });
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isInstalling.value = false;
    }
  }

  /** 卸载技能 */
  async function uninstallSkill(skill: UnifiedPlugin): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'allagents_skill_remove',
        {
          workspacePath: '',
          name: skill.name,
          plugin: skill.parentPlugin,
          scope: 'project',
        }
      );

      if (result.success) {
        const idx = skills.value.findIndex(s => s.id === skill.id);
        if (idx >= 0) {
          skills.value[idx].installed = false;
          skills.value[idx].syncStatus = 'pending';
        }
        return true;
      } else {
        error.value = result.error ?? '卸载失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  /** 检查技能是否已安装 */
  function isSkillInstalled(skillName: string): boolean {
    return skills.value.some(
      s => s.name === skillName && s.installed
    );
  }

  // ==========================================================================
  // 仓库管理 (skill-import)
  // ==========================================================================

  /** 加载仓库列表 */
  async function fetchRepositories(): Promise<void> {
    try {
      const result = await invoke<{ success: boolean; data?: any }>(
        'get_repositories'
      );

      if (result.success && result.data) {
        repositories.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  /** 添加仓库 */
  async function addRepository(url: string, name?: string): Promise<boolean> {
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'add_repository',
        { url, name }
      );

      if (result.success) {
        await fetchRepositories();
        return true;
      } else {
        error.value = result.error ?? '添加仓库失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  /** 同步仓库 */
  async function syncRepository(repoId: string): Promise<boolean> {
    isSyncing.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'sync_repository',
        { repoId }
      );

      if (result.success) {
        await fetchRepositories();
        return true;
      } else {
        error.value = result.error ?? '同步仓库失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  // ==========================================================================
  // 同步目标
  // ==========================================================================

  /** 加载同步目标 */
  async function fetchSyncTargets(): Promise<void> {
    try {
      const result = await invoke<{ success: boolean; data?: any }>(
        'get_sync_targets'
      );

      if (result.success && result.data) {
        syncTargets.value = result.data;
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  /** 同步技能到目标 */
  async function syncSkillToTarget(
    skillName: string,
    target: SyncTarget
  ): Promise<boolean> {
    isSyncing.value = true;
    error.value = null;

    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        'sync_skill_to_target',
        {
          skillName,
          localPath: '',
          target: {
            id: target.client,
            name: target.client,
            path: target.path,
            enabled: true,
          },
        }
      );

      if (result.success) {
        return true;
      } else {
        error.value = result.error ?? '同步失败';
        return false;
      }
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isSyncing.value = false;
    }
  }

  // ==========================================================================
  // 事件监听
  // ==========================================================================

  /** 开始监听安装进度 */
  function startListening() {
    if (progressUnlisten) return;

    listen<SkillInstallProgress>('skill-install-progress', (event) => {
      // 进度事件处理（需要匹配技能 ID）
      // 实际实现中需要从事件 payload 中提取技能 ID
    }).then((unlisten) => {
      progressUnlisten = unlisten;
    });
  }

  /** 停止监听 */
  function stopListening() {
    if (progressUnlisten) {
      progressUnlisten();
      progressUnlisten = null;
    }
  }

  // ==========================================================================
  // 工具函数
  // ==========================================================================

  /** 清除错误 */
  function clearError() {
    error.value = null;
  }

  /** 重置状态 */
  function reset() {
    skills.value = [];
    sources.value = [];
    repositories.value = [];
    syncTargets.value = [];
    searchKeyword.value = '';
    selectedCategory.value = null;
    currentPage.value = 1;
    error.value = null;
    installProgress.value.clear();
  }

  // ==========================================================================
  // 转换函数
  // ==========================================================================

  /** 将 allagents 技能转换为 UnifiedPlugin */
  function convertAllagentsSkill(raw: any): UnifiedPlugin {
    return {
      id: raw.name,
      name: raw.name,
      description: raw.description,
      source: {
        type: raw.plugin ? 'github' : 'local',
        repo: raw.plugin,
      },
      scope: 'project',
      type: 'skill',
      tags: raw.tags ?? [],
      categories: raw.categories ?? [],
      installed: true,
      enabled: raw.enabled ?? true,
      syncTargets: [],
      syncStatus: 'synced',
      targetClients: [],
      allagentsSpec: raw.plugin ? `${raw.name}@${raw.plugin}` : raw.name,
      skillPath: raw.path,
      parentPlugin: raw.plugin,
    };
  }

  /** 将本地 DB 技能转换为 UnifiedPlugin */
  function convertLocalSkill(raw: any): UnifiedPlugin {
    return {
      id: raw.id ?? raw.name,
      name: raw.name,
      description: raw.description,
      source: { type: 'local' },
      scope: 'project',
      type: 'skill',
      tags: [],
      categories: [],
      installed: true,
      enabled: true,
      syncTargets: [],
      syncStatus: 'synced',
      targetClients: [],
      skillPath: raw.filePath,
    };
  }

  /** 解析 marketplace 列表 */
  function parseMarketplaceList(output: string): SkillSourceInfo[] {
    const sources: SkillSourceInfo[] = [];
    // 简单解析，实际应根据 allagents 输出格式调整
    const lines = output.split('\n');
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed && !trimmed.startsWith('#')) {
        sources.push({
          id: trimmed,
          name: trimmed,
          type: 'marketplace',
          enabled: true,
        });
      }
    }
    return sources;
  }

  // ==========================================================================
  // 返回
  // ==========================================================================

  return {
    // 状态
    skills,
    sources,
    activeSourceId,
    repositories,
    syncTargets,
    searchKeyword,
    selectedCategory,
    currentPage,
    pageSize,
    totalSkills,
    isLoading,
    isInstalling,
    isSyncing,
    error,
    installProgress,

    // 计算属性
    skillsBySource,
    installedSkills,
    currentSourceSkills,
    filteredSkills,
    paginatedSkills,
    totalPages,

    // 来源管理
    fetchSources,
    switchSource,

    // 技能加载
    fetchSkills,
    fetchLocalSkills,

    // 安装/卸载
    installSkill,
    uninstallSkill,
    isSkillInstalled,

    // 仓库管理
    fetchRepositories,
    addRepository,
    syncRepository,

    // 同步目标
    fetchSyncTargets,
    syncSkillToTarget,

    // 事件监听
    startListening,
    stopListening,

    // 工具
    clearError,
    reset,
  };
});

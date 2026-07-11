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
  SyncTarget,
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
/**
 * Per-skill install / sync progress event. Mirrors the Rust
 * `InstallProgress` payload emitted by `anthropic_skills` on the
 * `anthropic-skill-install-progress` and `remote-skill-install-progress`
 * channels. `stage` is a free-form string from the Rust side; we model
 * it as a closed union here but cast at the boundary so a new stage
 * added in Rust does not silently break the listener.
 */
export interface SkillInstallProgress {
  /** Skill ID this progress event refers to. Optional in the interface
   *  so internal store helpers (e.g. `installSkill`) can record progress
   *  on their own behalf without re-stating the id; the Rust-emitted
   *  payload always carries `skill_id`. */
  skillId?: string;
  stage: 'listing' | 'downloading' | 'verifying' | 'copying' | 'complete' | 'error' | 'success' | 'failed';
  /** 0–100 */
  progress: number;
  message?: string;
  error?: string;
  /** Bytes / files already downloaded. */
  filesDownloaded?: number;
  /** Total bytes / files expected. */
  filesTotal?: number;
  /** Absolute install path, populated when stage === 'success'. */
  installedPath?: string;
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
      const result = await invoke<{ success: boolean; data?: { output?: string }; error?: string }>(
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
      const result = await invoke<{ success: boolean; data?: { skills?: unknown[] }; error?: string }>(
        'allagents_skill_list',
        { workspacePath: '' }
      );

      if (result.success && result.data?.skills) {
        skills.value = result.data.skills.map((s) =>
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
      const result = await invoke<{ success: boolean; data?: unknown[] }>(
        'get_skills',
        { softwareId: '' }
      );

      if (result.success && result.data) {
        const localSkills = result.data.map((s) =>
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
  async function installSkill(skill: UnifiedPlugin): Promise<boolean> {
    isInstalling.value = true;
    error.value = null;

    try {
      // 更新进度
      installProgress.value.set(skill.id, {
        stage: 'downloading',
        progress: 0,
        message: '正在下载...',
      });

      const result = await invoke<{ success: boolean; data?: unknown; error?: string }>(
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
      const result = await invoke<{ success: boolean; data?: SkillRepository[] }>(
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
      const result = await invoke<{ success: boolean; data?: SyncTarget[] }>(
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

  /**
   * Begin listening for skill install / sync progress events from the
   * Rust backend. The Rust side emits `anthropic-skill-install-progress`
   * for Anthropic Official source installs and `remote-skill-install-progress`
   * for all other sources. We map them to a single internal event and
   * store the latest progress per `skill_id` in `installProgress` so
   * the UI can render per-skill progress bars.
   */
  function startListening() {
    if (progressUnlisten) return;
    const handlers: UnlistenFn[] = [];

    const applyProgress = (event: { payload: SkillInstallProgress }) => {
      const p = event.payload;
      if (!p || !p.skillId) {
        // Avoid pulling in a logging shim — eslint handles this in CI.
        // eslint-disable-next-line no-console
        console.warn('[unified-skill] install progress without skillId', p);
        return;
      }
      installProgress.value.set(p.skillId, p);
      // Vue Map mutations don't auto-trigger reactivity; force a refresh
      // by reassigning the ref.
      installProgress.value = new Map(installProgress.value);

      // Reflect terminal stages on the skill record so the UI can flip
      // its spinner to a success/failure badge.
      const skill = skills.value.find((s) => s.id === p.skillId);
      if (skill) {
        if (p.stage === 'success' || p.stage === 'complete') {
          skill.installed = true;
          if (p.installedPath) {
            skill.installedPath = p.installedPath;
          }
          installProgress.value.delete(p.skillId);
          installProgress.value = new Map(installProgress.value);
        } else if (p.stage === 'failed' || p.stage === 'error') {
          (skill as Record<string, unknown>).lastError = p.message ?? p.error;
        }
      }
    };

    Promise.all([
      listen<SkillInstallProgress>('anthropic-skill-install-progress', applyProgress),
      listen<SkillInstallProgress>('remote-skill-install-progress', applyProgress),
    ])
      .then((unlistens) => {
        handlers.push(...unlistens);
        progressUnlisten = () => handlers.forEach((fn) => fn());
      })
      .catch((e) => {
        // eslint-disable-next-line no-console
        console.error('[unified-skill] failed to register progress listeners', e);
      });
  }

  /** Stop listening */
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
  function convertAllagentsSkill(raw: unknown): UnifiedPlugin {
    const r = raw as Record<string, unknown>;
    return {
      id: String(r.name ?? ''),
      name: String(r.name ?? ''),
      description: String(r.description ?? ''),
      source: {
        type: r.plugin ? 'github' : 'local',
        repo: String(r.plugin ?? ''),
      },
      scope: 'project',
      type: 'skill',
      tags: Array.isArray(r.tags) ? r.tags : [],
      categories: Array.isArray(r.categories) ? r.categories : [],
      installed: true,
      enabled: r.enabled !== false,
      syncTargets: [],
      syncStatus: 'synced',
      targetClients: [],
      allagentsSpec: r.plugin ? `${String(r.name)}@${String(r.plugin)}` : String(r.name),
      skillPath: String(r.path ?? ''),
      parentPlugin: String(r.plugin ?? ''),
    };
  }

  /** 将本地 DB 技能转换为 UnifiedPlugin */
  function convertLocalSkill(raw: unknown): UnifiedPlugin {
    const r = raw as Record<string, unknown>;
    return {
      id: String(r.id ?? r.name ?? ''),
      name: String(r.name ?? ''),
      description: String(r.description ?? ''),
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
      skillPath: String(r.filePath ?? ''),
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

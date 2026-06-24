// Recommended Sites Store - State management for "推荐技能网站"
// Each entry is a curated website that points to a skills marketplace,
// GitHub repo, or similar resource. The user can add, update, and
// delete entries; the list is persisted to localStorage so that custom
// additions survive across app restarts.

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export type SiteRegion = 'international' | 'china' | 'github' | 'other';

export interface RecommendedSite {
  id: string;
  name: string;
  /** Single-paragraph description shown under the name. */
  description: string;
  /** External URL opened by the jump button. */
  url: string;
  /** Optional ISO region tag. Used to colour the region chip. */
  region: SiteRegion;
  createdAt: string;
  updatedAt: string;
}

const STORAGE_KEY = 'aem-recommended-sites';

// Seed the store with the 8 websites that previously lived in the
// marketplace source-tabs. We keep them here so the user's first visit
// to the new feature feels familiar.
const DEFAULT_SITES: RecommendedSite[] = [
  {
    id: 'skillmp',
    name: 'SkillMP',
    description: 'International AI skills marketplace — discover community-curated skills across development, business, search, writing, automation, and AI categories.',
    url: 'https://skillsmp.com',
    region: 'international',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'skillzwave',
    name: 'SkillzWave',
    description: 'AI skills platform for developers — focused on integration-ready skills that drop into your existing dev environment without extra glue code.',
    url: 'https://skillzwave.ai',
    region: 'international',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'agensi',
    name: 'Agensi',
    description: 'Enterprise skills library — production-grade skills aimed at teams that need governance, review trails, and stable long-term support.',
    url: 'https://agensi.io',
    region: 'international',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'skills-marketplace',
    name: 'Skills Marketplace',
    description: 'Community-driven skills marketplace — an open submission portal where individual contributors publish and version their own skills.',
    url: 'https://skills.marketplace',
    region: 'international',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'clawhub',
    name: 'ClawHub',
    description: '国内 AI 技能聚合平台 — 中文环境下的 AI 技能聚合，涵盖开发、写作、商业、自动化等常见场景。',
    url: 'https://clawhub.ai',
    region: 'china',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'skill-cn',
    name: 'Skill Hub 中国',
    description: '中文 AI 技能市场 — 面向中文用户的技能市场，描述与标签均为中文，方便本地团队直接复用。',
    url: 'https://skill-cn.com',
    region: 'china',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'agskills',
    name: 'agskills.dev',
    description: '开发者技能平台 — 面向工程师的技能平台，强调可读性、可测试性以及与主流 IDE 工具链的兼容性。',
    url: 'https://agskills.dev',
    region: 'china',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'awesome-skills',
    name: 'Awesome-Skills',
    description: 'GitHub 技能聚合仓库 — 由社区维护的精选列表，按场景分类整理了大量可一键复用的技能资源。',
    url: 'https://github.com/Sec-Dome/Awesome-Skills',
    region: 'github',
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  },
];

function readFromStorage(): RecommendedSite[] {
  if (typeof localStorage === 'undefined') return [...DEFAULT_SITES];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [...DEFAULT_SITES];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [...DEFAULT_SITES];
    // Light validation: keep only entries that have the required fields.
    return parsed.filter(
      (s): s is RecommendedSite =>
        s &&
        typeof s.id === 'string' &&
        typeof s.name === 'string' &&
        typeof s.description === 'string' &&
        typeof s.url === 'string'
    );
  } catch (e) {
    console.warn('Failed to read recommended sites from storage:', e);
    return [...DEFAULT_SITES];
  }
}

function writeToStorage(sites: RecommendedSite[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sites));
  } catch (e) {
    console.warn('Failed to persist recommended sites:', e);
  }
}

function makeId(name: string): string {
  const slug = name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9\u4e00-\u9fa5]+/g, '-')
    .replace(/^-+|-+$/g, '');
  const base = slug || 'site';
  // Append a short timestamp to avoid collisions when two sites share a slug.
  return `${base}-${Date.now().toString(36)}`;
}

export const useRecommendedSitesStore = defineStore('recommendedSites', () => {
  // State
  const sites = ref<RecommendedSite[]>(readFromStorage());

  // Computed
  const count = computed(() => sites.value.length);

  const regionLabels: Record<SiteRegion, string> = {
    international: '国际',
    china: '国内',
    github: 'GitHub',
    other: '其他',
  };

  const regionColorVar: Record<SiteRegion, string> = {
    international: 'var(--info)',
    china: 'var(--error)',
    github: 'var(--fg)',
    other: 'var(--fg-muted)',
  };

  function regionLabel(region: SiteRegion): string {
    return regionLabels[region] ?? '其他';
  }

  function regionColor(region: SiteRegion): string {
    return regionColorVar[region] ?? 'var(--fg-muted)';
  }

  // Actions
  function persist(): void {
    writeToStorage(sites.value);
  }

  function addSite(input: {
    name: string;
    description: string;
    url: string;
    region: SiteRegion;
  }): RecommendedSite {
    const now = new Date().toISOString();
    const site: RecommendedSite = {
      id: makeId(input.name),
      name: input.name.trim(),
      description: input.description.trim(),
      url: input.url.trim(),
      region: input.region,
      createdAt: now,
      updatedAt: now,
    };
    sites.value = [site, ...sites.value];
    persist();
    return site;
  }

  function updateSite(
    id: string,
    input: {
      name: string;
      description: string;
      url: string;
      region: SiteRegion;
    }
  ): boolean {
    const idx = sites.value.findIndex(s => s.id === id);
    if (idx === -1) return false;
    const previous = sites.value[idx];
    const updated: RecommendedSite = {
      ...previous,
      name: input.name.trim(),
      description: input.description.trim(),
      url: input.url.trim(),
      region: input.region,
      updatedAt: new Date().toISOString(),
    };
    // Replace immutably to keep reactivity consistent.
    sites.value = [
      ...sites.value.slice(0, idx),
      updated,
      ...sites.value.slice(idx + 1),
    ];
    persist();
    return true;
  }

  function removeSite(id: string): boolean {
    const next = sites.value.filter(s => s.id !== id);
    if (next.length === sites.value.length) return false;
    sites.value = next;
    persist();
    return true;
  }

  function resetToDefaults(): void {
    sites.value = [...DEFAULT_SITES];
    persist();
  }

  return {
    // State
    sites,
    // Computed
    count,
    // Helpers
    regionLabel,
    regionColor,
    // Actions
    addSite,
    updateSite,
    removeSite,
    resetToDefaults,
  };
});

// Skill Source Types - Multi-source skill marketplace

export type SourceRegion = 'international' | 'china' | 'github';
export type SyncMethod = 'copy' | 'symlink';
export type InstallStatus = 'pending' | 'downloading' | 'installing' | 'syncing' | 'success' | 'failed' | 'conflict';

export interface SkillSource {
  id: string;
  name: string;
  nameZh?: string;
  region: SourceRegion;
  url: string;
  apiEndpoint: string;
  description: string;
  icon?: string;
  isAvailable: boolean;
  lastChecked?: string;
  skillCount?: number;
}

export interface MarketplaceSkill {
  id: string;
  sourceId: string;
  name: string;
  description: string;
  longDescription?: string;
  author?: string;
  version?: string;
  categories: string[];
  tags: string[];
  installCommand?: string;
  installPath?: string;
  repository?: string;
  homepage?: string;
  license?: string;
  stars?: number;
  downloads?: number;
  lastUpdated?: string;
  metadata?: Record<string, unknown>;
}

export interface PaginatedSkills {
  items: MarketplaceSkill[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface SyncTarget {
  id: string;
  name: string;
  path: string;
  method: SyncMethod;
  isValid: boolean;
  exists?: boolean;
}

export interface SyncConfig {
  targets: SyncTarget[];
  defaultMethod: SyncMethod;
}

export interface InstallProgress {
  skillId: string;
  skillName: string;
  stage: InstallStatus;
  progress: number;
  message: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export interface SyncProgress {
  skillName: string;
  targetName: string;
  method: SyncMethod;
  stage: 'pending' | 'syncing' | 'success' | 'failed';
  progress: number;
  error?: string;
}

export interface MarketplaceError {
  code: string;
  message: string;
  source?: string;
  details?: Record<string, unknown>;
}

// Category definitions
export const SKILL_CATEGORIES = {
  development: {
    label: 'Development',
    labelZh: '开发',
    tags: ['code', 'debug', 'test', 'refactor', 'security', 'database', 'api', 'frontend', 'backend'],
  },
  business: {
    label: 'Business',
    labelZh: '商业',
    tags: ['business', 'crm', 'analytics', 'finance', 'marketing', 'sales', 'hr'],
  },
  search: {
    label: 'Search & Research',
    labelZh: '搜索研究',
    tags: ['search', 'web', 'research', 'scraping', 'data'],
  },
  writing: {
    label: 'Writing & Content',
    labelZh: '写作内容',
    tags: ['writing', 'content', 'translation', 'summarization', 'copywriting'],
  },
  tools: {
    label: 'Tools & Automation',
    labelZh: '工具自动化',
    tags: ['automation', 'system', 'file', 'network', 'cloud', 'devops'],
  },
  ai: {
    label: 'AI & Machine Learning',
    labelZh: 'AI与机器学习',
    tags: ['llm', 'ml', 'nlp', 'vision', 'agent', 'prompt'],
  },
} as const;

export type CategoryKey = keyof typeof SKILL_CATEGORIES;

// Default preset sources
export const PRESET_SOURCES: SkillSource[] = [
  {
    id: 'skillmp',
    name: 'SkillMP',
    region: 'international',
    url: 'https://skillsmp.com',
    apiEndpoint: 'https://api.skillsmp.com/v1/skills',
    description: 'International AI skills marketplace',
    isAvailable: true,
  },
  {
    id: 'skillzwave',
    name: 'SkillzWave',
    region: 'international',
    url: 'https://skillzwave.ai',
    apiEndpoint: 'https://api.skillzwave.ai/v1/skills',
    description: 'AI skills platform for developers',
    isAvailable: true,
  },
  {
    id: 'agensi',
    name: 'Agensi',
    region: 'international',
    url: 'https://agensi.io',
    apiEndpoint: 'https://agensi.io/skills/api/v1',
    description: 'Enterprise skills library',
    isAvailable: true,
  },
  {
    id: 'skills-marketplace',
    name: 'Skills Marketplace',
    region: 'international',
    url: 'https://skills.marketplace',
    apiEndpoint: 'https://skills.marketplace/api/v1',
    description: 'Community-driven skills marketplace',
    isAvailable: true,
  },
  {
    id: 'clawhub',
    name: 'ClawHub',
    nameZh: '爪哇市场',
    region: 'china',
    url: 'https://clawhub.ai',
    apiEndpoint: 'https://clawhub.ai/api/skills',
    description: '国内 AI 技能聚合平台',
    isAvailable: true,
  },
  {
    id: 'skill-cn',
    name: 'Skill Hub 中国',
    region: 'china',
    url: 'https://skill-cn.com',
    apiEndpoint: 'https://skill-cn.com/api/skills',
    description: '中文 AI 技能市场',
    isAvailable: true,
  },
  {
    id: 'agskills',
    name: 'agskills.dev',
    region: 'china',
    url: 'https://agskills.dev',
    apiEndpoint: 'https://agskills.dev/api/skills',
    description: '开发者技能平台',
    isAvailable: true,
  },
  {
    id: 'awesome-skills',
    name: 'Awesome-Skills',
    nameZh: '技能聚合',
    region: 'github',
    url: 'https://github.com/Sec-Dome/Awesome-Skills',
    apiEndpoint: 'https://api.github.com/repos/Sec-Dome/Awesome-Skills/contents',
    description: 'GitHub aggregated skills collection',
    isAvailable: true,
  },
];

// Default sync targets
export const DEFAULT_SYNC_TARGETS: SyncTarget[] = [
  {
    id: 'cursor-default',
    name: 'Cursor',
    path: '~/.cursor/skills/',
    method: 'copy',
    isValid: true,
  },
  {
    id: 'claude-default',
    name: 'Claude Desktop',
    path: '~/.claude/skills/',
    method: 'symlink',
    isValid: true,
  },
];

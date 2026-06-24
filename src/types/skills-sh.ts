// skills.sh TypeScript Type Definitions

export type SkillsShView = 'all-time' | 'trending' | 'hot';

export interface SkillsShSkill {
  id: string;
  slug: string;
  name: string;
  source: string;
  installs: number;
  sourceType: 'github' | 'well-known';
  installUrl: string | null;
  url: string;
  isDuplicate?: boolean;
  // hot view
  installsYesterday?: number;
  change?: number;
}

export interface SkillsShPage {
  data: SkillsShSkill[];
  pagination: SkillsShPagination;
}

export interface SkillsShPagination {
  page: number;
  perPage: number;
  total: number;
  hasMore: boolean;
}

export interface SkillsShCuratedOwner {
  owner: string;
  totalInstalls: number;
  featuredRepo: string;
  featuredSkill: string;
  skills: SkillsShSkill[];
}

export interface SkillsShCuratedResponse {
  data: SkillsShCuratedOwner[];
  totalOwners: number;
  totalSkills: number;
  generatedAt: string;
}

export interface SkillsShSkillFile {
  path: string;
  contents: string | null;
}

export interface SkillsShSkillDetail {
  id: string;
  source: string;
  slug: string;
  installs: number;
  hash: string | null;
  files: SkillsShSkillFile[] | null;
}

export interface SkillsShAuditEntry {
  provider: string;
  slug: string;
  status: 'pass' | 'warn' | 'fail';
  summary: string;
  auditedAt: string;
  riskLevel?: 'NONE' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  categories?: string[];
}

export interface SkillsShAuditResponse {
  id: string;
  source: string;
  slug: string;
  audits: SkillsShAuditEntry[];
}

export interface SkillsShInstallResult {
  success: boolean;
  message: string;
  output?: string;
}

// Source group for Browse by Source view
export interface SkillsShSourceGroup {
  source: string;
  totalInstalls: number;
  skillCount: number;
  skills: SkillsShSkill[];
}

// Curated owner (from API response)
export interface SkillsShCuratedOwnerGroup {
  owner: string;
  totalInstalls: number;
  featuredRepo: string;
  featuredSkill: string;
  skills: SkillsShSkill[];
}

export * from './software';
export * from './plugin';
export * from './skill';
export {
  type SkillSource,
  type MarketplaceSkill,
  type PaginatedSkills,
  type SyncTarget,
  type SyncConfig,
  type InstallProgress as MarketplaceInstallProgress,
  type SyncProgress,
  type InstallStatus,
  type SyncMethod,
  type MarketplaceError,
  type CategoryKey,
  SKILL_CATEGORIES,
  PRESET_SOURCES,
  DEFAULT_SYNC_TARGETS
} from './skill-marketplace';
export * from './skill-import';
export * from './mcp';

// `mcp-marketplace` defines a *remote catalogue* MCPServer with fields
// like `sourceId` / `installCommand`, while `./mcp` defines the *local
// installed* MCPServer used by health/audit code. Both are valid
// `MCPServer`s in the domain. To avoid the `TS2308: has already exported
// a member named` conflict we expose:
//   * `MCPServer`            — the local-service definition from `./mcp`
//                              (kept for backwards compatibility; what
//                              backend commands return when listing local
//                              services or running health checks)
//   * `MarketplaceMCPServer` — the rich catalogue shape from `./mcp-marketplace`
//                              (used by marketplace stores/views/components)
//   * `MCPServerUnion` — intersection (intersection keeps every field on
//                              either side so Vue templates can access
//                              marketplace-only fields such as `author` /
//                              `categories` without per-field narrowing)
export {
  type MCPServer as MarketplaceMCPServer,
  type MCPSource as MarketplaceMCPSource,
  type ServerProtocol as MarketplaceServerProtocol,
  type MCPInstallProgress as MarketplaceMCPInstallProgress,
  type PaginatedMCPServers,
  type MCPSyncTarget as MarketplaceMCPSyncTarget,
  type MCPSyncProgress as MarketplaceMCPSyncProgress,
  type MCPSourceRegion,
  type EnvVar,
  PRESET_MCP_SOURCES,
  MCP_CATEGORIES,
  type CategoryKey as MarketplaceCategoryKey,
  DEFAULT_SYNC_TARGETS as DEFAULT_MARKETPLACE_SYNC_TARGETS,
} from './mcp-marketplace';

import type { MCPServer, MCPInstallProgress } from './mcp';
import type { MCPServer as MarketplaceMCPServer, MCPInstallProgress as MarketplaceMCPInstallProgress } from './mcp-marketplace';

// `MCPServerUnion` is the intersection of the *local* `MCPServer` (used by
// health/audit code) and the *marketplace* `MCPServer` (rich catalogue
// shape). Components that can render either — e.g. the details dialog —
// accept this union so template code can access fields from both sides
// without per-access narrowing. `MCPInstallProgressUnion` is its sibling
// for install progress.
export type MCPServerUnion = MCPServer & MarketplaceMCPServer;
export type MCPInstallProgressUnion = MCPInstallProgress & MarketplaceMCPInstallProgress;
export * from './rule';
export * from './backup';

export * from './plugin-marketplace';
export * from './plugin-capabilities';
export * from './mcp-bridge';
export * from './installed-registry';
export * from './anthropic-skills';
export * from './skills-sh';
export * from './agent';

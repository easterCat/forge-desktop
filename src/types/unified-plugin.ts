/**
 * 统一插件类型系统
 *
 * 替代现有的 4 套 Skill 接口和 2 套 Plugin 接口，
 * 提供统一的类型定义用于所有插件/技能/代理/规则/MCP 管理。
 */

// ============================================================================
// 基础类型
// ============================================================================

/** 插件来源类型 */
export type PluginSourceType = 'marketplace' | 'github' | 'local' | 'url';

/** 插件内容分类 */
export type PluginContentType = 'skill' | 'agent' | 'rule' | 'mcp' | 'hook' | 'command';

/** 同步状态 */
export type SyncStatus =
  | 'synced'
  | 'pending'
  | 'partial'
  | 'error'
  | 'conflict'
  | 'unknown';

/** 作用域 */
export type PluginScope = 'user' | 'project';

/** 安装模式 */
export type InstallMode = 'file' | 'native';

// ============================================================================
// 核心类型
// ============================================================================

/** 插件来源信息 */
export interface PluginSource {
  type: PluginSourceType;
  marketplace?: string;
  repo?: string;
  path?: string;
  url?: string;
}

/** 同步目标 */
export interface SyncTarget {
  /** 目标客户端 (如 "claude", "cursor", "copilot") */
  client: string;
  /** 同步后的文件路径 */
  path: string;
  /** 同步状态 */
  status: SyncStatus;
  /** 最后同步时间 */
  lastSyncedAt?: string;
}

/** 审计日志条目 */
export interface AuditEntry {
  timestamp: string;
  action: string;
  detail: string;
  userId?: string;
}

// ============================================================================
// 统一插件类型
// ============================================================================

/** 统一插件 —— 所有内容类型的通用表示 */
export interface UnifiedPlugin {
  /** 唯一标识 */
  id: string;
  /** 插件名称 */
  name: string;
  /** 描述 */
  description?: string;
  /** 版本 */
  version?: string;

  // 来源信息
  source: PluginSource;
  /** 作用域 (user 全局 / project 项目级) */
  scope: PluginScope;

  // 内容分类
  type: PluginContentType;
  /** 标签 (如 "coding", "research", "productivity") */
  tags: string[];
  /** 分类 (如 "development", "design", "documentation") */
  categories: string[];

  // 安装状态
  installed: boolean;
  enabled: boolean;
  installedAt?: string;
  installedPath?: string;

  // 同步信息
  syncTargets: SyncTarget[];
  syncStatus: SyncStatus;

  // 客户端过滤
  /** 目标客户端列表 (空数组表示全部) */
  targetClients: string[];

  // allagents 特有
  /** allagents 插件规格 (如 "plugin@owner/repo") */
  allagentsSpec?: string;
  /** 安装模式 */
  installMode?: InstallMode;

  // 技能特有
  /** 技能文件路径 (SKILL.md) */
  skillPath?: string;
  /** 父插件名称 (如果此技能属于某个插件) */
  parentPlugin?: string;

  // 代理特有
  /** 代理部门 */
  department?: string;
  /** 代理内容 (Markdown) */
  content?: string;

  // 规则特有
  /** 规则类型 (md / mdc) */
  ruleType?: 'md' | 'mdc';
  /** 规则是否激活 */
  ruleActive?: boolean;

  // 元数据
  createdAt?: string;
  updatedAt?: string;
}

// ============================================================================
// MCP 服务类型
// ============================================================================

/** MCP 传输协议 */
export type McpTransport = 'http' | 'stdio';

/** MCP 健康状态 */
export type McpHealthStatus = 'healthy' | 'unhealthy' | 'unknown';

/** MCP 服务 —— 扩展 allagents 的 MCP 定义 */
export interface UnifiedMCP {
  /** 服务名称 */
  name: string;
  /** 传输协议 */
  transport: McpTransport;

  // HTTP 模式
  /** HTTP 端点 URL */
  url?: string;
  /** HTTP 请求头 */
  headers?: Record<string, string>;

  // stdio 模式
  /** 命令 */
  command?: string;
  /** 命令参数 */
  args?: string[];

  // 通用
  /** 环境变量 */
  env?: Record<string, string>;
  /** 目标客户端 (空数组表示全部) */
  clients?: string[];

  // Forge Desktop 扩展功能
  /** 分组 ID */
  groupIds: string[];
  /** 标签 */
  tags: string[];
  /** 健康状态 */
  healthStatus: McpHealthStatus;
  /** 最后健康检查时间 */
  lastHealthCheck?: string;
  /** 审计日志 */
  auditLog: AuditEntry[];

  // 元数据
  createdAt?: string;
  updatedAt?: string;
}

// ============================================================================
// 工作区配置类型
// ============================================================================

/** workspace.yaml 工作区配置 */
export interface WorkspaceConfig {
  workspace?: {
    source?: string;
    files?: Array<string | { source: string; dest?: string }>;
  };
  repositories?: Array<{
    path: string;
    source?: string;
    repo?: string;
    description?: string;
  }>;
  plugins?: Array<string | {
    name: string;
    source?: string;
    clients?: string[];
    install?: InstallMode;
    exclude?: string[];
    skills?: string[] | { exclude: string[] };
    pin?: string;
  }>;
  clients?: string[];
  mcpServers?: Record<string, {
    type: McpTransport;
    url?: string;
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    headers?: Record<string, string>;
    clients?: string[];
  }>;
  mcpProxy?: {
    clients?: string[];
    servers?: Record<string, { proxy: string[] }>;
  };
  syncMode?: 'copy' | 'symlink';
}

// ============================================================================
// 支持的客户端列表
// ============================================================================

/** AllAgents 支持的客户端 (23个) */
export const SUPPORTED_CLIENTS = [
  // 通用客户端 (Universal Clients - 共享 .agents/skills/)
  'copilot',
  'codex',
  'opencode',
  'gemini',
  'ampcode',
  'vscode',
  'replit',
  'kimi',

  // 提供商专属客户端 (Provider-Specific Clients)
  'claude',
  'cursor',
  'factory',
  'openclaw',
  'windsurf',
  'cline',
  'continue',
  'roo',
  'kilo',
  'trae',
  'augment',
  'zencoder',
  'junie',
  'openhands',
  'kiro',
] as const;

export type ClientType = typeof SUPPORTED_CLIENTS[number];

/** 客户端显示名称映射 */
export const CLIENT_DISPLAY_NAMES: Record<ClientType, string> = {
  claude: 'Claude Code',
  copilot: 'GitHub Copilot',
  codex: 'OpenAI Codex',
  cursor: 'Cursor',
  opencode: 'OpenCode',
  gemini: 'Gemini',
  factory: 'Factory',
  ampcode: 'Amp Code',
  vscode: 'VS Code',
  windsurf: 'Windsurf',
  cline: 'Cline',
  continue: 'Continue',
  roo: 'Roo',
  kilo: 'Kilo',
  trae: 'Trae',
  augment: 'Augment',
  zencoder: 'Zencoder',
  junie: 'Junie',
  openhands: 'OpenHands',
  kiro: 'Kiro',
  replit: 'Replit',
  kimi: 'Kimi',
  openclaw: 'OpenClaw',
};

/** 客户端图标路径映射 */
export const CLIENT_ICONS: Record<ClientType, string> = {
  claude: '/icons/claude.svg',
  copilot: '/icons/copilot.svg',
  codex: '/icons/codex.svg',
  cursor: '/icons/cursor.svg',
  opencode: '/icons/opencode.svg',
  gemini: '/icons/gemini.svg',
  factory: '/icons/factory.svg',
  ampcode: '/icons/ampcode.svg',
  vscode: '/icons/vscode.svg',
  windsurf: '/icons/windsurf.svg',
  cline: '/icons/cline.svg',
  continue: '/icons/continue.svg',
  roo: '/icons/roo.svg',
  kilo: '/icons/kilo.svg',
  trae: '/icons/trae.svg',
  augment: '/icons/augment.svg',
  zencoder: '/icons/zencoder.svg',
  junie: '/icons/junie.svg',
  openhands: '/icons/openhands.svg',
  kiro: '/icons/kiro.svg',
  replit: '/icons/replit.svg',
  kimi: '/icons/kimi.svg',
  openclaw: '/icons/openclaw.svg',
};

/** 客户端品牌主色映射 */
export const CLIENT_COLORS: Record<ClientType, string> = {
  claude: '#D97706',
  copilot: '#6E40C9',
  codex: '#10A37F',
  cursor: '#7C3AED',
  opencode: '#3B82F6',
  gemini: '#4285F4',
  factory: '#F59E0B',
  ampcode: '#8B5CF6',
  vscode: '#007ACC',
  windsurf: '#3B82F6',
  cline: '#10B981',
  continue: '#6366F1',
  roo: '#EC4899',
  kilo: '#8B5CF6',
  trae: '#F97316',
  augment: '#6366F1',
  zencoder: '#14B8A6',
  junie: '#84CC16',
  openhands: '#F59E0B',
  kiro: '#3B82F6',
  replit: '#F97316',
  kimi: '#8B5CF6',
  openclaw: '#3B82F6',
};

// ============================================================================
// 工具函数
// ============================================================================

/** 生成 allagents 插件规格字符串 */
export function toAllagentsSpec(plugin: UnifiedPlugin): string {
  if (plugin.allagentsSpec) {
    return plugin.allagentsSpec;
  }

  if (plugin.source.type === 'github' && plugin.source.repo) {
    return `${plugin.name}@${plugin.source.repo}`;
  }

  if (plugin.source.type === 'marketplace' && plugin.source.marketplace) {
    return `${plugin.name}@${plugin.source.marketplace}`;
  }

  return plugin.name;
}

/** 从 allagents 规格字符串解析插件信息 */
export function parseAllagentsSpec(spec: string): {
  name: string;
  source?: string;
} {
  const atIndex = spec.indexOf('@');
  if (atIndex === -1) {
    return { name: spec };
  }

  return {
    name: spec.substring(0, atIndex),
    source: spec.substring(atIndex + 1),
  };
}

/** 获取插件的显示标签 */
export function getPluginDisplayTags(plugin: UnifiedPlugin): string[] {
  const tags = [...plugin.tags];

  if (plugin.type === 'skill') tags.unshift('Skill');
  if (plugin.type === 'agent') tags.unshift('Agent');
  if (plugin.type === 'mcp') tags.unshift('MCP');
  if (plugin.type === 'rule') tags.unshift('Rule');
  if (plugin.type === 'hook') tags.unshift('Hook');
  if (plugin.type === 'command') tags.unshift('Command');

  return tags;
}

// MCP Marketplace Types - Multi-source MCP server marketplace

export type MCPSourceRegion = 'international' | 'china' | 'github' | 'mcp-specific';
export type SyncMethod = 'copy' | 'symlink';
export type InstallStatus = 'pending' | 'downloading' | 'installing' | 'extracting' | 'syncing' | 'success' | 'failed' | 'conflict';
export type ServerProtocol = 'stdio' | 'sse' | 'http';

export interface MCPSource {
  id: string;
  name: string;
  nameZh?: string;
  region: MCPSourceRegion;
  url: string;
  apiEndpoint: string;
  description: string;
  icon?: string;
  isAvailable: boolean;
  lastChecked?: string;
  serverCount?: number;
  requiresAuth?: boolean;
}

export interface MCPServer {
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
  npmPackage?: string;
  protocol: ServerProtocol;
  requiredEnvVars?: EnvVar[];
  requiredPermissions?: string[];
  lastUpdated?: string;
  stars?: number;
  downloads?: number;
  metadata?: Record<string, unknown>;
  // Local server specific fields (populated by backend)
  endpoint?: string;
  auth?: string;
  healthy?: boolean;
  tools?: number;
  lastChecked?: string;
}

export interface EnvVar {
  name: string;
  description?: string;
  required: boolean;
  defaultValue?: string;
  example?: string;
}

export interface PaginatedMCPServers {
  items: MCPServer[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface MCPSyncTarget {
  id: string;
  name: string;
  path: string;
  method: SyncMethod;
  isValid: boolean;
  exists?: boolean;
  configFile?: string;
}

export interface MCPInstallProgress {
  serverId: string;
  serverName: string;
  stage: InstallStatus;
  progress: number;
  message: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export interface MCPSyncProgress {
  serverName: string;
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

// Category definitions for MCP servers
export const MCP_CATEGORIES = {
  development: {
    label: 'Development',
    labelZh: '开发',
    tags: ['code', 'debug', 'test', 'refactor', 'security', 'database', 'api', 'frontend', 'backend', 'git', 'docker'],
  },
  business: {
    label: 'Business',
    labelZh: '商业',
    tags: ['crm', 'analytics', 'finance', 'marketing', 'sales', 'hr', 'erp'],
  },
  search: {
    label: 'Search & Research',
    labelZh: '搜索研究',
    tags: ['search', 'web', 'research', 'scraping', 'data', 'browser', 'crawl'],
  },
  productivity: {
    label: 'Productivity',
    labelZh: '效率办公',
    tags: ['office', 'document', 'calendar', 'email', 'notes', 'task', 'collaboration'],
  },
  data: {
    label: 'Data & Analytics',
    labelZh: '数据分析',
    tags: ['data', 'analytics', 'visualization', 'ml', 'ai', 'llm', 'nlp'],
  },
  writing: {
    label: 'Writing & Content',
    labelZh: '写作内容',
    tags: ['writing', 'content', 'translation', 'summarization', 'copywriting', 'seo'],
  },
  security: {
    label: 'Security',
    labelZh: '安全',
    tags: ['security', 'auth', 'vulnerability', 'scan', 'audit', 'compliance'],
  },
  infrastructure: {
    label: 'Infrastructure',
    labelZh: '基础设施',
    tags: ['cloud', 'aws', 'gcp', 'azure', 'k8s', 'docker', 'terraform', 'devops'],
  },
} as const;

export type CategoryKey = keyof typeof MCP_CATEGORIES;

// Default MCP marketplace sources
export const PRESET_MCP_SOURCES: MCPSource[] = [
  // International sources
  {
    id: 'mcpmarket',
    name: 'MCPMarket.com',
    region: 'mcp-specific',
    url: 'https://mcpmarket.com',
    apiEndpoint: 'https://mcpmarket.com/api/v1/servers',
    description: 'Comprehensive MCP server marketplace with 10000+ servers',
    isAvailable: true,
    serverCount: 10000,
  },
  {
    id: 'mcpservers-org',
    name: 'MCPServers.org',
    region: 'mcp-specific',
    url: 'https://mcpservers.org',
    apiEndpoint: 'https://api.mcpservers.org/v1/servers',
    description: 'Community-driven MCP server directory',
    isAvailable: true,
    serverCount: 5000,
  },
  {
    id: 'mcplug',
    name: 'MCPlug.store',
    region: 'mcp-specific',
    url: 'https://mcplug.store',
    apiEndpoint: 'https://api.mcplug.store/v1/servers',
    description: 'Premium MCP plugins and integrations',
    isAvailable: true,
    serverCount: 2000,
  },
  {
    id: 'agenticskills',
    name: 'AgenticSkills.io',
    region: 'international',
    url: 'https://agenticskills.io',
    apiEndpoint: 'https://api.agenticskills.io/mcp/servers',
    description: '69,000+ AI agent skills and MCP servers',
    isAvailable: true,
    serverCount: 69000,
  },
  {
    id: 'agentpatch',
    name: 'AgentPatch.ai',
    region: 'international',
    url: 'https://agentpatch.ai',
    apiEndpoint: 'https://api.agentpatch.ai/v1/servers',
    description: 'Open source MCP server patches and enhancements',
    isAvailable: true,
    serverCount: 500,
  },
  {
    id: 'agentpowers',
    name: 'AgentPowers.ai',
    region: 'international',
    url: 'https://agentpowers.ai',
    apiEndpoint: 'https://api.agentpowers.ai/servers',
    description: 'Enterprise-grade agent capabilities and MCP integrations',
    isAvailable: true,
    serverCount: 3000,
  },
  // Chinese sources
  {
    id: 'openclaw',
    name: 'OpenClaw / ClawHub',
    nameZh: '爪哇市场',
    region: 'china',
    url: 'https://clawhub.ai',
    apiEndpoint: 'https://clawhub.ai/api/mcp/servers',
    description: 'OpenClaw 官方 MCP 服务器市场',
    isAvailable: true,
    serverCount: 3000,
  },
  {
    id: 'mcp-cn',
    name: 'MCP 中文社区',
    nameZh: 'MCP中文社区',
    region: 'china',
    url: 'https://mcp-cn.com',
    apiEndpoint: 'https://mcp-cn.com/api/servers',
    description: '国内 MCP 开发者社区',
    isAvailable: true,
    serverCount: 1500,
  },
  {
    id: 'agskills-mcp',
    name: 'AGSkills MCP',
    region: 'china',
    url: 'https://agskills.dev',
    apiEndpoint: 'https://agskills.dev/api/mcp/servers',
    description: '开发者技能平台的 MCP 服务',
    isAvailable: true,
    serverCount: 2000,
  },
  // GitHub sources
  {
    id: 'awesome-mcp',
    name: 'Awesome MCP Servers',
    nameZh: 'MCP服务器聚合',
    region: 'github',
    url: 'https://github.com/punksecurity/awesome-mcp-servers',
    apiEndpoint: 'https://api.github.com/repos/punksecurity/awesome-mcp-servers/contents',
    description: 'GitHub aggregated MCP servers collection',
    isAvailable: true,
    serverCount: 800,
  },
  {
    id: 'mcp-gitHub',
    name: 'MCP by GitHub',
    nameZh: 'GitHub官方MCP',
    region: 'github',
    url: 'https://github.com/github/github-mcp-server',
    apiEndpoint: 'https://api.github.com/repos/github/github-mcp-server',
    description: 'GitHub official MCP server',
    isAvailable: true,
    serverCount: 1,
  },
];

// Default agent sync targets
export const DEFAULT_SYNC_TARGETS: MCPSyncTarget[] = [
  {
    id: 'claude-desktop',
    name: 'Claude Desktop',
    path: '~/Library/Application Support/Claude/claude_desktop_config.json',
    method: 'symlink',
    isValid: true,
    configFile: 'claude_desktop_config.json',
  },
  {
    id: 'cursor-mcp',
    name: 'Cursor',
    path: '~/.cursor/mcp.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp.json',
  },
  {
    id: 'openclaw-mcp',
    name: 'OpenClaw',
    path: '~/.openclaw/mcp.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp.json',
  },
  {
    id: 'cline-mcp',
    name: 'Cline',
    path: '~/.cline/mcp_servers.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp_servers.json',
  },
  {
    id: 'gemini-cli',
    name: 'Gemini CLI',
    path: '~/.gemini/mcp_config.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp_config.json',
  },
  {
    id: 'zed-mcp',
    name: 'Zed',
    path: '~/.config/zed/mcp.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp.json',
  },
  {
    id: 'windsurf-mcp',
    name: 'Windsurf',
    path: '~/.windsurf/mcp.json',
    method: 'copy',
    isValid: true,
    configFile: 'mcp.json',
  },
  {
    id: 'vscode-mcp',
    name: 'VS Code',
    path: '~/.config/Code/User/globalStorage/script(s)/mcp/settings.json',
    method: 'copy',
    isValid: true,
    configFile: 'settings.json',
  },
];

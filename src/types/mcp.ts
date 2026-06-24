// =============================================================================
// MCP Types - Extended for FEAT-022 Universal MCP Servers Manager
// =============================================================================

// --- Enums ---

export type AuthType = 'none' | 'bearer' | 'api-key';
export type ServerProtocol = 'stdio' | 'sse' | 'http';
export type MCPHealthStatus = 'online' | 'offline' | 'error' | 'checking';

// --- Core Service Types ---

export interface MCPService {
  id: string;
  softwareId: string;
  name: string;
  endpoint: string;
  authType: AuthType;
  config: string;
  isHealthy: boolean;
  lastChecked: string;
  // Extended fields for FEAT-022
  protocol: ServerProtocol;
  port?: number;
  groupIds: string[];
  tags: string[];
  createdAt: string;
  updatedAt: string;
  // Discovery cache
  discoveryCache?: MCPDiscoveryCache;
}

export interface MCPServiceDetail extends MCPService {
  healthHistory: MCPHealthRecord[];
  recentActivity: MCPAuditEntry[];
}

export interface MCPDiscoveryCache {
  tools: MCPTool[];
  resources: MCPResource[];
  prompts: MCPPrompt[];
  cachedAt: string;
  expiresAt: string;
}

// --- Tool, Resource, Prompt Types ---

export interface MCPTool {
  name: string;
  description?: string;
  inputSchema: JSONSchema;
}

export interface MCPResource {
  uri: string;
  name?: string;
  description?: string;
  mimeType?: string;
}

export interface MCPPrompt {
  name: string;
  description?: string;
  arguments?: MCPArgument[];
}

export interface MCPArgument {
  name: string;
  description?: string;
  required: boolean;
  default?: unknown;
}

// --- JSON Schema Types ---

export interface JSONSchema {
  type?: string;
  properties?: Record<string, JSONSchemaProperty>;
  required?: string[];
  title?: string;
  description?: string;
  enum?: unknown[];
  default?: unknown;
}

export interface JSONSchemaProperty {
  type?: string;
  title?: string;
  description?: string;
  default?: unknown;
  enum?: unknown[];
  items?: JSONSchemaProperty;
  properties?: Record<string, JSONSchemaProperty>;
}

// --- Health Record Type ---

export interface MCPHealthRecord {
  id: string;
  serviceId: string;
  status: MCPHealthStatus;
  latencyMs?: number;
  errorMessage?: string;
  checkedAt: string;
}

// --- Group Types ---

export interface MCPGroup {
  id: string;
  name: string;
  color: string;
  isVisible: boolean;
  createdAt: string;
  serverCount?: number;
}

// --- Audit Entry Types ---

export type MCPAuditAction = 'create' | 'update' | 'delete' | 'health_check' | 'invoke';
export type MCPAuditStatus = 'success' | 'failure';

export interface MCPAuditEntry {
  id: string;
  actor: string;
  action: MCPAuditAction;
  serviceId?: string;
  serviceName?: string;
  details: string;
  status: MCPAuditStatus;
  createdAt: string;
}

export interface MCPAuditFilters {
  action?: MCPAuditAction | '';
  serviceId?: string;
  serviceName?: string;
  actor?: string;
  dateFrom?: string;
  dateTo?: string;
  status?: MCPAuditStatus | '';
}

export interface MCPAuditPage {
  items: MCPAuditEntry[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

// --- Invocation Types ---

export interface MCPInvocationParams {
  serviceId: string;
  toolName: string;
  args: Record<string, unknown>;
}

export interface MCPInvocationResult {
  success: boolean;
  content?: MCPContentBlock[];
  error?: string;
  durationMs: number;
}

export interface MCPContentBlock {
  type: 'text' | 'image' | 'resource';
  text?: string;
  data?: string;
  mimeType?: string;
  uri?: string;
}

// --- Import/Export Types ---

export type MCPExportFormat = 'json' | 'yaml';

export interface MCPExportData {
  version: '1.0';
  exportedAt: string;
  services: MCPExportService[];
}

export interface MCPExportService {
  name: string;
  endpoint: string;
  protocol: ServerProtocol;
  args?: string[];
  authType: AuthType;
  config: Record<string, unknown>;
  groups?: string[];
  tags?: string[];
}

export interface MCPImportResult {
  imported: number;
  skipped: number;
  overwritten: number;
  errors: string[];
}

export interface MCPServiceFormData {
  name: string;
  endpoint: string;
  port?: number;
  protocol: ServerProtocol;
  authType: AuthType;
  config: string;
  groupIds: string[];
  tags: string[];
}

// --- Legacy Types (from existing codebase) ---

export interface MCPServer {
  id: string;
  name: string;
  description: string;
  protocol: 'stdio' | 'sse' | 'http';
  categories: string[];
  tags: string[];
  author?: string;
  version?: string;
  license?: string;
  stars?: number;
  downloads?: number;
  lastUpdated?: string;
  npmPackage?: string;
  installCommand?: string;
  repository?: string;
  homepage?: string;
  requiredEnvVars?: Array<{
    name: string;
    description?: string;
    required?: boolean;
    defaultValue?: string;
    example?: string;
  }>;
  requiredPermissions?: string[];
  sourceId: string;
}

export interface MCPInstallProgress {
  stage: 'pending' | 'downloading' | 'installing' | 'extracting' | 'syncing' | 'success' | 'failed' | 'conflict';
  progress: number;
  message?: string;
}

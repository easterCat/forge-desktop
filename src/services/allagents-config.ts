/**
 * workspace.yaml 生成器
 *
 * 将 Forge Desktop 的插件/技能/MCP 状态转换为 allagents 的 workspace.yaml 配置。
 * 支持增量更新，保留用户手动修改的部分。
 */

import type {
  UnifiedPlugin,
  UnifiedMCP,
  WorkspaceConfig,
  ClientType,
} from '@/types/unified-plugin';
import { SUPPORTED_CLIENTS } from '@/types/unified-plugin';

// ============================================================================
// 配置生成器
// ============================================================================

export interface ConfigGeneratorOptions {
  /** 工作区根路径 */
  workspacePath: string;
  /** 配置源目录 (相对于工作区) */
  configSource?: string;
  /** 同步模式 */
  syncMode?: 'copy' | 'symlink';
}

/**
 * 从 UnifiedPlugin 和 UnifiedMCP 列表生成 workspace.yaml 配置
 */
export function generateWorkspaceConfig(
  plugins: UnifiedPlugin[],
  mcpServers: UnifiedMCP[],
  targetClients: ClientType[],
  options: ConfigGeneratorOptions = { workspacePath: '.' }
): WorkspaceConfig {
  const config: WorkspaceConfig = {
    workspace: {
      source: options.configSource ?? '.forge/config',
      files: generateWorkspaceFiles(plugins),
    },
    plugins: generatePluginEntries(plugins),
    clients: targetClients.length > 0 ? targetClients : [...SUPPORTED_CLIENTS],
    mcpServers: generateMcpEntries(mcpServers),
    syncMode: options.syncMode ?? 'copy',
  };

  return config;
}

// ============================================================================
// 插件条目生成
// ============================================================================

function generatePluginEntries(
  plugins: UnifiedPlugin[]
): WorkspaceConfig['plugins'] {
  const entries: WorkspaceConfig['plugins'] = [];

  for (const plugin of plugins) {
    if (!plugin.installed || !plugin.allagentsSpec) {
      continue;
    }

    // 基础规格字符串
    const spec = plugin.allagentsSpec;

    // 如果有客户端过滤或技能过滤，使用详细格式
    if (
      plugin.targetClients.length > 0 ||
      (plugin.type === 'skill' && plugin.parentPlugin)
    ) {
      const entry: Record<string, unknown> = {
        name: spec,
      };

      if (plugin.targetClients.length > 0) {
        entry.clients = plugin.targetClients;
      }

      if (plugin.installMode) {
        entry.install = plugin.installMode;
      }

      entries?.push(entry as any);
    } else {
      // 简单格式
      entries?.push(spec);
    }
  }

  return entries;
}

// ============================================================================
// MCP 条目生成
// ============================================================================

function generateMcpEntries(
  mcpServers: UnifiedMCP[]
): WorkspaceConfig['mcpServers'] {
  const entries: Record<string, NonNullable<WorkspaceConfig['mcpServers']>[string]> = {};

  for (const server of mcpServers) {
    entries[server.name] = {
      type: server.transport,
      url: server.url,
      command: server.command,
      args: server.args,
      env: server.env,
      headers: server.headers,
      clients: server.clients,
    };
  }

  return Object.keys(entries).length > 0 ? entries : undefined;
}

// ============================================================================
// 工作区文件生成
// ============================================================================

function generateWorkspaceFiles(
  plugins: UnifiedPlugin[]
): NonNullable<WorkspaceConfig['workspace']>['files'] {
  const files: NonNullable<WorkspaceConfig['workspace']>['files'] = [];

  // 基础规则文件
  files.push('AGENTS.md');
  files.push('CLAUDE.md');

  // 检查是否有 rules 类型的插件需要同步
  const rules = plugins.filter(p => p.type === 'rule' && p.installed);
  for (const rule of rules) {
    if (rule.ruleType === 'md' && rule.installedPath) {
      files.push({
        source: rule.installedPath,
        dest: rule.name,
      });
    }
  }

  return files.length > 0 ? files : undefined;
}

// ============================================================================
// 增量更新
// ============================================================================

/**
 * 合并新旧配置，保留用户手动修改的部分
 */
export function mergeWorkspaceConfigs(
  existing: WorkspaceConfig,
  updates: Partial<WorkspaceConfig>
): WorkspaceConfig {
  const merged = { ...existing };

  // 合并 workspace 部分
  if (updates.workspace) {
    merged.workspace = {
      ...merged.workspace,
      ...updates.workspace,
    };
  }

  // 合并插件列表（去重）
  if (updates.plugins) {
    const existingSpecs = new Set(
      (merged.plugins ?? []).map(p =>
        typeof p === 'string' ? p : p.name
      )
    );

    const newPlugins = (updates.plugins ?? []).filter(p => {
      const spec = typeof p === 'string' ? p : p.name;
      return !existingSpecs.has(spec);
    });

    merged.plugins = [...(merged.plugins ?? []), ...newPlugins];
  }

  // 合并客户端列表（去重）
  if (updates.clients) {
    const existingSet = new Set(merged.clients ?? []);
    const newClients = (updates.clients ?? []).filter(c => !existingSet.has(c));
    merged.clients = [...(merged.clients ?? []), ...newClients];
  }

  // 合并 MCP 服务器（覆盖同名）
  if (updates.mcpServers) {
    merged.mcpServers = {
      ...merged.mcpServers,
      ...updates.mcpServers,
    };
  }

  return merged;
}

// ============================================================================
// 验证
// ============================================================================

export interface ConfigValidationError {
  field: string;
  message: string;
  severity: 'error' | 'warning';
}

/**
 * 验证 workspace.yaml 配置
 */
export function validateWorkspaceConfig(
  config: WorkspaceConfig
): ConfigValidationError[] {
  const errors: ConfigValidationError[] = [];

  // 验证客户端列表
  if (config.clients) {
    for (const client of config.clients) {
      if (!SUPPORTED_CLIENTS.includes(client as ClientType)) {
        errors.push({
          field: `clients.${client}`,
          message: `Unknown client: ${client}`,
          severity: 'warning',
        });
      }
    }
  }

  // 验证 MCP 服务器
  if (config.mcpServers) {
    for (const [name, server] of Object.entries(config.mcpServers)) {
      if (server.type === 'http' && !server.url) {
        errors.push({
          field: `mcpServers.${name}.url`,
          message: `HTTP MCP server "${name}" requires a URL`,
          severity: 'error',
        });
      }
      if (server.type === 'stdio' && !server.command) {
        errors.push({
          field: `mcpServers.${name}.command`,
          message: `Stdio MCP server "${name}" requires a command`,
          severity: 'error',
        });
      }
    }
  }

  return errors;
}

// ============================================================================
// YAML 序列化（简化版，生产环境应使用 js-yaml）
// ============================================================================

/**
 * 将配置转换为 YAML 字符串
 * 注意: 这是一个简化实现，生产环境应使用 js-yaml 库
 */
export function configToYaml(config: WorkspaceConfig): string {
  const lines: string[] = [];

  // workspace 部分
  if (config.workspace) {
    lines.push('workspace:');
    if (config.workspace.source) {
      lines.push(`  source: ${config.workspace.source}`);
    }
    if (config.workspace.files && config.workspace.files.length > 0) {
      lines.push('  files:');
      for (const file of config.workspace.files) {
        if (typeof file === 'string') {
          lines.push(`    - ${file}`);
        } else {
          lines.push(`    - source: ${file.source}`);
          if (file.dest) {
            lines.push(`      dest: ${file.dest}`);
          }
        }
      }
    }
    lines.push('');
  }

  // plugins 部分
  if (config.plugins && config.plugins.length > 0) {
    lines.push('plugins:');
    for (const plugin of config.plugins) {
      if (typeof plugin === 'string') {
        lines.push(`  - ${plugin}`);
      } else {
        lines.push(`  - name: ${plugin.name}`);
        if (plugin.clients && plugin.clients.length > 0) {
          lines.push(`    clients: [${plugin.clients.join(', ')}]`);
        }
      }
    }
    lines.push('');
  }

  // clients 部分
  if (config.clients && config.clients.length > 0) {
    lines.push('clients:');
    for (const client of config.clients) {
      lines.push(`  - ${client}`);
    }
    lines.push('');
  }

  // mcpServers 部分
  if (config.mcpServers && Object.keys(config.mcpServers).length > 0) {
    lines.push('mcpServers:');
    for (const [name, server] of Object.entries(config.mcpServers)) {
      lines.push(`  ${name}:`);
      lines.push(`    type: ${server.type}`);
      if (server.url) {
        lines.push(`    url: ${server.url}`);
      }
      if (server.command) {
        lines.push(`    command: ${server.command}`);
      }
      if (server.args) {
        lines.push(`    args:`);
        for (const arg of server.args) {
          lines.push(`      - ${arg}`);
        }
      }
      if (server.env) {
        lines.push(`    env:`);
        for (const [key, value] of Object.entries(server.env)) {
          lines.push(`      ${key}: ${value}`);
        }
      }
    }
    lines.push('');
  }

  // syncMode 部分
  if (config.syncMode) {
    lines.push(`syncMode: ${config.syncMode}`);
    lines.push('');
  }

  return lines.join('\n');
}

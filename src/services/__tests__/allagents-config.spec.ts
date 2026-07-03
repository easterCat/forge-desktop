import { describe, it, expect } from 'vitest';
import {
  generateWorkspaceConfig,
  mergeWorkspaceConfigs,
  validateWorkspaceConfig,
  configToYaml,
} from '@/services/allagents-config';
import type {
  UnifiedPlugin,
  UnifiedMCP,
  WorkspaceConfig,
} from '@/types/unified-plugin';

describe('allagents-config', () => {
  describe('generateWorkspaceConfig', () => {
    it('should generate config with clients', () => {
      const config = generateWorkspaceConfig([], [], ['claude', 'cursor']);
      expect(config.clients).toEqual(['claude', 'cursor']);
    });

    it('should generate config with plugins', () => {
      const plugins: UnifiedPlugin[] = [
        {
          id: '1',
          name: 'code-review',
          source: { type: 'marketplace', marketplace: 'official' },
          scope: 'project',
          type: 'skill',
          tags: [],
          categories: [],
          installed: true,
          enabled: true,
          syncTargets: [],
          syncStatus: 'synced',
          targetClients: [],
          allagentsSpec: 'code-review@official',
        },
      ];

      const config = generateWorkspaceConfig(plugins, [], ['claude']);
      expect(config.plugins).toBeDefined();
      expect(config.plugins!.length).toBe(1);
    });

    it('should generate config with MCP servers', () => {
      const mcpServers: UnifiedMCP[] = [
        {
          name: 'test-server',
          transport: 'http',
          url: 'https://example.com/mcp',
          groupIds: [],
          tags: [],
          healthStatus: 'unknown',
          auditLog: [],
        },
      ];

      const config = generateWorkspaceConfig([], mcpServers, ['claude']);
      expect(config.mcpServers).toBeDefined();
      expect(config.mcpServers!['test-server']).toBeDefined();
      expect(config.mcpServers!['test-server'].type).toBe('http');
      expect(config.mcpServers!['test-server'].url).toBe('https://example.com/mcp');
    });

    it('should default to copy sync mode', () => {
      const config = generateWorkspaceConfig([], [], ['claude']);
      expect(config.syncMode).toBe('copy');
    });
  });

  describe('mergeWorkspaceConfigs', () => {
    it('should merge plugins without duplicates', () => {
      const existing: WorkspaceConfig = {
        plugins: ['plugin-a@source'],
      };

      const updates: Partial<WorkspaceConfig> = {
        plugins: ['plugin-a@source', 'plugin-b@source'],
      };

      const merged = mergeWorkspaceConfigs(existing, updates);
      expect(merged.plugins!.length).toBe(2);
    });

    it('should merge clients without duplicates', () => {
      const existing: WorkspaceConfig = {
        clients: ['claude', 'cursor'],
      };

      const updates: Partial<WorkspaceConfig> = {
        clients: ['cursor', 'copilot'],
      };

      const merged = mergeWorkspaceConfigs(existing, updates);
      expect(merged.clients).toEqual(['claude', 'cursor', 'copilot']);
    });

    it('should merge MCP servers (overwrite same name)', () => {
      const existing: WorkspaceConfig = {
        mcpServers: {
          'server-a': { type: 'http', url: 'https://old.com' },
        },
      };

      const updates: Partial<WorkspaceConfig> = {
        mcpServers: {
          'server-a': { type: 'http', url: 'https://new.com' },
          'server-b': { type: 'stdio', command: 'npx' },
        },
      };

      const merged = mergeWorkspaceConfigs(existing, updates);
      expect(merged.mcpServers!['server-a'].url).toBe('https://new.com');
      expect(merged.mcpServers!['server-b'].command).toBe('npx');
    });
  });

  describe('validateWorkspaceConfig', () => {
    it('should return no errors for valid config', () => {
      const config: WorkspaceConfig = {
        clients: ['claude', 'cursor'],
        mcpServers: {
          'server-a': { type: 'http', url: 'https://example.com' },
        },
      };

      const errors = validateWorkspaceConfig(config);
      expect(errors.filter(e => e.severity === 'error').length).toBe(0);
    });

    it('should warn about unknown clients', () => {
      const config: WorkspaceConfig = {
        clients: ['unknown-client'],
      };

      const errors = validateWorkspaceConfig(config);
      expect(errors.some(e => e.message.includes('unknown-client'))).toBe(true);
    });

    it('should error on HTTP MCP without URL', () => {
      const config: WorkspaceConfig = {
        mcpServers: {
          'server-a': { type: 'http' },
        },
      };

      const errors = validateWorkspaceConfig(config);
      expect(errors.some(e => e.severity === 'error' && e.message.includes('URL'))).toBe(true);
    });

    it('should error on stdio MCP without command', () => {
      const config: WorkspaceConfig = {
        mcpServers: {
          'server-a': { type: 'stdio' },
        },
      };

      const errors = validateWorkspaceConfig(config);
      expect(errors.some(e => e.severity === 'error' && e.message.includes('command'))).toBe(true);
    });
  });

  describe('configToYaml', () => {
    it('should generate valid YAML for simple config', () => {
      const config: WorkspaceConfig = {
        clients: ['claude', 'cursor'],
        syncMode: 'copy',
      };

      const yaml = configToYaml(config);
      expect(yaml).toContain('clients:');
      expect(yaml).toContain('- claude');
      expect(yaml).toContain('- cursor');
      expect(yaml).toContain('syncMode: copy');
    });

    it('should generate YAML with plugins', () => {
      const config: WorkspaceConfig = {
        plugins: ['code-review@official'],
      };

      const yaml = configToYaml(config);
      expect(yaml).toContain('plugins:');
      expect(yaml).toContain('- code-review@official');
    });

    it('should generate YAML with MCP servers', () => {
      const config: WorkspaceConfig = {
        mcpServers: {
          'test-server': {
            type: 'http',
            url: 'https://example.com/mcp',
          },
        },
      };

      const yaml = configToYaml(config);
      expect(yaml).toContain('mcpServers:');
      expect(yaml).toContain('test-server:');
      expect(yaml).toContain('type: http');
      expect(yaml).toContain('url: https://example.com/mcp');
    });

    it('should generate YAML with workspace files', () => {
      const config: WorkspaceConfig = {
        workspace: {
          source: '.forge/config',
          files: ['AGENTS.md', { source: 'docs/guide.md', dest: 'GUIDE.md' }],
        },
      };

      const yaml = configToYaml(config);
      expect(yaml).toContain('workspace:');
      expect(yaml).toContain('source: .forge/config');
      expect(yaml).toContain('- AGENTS.md');
      expect(yaml).toContain('source: docs/guide.md');
    });
  });
});

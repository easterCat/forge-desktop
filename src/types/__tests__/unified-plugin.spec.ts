import { describe, it, expect } from 'vitest';
import {
  SUPPORTED_CLIENTS,
  CLIENT_DISPLAY_NAMES,
  toAllagentsSpec,
  parseAllagentsSpec,
  getPluginDisplayTags,
} from '@/types/unified-plugin';
import type { UnifiedPlugin, PluginSource } from '@/types/unified-plugin';

describe('unified-plugin types', () => {
  describe('SUPPORTED_CLIENTS', () => {
    it('should contain 23 clients', () => {
      expect(SUPPORTED_CLIENTS.length).toBe(23);
    });

    it('should include core clients', () => {
      expect(SUPPORTED_CLIENTS).toContain('claude');
      expect(SUPPORTED_CLIENTS).toContain('copilot');
      expect(SUPPORTED_CLIENTS).toContain('cursor');
      expect(SUPPORTED_CLIENTS).toContain('codex');
    });

    it('should have display names for all clients', () => {
      for (const client of SUPPORTED_CLIENTS) {
        expect(CLIENT_DISPLAY_NAMES[client]).toBeDefined();
        expect(CLIENT_DISPLAY_NAMES[client].length).toBeGreaterThan(0);
      }
    });
  });

  describe('toAllagentsSpec', () => {
    it('should generate spec from explicit allagentsSpec', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'test',
        source: { type: 'marketplace' },
        scope: 'project',
        type: 'skill',
        tags: [],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
        allagentsSpec: 'custom-spec@custom-source',
      };

      expect(toAllagentsSpec(plugin)).toBe('custom-spec@custom-source');
    });

    it('should generate spec from github source', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'my-skill',
        source: { type: 'github', repo: 'owner/repo' },
        scope: 'project',
        type: 'skill',
        tags: [],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
      };

      expect(toAllagentsSpec(plugin)).toBe('my-skill@owner/repo');
    });

    it('should generate spec from marketplace source', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'code-review',
        source: { type: 'marketplace', marketplace: 'claude-plugins-official' },
        scope: 'project',
        type: 'skill',
        tags: [],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
      };

      expect(toAllagentsSpec(plugin)).toBe('code-review@claude-plugins-official');
    });

    it('should fallback to name only', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'local-skill',
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
      };

      expect(toAllagentsSpec(plugin)).toBe('local-skill');
    });
  });

  describe('parseAllagentsSpec', () => {
    it('should parse spec with source', () => {
      const result = parseAllagentsSpec('plugin@owner/repo');
      expect(result.name).toBe('plugin');
      expect(result.source).toBe('owner/repo');
    });

    it('should parse spec without source', () => {
      const result = parseAllagentsSpec('plugin');
      expect(result.name).toBe('plugin');
      expect(result.source).toBeUndefined();
    });

    it('should handle empty string', () => {
      const result = parseAllagentsSpec('');
      expect(result.name).toBe('');
      expect(result.source).toBeUndefined();
    });
  });

  describe('getPluginDisplayTags', () => {
    it('should prepend type tag', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'test',
        source: { type: 'local' },
        scope: 'project',
        type: 'skill',
        tags: ['coding', 'test'],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
      };

      const displayTags = getPluginDisplayTags(plugin);
      expect(displayTags[0]).toBe('Skill');
      expect(displayTags).toContain('coding');
      expect(displayTags).toContain('test');
    });

    it('should handle agent type', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'test',
        source: { type: 'local' },
        scope: 'project',
        type: 'agent',
        tags: [],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
      };

      const displayTags = getPluginDisplayTags(plugin);
      expect(displayTags[0]).toBe('Agent');
    });

    it('should handle mcp type', () => {
      const plugin: UnifiedPlugin = {
        id: '1',
        name: 'test',
        source: { type: 'local' },
        scope: 'project',
        type: 'mcp',
        tags: [],
        categories: [],
        installed: true,
        enabled: true,
        syncTargets: [],
        syncStatus: 'synced',
        targetClients: [],
      };

      const displayTags = getPluginDisplayTags(plugin);
      expect(displayTags[0]).toBe('MCP');
    });
  });
});

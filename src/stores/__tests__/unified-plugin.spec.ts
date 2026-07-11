import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUnifiedPluginStore } from '../unified-plugin';
import type { UnifiedPlugin } from '@/types/unified-plugin';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

function makePlugin(overrides: Partial<UnifiedPlugin> = {}): UnifiedPlugin {
  return {
    id: 'p1',
    name: 'test-plugin',
    type: 'skill',
    source: { type: 'marketplace', marketplace: 'anthropic' },
    scope: 'user',
    installed: false,
    enabled: true,
    tags: [],
    categories: [],
    syncStatus: 'unknown',
    syncTargets: [],
    targetClients: [],
    ...overrides,
  };
}

describe('useUnifiedPluginStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('defaults to empty plugins and MCP servers', () => {
    const store = useUnifiedPluginStore();
    expect(store.plugins).toEqual([]);
    expect(store.mcpServers).toEqual([]);
  });

  it('pluginsByType returns 6 empty arrays', () => {
    const store = useUnifiedPluginStore();
    const byType = store.pluginsByType;
    expect(Object.keys(byType)).toHaveLength(6);
    expect(byType.skill).toEqual([]);
    expect(byType.agent).toEqual([]);
  });

  it('pluginsByType groups by type', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', type: 'skill' }),
      makePlugin({ id: '2', type: 'agent' }),
      makePlugin({ id: '3', type: 'skill' }),
    ];
    expect(store.pluginsByType.skill).toHaveLength(2);
    expect(store.pluginsByType.agent).toHaveLength(1);
  });

  it('installedPlugins filters installed', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', installed: true }),
      makePlugin({ id: '2', installed: false }),
      makePlugin({ id: '3', installed: true }),
    ];
    expect(store.installedPlugins).toHaveLength(2);
  });

  it('installedSkills filters type=skill and installed', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', type: 'skill', installed: true }),
      makePlugin({ id: '2', type: 'skill', installed: false }),
      makePlugin({ id: '3', type: 'agent', installed: true }),
    ];
    expect(store.installedSkills).toHaveLength(1);
  });

  it('installedAgents filters type=agent and installed', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', type: 'agent', installed: true }),
      makePlugin({ id: '2', type: 'agent', installed: false }),
    ];
    expect(store.installedAgents).toHaveLength(1);
  });

  it('installedRules filters type=rule and installed', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', type: 'rule', installed: true }),
      makePlugin({ id: '2', type: 'rule', installed: false }),
    ];
    expect(store.installedRules).toHaveLength(1);
  });

  it('syncStats counts statuses correctly', () => {
    const store = useUnifiedPluginStore();
    store.plugins = [
      makePlugin({ id: '1', syncStatus: 'synced' }),
      makePlugin({ id: '2', syncStatus: 'synced' }),
      makePlugin({ id: '3', syncStatus: 'pending' }),
      makePlugin({ id: '4', syncStatus: 'error' }),
      makePlugin({ id: '5', syncStatus: 'conflict' }),
    ];
    const stats = store.syncStats;
    expect(stats.synced).toBe(2);
    expect(stats.pending).toBe(1);
    expect(stats.errors).toBe(1);
    expect(stats.conflict).toBe(1);
  });

  it('currentConfig returns generated config', () => {
    const store = useUnifiedPluginStore();
    const config = store.currentConfig;
    expect(config).toBeDefined();
    expect(config).toHaveProperty('workspace');
    expect(config).toHaveProperty('plugins');
  });

  describe('initWorkspace', () => {
    it('sets isLoading then false on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      expect(store.isLoading).toBe(false);
      const p = store.initWorkspace('/test/path');
      expect(store.isLoading).toBe(true);
      await p;
      expect(store.isLoading).toBe(false);
    });

    it('sets workspacePath on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      await store.initWorkspace('/test/path');
      expect(store.workspacePath).toBe('/test/path');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: false, error: 'init failed', data: null });
      const store = useUnifiedPluginStore();
      const result = await store.initWorkspace('/test');
      expect(result).toBe(false);
      expect(store.error).toBe('init failed');
    });

    it(' throws and sets error on exception', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('network error'));
      const store = useUnifiedPluginStore();
      await store.initWorkspace('/test');
      expect(store.error).toBeTruthy();
    });
  });

  describe('syncAll', () => {
    it('sets syncing flag', async () => {
      let resolve: (v: any) => void;
      vi.mocked(invoke).mockImplementation(() => new Promise(r => (resolve = r)));
      const store = useUnifiedPluginStore();
      const p = store.syncAll();
      expect(store.syncStatus.syncing).toBe(true);
      resolve!({ success: true, data: { synced_count: 0, error_count: 0 } });
      await p;
      expect(store.syncStatus.syncing).toBe(false);
    });

    it('updates syncStatus counts on success', async () => {
      vi.mocked(invoke).mockResolvedValue({
        success: true,
        data: { synced_count: 5, error_count: 1 },
        error: null,
      });
      const store = useUnifiedPluginStore();
      store.plugins = [makePlugin({ id: '1' })];
      await store.syncAll();
      expect(store.syncStatus.syncedCount).toBe(5);
      expect(store.syncStatus.errorCount).toBe(1);
    });

    it('sets plugin syncStatus to partial on errors', async () => {
      vi.mocked(invoke).mockResolvedValue({
        success: true,
        data: { synced_count: 5, error_count: 1 },
        error: null,
      });
      const store = useUnifiedPluginStore();
      store.plugins = [makePlugin({ id: '1', syncStatus: 'unknown' })];
      await store.syncAll();
      expect(store.plugins[0].syncStatus).toBe('partial');
    });

    it('sets plugin syncStatus to synced on clean sync', async () => {
      vi.mocked(invoke).mockResolvedValue({
        success: true,
        data: { synced_count: 5, error_count: 0 },
        error: null,
      });
      const store = useUnifiedPluginStore();
      store.plugins = [makePlugin({ id: '1', syncStatus: 'unknown' })];
      await store.syncAll();
      expect(store.plugins[0].syncStatus).toBe('synced');
    });

    it('sets error and returns false on failed sync', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: false, error: 'sync error', data: null });
      const store = useUnifiedPluginStore();
      const result = await store.syncAll();
      expect(result).toBe(false);
      expect(store.error).toBe('sync error');
    });
  });

  describe('installPlugin', () => {
    it('marks plugin installed on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      const plugin = makePlugin();
      await store.installPlugin(plugin);
      expect(plugin.installed).toBe(true);
      expect(plugin.syncStatus).toBe('pending');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: false, error: 'install failed', data: null });
      const store = useUnifiedPluginStore();
      const plugin = makePlugin();
      const result = await store.installPlugin(plugin);
      expect(result).toBe(false);
      expect(store.error).toBe('install failed');
    });
  });

  describe('uninstallPlugin', () => {
    it('marks plugin uninstalled on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      const plugin = makePlugin({ installed: true });
      await store.uninstallPlugin(plugin);
      expect(plugin.installed).toBe(false);
      expect(plugin.syncStatus).toBe('pending');
    });
  });

  describe('togglePlugin', () => {
    it('flips enabled flag', () => {
      const store = useUnifiedPluginStore();
      const plugin = makePlugin({ enabled: true });
      store.togglePlugin(plugin);
      expect(plugin.enabled).toBe(false);
    });

    it('marks syncStatus as pending', () => {
      const store = useUnifiedPluginStore();
      const plugin = makePlugin({ syncStatus: 'synced' });
      store.togglePlugin(plugin);
      expect(plugin.syncStatus).toBe('pending');
    });
  });

  describe('addMcpServer', () => {
    it('adds to mcpServers on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      await store.addMcpServer({ name: 'server1', transport: 'stdio', command: 'npx' });
      expect(store.mcpServers).toHaveLength(1);
      expect(store.mcpServers[0].name).toBe('server1');
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: false, error: 'add failed', data: null });
      const store = useUnifiedPluginStore();
      await store.addMcpServer({ name: 'server1', transport: 'stdio', command: 'npx' });
      expect(store.error).toBe('add failed');
    });
  });

  describe('removeMcpServer', () => {
    it('removes from mcpServers on success', async () => {
      vi.mocked(invoke).mockResolvedValue({ success: true, data: null, error: null });
      const store = useUnifiedPluginStore();
      store.mcpServers = [{ name: 'server1', transport: 'stdio', command: 'npx', groupIds: [], tags: [], healthStatus: 'unknown', auditLog: [] }];
      await store.removeMcpServer('server1');
      expect(store.mcpServers).toHaveLength(0);
    });
  });

  describe('setTargetClients', () => {
    it('updates targetClients', () => {
      const store = useUnifiedPluginStore();
      store.setTargetClients(['claude', 'cursor']);
      expect(store.targetClients).toEqual(['claude', 'cursor']);
    });
  });

  describe('clearError', () => {
    it('clears error', () => {
      const store = useUnifiedPluginStore();
      store.error = 'some error';
      store.clearError();
      expect(store.error).toBeNull();
    });
  });

  describe('loadStatus', () => {
    it('updates targetClients from backend', async () => {
      vi.mocked(invoke).mockResolvedValue({
        success: true,
        data: { clients: ['claude'] },
        error: null,
      });
      const store = useUnifiedPluginStore();
      await store.loadStatus();
      expect(store.targetClients).toContain('claude');
    });
  });

  describe('generateConfig', () => {
    it('returns YAML string', async () => {
      const store = useUnifiedPluginStore();
      const yaml = await store.generateConfig();
      expect(typeof yaml).toBe('string');
      expect(yaml).toContain('workspace:');
    });
  });
});

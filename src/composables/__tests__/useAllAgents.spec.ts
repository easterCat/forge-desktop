import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

// Store-level mock: must use vi.hoisted so it's evaluated before vi.mock hoisting
const { mockInstallPlugin, mockUninstallPlugin, mockAddMcpServer, mockRemoveMcpServer, mockSyncAll, mockClearError, mockSetTargetClients, mockPlugins, mockTargetClients } = vi.hoisted(() => {
  const plugins: unknown[] = [];
  const targetClients: string[] = ['claude', 'cursor'];
  return {
    mockPlugins: plugins,
    mockTargetClients: targetClients,
    mockInstallPlugin: vi.fn().mockResolvedValue(true),
    mockUninstallPlugin: vi.fn().mockResolvedValue(true),
    mockAddMcpServer: vi.fn().mockResolvedValue(true),
    mockRemoveMcpServer: vi.fn().mockResolvedValue(true),
    mockSyncAll: vi.fn().mockResolvedValue(true),
    mockClearError: vi.fn(),
    mockSetTargetClients: vi.fn(),
  };
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({ success: true, data: null, error: null }),
}));

vi.mock('@/stores/unified-plugin', () => ({
  useUnifiedPluginStore: () => ({
    plugins: mockPlugins,
    mcpServers: [],
    isLoading: false,
    error: null,
    syncStatus: {
      syncing: false,
      lastSyncAt: undefined,
      syncedCount: 0,
      errorCount: 0,
    },
    syncStats: { synced: 0, pending: 0, errors: 0, conflict: 0 },
    targetClients: mockTargetClients,
    pluginsByType: { skill: [], agent: [], rule: [], mcp: [], hook: [], command: [] },
    installedPlugins: [],
    installedSkills: [],
    installedAgents: [],
    installedRules: [],
    currentConfig: {},
    installPlugin: mockInstallPlugin,
    uninstallPlugin: mockUninstallPlugin,
    addMcpServer: mockAddMcpServer,
    removeMcpServer: mockRemoveMcpServer,
    syncAll: mockSyncAll,
    clearError: mockClearError,
    setTargetClients: mockSetTargetClients,
  }),
}));

import { useAllAgents } from '../useAllAgents';

describe('useAllAgents', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mockPlugins.length = 0;
    mockTargetClients.length = 0;
    mockTargetClients.push('claude', 'cursor');
    mockInstallPlugin.mockResolvedValue(true);
    mockUninstallPlugin.mockResolvedValue(true);
    mockAddMcpServer.mockResolvedValue(true);
    mockRemoveMcpServer.mockResolvedValue(true);
    mockSyncAll.mockResolvedValue(true);
    mockClearError.mockClear();
    mockSetTargetClients.mockClear();
  });

  it('returns computed isLoading, error, syncStatus, syncStats', () => {
    const { isLoading, error, syncStatus } = useAllAgents();
    expect(isLoading.value).toBeDefined();
    expect(error.value).toBeDefined();
    expect(syncStatus.value).toBeDefined();
  });

  it('searchPlugins returns empty when no plugins match', () => {
    const { searchPlugins } = useAllAgents();
    expect(searchPlugins('test')).toEqual([]);
  });

  it('installAndSync calls store and syncs', async () => {
    const { installAndSync } = useAllAgents();
    await installAndSync({ id: 'p1', name: 'test', type: 'skill', installed: false, enabled: false, syncStatus: 'pending', targetClients: [], tags: [] } as any);
    expect(mockInstallPlugin).toHaveBeenCalled();
    expect(mockSyncAll).toHaveBeenCalled();
  });

  it('installAndSync skips sync when autoSync is false', async () => {
    const { installAndSync } = useAllAgents();
    await installAndSync({ id: 'p1', name: 'test', type: 'skill', installed: false, enabled: false, syncStatus: 'pending', targetClients: [], tags: [] } as any, { autoSync: false });
    expect(mockInstallPlugin).toHaveBeenCalled();
    expect(mockSyncAll).not.toHaveBeenCalled();
  });

  it('uninstallAndSync calls store and syncs', async () => {
    const { uninstallAndSync } = useAllAgents();
    await uninstallAndSync({ id: 'p1', name: 'test', type: 'skill', installed: false, enabled: false, syncStatus: 'pending', targetClients: [], tags: [] } as any);
    expect(mockUninstallPlugin).toHaveBeenCalled();
    expect(mockSyncAll).toHaveBeenCalled();
  });

  it('addMcpAndSync calls store and syncs', async () => {
    const { addMcpAndSync } = useAllAgents();
    const server = { name: 'test-server', transport: 'stdio' as const };
    await addMcpAndSync(server);
    expect(mockAddMcpServer).toHaveBeenCalled();
    expect(mockSyncAll).toHaveBeenCalled();
  });

  it('removeMcpAndSync calls store and syncs', async () => {
    const { removeMcpAndSync } = useAllAgents();
    await removeMcpAndSync('test-server');
    expect(mockRemoveMcpServer).toHaveBeenCalledWith('test-server');
    expect(mockSyncAll).toHaveBeenCalled();
  });

  it('toggleClient adds client when not selected', () => {
    const { toggleClient } = useAllAgents();
    mockTargetClients.length = 0;
    mockTargetClients.push('claude');
    toggleClient('cursor');
    expect(mockSetTargetClients).toHaveBeenCalled();
  });

  it('toggleClient removes client when already selected', () => {
    const { toggleClient } = useAllAgents();
    mockTargetClients.length = 0;
    mockTargetClients.push('claude', 'cursor');
    toggleClient('claude');
    expect(mockSetTargetClients).toHaveBeenCalled();
  });

  it('selectAllClients sets all supported clients', () => {
    const { selectAllClients } = useAllAgents();
    selectAllClients();
    expect(mockSetTargetClients).toHaveBeenCalled();
  });

  it('deselectAllClients clears all clients', () => {
    const { deselectAllClients } = useAllAgents();
    deselectAllClients();
    expect(mockSetTargetClients).toHaveBeenCalledWith([]);
  });

  it('sync calls store.syncAll', async () => {
    const { sync } = useAllAgents();
    mockSyncAll.mockResolvedValue(true);
    await sync({ offline: true });
    expect(mockSyncAll).toHaveBeenCalledWith({ offline: true });
  });

  it('previewSync calls store.syncAll with dryRun', async () => {
    const { previewSync } = useAllAgents();
    mockSyncAll.mockResolvedValue({ syncedCount: 0, errorCount: 0 });
    const result = await previewSync();
    expect(mockSyncAll).toHaveBeenCalledWith({ dryRun: true });
    expect(result).toHaveProperty('syncedCount');
    expect(result).toHaveProperty('errorCount');
  });

  it('clearError calls store.clearError', () => {
    const { clearError } = useAllAgents();
    clearError();
    expect(mockClearError).toHaveBeenCalled();
  });
});

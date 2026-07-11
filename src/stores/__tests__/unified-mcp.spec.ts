import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUnifiedMcpStore } from '../unified-mcp';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('useUnifiedMcpStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('starts with empty servers', () => {
    const store = useUnifiedMcpStore();
    expect(store.servers).toEqual([]);
    expect(store.groups).toEqual([]);
    expect(store.isLoading).toBe(false);
    expect(store.isSyncing).toBe(false);
    expect(store.error).toBeNull();
  });

  it('serversByTransport groups by http and stdio', () => {
    const store = useUnifiedMcpStore();
    store.servers = [
      { name: 's1', transport: 'http' as const, url: 'https://x.com', groupIds: [], tags: [], healthStatus: 'unknown', auditLog: [] },
      { name: 's2', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: [], healthStatus: 'unknown', auditLog: [] },
    ];
    expect(store.serversByTransport.http).toHaveLength(1);
    expect(store.serversByTransport.stdio).toHaveLength(1);
  });

  it('filteredServers applies search keyword', () => {
    const store = useUnifiedMcpStore();
    store.servers = [
      { name: 'filesystem', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: ['fs', 'disk'], healthStatus: 'unknown', auditLog: [] },
      { name: 'github-api', transport: 'http' as const, url: 'https://x.com', groupIds: [], tags: ['api'], healthStatus: 'unknown', auditLog: [] },
    ];
    store.searchKeyword = 'fs';
    expect(store.filteredServers).toHaveLength(1);
    expect(store.filteredServers[0].name).toBe('filesystem');
    store.searchKeyword = '';
    expect(store.filteredServers).toHaveLength(2);
  });

  it('healthStats counts statuses correctly', () => {
    const store = useUnifiedMcpStore();
    store.servers = [
      { name: 's1', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: [], healthStatus: 'healthy', auditLog: [] },
      { name: 's2', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: [], healthStatus: 'healthy', auditLog: [] },
      { name: 's3', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: [], healthStatus: 'unhealthy', auditLog: [] },
      { name: 's4', transport: 'stdio' as const, command: 'npx', groupIds: [], tags: [], healthStatus: 'unknown', auditLog: [] },
    ];
    const stats = store.healthStats;
    expect(stats.healthy).toBe(2);
    expect(stats.unhealthy).toBe(1);
    expect(stats.unknown).toBe(1);
    expect(stats.total).toBe(4);
  });

  it('groupStats counts servers per group', () => {
    const store = useUnifiedMcpStore();
    store.groups = [
      { id: 'g1', name: 'dev-tools', serverNames: ['s1', 's2'] },
      { id: 'g2', name: 'ai', serverNames: ['s3'] },
    ];
    const stats = store.groupStats;
    expect(stats['dev-tools']).toBe(2);
    expect(stats['ai']).toBe(1);
  });

  it('clearError sets error to null', () => {
    const store = useUnifiedMcpStore();
    store.error = 'some error';
    store.clearError();
    expect(store.error).toBeNull();
  });
});

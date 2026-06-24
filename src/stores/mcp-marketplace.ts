// MCP Marketplace Store - State management for MCP server marketplace

import { defineStore } from 'pinia';
import { ref, computed, shallowRef } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  MCPSource,
  MCPServer,
  PaginatedMCPServers,
  MCPSyncTarget,
  MCPInstallProgress,
  CategoryKey,
} from '@/types';

export const useMCPMarketplaceStore = defineStore('mcpMarketplace', () => {
  // State
  const sources = ref<MCPSource[]>([]);
  const currentSource = ref<MCPSource | null>(null);
  const servers = ref<MCPServer[]>([]);
  const localServers = ref<MCPServer[]>([]);
  const syncTargets = ref<MCPSyncTarget[]>([]);
  
  // Pagination state
  const currentPage = ref(1);
  const pageSize = ref(20);
  const totalServers = ref(0);
  const totalPages = ref(0);
  
  // Filter state
  const selectedCategory = ref<string | null>(null);
  const searchKeyword = ref('');
  
  // Loading states
  const isLoadingSources = ref(false);
  const isLoadingServers = ref(false);
  const isInstalling = ref(false);
  const isSyncing = ref(false);
  
  // Error state
  const error = ref<string | null>(null);
  
  // Install progress tracking - use shallowRef to avoid deep reactivity overhead
  const installProgress = shallowRef<Map<string, MCPInstallProgress>>(new Map());
  
  // Computed
  const hasNextPage = computed(() => currentPage.value < totalPages.value);
  const hasPrevPage = computed(() => currentPage.value > 1);
  
  const installedServerNames = computed(() => 
    new Set(localServers.value.map(s => s.name))
  );
  
  const isServerInstalled = (serverName: string) => 
    installedServerNames.value.has(serverName);
  
  const serversByRegion = computed(() => {
    const grouped: Record<string, MCPSource[]> = {
      'mcp-specific': [],
      'international': [],
      'china': [],
      'github': [],
    };
    
    for (const source of sources.value) {
      if (grouped[source.region]) {
        grouped[source.region].push(source);
      } else {
        grouped['international'].push(source);
      }
    }
    
    return grouped;
  });
  
  // Actions
  async function fetchSources() {
    try {
      isLoadingSources.value = true;
      error.value = null;
      sources.value = await invoke<MCPSource[]>('get_mcp_sources');
      
      // Select first source by default
      if (sources.value.length > 0 && !currentSource.value) {
        currentSource.value = sources.value[0];
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch sources';
      console.error('Failed to fetch sources:', e);
    } finally {
      isLoadingSources.value = false;
    }
  }
  
  async function fetchServers() {
    if (!currentSource.value) return;
    
    try {
      isLoadingServers.value = true;
      error.value = null;
      
      const result = await invoke<PaginatedMCPServers>('fetch_mcp_servers', {
        sourceId: currentSource.value.id,
        page: currentPage.value,
        pageSize: pageSize.value,
        category: selectedCategory.value,
        keyword: searchKeyword.value || null,
      });
      
      servers.value = result.items;
      totalServers.value = result.total;
      totalPages.value = result.totalPages;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch servers';
      console.error('Failed to fetch servers:', e);
    } finally {
      isLoadingServers.value = false;
    }
  }
  
  async function fetchLocalServers() {
    try {
      localServers.value = await invoke<MCPServer[]>('get_local_mcp_servers');
    } catch (e) {
      console.error('Failed to fetch local servers:', e);
    }
  }
  
  function selectSource(source: MCPSource) {
    currentSource.value = source;
    currentPage.value = 1;
    fetchServers();
  }
  
  function setPage(page: number) {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page;
      fetchServers();
    }
  }
  
  function setPageSize(size: number) {
    pageSize.value = size;
    currentPage.value = 1;
    fetchServers();
  }
  
  function setCategory(category: string | null) {
    selectedCategory.value = category;
    currentPage.value = 1;
    fetchServers();
  }
  
  function setSearchKeyword(keyword: string) {
    searchKeyword.value = keyword;
    currentPage.value = 1;
    fetchServers();
  }
  
  async function installServer(server: MCPServer, installDir?: string) {
    const progress: MCPInstallProgress = {
      serverId: server.id,
      serverName: server.name,
      stage: 'downloading',
      progress: 0,
      message: '正在下载 MCP 服务器...',
      startedAt: new Date().toISOString(),
    };
    
    // Create a new Map to trigger shallowRef reactivity
    const newMap = new Map(installProgress.value);
    newMap.set(server.id, progress);
    installProgress.value = newMap;
    isInstalling.value = true;
    
    try {
      // Stage 1: Downloading (0-30%)
      progress.progress = 10;
      progress.message = '正在连接服务器...';
      const map1 = new Map(installProgress.value);
      map1.set(server.id, { ...progress });
      installProgress.value = map1;
      
      const result = await invoke<{ success: boolean; path?: string; error?: string }>('install_mcp_server', {
        server,
        installDir: installDir || '',
      });
      
      if (!result.success) {
        throw new Error(result.error || '安装失败');
      }
      
      // Stage 2: Installing (30-70%)
      progress.progress = 50;
      progress.stage = 'installing';
      progress.message = '正在安装服务器...';
      const map2 = new Map(installProgress.value);
      map2.set(server.id, { ...progress });
      installProgress.value = map2;
      
      // Stage 3: Extracting (70-90%)
      progress.progress = 80;
      progress.stage = 'extracting';
      progress.message = '正在解压文件...';
      const map3 = new Map(installProgress.value);
      map3.set(server.id, { ...progress });
      installProgress.value = map3;
      
      // Complete
      progress.progress = 100;
      progress.stage = 'success';
      progress.message = `服务器已安装到 ${result.path}`;
      progress.completedAt = new Date().toISOString();
      const map4 = new Map(installProgress.value);
      map4.set(server.id, { ...progress });
      installProgress.value = map4;
      
      // Refresh local servers
      await fetchLocalServers();
      
      return { success: true, path: result.path };
    } catch (e) {
      progress.stage = 'failed';
      progress.error = e instanceof Error ? e.message : String(e);
      progress.message = `安装失败: ${progress.error}`;
      const mapError = new Map(installProgress.value);
      mapError.set(server.id, { ...progress });
      installProgress.value = mapError;
      
      return { success: false, error: e };
    } finally {
      isInstalling.value = false;
      
      // Clear progress after delay
      setTimeout(() => {
        const mapClear = new Map(installProgress.value);
        mapClear.delete(server.id);
        installProgress.value = mapClear;
      }, 5000);
    }
  }
  
  async function syncServerToTarget(
    serverName: string,
    installDir: string,
    target: MCPSyncTarget
  ) {
    isSyncing.value = true;
    
    try {
      const result = await invoke<{ success: boolean; error?: string }>('sync_mcp_to_target', {
        serverName,
        installDir,
        target,
      });
      
      return result;
    } catch (e) {
      return { success: false, error: e instanceof Error ? e.message : String(e) };
    } finally {
      isSyncing.value = false;
    }
  }
  
  async function fetchSyncTargets() {
    try {
      syncTargets.value = await invoke<MCPSyncTarget[]>('get_mcp_sync_targets');
    } catch (e) {
      console.error('Failed to fetch sync targets:', e);
    }
  }
  
  async function addSyncTarget(target: MCPSyncTarget) {
    try {
      const newTarget = await invoke<MCPSyncTarget>('add_mcp_sync_target', { target });
      syncTargets.value.push(newTarget);
      return newTarget;
    } catch (e) {
      throw e;
    }
  }
  
  async function removeSyncTarget(targetId: string) {
    try {
      await invoke('remove_mcp_sync_target', { targetId });
      syncTargets.value = syncTargets.value.filter(t => t.id !== targetId);
    } catch (e) {
      throw e;
    }
  }
  
  function getInstallProgress(serverId: string): MCPInstallProgress | undefined {
    return installProgress.value.get(serverId);
  }
  
  function clearError() {
    error.value = null;
  }
  
  return {
    // State
    sources,
    currentSource,
    servers,
    localServers,
    syncTargets,
    currentPage,
    pageSize,
    totalServers,
    totalPages,
    selectedCategory,
    searchKeyword,
    isLoadingSources,
    isLoadingServers,
    isInstalling,
    isSyncing,
    error,
    installProgress,
    
    // Computed
    hasNextPage,
    hasPrevPage,
    installedServerNames,
    serversByRegion,
    
    // Actions
    fetchSources,
    fetchServers,
    fetchLocalServers,
    selectSource,
    setPage,
    setPageSize,
    setCategory,
    setSearchKeyword,
    installServer,
    syncServerToTarget,
    fetchSyncTargets,
    addSyncTarget,
    removeSyncTarget,
    getInstallProgress,
    clearError,
    isServerInstalled,
  };
});

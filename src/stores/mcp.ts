import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  MCPService,
  MCPGroup,
  MCPServiceDetail,
  MCPDiscoveryCache,
  MCPHealthRecord,
  MCPAuditEntry,
  MCPAuditFilters,
  MCPAuditPage,
  MCPInvocationResult,
  MCPImportResult,
  MCPExportFormat,
  MCPHealthStatus,
} from '@/types';

export const useMCPStore = defineStore('mcp', () => {
  // === Existing State ===
  const services = ref<MCPService[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // === NEW: Groups State ===
  const groups = ref<MCPGroup[]>([]);
  const activeGroupId = ref<string | null>(null);
  const isLoadingGroups = ref(false);

  // === NEW: Discovery Cache State ===
  const discoveryCache = ref<Record<string, MCPDiscoveryCache>>({});
  const isDiscovering = ref<Record<string, boolean>>({});

  // === NEW: Health History State ===
  const healthHistory = ref<Record<string, MCPHealthRecord[]>>({});

  // === NEW: Audit Log State ===
  const auditLog = ref<MCPAuditEntry[]>([]);
  const auditLogTotal = ref(0);
  const auditLogPage = ref(1);
  const auditLogPageSize = ref(20);
  const isLoadingAudit = ref(false);

  // === NEW: UI State ===
  const searchQuery = ref('');
  const statusFilter = ref<MCPHealthStatus | 'all'>('all');
  const sortBy = ref<'name' | 'lastChecked' | 'status'>('name');
  const sortOrder = ref<'asc' | 'desc'>('asc');
  const viewMode = ref<'table' | 'grid'>('table');

  // === Existing Actions ===

  async function fetchServices(softwareId: string = '') {
    try {
      isLoading.value = true;
      error.value = null;
      const result = await invoke<MCPService[]>('get_mcp_services', { softwareId });
      services.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch MCP services';
    } finally {
      isLoading.value = false;
    }
  }

  async function addService(
    softwareId: string,
    name: string,
    endpoint: string,
    authType: string = 'none',
    config: string = '{}'
  ) {
    try {
      isLoading.value = true;
      error.value = null;
      const now = new Date().toISOString();
      const service: MCPService = {
        id: crypto.randomUUID(),
        softwareId,
        name,
        endpoint,
        authType: authType as MCPService['authType'],
        config,
        isHealthy: false,
        lastChecked: now,
        protocol: 'http',
        groupIds: [],
        tags: [],
        createdAt: now,
        updatedAt: now,
      };
      await invoke('add_mcp_service', { mcpService: service });
      services.value.push(service);
      return service;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add MCP service';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateService(service: MCPService) {
    try {
      isLoading.value = true;
      error.value = null;
      service.updatedAt = new Date().toISOString();
      await invoke('update_mcp_service', { mcpService: service });
      const index = services.value.findIndex(s => s.id === service.id);
      if (index !== -1) {
        services.value[index] = service;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update MCP service';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteService(serviceId: string) {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_mcp_service', { serviceId });
      services.value = services.value.filter(s => s.id !== serviceId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete MCP service';
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function checkHealth(serviceId: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('check_mcp_service_health', { serviceId });
      const service = services.value.find(s => s.id === serviceId);
      if (service) {
        service.isHealthy = result;
        service.lastChecked = new Date().toISOString();
      }
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to check MCP service health';
      return false;
    }
  }

  function getServicesBySoftware(softwareId: string): MCPService[] {
    return services.value.filter(s => s.softwareId === softwareId);
  }

  // === NEW: Group Actions ===

  async function fetchGroups() {
    try {
      isLoadingGroups.value = true;
      const result = await invoke<MCPGroup[]>('get_mcp_groups');
      groups.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch MCP groups';
    } finally {
      isLoadingGroups.value = false;
    }
  }

  async function createGroup(name: string, color: string): Promise<MCPGroup> {
    const result = await invoke<MCPGroup>('create_mcp_group', { name, color });
    groups.value.push(result);
    return result;
  }

  async function updateGroup(group: MCPGroup): Promise<MCPGroup> {
    const result = await invoke<MCPGroup>('update_mcp_group', {
      id: group.id,
      name: group.name,
      color: group.color,
      isVisible: group.isVisible,
    });
    const index = groups.value.findIndex(g => g.id === group.id);
    if (index !== -1) {
      groups.value[index] = result;
    }
    return result;
  }

  async function deleteGroup(id: string) {
    await invoke('delete_mcp_group', { id });
    groups.value = groups.value.filter(g => g.id !== id);
    if (activeGroupId.value === id) {
      activeGroupId.value = null;
    }
  }

  // === NEW: Discovery Actions ===

  async function discoverService(serviceId: string): Promise<MCPDiscoveryCache> {
    try {
      isDiscovering.value[serviceId] = true;
      const result = await invoke<MCPDiscoveryCache>('discover_mcp_service', { serviceId });
      discoveryCache.value[serviceId] = result;
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to discover MCP service';
      throw e;
    } finally {
      isDiscovering.value[serviceId] = false;
    }
  }

  async function invokeTool(
    serviceId: string,
    toolName: string,
    args: Record<string, unknown>
  ): Promise<MCPInvocationResult> {
    try {
      const result = await invoke<MCPInvocationResult>('invoke_mcp_tool', {
        serviceId,
        toolName,
        args,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to invoke MCP tool';
      throw e;
    }
  }

  // === NEW: Health History Actions ===

  async function fetchHealthHistory(serviceId: string, limit: number = 20) {
    try {
      const result = await invoke<MCPHealthRecord[]>('get_mcp_health_history', {
        serviceId,
        limit,
      });
      healthHistory.value[serviceId] = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch health history';
    }
  }

  // === NEW: Audit Log Actions ===

  async function fetchAuditLog(filters: MCPAuditFilters) {
    try {
      isLoadingAudit.value = true;
      const result = await invoke<MCPAuditPage>('get_mcp_audit_log', {
        filters,
        page: auditLogPage.value,
        pageSize: auditLogPageSize.value,
      });
      auditLog.value = result.items;
      auditLogTotal.value = result.total;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch audit log';
    } finally {
      isLoadingAudit.value = false;
    }
  }

  // === NEW: Import/Export Actions ===

  async function exportServices(
    ids: string[] | null,
    format: MCPExportFormat
  ): Promise<string> {
    try {
      const result = await invoke<string>('export_mcp_services', {
        ids,
        format,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to export services';
      throw e;
    }
  }

  async function importServices(
    data: string,
    mode: 'skip' | 'overwrite'
  ): Promise<MCPImportResult> {
    try {
      const result = await invoke<MCPImportResult>('import_mcp_services', {
        data,
        mode,
      });
      await fetchServices();
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import services';
      throw e;
    }
  }

  // === NEW: Service Detail ===

  async function fetchServiceDetail(serviceId: string): Promise<MCPServiceDetail> {
    try {
      const result = await invoke<MCPServiceDetail>('get_mcp_service_detail', {
        serviceId,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch service detail';
      throw e;
    }
  }

  // === Computed ===

  const filteredServices = computed(() => {
    let result = services.value;

    // Search filter
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        s =>
          s.name.toLowerCase().includes(q) ||
          s.endpoint.toLowerCase().includes(q) ||
          s.tags.some(t => t.toLowerCase().includes(q))
      );
    }

    // Status filter
    if (statusFilter.value !== 'all') {
      result = result.filter(s => s.isHealthy === (statusFilter.value === 'online'));
    }

    // Group filter
    if (activeGroupId.value) {
      result = result.filter(s => s.groupIds.includes(activeGroupId.value!));
    }

    // Sorting — spread creates a shallow copy; mutating sort in-place is acceptable
    // since we operate on the copy, not the original services array.
    result = [...result].sort((a, b) => {
      let aVal: string | number | boolean;
      let bVal: string | number | boolean;

      switch (sortBy.value) {
        case 'name':
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
          break;
        case 'lastChecked':
          aVal = a.lastChecked;
          bVal = b.lastChecked;
          break;
        case 'status':
          aVal = a.isHealthy ? 1 : 0;
          bVal = b.isHealthy ? 1 : 0;
          break;
        default:
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
      }

      const order = sortOrder.value === 'asc' ? 1 : -1;

      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return aVal.localeCompare(bVal) * order;
      }
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return (aVal - bVal) * order;
      }
      // Fallback comparison for mixed-type values (e.g. comparing a string
      // against a number after a `status` sort). Treat non-comparable
      // operands as 0 so the comparator returns a stable direction.
      const aNum = Number(aVal);
      const bNum = Number(bVal);
      if (Number.isFinite(aNum) && Number.isFinite(bNum)) {
        return (aNum - bNum) * order;
      }
      return 0;
    });

    return result;
  });

  const serviceStats = computed(() => ({
    total: services.value.length,
    online: services.value.filter(s => s.isHealthy).length,
    offline: services.value.filter(s => !s.isHealthy).length,
    byGroup: groups.value.reduce(
      (acc, group) => {
        acc[group.id] = services.value.filter(s => s.groupIds.includes(group.id)).length;
        return acc;
      },
      {} as Record<string, number>
    ),
  }));

  // === Error Handling ===

  class MCPPersistenceError extends Error {
    constructor(
      public context: string,
      message: string,
      public code?: string
    ) {
      super(message);
      this.name = 'MCPPersistenceError';
    }
  }

  function handleError(context: string, err: unknown): never {
    const message = err instanceof Error ? err.message : 'Unknown error';
    const code = (err as { code?: string }).code;

    if (import.meta.env.DEV) {
      console.error(`[MCP Store] ${context}:`, err);
    }

    throw new MCPPersistenceError(context, message, code);
  }

  return {
    // Existing exports
    services,
    isLoading,
    error,
    fetchServices,
    addService,
    updateService,
    deleteService,
    checkHealth,
    getServicesBySoftware,
    // NEW exports
    groups,
    activeGroupId,
    isLoadingGroups,
    fetchGroups,
    createGroup,
    updateGroup,
    deleteGroup,
    discoveryCache,
    isDiscovering,
    discoverService,
    invokeTool,
    healthHistory,
    fetchHealthHistory,
    auditLog,
    auditLogTotal,
    auditLogPage,
    auditLogPageSize,
    isLoadingAudit,
    fetchAuditLog,
    searchQuery,
    statusFilter,
    sortBy,
    sortOrder,
    viewMode,
    filteredServices,
    serviceStats,
    exportServices,
    importServices,
    fetchServiceDetail,
    // Error handling
    MCPPersistenceError,
    handleError,
  };
});

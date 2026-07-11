import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useClientSync } from '../useClientSync'
import { SUPPORTED_CLIENTS } from '@/types/unified-plugin'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock showNotification function
const mockShowNotification = vi.fn()
vi.mock('vue', async () => {
  const actual = await vi.importActual('vue')
  return {
    ...actual,
    inject: vi.fn((key: string) => {
      if (key === 'showNotification') return mockShowNotification
      return undefined
    })
  }
})

// Mock unified-plugin store
const mockWorkspacePath = '/test/workspace'
vi.mock('@/stores/unified-plugin', () => ({
  useUnifiedPluginStore: vi.fn(() => ({
    workspacePath: mockWorkspacePath
  }))
}))

// Mock useErrorHandler
const mockHandleError = vi.fn((error: unknown) => ({
  code: 'UNKNOWN_ERROR',
  message: error instanceof Error ? error.message : String(error),
  userMessage: '未知错误',
  recoverable: true,
  timestamp: Date.now()
}))
vi.mock('../useErrorHandler', () => ({
  useErrorHandler: vi.fn(() => ({
    currentError: { value: null },
    clearError: vi.fn(),
    handleError: mockHandleError,
    withRetry: vi.fn(async (fn: () => Promise<unknown>) => fn()),
    wrapAsync: vi.fn(async (fn: () => Promise<unknown>) => {
      try {
        const data = await fn()
        return { data, error: null }
      } catch (error) {
        return { data: null, error: mockHandleError(error) }
      }
    })
  }))
}))

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value }),
    removeItem: vi.fn((key: string) => { delete store[key] }),
    clear: vi.fn(() => { store = {} }),
    get length() { return Object.keys(store).length },
    key: vi.fn((index: number) => Object.keys(store)[index] || null)
  }
})()

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock, writable: true })

describe('useClientSync', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // 清除 localStorage 中的同步状态
    localStorage.clear()
  })

  it('should initialize with empty clients list', () => {
    const { clients, isDialogOpen, syncingClient, isLoading } = useClientSync()

    expect(clients.value).toEqual([])
    expect(isDialogOpen.value).toBe(false)
    expect(syncingClient.value).toBeNull()
    expect(isLoading.value).toBe(false)
  })

  it('should toggle dialog state', () => {
    const { isDialogOpen, toggleDialog } = useClientSync()

    expect(isDialogOpen.value).toBe(false)
    toggleDialog()
    expect(isDialogOpen.value).toBe(true)
    toggleDialog()
    expect(isDialogOpen.value).toBe(false)
  })

  it('should compute totalSyncedCount correctly', () => {
    const { clients, totalSyncedCount } = useClientSync()

    // Initially 0
    expect(totalSyncedCount.value).toBe(0)

    // Add some clients
    clients.value = [
      { key: 'claude', name: 'Claude', icon: '/icons/claude.svg', color: '#D97706', isInstalled: true, isSynced: true },
      { key: 'cursor', name: 'Cursor', icon: '/icons/cursor.svg', color: '#7C3AED', isInstalled: true, isSynced: false }
    ]

    expect(totalSyncedCount.value).toBe(1)
  })

  it('should initialize clients from allagents status', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 后端返回的是 CommandResult 包装器
    mockInvoke.mockResolvedValue({
      success: true,
      data: {
        workspace_path: '/test/workspace',
        clients: ['claude', 'cursor'],
        plugins: [],
        mcp_servers: [],
        last_sync: undefined
      },
      error: null
    })

    const { clients, initClients, isLoading } = useClientSync()

    await initClients()

    expect(isLoading.value).toBe(false)
    expect(clients.value.length).toBe(SUPPORTED_CLIENTS.length) // 23 clients

    // 检查 claude 客户端 - 已配置的客户端
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude).toBeDefined()
    expect(claude?.isInstalled).toBe(true)
    expect(claude?.isSynced).toBe(false)  // 已配置但未同步
    expect(claude?.name).toBe('Claude Code')

    // 检查 cursor 客户端 - 已配置的客户端
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(cursor).toBeDefined()
    expect(cursor?.isInstalled).toBe(true)
    expect(cursor?.isSynced).toBe(false)  // 已配置但未同步

    // 检查未配置的客户端
    const copilot = clients.value.find(c => c.key === 'copilot')
    expect(copilot).toBeDefined()
    expect(copilot?.isInstalled).toBe(false)
    expect(copilot?.isSynced).toBe(false)
  })

  it('should set isLoading to true during initClients', async () => {
    const mockInvoke = vi.mocked(invoke)
    // Use a deferred promise to check loading state
    let resolvePromise: (value: unknown) => void
    mockInvoke.mockImplementation(() =>
      new Promise((resolve) => { resolvePromise = resolve })
    )

    const { initClients, isLoading } = useClientSync()

    const promise = initClients()
    expect(isLoading.value).toBe(true)

    resolvePromise!({ success: true, data: { workspace_path: '/test', clients: [], plugins: [], mcp_servers: [] }, error: null })
    await promise

    expect(isLoading.value).toBe(false)
  })

  it('should handle initClients error', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValue(new Error('CLI not found'))

    const { initClients, isLoading } = useClientSync()

    await initClients()

    expect(isLoading.value).toBe(false)
    // useErrorHandler 会将错误转换为用户友好的消息
    expect(mockShowNotification).toHaveBeenCalled()
  })

  it('should call invoke with allagents_status command', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: [], plugins: [], mcp_servers: [] },
      error: null
    })

    const { initClients } = useClientSync()

    await initClients()

    // 验证调用时传递了 workspacePath 参数
    expect(mockInvoke).toHaveBeenCalledWith('allagents_status', { workspacePath: mockWorkspacePath })
  })

  it('should set installStatus correctly', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { clients, initClients } = useClientSync()

    await initClients()

    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude?.installStatus).toBe('installed')

    const copilot = clients.value.find(c => c.key === 'copilot')
    expect(copilot?.installStatus).toBe('notinstalled')
  })

  it('should toggle sync status for installed client', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 初始化 mock
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()

    // claude is installed but not synced
    const claudeBefore = clients.value.find(c => c.key === 'claude')
    expect(claudeBefore?.isSynced).toBe(false)

    // Mock update command - 返回成功的 CommandResult
    mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })

    await toggleSync('claude')

    const claudeAfter = clients.value.find(c => c.key === 'claude')
    expect(claudeAfter?.isSynced).toBe(true)  // Toggled from false to true
  })

  it('should show warning when trying to sync uninstalled client', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: [], plugins: [], mcp_servers: [] },
      error: null
    })

    const { initClients, toggleSync } = useClientSync()
    await initClients()

    await toggleSync('claude')

    // toast.warning should be called
    expect(mockShowNotification).toHaveBeenCalled()
  })

  it('should handle sync error', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 初始化 mock
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()

    // Mock update to fail - 后端返回失败的 CommandResult
    mockInvoke.mockResolvedValue({ success: false, data: null, error: 'Sync failed' })

    await toggleSync('claude')

    // State should not change - configured clients start as not synced
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude?.isSynced).toBe(false)
  })

  it('should set syncingClient during sync operation', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 初始化 mock
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { syncingClient, initClients, toggleSync } = useClientSync()
    await initClients()

    let resolvePromise: (value: unknown) => void
    mockInvoke.mockImplementation(() =>
      new Promise((resolve) => { resolvePromise = resolve })
    )

    const promise = toggleSync('claude')
    expect(syncingClient.value).toBe('claude')

    resolvePromise!({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })
    await promise

    expect(syncingClient.value).toBeNull()
  })

  it('should sync all unsynced installed clients', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 初始化 mock
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { clients, initClients, syncAll } = useClientSync()
    await initClients()

    // Mock update command - 返回成功的 CommandResult
    mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })

    await syncAll()

    // Configured clients are initially not synced, toggleSync will toggle them to true
    const claude = clients.value.find(c => c.key === 'claude')
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(claude?.isSynced).toBe(true)
    expect(cursor?.isSynced).toBe(true)
  })

  it('should show info when no clients to sync', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: [], plugins: [], mcp_servers: [] },
      error: null
    })

    const { initClients, syncAll } = useClientSync()
    await initClients()

    await syncAll()

    expect(mockShowNotification).toHaveBeenCalledWith('没有需要同步的客户端', 'info')
  })

  it('should call invoke with correct allagents_update command', async () => {
    const mockInvoke = vi.mocked(invoke)
    // 初始化 mock
    mockInvoke.mockResolvedValue({
      success: true,
      data: { workspace_path: '/test', clients: ['claude'], plugins: [], mcp_servers: [] },
      error: null
    })

    const { initClients, toggleSync } = useClientSync()
    await initClients()

    mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })

    await toggleSync('claude')

    // 验证调用时传递了 workspacePath 和 client 参数
    expect(mockInvoke).toHaveBeenCalledWith('allagents_update', { workspacePath: mockWorkspacePath, client: 'claude' })
  })

  describe('Integration', () => {
    it('should complete full sync flow', async () => {
      const mockInvoke = vi.mocked(invoke)

      // Step 1: Initialize - 使用 CommandResult 格式
      mockInvoke.mockResolvedValue({
        success: true,
        data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
        error: null
      })

      const { initClients, toggleSync, totalSyncedCount } = useClientSync()
      await initClients()

      // Configured clients are initially not synced
      expect(totalSyncedCount.value).toBe(0)

      // Step 2: Sync first client (false -> true)
      mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })
      await toggleSync('claude')

      expect(totalSyncedCount.value).toBe(1)

      // Step 3: Sync second client (false -> true)
      await toggleSync('cursor')

      expect(totalSyncedCount.value).toBe(2)
    })

    it('should handle partial sync failure', async () => {
      const mockInvoke = vi.mocked(invoke)

      // 初始化 mock
      mockInvoke.mockResolvedValue({
        success: true,
        data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
        error: null
      })

      const { clients, initClients, toggleSync, totalSyncedCount } = useClientSync()
      await initClients()

      // First toggle succeeds (false -> true)
      mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })
      await toggleSync('claude')
      expect(totalSyncedCount.value).toBe(1)

      // Second toggle fails - 后端返回失败的 CommandResult
      mockInvoke.mockResolvedValue({ success: false, data: null, error: 'Network error' })
      await toggleSync('cursor')

      // Only claude should be synced (cursor toggle failed, so it stays false)
      expect(totalSyncedCount.value).toBe(1)
      const cursor = clients.value.find(c => c.key === 'cursor')
      expect(cursor?.isSynced).toBe(false)
    })

    it('should persist sync state to localStorage', async () => {
      const mockInvoke = vi.mocked(invoke)

      // Step 1: Initialize
      mockInvoke.mockResolvedValue({
        success: true,
        data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
        error: null
      })

      const { initClients, toggleSync } = useClientSync()
      await initClients()

      // Step 2: Sync claude
      mockInvoke.mockResolvedValue({ success: true, data: { synced_count: 1, error_count: 0, skipped_count: 0 }, error: null })
      await toggleSync('claude')

      // Check localStorage was updated
      const stored = JSON.parse(localStorage.getItem('forge_synced_clients') || '[]')
      expect(stored).toContain('claude')
      expect(stored).not.toContain('cursor')
    })

    it('should restore sync state from localStorage after refresh', async () => {
      const mockInvoke = vi.mocked(invoke)

      // Simulate previous sync state in localStorage
      localStorage.setItem('forge_synced_clients', JSON.stringify(['claude']))

      // Initialize
      mockInvoke.mockResolvedValue({
        success: true,
        data: { workspace_path: '/test', clients: ['claude', 'cursor'], plugins: [], mcp_servers: [] },
        error: null
      })

      const { clients, initClients } = useClientSync()
      await initClients()

      // claude should be synced (from localStorage)
      const claude = clients.value.find(c => c.key === 'claude')
      expect(claude?.isSynced).toBe(true)

      // cursor should not be synced
      const cursor = clients.value.find(c => c.key === 'cursor')
      expect(cursor?.isSynced).toBe(false)
    })
  })
})

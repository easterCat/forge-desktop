import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-toastification'
import { useClientSync } from '../useClientSync'
import { SUPPORTED_CLIENTS } from '@/types/unified-plugin'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock vue-toastification
vi.mock('vue-toastification', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn()
  }
}))

describe('useClientSync', () => {
  beforeEach(() => {
    vi.clearAllMocks()
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
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: ['claude']
    })

    const { clients, initClients, isLoading } = useClientSync()

    await initClients()

    expect(isLoading.value).toBe(false)
    expect(clients.value.length).toBe(SUPPORTED_CLIENTS.length) // 23 clients

    // 检查 claude 客户端
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude).toBeDefined()
    expect(claude?.isInstalled).toBe(true)
    expect(claude?.isSynced).toBe(true)
    expect(claude?.name).toBe('Claude Code')

    // 检查 cursor 客户端
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(cursor).toBeDefined()
    expect(cursor?.isInstalled).toBe(true)
    expect(cursor?.isSynced).toBe(false)

    // 检查未安装的客户端
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

    resolvePromise!({ installedClients: [], syncedClients: [] })
    await promise

    expect(isLoading.value).toBe(false)
  })

  it('should handle initClients error', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValue(new Error('CLI not found'))

    const { initClients, isLoading } = useClientSync()

    await initClients()

    expect(isLoading.value).toBe(false)
    expect(toast.error).toHaveBeenCalledWith('获取客户端状态失败')
  })

  it('should call invoke with allagents_status command', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: [],
      syncedClients: []
    })

    const { initClients } = useClientSync()

    await initClients()

    expect(mockInvoke).toHaveBeenCalledWith('allagents_status')
  })

  it('should set installStatus correctly', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: []
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
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()

    // claude is installed but not synced
    const claudeBefore = clients.value.find(c => c.key === 'claude')
    expect(claudeBefore?.isSynced).toBe(false)

    // Mock update command
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })

    await toggleSync('claude')

    const claudeAfter = clients.value.find(c => c.key === 'claude')
    expect(claudeAfter?.isSynced).toBe(true)
  })

  it('should show warning when trying to sync uninstalled client', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: [],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()

    await toggleSync('claude')

    // toast.warning should be called
    expect(toast.warning).toHaveBeenCalled()
  })

  it('should handle sync error', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()

    // Mock update to fail
    mockInvoke.mockRejectedValue(new Error('Sync failed'))

    await toggleSync('claude')

    // State should not change
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude?.isSynced).toBe(false)
  })

  it('should set syncingClient during sync operation', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: []
    })

    const { syncingClient, initClients, toggleSync } = useClientSync()
    await initClients()

    let resolvePromise: (value: unknown) => void
    mockInvoke.mockImplementation(() =>
      new Promise((resolve) => { resolvePromise = resolve })
    )

    const promise = toggleSync('claude')
    expect(syncingClient.value).toBe('claude')

    resolvePromise!({ synced_count: 1, error_count: 0, skipped_count: 0 })
    await promise

    expect(syncingClient.value).toBeNull()
  })

  it('should sync all unsynced installed clients', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, syncAll } = useClientSync()
    await initClients()

    // Mock update command
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })

    await syncAll()

    // Both should be synced now
    const claude = clients.value.find(c => c.key === 'claude')
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(claude?.isSynced).toBe(true)
    expect(cursor?.isSynced).toBe(true)
  })

  it('should show info when no clients to sync', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: ['claude']
    })

    const { initClients, syncAll } = useClientSync()
    await initClients()

    await syncAll()

    expect(toast.info).toHaveBeenCalledWith('没有需要同步的客户端')
  })

  it('should call invoke with correct allagents_update command', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: []
    })

    const { initClients, toggleSync } = useClientSync()
    await initClients()

    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })

    await toggleSync('claude')

    expect(mockInvoke).toHaveBeenCalledWith('allagents_update', { client: 'claude' })
  })
})

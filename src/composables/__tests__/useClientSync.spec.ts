import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useClientSync } from '../useClientSync'

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
})

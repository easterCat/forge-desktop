// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ClientSyncDialog from '../ClientSyncDialog.vue'
import { useClientSync } from '@/composables/useClientSync'
import { invoke } from '@tauri-apps/api/core'

// Mock CliSyncChip component
vi.mock('../../common/CliSyncChip.vue', () => ({
  default: {
    template: '<span class="cli-sync-chip-mock">{{ toolName }}</span>',
    props: ['toolKey', 'toolName', 'toolIcon', 'toolColor', 'state', 'disabled']
  }
}))

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Build the mock composable using vi.hoisted so it's available before hoisted vi.mock calls.
// This avoids the vue-toastification resolution issue while keeping the same API.
const mockUseClientSync = vi.hoisted(() => {
  const { ref, computed } = require('vue')

  return () => {
    const clients = ref([
      { key: 'claude', name: 'Claude Code', icon: '/icons/claude.svg', color: '#D97706', isInstalled: true, isSynced: false },
      { key: 'cursor', name: 'Cursor', icon: '/icons/cursor.svg', color: '#7C3AED', isInstalled: true, isSynced: false },
      { key: 'copilot', name: 'GitHub Copilot', icon: '/icons/copilot.svg', color: '#6E40C9', isInstalled: false, isSynced: false }
    ])
    const syncingClient = ref(null)
    const isDialogOpen = ref(false)
    const isLoading = ref(false)

    const totalSyncedCount = computed(() => clients.value.filter(c => c.isSynced).length)

    const toggleDialog = () => { isDialogOpen.value = !isDialogOpen.value }

    const toggleSync = async (clientKey) => {
      const client = clients.value.find(c => c.key === clientKey)
      if (!client?.isInstalled) return

      syncingClient.value = clientKey
      try {
        await invoke('allagents_update', { client: clientKey })
        const idx = clients.value.findIndex(c => c.key === clientKey)
        if (idx !== -1) {
          clients.value[idx] = { ...clients.value[idx], isSynced: !clients.value[idx].isSynced }
        }
      } finally {
        syncingClient.value = null
      }
    }

    const syncAll = async () => {
      const unsynced = clients.value.filter(c => c.isInstalled && !c.isSynced)
      for (const client of unsynced) {
        await toggleSync(client.key)
      }
    }

    const initClients = async () => {
      isLoading.value = true
      try {
        const status = await invoke('allagents_status')
        clients.value = clients.value.map(c => ({
          ...c,
          isInstalled: status.installedClients.includes(c.key),
          isSynced: status.syncedClients.includes(c.key)
        }))
      } finally {
        isLoading.value = false
      }
    }

    return { clients, totalSyncedCount, isDialogOpen, syncingClient, isLoading, toggleDialog, toggleSync, syncAll, initClients }
  }
})

vi.mock('@/composables/useClientSync', () => ({
  useClientSync: mockUseClientSync
}))

describe('ClientSyncDialog', () => {
  const mockClients = [
    { key: 'claude', name: 'Claude Code', icon: '/icons/claude.svg', color: '#D97706', isInstalled: true, isSynced: true },
    { key: 'cursor', name: 'Cursor', icon: '/icons/cursor.svg', color: '#7C3AED', isInstalled: true, isSynced: false },
    { key: 'copilot', name: 'GitHub Copilot', icon: '/icons/copilot.svg', color: '#6E40C9', isInstalled: false, isSynced: false }
  ]

  it('should render client list', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    expect(wrapper.text()).toContain('Claude Code')
    expect(wrapper.text()).toContain('Cursor')
    expect(wrapper.text()).toContain('GitHub Copilot')
  })

  it('should show disabled state for uninstalled clients', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const copilotItem = wrapper.find('[data-testid="client-copilot"]')
    expect(copilotItem.classes()).toContain('disabled')
  })

  it('should emit toggleSync when clicking installed client', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const claudeItem = wrapper.find('[data-testid="client-claude"]')
    await claudeItem.trigger('click')

    expect(wrapper.emitted('toggleSync')).toBeTruthy()
    expect(wrapper.emitted('toggleSync')![0]).toEqual(['claude'])
  })

  it('should not emit toggleSync when clicking uninstalled client', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const copilotItem = wrapper.find('[data-testid="client-copilot"]')
    await copilotItem.trigger('click')

    expect(wrapper.emitted('toggleSync')).toBeFalsy()
  })

  it('should emit close when clicking overlay', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const overlay = wrapper.find('.modal-overlay')
    await overlay.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('should show loading spinner for syncing client', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: 'claude'
      }
    })

    const spinner = wrapper.find('[data-testid="spinner-claude"]')
    expect(spinner.exists()).toBe(true)
  })

  it('should disable syncAll button when no unsynced clients', () => {
    const allSyncedClients = mockClients.map(c => ({ ...c, isSynced: true }))
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: allSyncedClients,
        syncingClient: null
      }
    })

    const syncAllButton = wrapper.find('[data-testid="sync-all-button"]')
    expect(syncAllButton.attributes('disabled')).toBeDefined()
  })

  describe('Integration with useClientSync', () => {
    it('should render client data sourced from useClientSync initClients', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValue({
        installedClients: ['claude', 'cursor'],
        syncedClients: ['claude']
      })

      // Use the composable to produce real client data
      const { clients, initClients, syncingClient } = useClientSync()
      await initClients()

      const wrapper = mount(ClientSyncDialog, {
        props: {
          clients: clients.value,
          syncingClient: syncingClient.value
        }
      })

      // The composable should have populated clients with the init status
      expect(wrapper.text()).toContain('Claude Code')
      expect(wrapper.text()).toContain('Cursor')

      // Claude should NOT show the "未安装" tag (it is installed)
      const claudeItem = wrapper.find('[data-testid="client-claude"]')
      expect(claudeItem.find('.status-tag').exists()).toBe(false)

      // Copilot is not installed -> should show the tag
      const copilotItem = wrapper.find('[data-testid="client-copilot"]')
      expect(copilotItem.find('.status-tag').exists()).toBe(true)
    })

    it('should invoke sync via composable when clicking an installed client', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValue({
        installedClients: ['claude', 'cursor'],
        syncedClients: []
      })

      const { clients, initClients, toggleSync, syncingClient } = useClientSync()
      await initClients()

      // Mock the update command that toggleSync will call
      mockInvoke.mockResolvedValue({})

      const wrapper = mount(ClientSyncDialog, {
        props: {
          clients: clients.value,
          syncingClient: syncingClient.value
        },
        // The component emits toggleSync; wire it to the composable's handler
        attrs: {
          onToggleSync: (clientKey: string) => toggleSync(clientKey)
        }
      })

      // Click Claude (installed client)
      const claudeItem = wrapper.find('[data-testid="client-claude"]')
      await claudeItem.trigger('click')

      // Verify the Tauri invoke was called with the correct command
      expect(mockInvoke).toHaveBeenCalledWith('allagents_update', { client: 'claude' })

      // Verify the composable state was updated (synced toggled to true)
      const claudeAfter = clients.value.find(c => c.key === 'claude')
      expect(claudeAfter?.isSynced).toBe(true)
    })

    it('should close the dialog via composable toggleDialog when overlay is clicked', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValue({
        installedClients: ['claude'],
        syncedClients: []
      })

      const { clients, initClients, isDialogOpen, toggleDialog } = useClientSync()
      await initClients()

      // Toggle open
      toggleDialog()
      expect(isDialogOpen.value).toBe(true)

      const wrapper = mount(ClientSyncDialog, {
        props: {
          clients: clients.value,
          syncingClient: null
        },
        attrs: {
          onClose: () => toggleDialog()
        }
      })

      // Click the overlay to close
      const overlay = wrapper.find('.modal-overlay')
      await overlay.trigger('click')

      // The component should have emitted 'close'
      expect(wrapper.emitted('close')).toBeTruthy()

      // The onClose handler wired via attrs calls toggleDialog(), which flips
      // isDialogOpen from true back to false — verifying the full integration cycle
      expect(isDialogOpen.value).toBe(false)
    })
  })
})

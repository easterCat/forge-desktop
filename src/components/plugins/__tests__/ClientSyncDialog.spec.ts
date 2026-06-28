// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ClientSyncDialog from '../ClientSyncDialog.vue'

// Mock CliSyncChip component
vi.mock('../../common/CliSyncChip.vue', () => ({
  default: {
    template: '<span class="cli-sync-chip-mock">{{ toolName }}</span>',
    props: ['toolKey', 'toolName', 'toolIcon', 'toolColor', 'state', 'disabled']
  }
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
})

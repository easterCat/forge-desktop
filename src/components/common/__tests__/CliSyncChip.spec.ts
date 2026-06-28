// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, computed } from 'vue'
import { mount } from '@vue/test-utils'
import CliSyncChip from '../CliSyncChip.vue'

// Mock useClientSync
vi.mock('../../../composables/useClientSync', () => ({
  useClientSync: vi.fn(() => ({
    clients: ref([]),
    totalSyncedCount: computed(() => 0),
    isDialogOpen: ref(false),
    syncingClient: ref(null),
    isLoading: ref(false),
    toggleDialog: vi.fn(),
    toggleSync: vi.fn(),
    syncAll: vi.fn(),
    initClients: vi.fn()
  }))
}))

// Mock ClientSyncDialog - must match the import path in CliSyncChip.vue
vi.mock('../../plugins/ClientSyncDialog.vue', () => ({
  default: {
    template: '<div class="dialog-mock" data-testid="client-sync-dialog"></div>',
    props: ['clients', 'syncingClient'],
    emits: ['toggleSync', 'close', 'syncAll']
  }
}))

describe('CliSyncChip', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render total count chip', () => {
    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced',
        showSyncCount: true
      }
    })

    expect(wrapper.find('.chip-badge').exists()).toBe(true)
  })

  it('should open dialog when clicking chip', async () => {
    const mockToggleDialog = vi.fn()
    const { useClientSync } = await import('../../../composables/useClientSync')
    vi.mocked(useClientSync).mockReturnValue({
      clients: ref([]),
      totalSyncedCount: computed(() => 0),
      isDialogOpen: ref(false),
      syncingClient: ref(null),
      isLoading: ref(false),
      toggleDialog: mockToggleDialog,
      toggleSync: vi.fn(),
      syncAll: vi.fn(),
      initClients: vi.fn()
    })

    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced',
        showSyncCount: true
      }
    })

    await wrapper.find('.cli-sync-chip-wrapper').trigger('click')
    expect(mockToggleDialog).toHaveBeenCalled()
  })

  it('should show dialog when isDialogOpen is true', async () => {
    const { useClientSync } = await import('../../../composables/useClientSync')
    vi.mocked(useClientSync).mockReturnValue({
      clients: ref([]),
      totalSyncedCount: computed(() => 0),
      isDialogOpen: ref(true),
      syncingClient: ref(null),
      isLoading: ref(false),
      toggleDialog: vi.fn(),
      toggleSync: vi.fn(),
      syncAll: vi.fn(),
      initClients: vi.fn()
    })

    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced'
      }
    })

    expect(wrapper.find('[data-testid="client-sync-dialog"]').exists()).toBe(true)
  })
})

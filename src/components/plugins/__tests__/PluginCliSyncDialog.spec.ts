// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import PluginCliSyncDialog from '../PluginCliSyncDialog.vue'
import type { MarketplacePlugin } from '@/types'

// Mock ToolIcon
vi.mock('@/components/common/ToolIcon.vue', () => ({
  default: {
    template: '<span class="tool-icon-mock">{{ toolKey }}</span>',
    props: ['toolKey', 'size']
  }
}))

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// ALL shared mock state must be hoisted together so the store factory can reference them
const {
  mockSyncStatuses,
  _mockSupportedCliTools,
  mockSyncPluginToCliTool,
  mockUnsyncPluginFromCliTool,
  mockShowNotification,
  storeFactory,
} = vi.hoisted(() => {
  // Plain object that we'll mutate directly — not a ref.
  // Vue's reactivity tracks property access on Proxy-wrapped objects returned
  // from defineStore, so direct mutations work the same as in production.
  const mockSyncStatuses = {} as Record<string, { synced: boolean }>
  const mockSupportedCliTools = [
    { key: 'claude-code', name: 'Claude Code', icon: 'CC', color: '#B8944A', pluginDir: '~/.claude/plugins' },
    { key: 'codex', name: 'Codex', icon: 'Co', color: '#059669', pluginDir: '~/.codex/plugins' },
    { key: 'qwen-code', name: 'Qwen Code', icon: 'QC', color: '#2563EB', pluginDir: null },
  ]
  const mockSyncPluginToCliTool = vi.fn()
  const mockUnsyncPluginFromCliTool = vi.fn()
  const mockShowNotification = vi.fn()

  const storeFactory = () => ({
    supportedCliTools: mockSupportedCliTools,
    syncStatuses: mockSyncStatuses,
    syncPluginToCliTool: mockSyncPluginToCliTool,
    unsyncPluginFromCliTool: mockUnsyncPluginFromCliTool,
  })

  return {
    mockSyncStatuses,
    mockSupportedCliTools,
    mockSyncPluginToCliTool,
    mockUnsyncPluginFromCliTool,
    mockShowNotification,
    storeFactory,
  }
})

vi.mock('@/stores/plugin-marketplace', () => ({
  usePluginMarketplaceStore: storeFactory
}))

vi.mock('vue', async importOriginal => {
  const actual = await importOriginal()
  return {
    ...actual,
    inject: vi.fn(() => mockShowNotification)
  }
})

const mockPlugin: MarketplacePlugin = {
  id: 'test-plugin-id',
  sourceId: 'agent-skills',
  name: 'test-plugin',
  description: 'A test plugin',
}

describe('PluginCliSyncDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset syncStatuses by clearing its keys (plain object, not ref)
    for (const key in mockSyncStatuses) delete mockSyncStatuses[key]
    mockSyncPluginToCliTool.mockResolvedValue({ success: true })
    mockUnsyncPluginFromCliTool.mockResolvedValue({ success: true })
  })

  function mountDialog(extraProps = {}) {
    return mount(PluginCliSyncDialog, {
      props: { plugin: mockPlugin, isOpen: true, ...extraProps },
      global: {
        stubs: {
          ToolIcon: { template: '<span class="tool-icon-stub">{{ toolKey }}</span>', props: ['toolKey', 'size'] }
        }
      }
    })
  }

  it('should render all three tool groups', async () => {
    const wrapper = mountDialog()
    await nextTick()
    expect(wrapper.text()).toContain('AllAgents · Universal Clients')
    expect(wrapper.text()).toContain('AllAgents · Provider Clients')
    expect(wrapper.text()).toContain('Local CLI Tools')
  })

  it('should show local tools including Claude Code and Codex', async () => {
    const wrapper = mountDialog()
    await nextTick()
    expect(wrapper.text()).toContain('Claude Code')
    expect(wrapper.text()).toContain('Codex')
    expect(wrapper.text()).toContain('Qwen Code')
  })

  it('should show "No plugin dir" tag for tools without pluginDir', async () => {
    const wrapper = mountDialog()
    await nextTick()
    expect(wrapper.text()).toContain('No plugin dir')
  })

  it('should emit update:isOpen=false when close button is clicked', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('.close-btn').trigger('click')
    expect(wrapper.emitted('update:isOpen')).toBeTruthy()
    expect(wrapper.emitted('update:isOpen')![0]).toEqual([false])
  })

  it('should emit update:isOpen=false when overlay is clicked', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('.modal-overlay').trigger('click')
    expect(wrapper.emitted('update:isOpen')).toBeTruthy()
    expect(wrapper.emitted('update:isOpen')![0]).toEqual([false])
  })

  it('should call syncPluginToCliTool and emit synced when clicking unsynced tool', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('[data-testid="tool-claude-code"]').trigger('click')
    await nextTick()
    expect(mockSyncPluginToCliTool).toHaveBeenCalledWith(mockPlugin, 'claude-code')
    expect(wrapper.emitted('synced')).toBeTruthy()
    expect(mockShowNotification).toHaveBeenCalledWith('Synced to Claude Code', 'success')
  })

  it('should call unsyncPluginFromCliTool and emit unsynced when clicking synced tool', async () => {
    mockSyncStatuses['agent-skills::test-plugin::claude-code'] = { synced: true }
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('[data-testid="tool-claude-code"]').trigger('click')
    await nextTick()
    expect(mockUnsyncPluginFromCliTool).toHaveBeenCalledWith(mockPlugin, 'claude-code')
    expect(wrapper.emitted('unsynced')).toBeTruthy()
    expect(mockShowNotification).toHaveBeenCalledWith('Unsynced from Claude Code', 'success')
  })

  it('should show warning when clicking tool with null pluginDir', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('[data-testid="tool-qwen-code"]').trigger('click')
    await nextTick()
    expect(mockSyncPluginToCliTool).toHaveBeenCalledWith(mockPlugin, 'qwen-code')
    expect(mockShowNotification).toHaveBeenCalledWith(
      expect.stringContaining('no plugin directory'),
      'warn'
    )
    expect(wrapper.emitted('synced')).toBeTruthy()
  })

  it('should show error toast when sync returns failure', async () => {
    mockSyncPluginToCliTool.mockResolvedValue({ success: false, error: 'Plugin cache not found' })
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('[data-testid="tool-claude-code"]').trigger('click')
    await nextTick()
    expect(mockShowNotification).toHaveBeenCalledWith('Plugin cache not found', 'error')
    expect(wrapper.emitted('synced')).toBeFalsy()
  })

  it('should filter tools by search query', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('.search-input').setValue('codex')
    await nextTick()
    expect(wrapper.text()).toContain('Codex')
    expect(wrapper.text()).not.toContain('Claude Code')
    expect(wrapper.text()).not.toContain('Cursor')
  })

  it('should show empty state when no tools match search', async () => {
    const wrapper = mountDialog()
    await nextTick()
    await wrapper.find('.search-input').setValue('xyznonexistent999')
    await nextTick()
    expect(wrapper.find('.empty-state').exists()).toBe(true)
    expect(wrapper.text()).toContain('xyznonexistent999')
  })

  it('should display plugin name in header', async () => {
    const wrapper = mountDialog()
    await nextTick()
    expect(wrapper.text()).toContain('test-plugin')
  })
})

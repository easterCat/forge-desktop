import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

// Mock the software store
vi.mock('@/stores/software', () => ({
  useSoftwareStore: () => ({
    addCustomCliTool: vi.fn().mockResolvedValue(undefined),
    removeCustomCliTool: vi.fn().mockResolvedValue(undefined),
  }),
}))

import CustomToolDialog from '../CustomToolDialog.vue'

describe('CustomToolDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders add mode correctly', () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    expect(wrapper.find('h3').text()).toBe('Add Custom Tool')
    expect(wrapper.findAll('input').length).toBeGreaterThan(0)
    expect(wrapper.find('button[type="submit"]').text()).toBe('Add Tool')
  })

  it('renders edit mode correctly', () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: { key: 'my-tool', name: 'My Tool', websiteUrl: 'https://example.com' } },
      global: { stubs: { teleport: true } },
    })
    expect(wrapper.find('h3').text()).toBe('Edit: My Tool')
    expect(wrapper.find('button[type="submit"]').text()).toBe('Save Changes')
  })

  it('auto-generates key from name', async () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    const nameInput = wrapper.findAll('input')[0]
    await nameInput.setValue('My Custom CLI')
    await nextTick()
    const keyInput = wrapper.findAll('input')[1]
    expect(keyInput.element.value).toBe('my-custom-cli')
  })

  it('auto-generates detect command from key', async () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    // Enter name to auto-generate key
    const inputs = wrapper.findAll('input')
    const nameInput = inputs[0]
    await nameInput.setValue('foo')
    await nextTick()
    // Key should be 'foo', detect should be 'foo --version'
    const keyInput = inputs[1]
    expect(keyInput.element.value).toBe('foo')
    const detectInput = inputs[3] // 0=name, 1=key, 2=website, 3=detect
    expect(detectInput.element.value).toBe('foo --version')
  })

  it('shows validation errors for empty required fields', async () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    await wrapper.find('button[type="submit"]').trigger('click')
    await nextTick()
    const errors = wrapper.findAll('.form-error')
    expect(errors.length).toBeGreaterThan(0)
  })

  it('emits close event on cancel', async () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    await wrapper.findAll('button')[0].trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('emits close on overlay click', async () => {
    const wrapper = mount(CustomToolDialog, {
      props: { tool: null },
      global: { stubs: { teleport: true } },
    })
    await wrapper.find('.dialog-overlay').trigger('click')
    expect(wrapper.emitted('close')).toBeTruthy()
  })
})

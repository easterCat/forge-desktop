// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import CollapsePanel from '@/components/common/CollapsePanel.vue';

describe('CollapsePanel', () => {
  it('renders with expanded=true by default', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    expect(wrapper.find('.panel-body').isVisible()).toBe(true);
  });

  it('renders with expanded=false', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test', expanded: false } });
    expect(wrapper.find('.panel-body').isVisible()).toBe(false);
  });

  it('toggles on header click', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    await wrapper.find('.panel-header').trigger('click');
    expect(wrapper.emitted('update:expanded')).toEqual([[false]]);
  });

  it('displays title text', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'My Panel' } });
    expect(wrapper.find('.panel-title').text()).toBe('My Panel');
  });

  it('renders header slot content', () => {
    const wrapper = mount(CollapsePanel, {
      props: { title: 'Test' },
      slots: { header: '<span class="custom">Custom</span>' },
    });
    expect(wrapper.find('.custom').exists()).toBe(true);
  });

  it('renders default slot content', () => {
    const wrapper = mount(CollapsePanel, {
      props: { title: 'Test' },
      slots: { default: '<p>Content</p>' },
    });
    expect(wrapper.find('.panel-body').html()).toContain('Content');
  });

  it('shows chevron icon that rotates when collapsed', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    expect(wrapper.find('.chevron').classes()).not.toContain('rotated');
    await wrapper.setProps({ expanded: false });
    expect(wrapper.find('.chevron').classes()).toContain('rotated');
  });

  it('toggles on Enter key', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    await wrapper.find('.panel-header').trigger('keydown.enter');
    expect(wrapper.emitted('update:expanded')).toEqual([[false]]);
  });

  it('toggles on Space key', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test', expanded: false } });
    await wrapper.find('.panel-header').trigger('keydown.space');
    expect(wrapper.emitted('update:expanded')).toEqual([[true]]);
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    const header = wrapper.find('.panel-header');
    expect(header.attributes('role')).toBe('button');
    expect(header.attributes('tabindex')).toBe('0');
    expect(header.attributes('aria-expanded')).toBe('true');
  });

  it('sets aria-expanded to false when collapsed', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test', expanded: false } });
    expect(wrapper.find('.panel-header').attributes('aria-expanded')).toBe('false');
  });
});

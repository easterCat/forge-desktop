// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Badge from '@/components/common/Badge.vue';

describe('Badge', () => {
  it('renders with default type info', () => {
    const wrapper = mount(Badge, { slots: { default: 'Label' } });
    expect(wrapper.find('.badge').exists()).toBe(true);
    expect(wrapper.find('.badge').classes()).toContain('info');
  });

  it('renders type success', () => {
    const wrapper = mount(Badge, { props: { type: 'success' } });
    expect(wrapper.find('.badge').classes()).toContain('success');
  });

  it('renders type warn', () => {
    const wrapper = mount(Badge, { props: { type: 'warn' } });
    expect(wrapper.find('.badge').classes()).toContain('warn');
  });

  it('renders type error', () => {
    const wrapper = mount(Badge, { props: { type: 'error' } });
    expect(wrapper.find('.badge').classes()).toContain('error');
  });

  it('renders type outline', () => {
    const wrapper = mount(Badge, { props: { type: 'outline' } });
    expect(wrapper.find('.badge').classes()).toContain('outline');
  });

  it('renders type progress', () => {
    const wrapper = mount(Badge, { props: { type: 'progress' } });
    expect(wrapper.find('.badge').classes()).toContain('progress');
  });

  it('renders slot content', () => {
    const wrapper = mount(Badge, { slots: { default: 'Installed' } });
    expect(wrapper.text()).toBe('Installed');
  });

  it('renders slot as HTML', () => {
    const wrapper = mount(Badge, { slots: { default: '<strong>Bold</strong>' } });
    expect(wrapper.find('.badge strong').exists()).toBe(true);
  });

  it('renders multiple badges independently', () => {
    const Wrapper = {
      components: { Badge },
      template: `
        <div>
          <Badge type="success">Active</Badge>
          <Badge type="error">Inactive</Badge>
        </div>
      `,
    };
    const wrapper = mount(Wrapper);
    const badges = wrapper.findAll('.badge');
    expect(badges).toHaveLength(2);
    expect(badges[0].classes()).toContain('success');
    expect(badges[1].classes()).toContain('error');
  });
});

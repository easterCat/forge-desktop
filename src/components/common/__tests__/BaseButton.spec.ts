// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BaseButton from '@/components/common/BaseButton.vue';

describe('BaseButton', () => {
  it('renders with default slot content', () => {
    const wrapper = mount(BaseButton, { slots: { default: 'Label' } });
    expect(wrapper.text()).toContain('Label');
  });

  it('renders variant-primary by default', () => {
    const wrapper = mount(BaseButton);
    expect(wrapper.find('.base-btn').classes()).toContain('variant-primary');
  });

  it('renders variant-secondary', () => {
    const wrapper = mount(BaseButton, { props: { variant: 'secondary' } });
    expect(wrapper.find('.base-btn').classes()).toContain('variant-secondary');
  });

  it('renders variant-ghost', () => {
    const wrapper = mount(BaseButton, { props: { variant: 'ghost' } });
    expect(wrapper.find('.base-btn').classes()).toContain('variant-ghost');
  });

  it('renders variant-danger', () => {
    const wrapper = mount(BaseButton, { props: { variant: 'danger' } });
    expect(wrapper.find('.base-btn').classes()).toContain('variant-danger');
  });

  it('renders size-sm', () => {
    const wrapper = mount(BaseButton, { props: { size: 'sm' } });
    expect(wrapper.find('.base-btn').classes()).toContain('size-sm');
  });

  it('renders size-md', () => {
    const wrapper = mount(BaseButton, { props: { size: 'md' } });
    expect(wrapper.find('.base-btn').classes()).toContain('size-md');
  });

  it('renders size-lg', () => {
    const wrapper = mount(BaseButton, { props: { size: 'lg' } });
    expect(wrapper.find('.base-btn').classes()).toContain('size-lg');
  });

  it('has disabled class when disabled prop is true', () => {
    const wrapper = mount(BaseButton, { props: { disabled: true } });
    expect(wrapper.find('.base-btn').classes()).toContain('disabled');
    expect(wrapper.find('button').attributes('disabled')).toBeDefined();
  });

  it('is disabled when loading', () => {
    const wrapper = mount(BaseButton, { props: { loading: true } });
    expect(wrapper.find('button').attributes('disabled')).toBeDefined();
  });

  it('shows spinner when loading', () => {
    const wrapper = mount(BaseButton, { props: { loading: true } });
    expect(wrapper.find('.btn-spinner').exists()).toBe(true);
  });

  it('does not show spinner when not loading', () => {
    const wrapper = mount(BaseButton);
    expect(wrapper.find('.btn-spinner').exists()).toBe(false);
  });

  it('has loading class when loading', () => {
    const wrapper = mount(BaseButton, { props: { loading: true } });
    expect(wrapper.find('.base-btn').classes()).toContain('loading');
  });
});

// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Button from '@/components/common/Button.vue';

describe('Button', () => {
  it('renders with default props', () => {
    const wrapper = mount(Button, { slots: { default: 'Click me' } });
    expect(wrapper.text()).toContain('Click me');
    expect(wrapper.find('button').exists()).toBe(true);
  });

  it('renders with primary variant', () => {
    const wrapper = mount(Button, { props: { variant: 'primary' } });
    expect(wrapper.find('.btn').classes()).toContain('btn-primary');
  });

  it('renders with secondary variant', () => {
    const wrapper = mount(Button, { props: { variant: 'secondary' } });
    expect(wrapper.find('.btn').classes()).toContain('btn-secondary');
  });

  it('renders with ghost variant', () => {
    const wrapper = mount(Button, { props: { variant: 'ghost' } });
    expect(wrapper.find('.btn').classes()).toContain('btn-ghost');
  });

  it('renders with icon variant', () => {
    const wrapper = mount(Button, { props: { variant: 'icon' } });
    expect(wrapper.find('.btn').classes()).toContain('btn-icon');
  });

  it('renders with sm size', () => {
    const wrapper = mount(Button, { props: { size: 'sm' } });
    expect(wrapper.find('.btn').classes()).toContain('btn-sm');
  });

  it('renders with md size', () => {
    const wrapper = mount(Button, { props: { size: 'md' } });
    expect(wrapper.find('.btn').classes()).not.toContain('btn-sm');
  });

  it('emits click when not disabled', async () => {
    const wrapper = mount(Button);
    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('click')).toBeTruthy();
  });

  it('does not emit click when disabled', async () => {
    const wrapper = mount(Button, { props: { disabled: true } });
    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('click')).toBeUndefined();
  });

  it('renders with type submit', () => {
    const wrapper = mount(Button, { props: { type: 'submit' } });
    expect(wrapper.find('button').attributes('type')).toBe('submit');
  });

  it('renders with type reset', () => {
    const wrapper = mount(Button, { props: { type: 'reset' } });
    expect(wrapper.find('button').attributes('type')).toBe('reset');
  });

  it('renders button type by default', () => {
    const wrapper = mount(Button);
    expect(wrapper.find('button').attributes('type')).toBe('button');
  });

  it('renders slot content', () => {
    const wrapper = mount(Button, { slots: { default: '<span>Inner</span>' } });
    expect(wrapper.text()).toContain('Inner');
  });
});

// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ToggleSwitch from '@/components/common/ToggleSwitch.vue';

describe('ToggleSwitch', () => {
  it('renders with modelValue false by default', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    expect(wrapper.find('.toggle').classes()).not.toContain('on');
  });

  it('renders with modelValue true', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: true } });
    expect(wrapper.find('.toggle').classes()).toContain('on');
  });

  it('emits update:modelValue on click', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    await wrapper.find('.toggle').trigger('click');
    expect(wrapper.emitted('update:modelValue')).toEqual([[true]]);
  });

  it('does not emit when disabled', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, disabled: true } });
    await wrapper.find('.toggle').trigger('click');
    expect(wrapper.emitted('update:modelValue')).toBeUndefined();
  });

  it('renders label when provided', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, label: 'Enable' } });
    expect(wrapper.find('.toggle-label').text()).toBe('Enable');
  });

  it('does not render label when not provided', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    expect(wrapper.find('.toggle-label').exists()).toBe(false);
  });

  it('toggles on Enter keydown', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    await wrapper.find('.toggle').trigger('keydown.enter');
    expect(wrapper.emitted('update:modelValue')).toEqual([[true]]);
  });

  it('toggles on Space keydown', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: true } });
    await wrapper.find('.toggle').trigger('keydown.space');
    expect(wrapper.emitted('update:modelValue')).toEqual([[false]]);
  });

  it('does not toggle on Enter when disabled', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, disabled: true } });
    await wrapper.find('.toggle').trigger('keydown.enter');
    expect(wrapper.emitted('update:modelValue')).toBeUndefined();
  });

  it('has aria-disabled attribute when disabled', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, disabled: true } });
    expect(wrapper.find('.toggle').attributes('aria-disabled')).toBe('true');
  });
});

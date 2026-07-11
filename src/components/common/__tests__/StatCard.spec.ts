import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StatCard from '@/components/common/StatCard.vue';

describe('StatCard', () => {
  it('renders label and value', () => {
    const wrapper = mount(StatCard, { props: { label: 'Downloads', value: 42 } });
    expect(wrapper.find('.stat-label').text()).toBe('Downloads');
    expect(wrapper.find('.stat-value').text()).toBe('42');
  });

  it('renders numeric value', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: 123 } });
    expect(wrapper.find('.stat-value').text()).toBe('123');
  });

  it('renders sub text when provided', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '10', sub: 'per day' } });
    expect(wrapper.find('.stat-sub').text()).toBe('per day');
  });

  it('does not render sub when not provided', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0' } });
    expect(wrapper.find('.stat-sub').exists()).toBe(false);
  });

  it('applies tint-warm class by default', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0' } });
    expect(wrapper.find('.stat-card').classes()).toContain('tint-warm');
  });

  it('applies tint-cool class', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0', tint: 'cool' } });
    expect(wrapper.find('.stat-card').classes()).toContain('tint-cool');
  });

  it('applies tint-soft class', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0', tint: 'soft' } });
    expect(wrapper.find('.stat-card').classes()).toContain('tint-soft');
  });

  it('applies tint-amber class', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0', tint: 'amber' } });
    expect(wrapper.find('.stat-card').classes()).toContain('tint-amber');
  });

  it('applies accent class to value when accent prop is true', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0', accent: true } });
    expect(wrapper.find('.stat-value').classes()).toContain('accent');
  });

  it('does not apply accent class by default', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0' } });
    expect(wrapper.find('.stat-value').classes()).not.toContain('accent');
  });

  it('applies valueColor style when provided', () => {
    const wrapper = mount(StatCard, { props: { label: 'X', value: '0', valueColor: 'rgb(255, 0, 0)' } });
    const statValue = wrapper.find('.stat-value');
    const style = statValue.attributes('style') ?? '';
    expect(style).toContain('255');
  });
});

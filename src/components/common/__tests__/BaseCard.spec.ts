// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BaseCard from '@/components/common/BaseCard.vue';

describe('BaseCard', () => {
  it('renders slot content', () => {
    const wrapper = mount(BaseCard, { slots: { default: 'Card content' } });
    expect(wrapper.text()).toContain('Card content');
  });

  it('has base-card class', () => {
    const wrapper = mount(BaseCard);
    expect(wrapper.find('.base-card').exists()).toBe(true);
  });

  it('renders variant-glass by default', () => {
    const wrapper = mount(BaseCard);
    expect(wrapper.find('.base-card').classes()).toContain('variant-glass');
  });

  it('renders variant-default', () => {
    const wrapper = mount(BaseCard, { props: { variant: 'default' } });
    expect(wrapper.find('.base-card').classes()).toContain('variant-default');
  });

  it('renders variant-elevated', () => {
    const wrapper = mount(BaseCard, { props: { variant: 'elevated' } });
    expect(wrapper.find('.base-card').classes()).toContain('variant-elevated');
  });

  it('adds liquid-glass class for non-default variants', () => {
    const wrapper = mount(BaseCard, { props: { variant: 'glass' } });
    expect(wrapper.find('.base-card').classes()).toContain('liquid-glass');
  });

  it('does not add liquid-glass for variant-default', () => {
    const wrapper = mount(BaseCard, { props: { variant: 'default' } });
    expect(wrapper.find('.base-card').classes()).not.toContain('liquid-glass');
  });

  it('renders pad-sm', () => {
    const wrapper = mount(BaseCard, { props: { padding: 'sm' } });
    expect(wrapper.find('.base-card').classes()).toContain('pad-sm');
  });

  it('renders pad-md by default', () => {
    const wrapper = mount(BaseCard);
    expect(wrapper.find('.base-card').classes()).toContain('pad-md');
  });

  it('renders pad-lg', () => {
    const wrapper = mount(BaseCard, { props: { padding: 'lg' } });
    expect(wrapper.find('.base-card').classes()).toContain('pad-lg');
  });

  it('adds interactive class when interactive', () => {
    const wrapper = mount(BaseCard, { props: { interactive: true } });
    expect(wrapper.find('.base-card').classes()).toContain('interactive');
  });

  it('does not add interactive class by default', () => {
    const wrapper = mount(BaseCard);
    expect(wrapper.find('.base-card').classes()).not.toContain('interactive');
  });
});

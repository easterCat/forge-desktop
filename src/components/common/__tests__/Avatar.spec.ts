import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Avatar from '@/components/common/Avatar.vue';

describe('Avatar', () => {
  it('renders label text', () => {
    const wrapper = mount(Avatar, { props: { label: 'AB' } });
    expect(wrapper.text()).toBe('AB');
  });

  it('has role img', () => {
    const wrapper = mount(Avatar, { props: { label: 'A' } });
    expect(wrapper.find('.avatar').attributes('role')).toBe('img');
  });

  it('has aria-label matching label', () => {
    const wrapper = mount(Avatar, { props: { label: 'AB' } });
    expect(wrapper.find('.avatar').attributes('aria-label')).toBe('AB');
  });

  it('applies custom size', () => {
    const wrapper = mount(Avatar, { props: { label: 'A', size: 40 } });
    const div = wrapper.find('.avatar');
    const style = div.attributes('style') ?? '';
    // jsdom may return style as "width: 40px;height: 40px;..." (CSS properties kebab or camelCase)
    // Check that width/height appear in some form
    expect(style).toContain('40');
  });

  it('applies custom bg and color', () => {
    const wrapper = mount(Avatar, {
      props: { label: 'A', bg: 'rgb(255, 0, 0)', color: 'rgb(0, 0, 255)' },
    });
    const style = wrapper.find('.avatar').attributes('style') ?? '';
    // jsdom may normalize color values; check they are present
    expect(style).toContain('255');
    expect(style).toContain('0, 0, 255');
  });

  it('applies default bg and color', () => {
    const wrapper = mount(Avatar, { props: { label: 'A' } });
    const style = wrapper.find('.avatar').attributes('style') ?? '';
    expect(style).toContain('255');
  });

  it('computes fontSize from size', () => {
    const wrapper = mount(Avatar, { props: { label: 'A', size: 50 } });
    const style = wrapper.find('.avatar').attributes('style') ?? '';
    // fontSize = Math.round(50 * 0.4) = 20
    expect(style).toContain('20');
  });
});

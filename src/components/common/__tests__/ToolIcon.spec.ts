// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ToolIcon from '@/components/common/ToolIcon.vue';

describe('ToolIcon', () => {
  it('renders fallback span for unknown toolKey', () => {
    const wrapper = mount(ToolIcon, {
      props: { toolKey: 'unknown-tool' },
    });
    expect(wrapper.find('.tool-icon-fallback').exists()).toBe(true);
  });

  it('empty toolKey shows ?? fallback', () => {
    const wrapper = mount(ToolIcon, {
      props: { toolKey: '' },
    });
    expect(wrapper.find('.tool-icon-fallback').text()).toBe('??');
  });
});

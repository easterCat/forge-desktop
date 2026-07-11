// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import DropdownMenu from '@/components/common/DropdownMenu.vue';

describe('DropdownMenu', () => {
  it('renders trigger slot when closed', () => {
    const wrapper = mount(DropdownMenu, {
      props: { modelValue: false },
      slots: {
        trigger: '<button>Open</button>',
        default: '<div>Menu item</div>',
      },
    });
    expect(wrapper.text()).toContain('Open');
  });

  it('does not render menu content when modelValue is false', () => {
    const wrapper = mount(DropdownMenu, {
      props: { modelValue: false },
      slots: { default: '<div class="menu-item">Item</div>' },
    });
    expect(wrapper.find('.dropdown-menu').exists()).toBe(false);
  });

  it('has dropdown-wrapper class', () => {
    const wrapper = mount(DropdownMenu, {
      props: { modelValue: false },
    });
    expect(wrapper.find('.dropdown-wrapper').exists()).toBe(true);
  });
});

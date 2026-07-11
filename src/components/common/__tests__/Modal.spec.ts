import { describe, it, expect, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import Modal from '@/components/common/Modal.vue';

describe('Modal', () => {
  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('renders with title when open', () => {
    const wrapper = mount(Modal, {
      props: { modelValue: true, title: 'My Modal' },
      global: {
        stubs: {
          Teleport: {
            template: '<div><slot /></div>',
          },
          Transition: {
            template: '<slot />',
          },
        },
      },
    });
    expect(wrapper.text()).toContain('My Modal');
  });

  it('renders modal-overlay when open', () => {
    const wrapper = mount(Modal, {
      props: { modelValue: true },
      global: {
        stubs: {
          Teleport: {
            template: '<div><slot /></div>',
          },
          Transition: {
            template: '<slot />',
          },
        },
      },
    });
    expect(wrapper.find('.modal-overlay').exists()).toBe(true);
  });

  it('does not render when modelValue is false', () => {
    const wrapper = mount(Modal, {
      props: { modelValue: false, title: 'Hidden' },
      global: {
        stubs: {
          Teleport: {
            template: '<div><slot /></div>',
          },
          Transition: {
            template: '<slot />',
          },
        },
      },
    });
    expect(wrapper.find('.modal-overlay').exists()).toBe(false);
  });

  it('renders slot content when open', () => {
    const wrapper = mount(Modal, {
      props: { modelValue: true },
      slots: { default: '<p>Modal content</p>' },
      global: {
        stubs: {
          Teleport: {
            template: '<div><slot /></div>',
          },
          Transition: {
            template: '<slot />',
          },
        },
      },
    });
    expect(wrapper.text()).toContain('Modal content');
  });
});

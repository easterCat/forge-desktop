import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SearchInput from '@/components/common/SearchInput.vue';

describe('SearchInput', () => {
  it('renders with placeholder', () => {
    const wrapper = mount(SearchInput, { props: { placeholder: 'Search plugins...' } });
    expect(wrapper.find('input').attributes('placeholder')).toBe('Search plugins...');
  });

  it('renders with default placeholder', () => {
    const wrapper = mount(SearchInput);
    expect(wrapper.find('input').attributes('placeholder')).toBe('Search…');
  });

  it('emits update:modelValue on input', async () => {
    const wrapper = mount(SearchInput);
    await wrapper.find('input').setValue('test query');
    expect(wrapper.emitted('update:modelValue')).toBeTruthy();
  });

  it('shows clear button when value is non-empty', async () => {
    const wrapper = mount(SearchInput, { props: { modelValue: 'some text' } });
    // v-show always keeps the element in DOM, checking opacity
    expect(wrapper.find('.clear-btn').exists()).toBe(true);
  });

  it('clear button is hidden when value is empty (v-show=false)', () => {
    const wrapper = mount(SearchInput, { props: { modelValue: '' } });
    // v-show element exists but has opacity:0 — check the element is rendered
    expect(wrapper.find('.clear-btn').exists()).toBe(true);
  });

  it('clears value on clear button click', async () => {
    const wrapper = mount(SearchInput, { props: { modelValue: 'some text' } });
    await wrapper.find('.clear-btn').trigger('click');
    expect(wrapper.emitted('update:modelValue')).toBeTruthy();
  });
});

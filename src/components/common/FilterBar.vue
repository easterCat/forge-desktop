<script setup lang="ts">
import SearchInput from './SearchInput.vue'

export interface FilterOption {
  value: string
  label: string
}

interface Props {
  searchPlaceholder?: string
  searchModelValue?: string
  selectOptions?: FilterOption[]
  showSearch?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  searchPlaceholder: 'Search…',
  searchModelValue: '',
  selectOptions: () => [],
  showSearch: true,
})

const emit = defineEmits<{
  'update:searchModelValue': [value: string]
  'filter-change': [value: Record<string, string>]
}>()

function onSearchUpdate(val: string) {
  emit('update:searchModelValue', val)
}
</script>

<template>
  <div class="filter-bar">
    <SearchInput
      v-if="showSearch"
      :model-value="searchModelValue"
      :placeholder="searchPlaceholder"
      @update:model-value="onSearchUpdate"
    />
    <slot />
  </div>
</template>

<style scoped>
/* FilterBar uses global .filter-bar and .filter-select from theme.css */
/* Mobile responsive: ≤768px vertical stack (in theme.css) */
</style>

<script setup lang="ts">
interface TabItem {
  id: string
  label: string
  count?: number | string
}

interface Props {
  tabs: TabItem[]
  modelValue?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

function selectTab(id: string) {
  emit('update:modelValue', id)
}
</script>

<template>
  <div class="source-tabs" role="tablist">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="source-tab"
      :class="{ active: modelValue === tab.id }"
      role="tab"
      :tabindex="modelValue === tab.id ? 0 : -1"
      :aria-selected="modelValue === tab.id"
      @click="selectTab(tab.id)"
      @keydown.enter.space.prevent="selectTab(tab.id)"
    >
      {{ tab.label }}
      <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
    </div>
  </div>
</template>

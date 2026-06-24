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
  <div class="tab-bar">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      :class="['tab-item', { active: modelValue === tab.id }]"
      @click="selectTab(tab.id)"
    >
      {{ tab.label }}
      <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
    </div>
  </div>
</template>

<style scoped>
.tab-count {
  margin-left: 4px;
  font-family: var(--font-mono);
  font-size: 10px;
  opacity: 0.5;
}
</style>

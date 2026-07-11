<template>
  <div
    ref="containerRef"
    class="virtual-grid-scroller"
    @scroll.passive="onScroll"
  >
    <!-- Spacer div provides the total scrollable height. Items are
         absolutely positioned relative to it (the spacer is the closest
         positioned ancestor). -->
    <div
      class="virtual-grid-spacer"
      :style="{ height: `${totalHeight}px` }"
    >
      <div
        v-for="{ item, index, top, left } in visibleItems"
        :key="getItemKey(item, index)"
        class="virtual-grid-item"
        :style="{
          position: 'absolute',
          top: `${top}px`,
          left: `${left}px`,
          width: `${itemWidth}px`,
          height: `${rowHeight}px`,
        }"
      >
        <slot :item="item" :index="index" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, isRef, type Ref } from 'vue';
import { useVirtualGrid } from '@/composables/useVirtualGrid';

// `defineProps` registers the prop type for template auto-destructure
// AND exposes a `props` binding used in setup. ESLint flags the macro's
// result as "unused" because it doesn't see the binding's runtime usage.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const props = withDefaults(
  defineProps<{
    /** Array of items to render (plain array or Ref — both accepted). */
    items: unknown[] | Ref<unknown[]>;
    columnWidth?: number;
    rowHeight?: number;
    gap?: number;
    overscanPx?: number;
  }>(),
  {
    columnWidth: 320,
    rowHeight: 280,
    gap: 16,
    overscanPx: 500,
  }
);

// Normalise to Ref so useVirtualGrid always receives the same shape.
const itemsRef = computed(() =>
  isRef(props.items) ? props.items.value : props.items
);

const { visibleItems, totalHeight, itemWidth, containerRef, onScroll, resetScroll } =
  useVirtualGrid({
    items: itemsRef as Ref<unknown[]>,
    columnWidth: props.columnWidth,
    rowHeight: props.rowHeight,
    gap: props.gap,
    overscanPx: props.overscanPx,
  });

function getItemKey(item: unknown, index: number): string | number {
  if (item && typeof item === 'object' && 'id' in item) {
    const id = (item as { id: unknown }).id;
    if (typeof id === 'string' || typeof id === 'number') return id;
  }
  return index;
}

defineExpose({ resetScroll });
</script>

<style scoped>
.virtual-grid-scroller {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}

.virtual-grid-spacer {
  position: relative;
  width: 100%;
  min-height: 100%;
}

.virtual-grid-item {
  box-sizing: border-box;
}

/* Slot content fills the fixed-height item so cards in the same row
   share a single, uniform height. */
.virtual-grid-item > :deep(*) {
  height: 100%;
}
</style>

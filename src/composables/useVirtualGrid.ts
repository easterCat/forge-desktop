/**
 * useVirtualGrid - Virtual scrolling composable for CSS Grid-based layouts.
 *
 * Renders only items within the visible viewport plus an overscan buffer,
 * using absolute positioning for each item. Column count adapts to container
 * width (responsive grid); per-item width is derived so items collectively
 * fill the container's full width.
 *
 * @example
 * const {
 *   visibleItems,   // Ref<VirtualGridItem[]>
 *   totalHeight,    // Ref<number> — height of the virtual scroll container
 *   itemWidth,      // Ref<number> — per-item width in px (fills container)
 *   containerRef,   // Ref<HTMLElement | null> — bind to the scroll container
 *   onScroll,       // Event handler for scroll events
 * } = useVirtualGrid({
 *   items: plugins,     // Ref<unknown[]>
 *   columnWidth: 320,    // preferred column width; actual width adapts
 *   rowHeight: 280,     // px per row (estimated)
 *   gap: 16,            // px between items
 *   overscanPx: 500,    // px buffer above/below viewport
 * });
 */

import { ref, computed, onMounted, onBeforeUnmount, type Ref } from 'vue';

export interface VirtualGridItem {
  item: unknown;
  index: number;
  top: number;
  left: number;
  col: number;
  row: number;
}

export interface UseVirtualGridOptions {
  items: Ref<unknown[]>;
  /**
   * Preferred per-column width in pixels. Used only to compute how many
   * columns fit in the container; the actual rendered width adapts to the
   * container's width so items collectively fill it (responsive).
   */
  columnWidth: number;
  rowHeight: number;
  gap: number;
  overscanPx: number;
}

export interface UseVirtualGridReturn {
  visibleItems: Ref<VirtualGridItem[]>;
  totalHeight: Ref<number>;
  /**
   * Per-item width in pixels. Equal to `columnWidth` when the column count
   * divides the container exactly; otherwise stretched so `columns * width
   * + (columns-1) * gap` covers the full container width.
   */
  itemWidth: Ref<number>;
  containerRef: Ref<HTMLElement | null>;
  onScroll: (e: Event) => void;
  scrollToTop: () => void;
  resetScroll: () => void;
}

export function useVirtualGrid({
  items,
  columnWidth,
  rowHeight,
  gap,
  overscanPx,
}: UseVirtualGridOptions): UseVirtualGridReturn {
  const scrollTop = ref(0);
  const containerHeight = ref(0);
  const containerWidth = ref(0);
  const containerRef = ref<HTMLElement | null>(null);

  let resizeObserver: ResizeObserver | null = null;

  const columns = computed(() =>
    Math.max(1, Math.floor((containerWidth.value + gap) / (columnWidth + gap)))
  );

  // Items collectively fill the container width. We divide the available
  // space (container minus inter-column gaps) by the number of columns so
  // the right edge of the last column lands flush with the spacer's edge.
  // Floor + remainder distribution keeps each column visually balanced.
  const itemWidth = computed(() => {
    const cols = columns.value;
    if (containerWidth.value <= 0) return columnWidth;
    const available = containerWidth.value - gap * (cols - 1);
    return Math.max(1, Math.floor(available / cols));
  });

  const totalRows = computed(() =>
    Math.ceil(items.value.length / columns.value)
  );

  const totalHeight = computed(() =>
    totalRows.value * (rowHeight + gap)
  );

  const startRow = computed(() => {
    const raw = Math.floor((scrollTop.value - overscanPx) / (rowHeight + gap));
    return Math.max(0, raw);
  });

  const endRow = computed(() => {
    const raw = Math.ceil(
      (scrollTop.value + containerHeight.value + overscanPx) / (rowHeight + gap)
    );
    return Math.min(totalRows.value - 1, raw);
  });

  const visibleItems = computed<VirtualGridItem[]>(() => {
    const cols = columns.value;
    const w = itemWidth.value;
    const result: VirtualGridItem[] = [];

    for (let r = startRow.value; r <= endRow.value; r++) {
      for (let c = 0; c < cols; c++) {
        const idx = r * cols + c;
        if (idx >= items.value.length) break;

        const item = items.value[idx];
        if (!item) continue;

        result.push({
          item,
          index: idx,
          top: r * (rowHeight + gap),
          left: c * (w + gap),
          col: c,
          row: r,
        });
      }
    }

    return result;
  });

  function onScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop.value = target.scrollTop;
  }

  function scrollToTop() {
    if (containerRef.value) {
      containerRef.value.scrollTop = 0;
      scrollTop.value = 0;
    }
  }

  function resetScroll() {
    scrollTop.value = 0;
    if (containerRef.value) {
      containerRef.value.scrollTop = 0;
    }
  }

  onMounted(() => {
    if (!containerRef.value) return;

    const el = containerRef.value;

    // Capture initial dimensions
    containerHeight.value = el.clientHeight;
    containerWidth.value = el.clientWidth;

    // Watch for container resize (window drag, CSS changes, etc.)
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { height, width } = entry.contentRect;
        containerHeight.value = height;
        containerWidth.value = width;
      }
    });

    resizeObserver.observe(el);
  });

  onBeforeUnmount(() => {
    resizeObserver?.disconnect();
    resizeObserver = null;
  });

  return {
    visibleItems,
    totalHeight,
    itemWidth,
    containerRef,
    onScroll,
    scrollToTop,
    resetScroll,
  };
}

/**
 * Unit tests for useVirtualGrid composable — pure logic tests.
 *
 * Covers:
 * - column count adapts to container width
 * - startRow / endRow with overscan
 * - visibleItems maps correct indices and positions
 * - empty / partial last-row handling
 * - boundary conditions (0 items, 1 item, scroll at 0)
 *
 * These tests run the core grid math in isolation (no DOM, no ResizeObserver),
 * which mirrors what useVirtualGrid computes internally.
 *
 * Run with: pnpm test src/composables/__tests__/useVirtualGrid.spec.ts
 */

import { describe, it, expect } from 'vitest';

/* ─────────────────────────────────────────────────────────────────────────────
   Pure computation helper — mirrors the core logic inside useVirtualGrid.
   ───────────────────────────────────────────────────────────────────────────── */

interface GridResult {
  cols: number;
  totalRows: number;
  totalHeight: number;
  itemWidth: number;
  startRow: number;
  endRow: number;
  visible: Array<{
    index: number;
    top: number;
    left: number;
    row: number;
    col: number;
  }>;
}

function computeGrid({
  itemCount,
  columnWidth = 320,
  rowHeight = 280,
  gap = 16,
  overscanPx = 500,
  scrollTop = 0,
  containerHeight = 700,
  containerWidth = 1100,
}: {
  itemCount: number;
  columnWidth?: number;
  rowHeight?: number;
  gap?: number;
  overscanPx?: number;
  scrollTop?: number;
  containerHeight?: number;
  containerWidth?: number;
}): GridResult {
  const cols = Math.max(1, Math.floor((containerWidth + gap) / (columnWidth + gap)));
  const totalRows = Math.ceil(itemCount / cols);
  const totalHeight = totalRows * (rowHeight + gap);

  // Item width stretches to fill the container; only the column count is
  // derived from the preferred `columnWidth` parameter.
  const availableWidth = Math.max(1, containerWidth - gap * (cols - 1));
  const itemWidth = Math.max(1, Math.floor(availableWidth / cols));

  const startRow = Math.max(
    0,
    Math.floor((scrollTop - overscanPx) / (rowHeight + gap))
  );
  const endRow = Math.min(
    totalRows - 1,
    Math.ceil((scrollTop + containerHeight + overscanPx) / (rowHeight + gap))
  );

  const visible: GridResult['visible'] = [];

  for (let r = startRow; r <= endRow; r++) {
    for (let c = 0; c < cols; c++) {
      const idx = r * cols + c;
      if (idx >= itemCount) break;
      visible.push({
        index: idx,
        top: r * (rowHeight + gap),
        left: c * (itemWidth + gap),
        row: r,
        col: c,
      });
    }
  }

  return { cols, totalRows, totalHeight, itemWidth, startRow, endRow, visible };
}

/* ─────────────────────────────────────────────────────────────────────────────
   Test cases
   ───────────────────────────────────────────────────────────────────────────── */

describe('useVirtualGrid — column count tests', () => {
  it('1 column at 300px container width', () => {
    expect(computeGrid({ itemCount: 10, containerWidth: 300 }).cols).toBe(1);
  });

  it('3 columns at ~1100px (default 1200px window)', () => {
    expect(computeGrid({ itemCount: 10, containerWidth: 1100 }).cols).toBe(3);
  });

  it('4 columns at 1400px container width', () => {
    expect(computeGrid({ itemCount: 20, containerWidth: 1400 }).cols).toBe(4);
  });
});

describe('useVirtualGrid — overscan 500px / rowHeight 280 / gap 16', () => {
  it('startRow = 0 at scrollTop=0 (clamped from -2)', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 0 });
    expect(r.startRow).toBe(0);
  });

  it('endRow = 5 at scrollTop=0 with 700px viewport', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 0 });
    expect(r.endRow).toBe(5);
  });

  it('first visible item is index 0', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 0 });
    expect(r.visible[0].index).toBe(0);
  });

  it('startRow = 0 after scrolling 700px (overscan still covers top)', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 700 });
    expect(r.startRow).toBe(0);
  });

  it('endRow = 7 after scrolling 700px', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 700 });
    expect(r.endRow).toBe(7);
  });

  it('startRow = 3 after scrolling 1400px', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 1400 });
    expect(r.startRow).toBe(3);
  });

  it('endRow = 9 after scrolling 1400px', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 1400 });
    expect(r.endRow).toBe(9);
  });

  it('endRow clamped to totalRows-1 (9) when scrolling past bottom', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 10000 });
    expect(r.endRow).toBe(9);
  });

  it('startRow clamped to 0 when scrollTop < overscanPx', () => {
    const r = computeGrid({ itemCount: 30, scrollTop: 200 });
    expect(r.startRow).toBe(0);
  });
});

describe('useVirtualGrid — visibleItems indices and positions', () => {
  it('0 items → no visible items', () => {
    expect(computeGrid({ itemCount: 0 }).visible.length).toBe(0);
  });

  it('1 item → 1 visible item', () => {
    expect(computeGrid({ itemCount: 1 }).visible.length).toBe(1);
  });

  it('1 item → index 0', () => {
    expect(computeGrid({ itemCount: 1 }).visible[0].index).toBe(0);
  });

  it('31 items / 3 cols → 11 rows (ceil(31/3))', () => {
    expect(computeGrid({ itemCount: 31 }).totalRows).toBe(11);
  });

  it('at scrollTop=0, last visible index is 17 (row 5, col 2) — items 18-30 are below the fold', () => {
    const r = computeGrid({ itemCount: 31 });
    const lastInRange = r.visible[r.visible.length - 1];
    expect(lastInRange.index).toBe(17);
  });

  it('1-column layout: cols = 1', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.cols).toBe(1);
  });

  it('1-column: item 0 top = 0', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[0].top).toBe(0);
  });

  it('1-column: item 0 left = 0', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[0].left).toBe(0);
  });

  it('1-column: item 1 top = 296', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[1].top).toBe(296);
  });

  it('1-column: item 2 top = 592', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[2].top).toBe(592);
  });

  it('3-column: cols = 3', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.cols).toBe(3);
  });

  it('3-column: itemWidth stretches to fill container', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.itemWidth).toBe(356);
  });

  it('3-column: last column right edge = container width (fills exactly)', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    const rightEdge = 2 * (r.itemWidth + 16) + r.itemWidth;
    expect(rightEdge).toBe(1100);
  });

  it('3-column: item 0 top = 0', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[0].top).toBe(0);
  });

  it('3-column: item 0 left = 0', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[0].left).toBe(0);
  });

  it('3-column: item 1 top = 0', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[1].top).toBe(0);
  });

  it('3-column: item 1 left = 356+16 (itemWidth+gap)', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[1].left).toBe(372);
  });

  it('3-column: item 2 top = 0', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[2].top).toBe(0);
  });

  it('3-column: item 2 left = 2*(itemWidth+gap)', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[2].left).toBe(744);
  });

  it('3-column: item 3 top = 296', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[3].top).toBe(296);
  });

  it('3-column: item 3 left = 0', () => {
    const r = computeGrid({ itemCount: 12, containerWidth: 1100, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.visible[3].left).toBe(0);
  });
});

describe('useVirtualGrid — totalHeight', () => {
  it('10 items / 3 cols / 4 rows → 4×296px = 1184px', () => {
    expect(computeGrid({ itemCount: 10 }).totalHeight).toBe(4 * 296);
  });

  it('0 items → 0px totalHeight', () => {
    expect(computeGrid({ itemCount: 0 }).totalHeight).toBe(0);
  });
});

describe('useVirtualGrid — fill-width behavior', () => {
  it('1-column fill: cols = 1', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.cols).toBe(1);
  });

  it('1-column fill: itemWidth = container width (no gaps)', () => {
    const r = computeGrid({ itemCount: 6, containerWidth: 400, columnWidth: 320, gap: 16, rowHeight: 280, scrollTop: 0 });
    expect(r.itemWidth).toBe(400);
  });

  it('4-column fill: cols = 4', () => {
    const r = computeGrid({ itemCount: 20, containerWidth: 1400, columnWidth: 320, gap: 16 });
    expect(r.cols).toBe(4);
  });

  it('4-column fill: itemWidth stretches to 338px', () => {
    const r = computeGrid({ itemCount: 20, containerWidth: 1400, columnWidth: 320, gap: 16 });
    expect(r.itemWidth).toBe(338);
  });

  it('4-column fill: last column right edge = 1400 (container width)', () => {
    const r = computeGrid({ itemCount: 20, containerWidth: 1400, columnWidth: 320, gap: 16 });
    const rightEdge = 3 * (r.itemWidth + 16) + r.itemWidth;
    expect(rightEdge).toBe(1400);
  });

  it('2-column fill: cols = 2', () => {
    const r = computeGrid({ itemCount: 8, containerWidth: 720, columnWidth: 320, gap: 16 });
    expect(r.cols).toBe(2);
  });

  it('2-column fill: itemWidth stretches to 352px', () => {
    const r = computeGrid({ itemCount: 8, containerWidth: 720, columnWidth: 320, gap: 16 });
    expect(r.itemWidth).toBe(352);
  });

  it('2-column fill: last column right edge = 720 (container width)', () => {
    const r = computeGrid({ itemCount: 8, containerWidth: 720, columnWidth: 320, gap: 16 });
    const rightEdge = 1 * (r.itemWidth + 16) + r.itemWidth;
    expect(rightEdge).toBe(720);
  });
});

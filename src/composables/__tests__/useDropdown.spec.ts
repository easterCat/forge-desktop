import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { useDropdown } from '../useDropdown';

describe('useDropdown', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns positionStyle and computePosition', () => {
    const trigger = ref<HTMLElement | null>(null);
    const { positionStyle, computePosition } = useDropdown(trigger);
    expect(positionStyle.value).toEqual({});
    expect(typeof computePosition).toBe('function');
  });

  it('computePosition sets empty style when trigger is null', async () => {
    const trigger = ref<HTMLElement | null>(null);
    const { positionStyle, computePosition } = useDropdown(trigger);
    await computePosition();
    expect(positionStyle.value).toEqual({});
  });

  it('computePosition sets position when trigger has bounding rect', async () => {
    const mockEl = {
      getBoundingClientRect: () => ({
        top: 200,
        bottom: 240,
        left: 100,
        right: 200,
      }),
    } as unknown as HTMLElement;
    const trigger = ref<HTMLElement | null>(mockEl);

    // Mock window dimensions
    const originalInnerWidth = window.innerWidth;
    Object.defineProperty(window, 'innerWidth', { value: 1200, configurable: true });
    const originalInnerHeight = window.innerHeight;
    Object.defineProperty(window, 'innerHeight', { value: 800, configurable: true });

    const { positionStyle, computePosition } = useDropdown(trigger);
    await computePosition(200, 300);

    expect(positionStyle.value).toMatchObject({
      position: 'fixed',
    });
    expect(positionStyle.value).toHaveProperty('top');
    expect(positionStyle.value).toHaveProperty('left');

    // Restore
    Object.defineProperty(window, 'innerWidth', { value: originalInnerWidth, configurable: true });
    Object.defineProperty(window, 'innerHeight', { value: originalInnerHeight, configurable: true });
  });

  it('computePosition opens upward when space above', async () => {
    const mockEl = {
      getBoundingClientRect: () => ({
        top: 500,
        bottom: 540,
        left: 100,
        right: 200,
      }),
    } as unknown as HTMLElement;
    const trigger = ref<HTMLElement | null>(mockEl);

    const originalInnerHeight = window.innerHeight;
    Object.defineProperty(window, 'innerHeight', { value: 600, configurable: true });

    const { positionStyle, computePosition } = useDropdown(trigger);
    await computePosition(200, 100);

    // Since top (500) - gap - menuHeight (100) = 394 > 0, should open upward
    expect(positionStyle.value).toHaveProperty('top');
    expect(positionStyle.value).toHaveProperty('left');

    Object.defineProperty(window, 'innerHeight', { value: originalInnerHeight, configurable: true });
  });

  it('computePosition clamps left when near right edge', async () => {
    const mockEl = {
      getBoundingClientRect: () => ({
        top: 200,
        bottom: 240,
        left: 1100,
        right: 1200,
      }),
    } as unknown as HTMLElement;
    const trigger = ref<HTMLElement | null>(mockEl);

    const originalInnerWidth = window.innerWidth;
    Object.defineProperty(window, 'innerWidth', { value: 1200, configurable: true });

    const { positionStyle, computePosition } = useDropdown(trigger);
    await computePosition(300, 200);

    // left = 1200 - 300 = 900; 900 + 300 = 1200 > 1200 - 8, should clamp
    expect(positionStyle.value.left).toBeDefined();

    Object.defineProperty(window, 'innerWidth', { value: originalInnerWidth, configurable: true });
  });
});

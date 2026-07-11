import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setActivePinia } from 'pinia';
import { createPinia } from 'pinia';
import { useOperationProgress, STAGE_CONFIG } from '../useOperationProgress';

describe('useOperationProgress', () => {
  let pinia: ReturnType<typeof createPinia>;

  beforeEach(() => {
    vi.useFakeTimers();
    pinia = createPinia();
    setActivePinia(pinia);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('starts with empty operations', () => {
    const { operations } = useOperationProgress();
    expect(operations.value.size).toBe(0);
  });

  it('activeOperation is null initially', () => {
    const { activeOperation } = useOperationProgress();
    expect(activeOperation.value).toBeNull();
  });

  it('isAnyActive is false initially', () => {
    const { isAnyActive } = useOperationProgress();
    expect(isAnyActive.value).toBe(false);
  });

  it('startOperation creates an operation in preparing stage', () => {
    const { operations, startOperation } = useOperationProgress();
    startOperation('claude-code');
    expect(operations.value.get('claude-code')).toMatchObject({
      toolKey: 'claude-code',
      stage: 'preparing',
      progress: 0,
      canCancel: true,
      canRetry: false,
    });
  });

  it('activeOperation returns the active operation', () => {
    const { startOperation, activeOperation } = useOperationProgress();
    startOperation('codex');
    expect(activeOperation.value).not.toBeNull();
    expect(activeOperation.value!.toolKey).toBe('codex');
  });

  it('updateProgress changes stage and progress', () => {
    const { startOperation, updateProgress, getOperation } = useOperationProgress();
    startOperation('codex');
    updateProgress('codex', 'downloading', 50, 'Downloading...');
    const op = getOperation('codex')!;
    expect(op.stage).toBe('downloading');
    expect(op.progress).toBe(50);
    expect(op.message).toBe('Downloading...');
  });

  it('updateProgress clamps progress to 0-100', () => {
    const { startOperation, updateProgress, getOperation } = useOperationProgress();
    startOperation('gemini');
    updateProgress('gemini', 'installing', 150, 'Over 100');
    expect(getOperation('gemini')!.progress).toBe(100);
    updateProgress('gemini', 'installing', -10, 'Under 0');
    expect(getOperation('gemini')!.progress).toBe(0);
  });

  it('updateProgress enables cancel during downloading/installing', () => {
    const { startOperation, updateProgress, getOperation } = useOperationProgress();
    startOperation('codex');
    updateProgress('codex', 'downloading', 10, 'dl');
    expect(getOperation('codex')!.canCancel).toBe(true);
    updateProgress('codex', 'installing', 50, 'install');
    expect(getOperation('codex')!.canCancel).toBe(true);
    updateProgress('codex', 'verifying', 90, 'verify');
    expect(getOperation('codex')!.canCancel).toBe(false);
  });

  it('completeOperation marks success', () => {
    const { startOperation, completeOperation, getOperation, activeOperation } = useOperationProgress();
    startOperation('codex');
    completeOperation('codex', true, 'Done!');
    const op = getOperation('codex')!;
    expect(op.stage).toBe('completed');
    expect(op.progress).toBe(100);
    expect(op.message).toBe('Done!');
    expect(op.canRetry).toBe(false);
    expect(op.canCancel).toBe(false);
    expect(activeOperation.value).toBeNull();
  });

  it('completeOperation marks failure', () => {
    const { startOperation, completeOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    completeOperation('codex', false, 'Network error');
    const op = getOperation('codex')!;
    expect(op.stage).toBe('failed');
    expect(op.canRetry).toBe(true);
    expect(op.canCancel).toBe(false);
  });

  it('completeOperation uses default message when not provided', () => {
    const { startOperation, completeOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    completeOperation('codex', true);
    expect(getOperation('codex')!.message).toBe('Completed successfully');
    completeOperation('codex', false);
    expect(getOperation('codex')!.message).toBe('Operation failed');
  });

  it('cancelOperation cancels when canCancel', () => {
    const { startOperation, updateProgress, cancelOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    updateProgress('codex', 'downloading', 30, 'dl');
    cancelOperation('codex');
    const op = getOperation('codex')!;
    expect(op.stage).toBe('cancelled');
    expect(op.canRetry).toBe(true);
  });

  it('cancelOperation does nothing when canCancel is false', () => {
    const { startOperation, updateProgress, cancelOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    updateProgress('codex', 'verifying', 90, 'verify');
    cancelOperation('codex');
    expect(getOperation('codex')!.stage).toBe('verifying');
  });

  it('retryOperation resets to preparing', () => {
    const { startOperation, completeOperation, retryOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    completeOperation('codex', false, 'Failed');
    retryOperation('codex');
    const op = getOperation('codex')!;
    expect(op.stage).toBe('preparing');
    expect(op.progress).toBe(0);
    expect(op.canCancel).toBe(true);
  });

  it('retryOperation does nothing when canRetry is false', () => {
    const { startOperation, retryOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    expect(getOperation('codex')!.canRetry).toBe(false);
    retryOperation('codex');
    expect(getOperation('codex')!.stage).toBe('preparing');
  });

  it('clearCompleted removes completed/failed/cancelled operations', () => {
    const { startOperation, completeOperation, clearCompleted, operations } = useOperationProgress();
    startOperation('a');
    startOperation('b');
    startOperation('c');
    completeOperation('a', true);
    completeOperation('b', false);
    clearCompleted();
    expect(operations.value.size).toBe(1);
    expect(operations.value.has('c')).toBe(true);
  });

  it('getOperation returns operation or undefined', () => {
    const { startOperation, getOperation } = useOperationProgress();
    startOperation('codex');
    expect(getOperation('codex')!.toolKey).toBe('codex');
    expect(getOperation('missing')).toBeUndefined();
  });
});

describe('STAGE_CONFIG', () => {
  it('contains label and icon for all stages', () => {
    const stages = ['idle', 'preparing', 'downloading', 'installing', 'verifying', 'completed', 'failed', 'cancelled'] as const;
    for (const stage of stages) {
      expect(STAGE_CONFIG[stage]).toHaveProperty('label');
      expect(STAGE_CONFIG[stage]).toHaveProperty('icon');
    }
  });
});

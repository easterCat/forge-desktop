import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export type OperationStage = 'idle' | 'preparing' | 'downloading' | 'installing' | 'verifying' | 'completed' | 'failed' | 'cancelled';

export interface OperationProgress {
  toolKey: string;
  stage: OperationStage;
  progress: number;
  message: string;
  error?: string;
  startTime: number;
  canRetry: boolean;
  canCancel: boolean;
}

export const useOperationsStore = defineStore('operations', () => {
  const operations = ref<Map<string, OperationProgress>>(new Map());

  const activeOperation = computed(() => {
    for (const op of operations.value.values()) {
      if (op.stage !== 'completed' && op.stage !== 'failed' && op.stage !== 'cancelled' && op.stage !== 'idle') {
        return op;
      }
    }
    return null;
  });

  const isAnyActive = computed(() => activeOperation.value !== null);

  const startOperation = (toolKey: string) => {
    const next = new Map(operations.value);
    next.set(toolKey, {
      toolKey,
      stage: 'preparing',
      progress: 0,
      message: 'Preparing...',
      startTime: Date.now(),
      canRetry: false,
      canCancel: true,
    });
    operations.value = next;
  };

  const updateProgress = (
    toolKey: string,
    stage: OperationStage,
    progress: number,
    message: string
  ) => {
    const existing = operations.value.get(toolKey);
    if (existing) {
      const next = new Map(operations.value);
      next.set(toolKey, {
        ...existing,
        stage,
        progress: Math.min(100, Math.max(0, progress)),
        message,
        canCancel: stage === 'downloading' || stage === 'installing',
        canRetry: false,
      });
      operations.value = next;
    }
  };

  const completeOperation = (toolKey: string, success: boolean, message?: string) => {
    const existing = operations.value.get(toolKey);
    if (existing) {
      const next = new Map(operations.value);
      next.set(toolKey, {
        ...existing,
        stage: success ? 'completed' : 'failed',
        progress: success ? 100 : existing.progress,
        message: message ?? (success ? 'Completed successfully' : 'Operation failed'),
        canRetry: !success,
        canCancel: false,
      });
      operations.value = next;
    }
  };

  const cancelOperation = (toolKey: string) => {
    const existing = operations.value.get(toolKey);
    if (existing && existing.canCancel) {
      const next = new Map(operations.value);
      next.set(toolKey, {
        ...existing,
        stage: 'cancelled',
        message: 'Cancelled by user',
        canRetry: true,
        canCancel: false,
      });
      operations.value = next;
    }
  };

  const retryOperation = (toolKey: string) => {
    const existing = operations.value.get(toolKey);
    if (existing && existing.canRetry) {
      const next = new Map(operations.value);
      next.set(toolKey, {
        ...existing,
        stage: 'preparing',
        progress: 0,
        message: 'Retrying...',
        canRetry: false,
        canCancel: true,
      });
      operations.value = next;
    }
  };

  const getOperation = (toolKey: string): OperationProgress | undefined => {
    return operations.value.get(toolKey);
  };

  const clearCompleted = () => {
    const next = new Map(operations.value);
    for (const [key, op] of next.entries()) {
      if (op.stage === 'completed' || op.stage === 'failed' || op.stage === 'cancelled') {
        next.delete(key);
      }
    }
    operations.value = next;
  };

  return {
    operations,
    activeOperation,
    isAnyActive,
    startOperation,
    updateProgress,
    completeOperation,
    cancelOperation,
    retryOperation,
    getOperation,
    clearCompleted,
  };
});

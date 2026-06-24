import { ref, reactive, computed, type ComputedRef } from 'vue';

export type OperationStage = 'idle' | 'preparing' | 'downloading' | 'installing' | 'verifying' | 'completed' | 'failed' | 'cancelled';

export interface OperationProgress {
  toolKey: string;
  stage: OperationStage;
  progress: number; // 0-100
  message: string;
  error?: string;
  startTime: number;
  canRetry: boolean;
  canCancel: boolean;
}

export interface UseOperationProgressReturn {
  operations: Map<string, OperationProgress>;
  activeOperation: ComputedRef<OperationProgress | null>;
  isAnyActive: ComputedRef<boolean>;
  startOperation: (toolKey: string) => void;
  updateProgress: (toolKey: string, stage: OperationStage, progress: number, message: string) => void;
  completeOperation: (toolKey: string, success: boolean, message?: string) => void;
  cancelOperation: (toolKey: string) => void;
  retryOperation: (toolKey: string) => void;
  getOperation: (toolKey: string) => OperationProgress | undefined;
  clearCompleted: () => void;
}

export function useOperationProgress(): UseOperationProgressReturn {
  const operations = reactive(new Map<string, OperationProgress>());

  const activeOperation = computed(() => {
    for (const op of operations.values()) {
      if (op.stage !== 'completed' && op.stage !== 'failed' && op.stage !== 'cancelled' && op.stage !== 'idle') {
        return op;
      }
    }
    return null;
  });

  const isAnyActive = computed(() => activeOperation.value !== null);

  const startOperation = (toolKey: string) => {
    operations.set(toolKey, {
      toolKey,
      stage: 'preparing',
      progress: 0,
      message: 'Preparing installation...',
      startTime: Date.now(),
      canRetry: false,
      canCancel: true,
    });
  };

  const updateProgress = (
    toolKey: string,
    stage: OperationStage,
    progress: number,
    message: string
  ) => {
    const existing = operations.get(toolKey);
    if (existing) {
      existing.stage = stage;
      existing.progress = Math.min(100, Math.max(0, progress));
      existing.message = message;
      // Allow cancel during downloading and installing
      existing.canCancel = stage === 'downloading' || stage === 'installing';
      existing.canRetry = false;
    }
  };

  const completeOperation = (toolKey: string, success: boolean, message?: string) => {
    const existing = operations.get(toolKey);
    if (existing) {
      if (success) {
        existing.stage = 'completed';
        existing.progress = 100;
        existing.message = message || 'Installation completed successfully';
        existing.canRetry = false;
        existing.canCancel = false;
      } else {
        existing.stage = 'failed';
        existing.message = message || 'Operation failed';
        existing.canRetry = true;
        existing.canCancel = false;
      }
    }
  };

  const cancelOperation = (toolKey: string) => {
    const existing = operations.get(toolKey);
    if (existing && existing.canCancel) {
      existing.stage = 'cancelled';
      existing.message = 'Operation cancelled by user';
      existing.canRetry = true;
      existing.canCancel = false;
    }
  };

  const retryOperation = (toolKey: string) => {
    const existing = operations.get(toolKey);
    if (existing && existing.canRetry) {
      existing.stage = 'preparing';
      existing.progress = 0;
      existing.message = 'Retrying...';
      existing.canRetry = false;
      existing.canCancel = true;
    }
  };

  const getOperation = (toolKey: string): OperationProgress | undefined => {
    return operations.get(toolKey);
  };

  const clearCompleted = () => {
    for (const [key, op] of operations.entries()) {
      if (op.stage === 'completed' || op.stage === 'failed' || op.stage === 'cancelled') {
        operations.delete(key);
      }
    }
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
}

// Stage display configuration
export const STAGE_CONFIG: Record<OperationStage, { label: string; icon: string }> = {
  idle: { label: 'Idle', icon: 'circle' },
  preparing: { label: 'Preparing', icon: 'loader' },
  downloading: { label: 'Downloading', icon: 'download' },
  installing: { label: 'Installing', icon: 'package' },
  verifying: { label: 'Verifying', icon: 'check-circle' },
  completed: { label: 'Completed', icon: 'check-circle' },
  failed: { label: 'Failed', icon: 'x-circle' },
  cancelled: { label: 'Cancelled', icon: 'x-circle' },
};

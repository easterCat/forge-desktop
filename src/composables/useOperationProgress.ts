import { storeToRefs } from 'pinia';
import { useOperationsStore } from '@/stores/operations';

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

export interface UseOperationProgressReturn {
  operations: typeof storeToRefs<ReturnType<typeof useOperationsStore>>['operations'];
  activeOperation: typeof storeToRefs<ReturnType<typeof useOperationsStore>>['activeOperation'];
  isAnyActive: typeof storeToRefs<ReturnType<typeof useOperationsStore>>['isAnyActive'];
  startOperation: (toolKey: string) => void;
  updateProgress: (toolKey: string, stage: OperationStage, progress: number, message: string) => void;
  completeOperation: (toolKey: string, success: boolean, message?: string) => void;
  cancelOperation: (toolKey: string) => void;
  retryOperation: (toolKey: string) => void;
  getOperation: (toolKey: string) => OperationProgress | undefined;
  clearCompleted: () => void;
}

export function useOperationProgress(): UseOperationProgressReturn {
  const store = useOperationsStore();
  const { operations, activeOperation, isAnyActive } = storeToRefs(store);

  return {
    operations,
    activeOperation,
    isAnyActive,
    startOperation: store.startOperation,
    updateProgress: store.updateProgress,
    completeOperation: store.completeOperation,
    cancelOperation: store.cancelOperation,
    retryOperation: store.retryOperation,
    getOperation: store.getOperation,
    clearCompleted: store.clearCompleted,
  };
}

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

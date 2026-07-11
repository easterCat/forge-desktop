import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useErrorHandler, classifyError, isRecoverable, ERROR_MESSAGES } from '../useErrorHandler';

describe('classifyError', () => {
  it('returns ALLAGENTS_NOT_FOUND for allagents not found', () => {
    expect(classifyError('allagents not found')).toBe('ALLAGENTS_NOT_FOUND');
  });

  it('returns ALLAGENTS_TIMEOUT for timeout errors', () => {
    expect(classifyError('Request timed out')).toBe('ALLAGENTS_TIMEOUT');
    expect(classifyError('command timed out')).toBe('ALLAGENTS_TIMEOUT');
  });

  it('returns ALLAGENTS_NETWORK for network errors', () => {
    expect(classifyError('network error')).toBe('ALLAGENTS_NETWORK');
    expect(classifyError('fetch failed')).toBe('ALLAGENTS_NETWORK');
  });

  it('returns ALLAGENTS_PERMISSION for permission errors', () => {
    expect(classifyError('Permission denied')).toBe('ALLAGENTS_PERMISSION');
    expect(classifyError('EACCES: permission')).toBe('ALLAGENTS_PERMISSION');
  });

  it('returns WORKSPACE_NOT_FOUND for workspace errors', () => {
    expect(classifyError('workspace not found')).toBe('WORKSPACE_NOT_FOUND');
  });

  it('returns PLUGIN_NOT_FOUND for plugin errors', () => {
    expect(classifyError('plugin not found')).toBe('PLUGIN_NOT_FOUND');
  });

  it('returns PLUGIN_ALREADY_INSTALLED', () => {
    expect(classifyError('already installed')).toBe('PLUGIN_ALREADY_INSTALLED');
  });

  it('returns MCP_INVALID_CONFIG for MCP errors', () => {
    expect(classifyError('mcp invalid config')).toBe('MCP_INVALID_CONFIG');
  });

  it('returns SYNC_FAILED for sync errors', () => {
    expect(classifyError('sync fail')).toBe('SYNC_FAILED');
  });

  it('returns UNKNOWN_ERROR as fallback', () => {
    expect(classifyError('something completely unrelated')).toBe('UNKNOWN_ERROR');
  });
});

describe('isRecoverable', () => {
  it('returns true for recoverable codes', () => {
    expect(isRecoverable('ALLAGENTS_TIMEOUT')).toBe(true);
    expect(isRecoverable('ALLAGENTS_NETWORK')).toBe(true);
    expect(isRecoverable('SYNC_FAILED')).toBe(true);
    expect(isRecoverable('UNKNOWN_ERROR')).toBe(true);
  });

  it('returns false for non-recoverable codes', () => {
    expect(isRecoverable('ALLAGENTS_NOT_FOUND')).toBe(false);
    expect(isRecoverable('WORKSPACE_NOT_FOUND')).toBe(false);
    expect(isRecoverable('PLUGIN_NOT_FOUND')).toBe(false);
    expect(isRecoverable('ALLAGENTS_PERMISSION')).toBe(false);
  });
});

describe('ERROR_MESSAGES', () => {
  it('contains all known error codes', () => {
    expect(ERROR_MESSAGES.ALLAGENTS_NOT_FOUND).toBeTruthy();
    expect(ERROR_MESSAGES.ALLAGENTS_TIMEOUT).toBeTruthy();
    expect(ERROR_MESSAGES.ALLAGENTS_NETWORK).toBeTruthy();
    expect(ERROR_MESSAGES.WORKSPACE_NOT_FOUND).toBeTruthy();
    expect(ERROR_MESSAGES.PLUGIN_NOT_FOUND).toBeTruthy();
    expect(ERROR_MESSAGES.SYNC_FAILED).toBeTruthy();
    expect(ERROR_MESSAGES.UNKNOWN_ERROR).toBeTruthy();
  });
});

describe('useErrorHandler', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('initializes with null error', () => {
    const { currentError } = useErrorHandler();
    expect(currentError.value).toBeNull();
  });

  it('handles Error instance', () => {
    const { handleError, currentError } = useErrorHandler();
    const err = new Error('timeout error');
    handleError(err);
    expect(currentError.value).not.toBeNull();
    expect(currentError.value!.message).toBe('timeout error');
  });

  it('handles string error', () => {
    const { handleError, currentError } = useErrorHandler();
    handleError('network error');
    expect(currentError.value!.code).toBe('ALLAGENTS_NETWORK');
  });

  it('handles plain object error', () => {
    const { handleError, currentError } = useErrorHandler();
    handleError({ code: 500 });
    expect(currentError.value!.code).toBe('UNKNOWN_ERROR');
  });

  it('pushes to errorHistory', () => {
    const { handleError, errorHistory } = useErrorHandler();
    handleError(new Error('err1'));
    handleError(new Error('err2'));
    expect(errorHistory.value).toHaveLength(2);
  });

  it('caps errorHistory at 50', () => {
    const { handleError, errorHistory } = useErrorHandler();
    for (let i = 0; i < 55; i++) {
      handleError(new Error(`err${i}`));
    }
    expect(errorHistory.value).toHaveLength(50);
  });

  it('clearError sets currentError to null', () => {
    const { handleError, clearError, currentError } = useErrorHandler();
    handleError(new Error('test'));
    clearError();
    expect(currentError.value).toBeNull();
  });

  it('clearHistory empties errorHistory', () => {
    const { handleError, clearHistory, errorHistory } = useErrorHandler();
    handleError(new Error('test'));
    clearHistory();
    expect(errorHistory.value).toHaveLength(0);
  });

  it('wrapAsync returns data on success', async () => {
    const { wrapAsync } = useErrorHandler();
    const result = await wrapAsync(() => Promise.resolve({ ok: true }));
    expect(result.data).toEqual({ ok: true });
    expect(result.error).toBeNull();
  });

  it('wrapAsync returns error on failure', async () => {
    const { wrapAsync } = useErrorHandler();
    const result = await wrapAsync(() => Promise.reject(new Error('fail')));
    expect(result.data).toBeNull();
    expect(result.error).not.toBeNull();
  });

  it('wrapAsync uses fallback when provided', async () => {
    const { wrapAsync } = useErrorHandler();
    const result = await wrapAsync(() => Promise.reject(new Error('fail')), { default: true });
    expect(result.data).toEqual({ default: true });
  });

  it('getRecentErrors returns last N errors', () => {
    const { handleError, getRecentErrors } = useErrorHandler();
    for (let i = 0; i < 15; i++) {
      handleError(new Error(`err${i}`));
    }
    const recent = getRecentErrors(3);
    expect(recent).toHaveLength(3);
  });

  it('getErrorsByCode filters correctly', () => {
    const { handleError, getErrorsByCode } = useErrorHandler();
    handleError(new Error('timeout error'));
    handleError(new Error('network error'));
    handleError(new Error('timeout again'));
    const timeoutErrors = getErrorsByCode('ALLAGENTS_TIMEOUT');
    expect(timeoutErrors).toHaveLength(2);
  });

  it('withRetry succeeds on first attempt', async () => {
    const { withRetry } = useErrorHandler({ maxRetries: 3 });
    const spy = vi.fn().mockResolvedValue('ok');
    const result = await withRetry(spy);
    expect(result).toBe('ok');
    expect(spy).toHaveBeenCalledTimes(1);
  });

  it('withRetry throws immediately on non-recoverable error', async () => {
    const { withRetry } = useErrorHandler({ maxRetries: 3 });
    const spy = vi.fn().mockRejectedValue(new Error('allagents not found'));
    await expect(withRetry(spy)).rejects.toBeDefined();
    expect(spy).toHaveBeenCalledTimes(1);
  });

  it('supports custom transform function', () => {
    const { handleError, currentError } = useErrorHandler({
      transform: () => ({
        code: 'CUSTOM',
        message: 'custom',
        userMessage: 'Custom error',
        recoverable: false,
        timestamp: Date.now(),
      }),
    });
    handleError(new Error('any'));
    expect(currentError.value!.code).toBe('CUSTOM');
  });
});

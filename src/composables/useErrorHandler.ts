/**
 * useErrorHandler 组合式函数
 *
 * 提供统一的错误处理、重试逻辑和用户友好的错误消息。
 * 所有 Unified Store 共享此错误处理机制。
 */

import { ref } from 'vue';

export interface AppError {
  /** 错误代码 */
  code: string;
  /** 错误消息 */
  message: string;
  /** 用户友好消息 */
  userMessage: string;
  /** 是否可恢复 */
  recoverable: boolean;
  /** 原始错误 */
  originalError?: unknown;
  /** 时间戳 */
  timestamp: number;
}

export interface ErrorHandlerOptions {
  /** 最大重试次数 */
  maxRetries?: number;
  /** 重试延迟 (ms) */
  retryDelay?: number;
  /** 自定义错误转换 */
  transform?: (error: unknown) => AppError;
}

// ============================================================================
// 错误代码映射
// ============================================================================

const ERROR_MESSAGES: Record<string, string> = {
  ALLAGENTS_NOT_FOUND: 'allagents CLI 未安装，请运行 npm install -g allagents',
  ALLAGENTS_TIMEOUT: 'allagents 命令执行超时，请检查网络连接',
  ALLAGENTS_NETWORK: '网络连接失败，请检查网络设置',
  ALLAGENTS_PERMISSION: '权限不足，请以管理员身份运行',
  WORKSPACE_NOT_FOUND: '工作区未初始化，请先执行初始化',
  PLUGIN_NOT_FOUND: '插件未找到，请检查插件名称',
  PLUGIN_ALREADY_INSTALLED: '插件已安装',
  MCP_INVALID_CONFIG: 'MCP 服务器配置无效，请检查参数',
  SYNC_FAILED: '同步失败，请重试',
  UNKNOWN_ERROR: '未知错误，请重试',
};

// ============================================================================
// 工具函数
// ============================================================================

/** 将未知错误转换为 AppError */
function toAppError(error: unknown, code = 'UNKNOWN_ERROR'): AppError {
  if (error instanceof Error) {
    const errorCode = classifyError(error.message);
    return {
      code: errorCode,
      message: error.message,
      userMessage: ERROR_MESSAGES[errorCode] ?? ERROR_MESSAGES.UNKNOWN_ERROR,
      recoverable: isRecoverable(errorCode),
      originalError: error,
      timestamp: Date.now(),
    };
  }

  if (typeof error === 'string') {
    const errorCode = classifyError(error);
    return {
      code: errorCode,
      message: error,
      userMessage: ERROR_MESSAGES[errorCode] ?? ERROR_MESSAGES.UNKNOWN_ERROR,
      recoverable: isRecoverable(errorCode),
      originalError: error,
      timestamp: Date.now(),
    };
  }

  return {
    code,
    message: String(error),
    userMessage: ERROR_MESSAGES[code] ?? ERROR_MESSAGES.UNKNOWN_ERROR,
    recoverable: isRecoverable(code),
    originalError: error,
    timestamp: Date.now(),
  };
}

/** 根据错误消息分类错误代码 */
function classifyError(message: string): string {
  const lower = message.toLowerCase();

  if (lower.includes('not found') && lower.includes('allagents')) {
    return 'ALLAGENTS_NOT_FOUND';
  }
  if (lower.includes('timeout') || lower.includes('timed out')) {
    return 'ALLAGENTS_TIMEOUT';
  }
  if (lower.includes('network') || lower.includes('fetch')) {
    return 'ALLAGENTS_NETWORK';
  }
  if (lower.includes('permission') || lower.includes('eacces')) {
    return 'ALLAGENTS_PERMISSION';
  }
  if (lower.includes('workspace') && lower.includes('not')) {
    return 'WORKSPACE_NOT_FOUND';
  }
  if (lower.includes('plugin') && lower.includes('not')) {
    return 'PLUGIN_NOT_FOUND';
  }
  if (lower.includes('already installed')) {
    return 'PLUGIN_ALREADY_INSTALLED';
  }
  if (lower.includes('mcp') && lower.includes('invalid')) {
    return 'MCP_INVALID_CONFIG';
  }
  if (lower.includes('sync') && lower.includes('fail')) {
    return 'SYNC_FAILED';
  }

  return 'UNKNOWN_ERROR';
}

/** 判断错误是否可恢复 */
function isRecoverable(code: string): boolean {
  const recoverableCodes = [
    'ALLAGENTS_TIMEOUT',
    'ALLAGENTS_NETWORK',
    'SYNC_FAILED',
    'UNKNOWN_ERROR',
  ];
  return recoverableCodes.includes(code);
}

// ============================================================================
// 组合式函数
// ============================================================================

export function useErrorHandler(options: ErrorHandlerOptions = {}) {
  const { maxRetries = 3, retryDelay = 1000 } = options;

  /** 当前错误 */
  const currentError = ref<AppError | null>(null);

  /** 错误历史 */
  const errorHistory = ref<AppError[]>([]);

  /** 是否正在重试 */
  const isRetrying = ref(false);

  /** 处理错误 */
  function handleError(error: unknown): AppError {
    const appError = options.transform
      ? options.transform(error)
      : toAppError(error);

    currentError.value = appError;
    errorHistory.value.push(appError);

    // 限制历史记录长度
    if (errorHistory.value.length > 50) {
      errorHistory.value = errorHistory.value.slice(-50);
    }

    return appError;
  }

  /** 清除当前错误 */
  function clearError() {
    currentError.value = null;
  }

  /** 清除错误历史 */
  function clearHistory() {
    errorHistory.value = [];
  }

  /** 重试操作 */
  async function withRetry<T>(
    operation: () => Promise<T>,
    retries = maxRetries
  ): Promise<T> {
    let lastError: unknown;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;
        const appError = toAppError(error);

        // 不可恢复的错误不重试
        if (!appError.recoverable || attempt === retries) {
          throw handleError(error);
        }

        // 等待后重试
        isRetrying.value = true;
        await new Promise(resolve => setTimeout(resolve, retryDelay * (attempt + 1)));
        isRetrying.value = false;
      }
    }

    throw handleError(lastError);
  }

  /** 包装异步操作，自动处理错误 */
  async function wrapAsync<T>(
    operation: () => Promise<T>,
    fallback?: T
  ): Promise<{ data: T | null; error: AppError | null }> {
    try {
      const data = await operation();
      return { data, error: null };
    } catch (error) {
      const appError = handleError(error);
      return { data: fallback ?? null, error: appError };
    }
  }

  /** 获取最近的错误 */
  function getRecentErrors(count = 10): AppError[] {
    return errorHistory.value.slice(-count);
  }

  /** 获取特定代码的错误 */
  function getErrorsByCode(code: string): AppError[] {
    return errorHistory.value.filter(e => e.code === code);
  }

  return {
    // 状态
    currentError,
    errorHistory,
    isRetrying,

    // 方法
    handleError,
    clearError,
    clearHistory,
    withRetry,
    wrapAsync,
    getRecentErrors,
    getErrorsByCode,
  };
}

// ============================================================================
// 导出工具函数
// ============================================================================

export { toAppError, classifyError, isRecoverable, ERROR_MESSAGES };

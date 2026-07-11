/**
 * Tauri IPC 命令返回结果统一类型
 *
 * 所有 Rust 命令通过 `#[command]` 暴露给前端，统一返回：
 * ```rust
 * #[derive(Serialize)]
 * pub struct CommandResult<T> {
 *     pub success: bool,
 *     pub data: Option<T>,
 *     pub error: Option<String>,
 * }
 * ```
 *
 * 前端调用时使用泛型 `invoke<CommandResult<T>>('xxx', args)`，避免 `any` 类型污染。
 */

/** 通用命令返回结构（无 data 负载） */
export interface CommandResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * 提取命令错误消息，缺失时回退到默认提示。
 * 集中处理以避免各 Store 重复相同的三元链。
 */
export function getCommandError<T>(
  result: CommandResult<T>,
  fallback = 'Operation failed',
): string {
  return result.error ?? fallback;
}

/**
 * 同步结果载荷（与 Rust 端 SyncReport 对应）
 */
export interface SyncReportPayload {
  synced_count: number;
  error_count: number;
  errors?: Array<{ plugin_id?: string; message: string }>;
}

/** 工作区状态载荷 */
export interface WorkspaceStatusPayload {
  clients?: string[];
  plugins?: number;
  mcp_servers?: number;
}
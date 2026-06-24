// MCP Bridge Types - mirrors the Rust McpProbeResult struct

export interface McpProbeResult {
  reachable: boolean;
  serverInfo?: Record<string, unknown>;
  error?: string;
  durationMs: number;
}

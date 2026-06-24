// Plugin Capabilities Types - mirrors the Rust PluginCapabilities struct
// All fields use camelCase to match the serde(rename_all = "camelCase") on the Rust side.

export interface PluginCapabilities {
  name: string;
  version?: string;
  author?: string;
  license?: string;
  description: string;
  repository?: string;
  /** "local" when the plugin is installed; "remote" when fetched from the network. */
  source: string;
  /** The source identifier from MarketplacePlugin.sourceId (e.g. "anthropics"). */
  sourceId: string;
  /** Absolute path to the plugin directory (empty string for remote-only). */
  installedPath: string;
  capabilities: PluginCapabilityCounts;
  skills: SkillInfo[];
  commands: CommandInfo[];
  hooks: HookInfo[];
  mcpServers: McpServerInfo[];
  lspServers: LspServerInfo[];
  dependencies: string[];
  /** Files listed in the plugin's manifest.files (if present). */
  manifestFiles: string[];
}

export interface PluginCapabilityCounts {
  skills: number;
  hooks: number;
  commands: number;
  mcpServers: number;
  lspServers: number;
}

export interface SkillInfo {
  /** CamelCase skill identifier derived from the directory name. */
  name: string;
  /** Human-readable description from YAML frontmatter. */
  description: string;
  /** Relative path from plugin root, e.g. "skills/using-superpowers/SKILL.md". */
  path: string;
  /** True when scripts/ subdirectory exists. */
  hasScripts: boolean;
  /** True when references/ subdirectory exists. */
  hasReferences: boolean;
}

export interface CommandInfo {
  /** CamelCase command identifier derived from the file name. */
  name: string;
  /** Human-readable description from YAML frontmatter. */
  description: string;
  /** Tools the command is allowed to invoke. */
  allowedTools: string[];
  /** Relative path from plugin root, e.g. "commands/my-command.md". */
  path: string;
}

export interface HookInfo {
  /** Lifecycle event name, e.g. "SessionStart". */
  event: string;
  /** Regex matcher, e.g. "startup|clear|compact". */
  matcher?: string;
  /** Command to execute, e.g. "bash hooks/session-start". */
  command: string;
  /** True when the command script file exists under hooks/. */
  scriptExists: boolean;
}

export interface McpServerInfo {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export interface LspServerInfo {
  name: string;
  command: string;
  args: string[];
}

export interface HookExecutionResult {
  event: string;
  matcher?: string;
  command: string;
  exitCode?: number;
  stdout: string;
  stderr: string;
  parsedJson?: Record<string, unknown>;
  startedAt: string;
  durationMs: number;
  logPath: string;
}

export interface ValidationIssue {
  severity: 'error' | 'warning';
  code: string;
  message: string;
  path?: string;
}

export interface ValidationCapabilityCounts {
  skills: number;
  hooks: number;
  commands: number;
  mcpServers: number;
  lspServers: number;
}

export interface ValidationReport {
  valid: boolean;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
  capabilities: ValidationCapabilityCounts;
}

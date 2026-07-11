// Plugin Marketplace Types - Multi-source plugin marketplace

export type PluginInstallStatus = 'pending' | 'downloading' | 'installing' | 'success' | 'failed' | 'updating';

/**
 * Repository type when adding a new source.
 *
 * - `"market"` — the repo is a marketplace index; plugin data comes from
 *   `.claude-plugin/marketplace.json` inside that repo.
 * - `"res"`    — the repo itself is a single plugin (resource); plugin
 *   info is read from `.claude-plugin/marketplace.json` first, falling
 *   back to `.claude-plugin/plugin.json`. The plugin count is always 1.
 */
export type RepoType = 'market' | 'res';

/**
 * Mirrors the Rust `PluginInstallSource` model. Carries enough information
 * for `install_plugin` to decide whether the plugin is local (already on
 * disk) or remote (needs a `git clone`).
 *
 * `type`:
 *  - `"local"`      — relative path inside the marketplace source repo
 *                     (e.g. `./plugins/agent-sdk-dev`); install_plugin just
 *                     copies it.
 *  - `"git-subdir"` — subdirectory of an external Git repo; install_plugin
 *                     does a sparse `git clone` of `url` and copies `path`.
 *  - `"url"`        — whole external repo; install_plugin does a
 *                     `git clone` of `url`.
 */
export interface PluginInstallSource {
  type: 'local' | 'git-subdir' | 'url' | string;
  url?: string;
  path?: string;
  ref?: string;
  sha?: string;
}

export interface PluginSource {
  id: string;
  name: string;
  nameZh?: string;
  command: string; // The exact marketplace add command (GitHub URL)
  description: string;
  icon?: string;
  pluginCount?: number;
  repoName?: string; // GitHub repository name (extracted from URL)
  /** Repository type: marketplace index (`market`) or single-plugin repo (`res`). */
  repoType?: RepoType;
}

export interface MarketplacePlugin {
  id: string;
  sourceId: string;
  name: string;
  description: string;
  longDescription?: string;
  author?: string;
  version?: string;
  latestVersion?: string;
  hasUpdate?: boolean;
  categories: string[];
  tags: string[];
  installCommand?: string;
  /// Absolute on-disk path of the installed plugin directory.
  /// Populated by the Rust `get_installed_plugins` command and the
  /// marketplace→installed mapper; the frontend uses it on the
  /// PluginsView Installed tab to render the "Installed at: …" row
  /// with a Copy button (mirrors `MarketplaceSkill.installPath`).
  installPath?: string;
  repository?: string;
  homepage?: string;
  license?: string;
  stars?: number;
  downloads?: number;
  lastUpdated?: string;
  isInstalled?: boolean;
  /// User-controlled enable/disable flag. Persisted in
  /// `$FORGE_HOME/plugins/marketplace.json` so the state survives app
  /// restarts and CLI syncs. Mirrors the `MarketplaceSkill.enabled`
  /// UI concept on the Skills side.
  disabled?: boolean;
  /// Original `source` block from the upstream marketplace manifest.
  /// `install_plugin` inspects this to decide between a local copy and
  /// a remote `git clone`. Optional for backward compatibility with
  /// plugins that came from the legacy fallback filesystem scan.
  installSource?: PluginInstallSource;
  metadata?: Record<string, unknown>;
  /// Key of the CLI tool this plugin is associated with (e.g. "claude-code").
  /// Used to display the CLI tool icon on the Installed tab and to determine
  /// the sync target directory. Populated from plugin metadata or manifest.
  cliToolKey?: string;
  /// All CLI tool keys this plugin supports (detected from marker directories).
  /// When non-empty, the Installed tab shows a sync button for each tool.
  cliToolKeys?: string[];
}

/**
 * Metadata for a CLI tool supported by the "Sync to" dialog.
 * Populated from the Rust `CliToolManager::get_supported_tools()` command.
 */
export interface CliToolMeta {
  /** Tool key, e.g. "claude-code", "codex", "gemini-cli" */
  key: string;
  /** Display name, e.g. "Claude Code" */
  name: string;
  /** Tooltip icon abbreviation (2-char), e.g. "CC" */
  icon: string;
  /** Brand accent color, e.g. "#B8944A" */
  color: string;
  /** Sync target plugin directory (null = not supported) */
  pluginDir: string | null;
}

export interface PluginInstallProgress {
  pluginId: string;
  pluginName: string;
  stage: PluginInstallStatus;
  progress: number;
  message: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export interface MarketplaceError {
  code: string;
  message: string;
  source?: string;
  details?: Record<string, unknown>;
}

// Result types for Rust commands
export interface PluginInstallResult {
  success: boolean;
  path?: string;
  error?: string;
}

export interface PluginUpdateResult {
  success: boolean;
  newVersion?: string;
  error?: string;
}

// Source status types (FEAT-016)
export interface SourceStatus {
  sourceId: string;
  name: string;
  nameZh?: string;
  repoUrl: string;
  isInstalled: boolean;
  installedPath?: string;
  installedPaths: string[];
}

export interface SourceInstallResult {
  success: boolean;
  sourceId: string;
  installedPath?: string;
  installedPaths: string[];
  error?: string;
}

export interface SourceInstallProgress {
  sourceId: string;
  stage: 'preparing' | 'cloning' | 'extracting' | 'success' | 'failed';
  progress: number;
  message: string;
}

// Real GitHub-backed marketplace sources.
// The id values (`anthropics` / `ccplugins` / `ananddtyagi` / `addyosmani`) match the source
// keys used in `$FORGE_HOME/plugins/marketplace.json` and are produced by
// `scripts/plugins/install.mjs` when syncing each repo.
export const PRESET_MARKETPLACE_SOURCES: PluginSource[] = [
  {
    id: 'anthropics',
    name: 'Anthropic Official',
    nameZh: 'Anthropic 官方',
    command: 'https://github.com/anthropics/claude-plugins-official',
    description: 'Anthropic 一手官方插件仓库（35+ 插件）',
    pluginCount: 0,
    repoName: 'claude-plugins-official',
    repoType: 'market',
  },
  {
    id: 'ananddtyagi',
    name: 'cc-marketplace',
    nameZh: '市场索引',
    command: 'https://github.com/ananddtyagi/cc-marketplace',
    description: 'Claude Code marketplace 索引仓库',
    pluginCount: 0,
    repoName: 'cc-marketplace',
    repoType: 'market',
  },
  {
    id: 'addyosmani',
    name: 'agent-skills',
    nameZh: 'Agent 技能库',
    command: 'https://github.com/addyosmani/agent-skills',
    description: 'Addy Osmani 的 Agent 技能集合，提供多种实用技能',
    pluginCount: 0,
    repoName: 'agent-skills',
    repoType: 'res',
  },
];

// Plugin sync to CLI tool types

export interface PluginSyncStatus {
  pluginId: string;      // synced: `${sourceId}::${pluginName}::${cliToolKey}`; unsynced: `${sourceId}::${pluginName}`
  cliToolKey: string;
  synced: boolean;
  syncedAt?: string;
  targetPath?: string;
}

export interface PluginSyncProgress {
  pluginId: string;
  cliToolKey: string;
  stage: 'syncing' | 'success' | 'failed' | 'unsyncing';
  progress: number;
  message?: string;
  error?: string;
}

export interface PluginSyncResult {
  success: boolean;
  targetPath?: string;
  error?: string;
}

// ---------------------------------------------------------------------------
// Marketplace manifest (mirrors Rust `MarketplaceManifest` in
// `src-tauri/src/services/plugin_marketplace.rs`).
//
// The manifest is the source of truth for what plugins the user has
// installed from which marketplace source, and is persisted in
// `$FORGE_HOME/plugins/marketplace.json` (legacy) or in the
// `kv_store.marketplace_manifest` key (current).
// ---------------------------------------------------------------------------

/** Per-source entry inside `MarketplaceManifest.sources`. */
export interface ManifestSource {
  repoUrl: string;
  /** Source kind: `"market"` (index repo) or `"res"` (single-plugin repo). */
  type: string;
  /** True if the source was added by the user, false for presets. */
  external: boolean;
  lastSyncAt: string | null;
  pluginCount: number;
}

/** Per-plugin entry inside `MarketplaceManifest.plugins[sourceId]`. */
export interface ManifestPlugin {
  name: string;
  description: string;
  version: string;
  /** Author is opaque in Rust (serde_json::Value) — string or object. */
  author: string | Record<string, unknown>;
  repoUrl: string;
  /** Absolute on-disk path of the installed plugin directory. */
  installedPath: string;
  external: boolean;
  dependencies: string[];
}

/** A record that a plugin was removed (for UI badges / "Recently removed"). */
export interface RemovedEntry {
  name: string;
  source: string;
  removedAt: string;
  reason: string;
}

/** The full marketplace manifest, returned by `get_marketplace_manifest`. */
export interface MarketplaceManifest {
  version: string;
  lastSyncAt: string | null;
  sources: Record<string, ManifestSource>;
  /** Map of source id → list of plugins from that source. */
  plugins: Record<string, ManifestPlugin[]>;
  removed: RemovedEntry[];
}

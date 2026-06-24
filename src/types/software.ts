export type SoftwareStatus = 'installed' | 'notinstalled' | 'outdated' | 'unknown';

export interface Software {
  id: string;
  name: string;
  key: string;
  version: string | null;
  installPath: string | null;
  configPath: string;
  isInstalled: boolean;
  lastChecked: string | null;
  latestVersion?: string | null;
  isUpgradable?: boolean;
  status?: SoftwareStatus;
  websiteUrl?: string | null;
  platform?: string;
}

export type InstallMethod = 'npm' | 'curl-bash' | 'npm-curl-fallback';

export interface CliTool {
  id: string;
  key: string;
  name: string;
  icon: string;
  description: string;
  installMethods: {
    method: InstallMethod;
    command: string;
    priority: number;
  }[];
  latestVersion: string | null;
  latestVersionChecked: string | null;
  npmPackage?: string;
  websiteUrl?: string;
  pluginDir?: string;  // Plugin directory path for this CLI tool (e.g. ~/.claude/plugins/)
}

export interface CliToolStatus {
  toolKey: string;
  isInstalled: boolean;
  installedVersion: string | null;
  installMethod: InstallMethod | null;
  installPath: string | null;
  hasConflict: boolean;
  conflictInfo: string | null;
  latestVersion: string | null;
  needsUpgrade: boolean;
}

export interface UpgradeResult {
  success: boolean;
  message: string;
  newVersion: string | null;
  method: InstallMethod;
}

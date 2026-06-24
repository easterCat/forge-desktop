// Installed Registry Types - mirrors Rust InstalledRegistry / InstalledEntry

export interface InstalledEntry {
  sourceId: string;
  pluginName: string;
  version?: string;
  installedAt?: string;
  lastInuseAt?: string;
  installPath: string;
}

export interface InstalledRegistry {
  version: string;
  plugins: Record<string, InstalledEntry>;
  lastSweepAt?: string;
}

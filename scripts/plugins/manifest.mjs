#!/usr/bin/env node
/**
 * marketplace.json manifest management
 * Handles reading, writing, and merging plugin data
 */

import { getPath, readJson, writeJson, FileLock } from './utils.mjs';

const MARKETPLACE_PATH = getPath('plugins', 'marketplace.json');

/**
 * Load marketplace manifest
 */
export function loadManifest() {
  const data = readJson(MARKETPLACE_PATH);
  if (!data) {
    // Return default structure if file doesn't exist
    return {
      version: '1.0.0',
      lastSyncAt: null,
      sources: {},
      plugins: {},
      removed: []
    };
  }
  return data;
}

/**
 * Save marketplace manifest with file lock
 */
export function saveManifest(data) {
  const lock = new FileLock(MARKETPLACE_PATH);
  try {
    lock.acquire(5000);
    writeJson(MARKETPLACE_PATH, data);
  } finally {
    lock.release();
  }
}

/**
 * Get source configuration by URL or name
 */
export function getSource(sourceName) {
  const manifest = loadManifest();
  return manifest.sources[sourceName] || null;
}

/**
 * Get all plugins for a source
 */
export function getPluginsForSource(sourceName) {
  const manifest = loadManifest();
  return manifest.plugins[sourceName] || [];
}

/**
 * Get all installed plugins
 */
export function getAllPlugins() {
  const manifest = loadManifest();
  const allPlugins = [];
  
  for (const [source, plugins] of Object.entries(manifest.plugins)) {
    if (Array.isArray(plugins)) {
      for (const plugin of plugins) {
        allPlugins.push({
          ...plugin,
          source
        });
      }
    }
  }
  
  return allPlugins;
}

/**
 * Get all removed plugins
 */
export function getRemovedPlugins() {
  const manifest = loadManifest();
  return manifest.removed || [];
}

/**
 * Find plugin by name in all sources
 */
export function findPlugin(pluginName, sourceName = null) {
  const manifest = loadManifest();
  
  if (sourceName) {
    const plugins = manifest.plugins[sourceName] || [];
    return plugins.find(p => p.name === pluginName) || null;
  }
  
  // Search all sources
  for (const [source, plugins] of Object.entries(manifest.plugins)) {
    if (Array.isArray(plugins)) {
      const found = plugins.find(p => p.name === pluginName);
      if (found) {
        return { ...found, source };
      }
    }
  }
  
  return null;
}

/**
 * Add or update plugin in manifest
 */
export function upsertPlugin(sourceName, pluginData) {
  const manifest = loadManifest();
  
  // Ensure sources structure exists
  if (!manifest.sources[sourceName]) {
    manifest.sources[sourceName] = {
      repoUrl: pluginData.repoUrl || '',
      type: 'unknown',
      external: false,
      lastSyncAt: null,
      pluginCount: 0
    };
  }
  
  // Ensure plugins array exists for source
  if (!Array.isArray(manifest.plugins[sourceName])) {
    manifest.plugins[sourceName] = [];
  }
  
  // Find existing plugin by name
  const existingIndex = manifest.plugins[sourceName].findIndex(
    p => p.name === pluginData.name
  );
  
  if (existingIndex >= 0) {
    // Update existing plugin (preserve some fields)
    manifest.plugins[sourceName][existingIndex] = {
      ...manifest.plugins[sourceName][existingIndex],
      ...pluginData,
      // Preserve installedPath if already set
      installedPath: manifest.plugins[sourceName][existingIndex].installedPath || pluginData.installedPath
    };
  } else {
    // Add new plugin
    manifest.plugins[sourceName].push(pluginData);
  }
  
  // Update plugin count
  manifest.sources[sourceName].pluginCount = manifest.plugins[sourceName].length;
  
  saveManifest(manifest);
  return true;
}

/**
 * Add multiple plugins for a source
 */
export function addPlugins(sourceName, plugins, sourceInfo = {}) {
  const manifest = loadManifest();
  
  // Update source info
  if (!manifest.sources[sourceName]) {
    manifest.sources[sourceName] = {
      repoUrl: sourceInfo.repoUrl || '',
      type: sourceInfo.type || 'unknown',
      external: sourceInfo.external || false,
      lastSyncAt: null,
      pluginCount: 0
    };
  }
  
  manifest.sources[sourceName] = {
    ...manifest.sources[sourceName],
    ...sourceInfo,
    lastSyncAt: new Date().toISOString()
  };
  
  // Ensure plugins array exists
  if (!Array.isArray(manifest.plugins[sourceName])) {
    manifest.plugins[sourceName] = [];
  }
  
  // Add plugins (skip duplicates by name)
  const existingNames = new Set(manifest.plugins[sourceName].map(p => p.name));
  for (const plugin of plugins) {
    if (!existingNames.has(plugin.name)) {
      manifest.plugins[sourceName].push({
        ...plugin,
        installedPath: `plugins/${sourceName}/${plugin.name}`
      });
      existingNames.add(plugin.name);
    }
  }
  
  // Update counts
  manifest.sources[sourceName].pluginCount = manifest.plugins[sourceName].length;
  manifest.lastSyncAt = new Date().toISOString();
  
  saveManifest(manifest);
  return true;
}

/**
 * Remove plugin from manifest (add to removed list)
 */
export function removePlugin(pluginName, sourceName, options = {}) {
  const manifest = loadManifest();
  
  if (!manifest.plugins[sourceName]) {
    return false;
  }
  
  const pluginIndex = manifest.plugins[sourceName].findIndex(
    p => p.name === pluginName
  );
  
  if (pluginIndex < 0) {
    return false;
  }
  
  // Remove from plugins array
  const removedPlugin = manifest.plugins[sourceName].splice(pluginIndex, 1)[0];
  
  // Add to removed list
  manifest.removed.push({
    name: removedPlugin.name,
    source: sourceName,
    removedAt: new Date().toISOString(),
    reason: options.reason || 'manual uninstall',
    // Keep metadata for reference
    description: removedPlugin.description,
    author: removedPlugin.author
  });
  
  // Update counts
  manifest.sources[sourceName].pluginCount = manifest.plugins[sourceName].length;
  
  // Purge from removed list if requested
  if (options.purge) {
    manifest.removed = manifest.removed.filter(r => r.name !== pluginName);
  }
  
  saveManifest(manifest);
  return true;
}

/**
 * Get marketplace path
 */
export function getMarketplacePath() {
  return MARKETPLACE_PATH;
}

export default {
  loadManifest,
  saveManifest,
  getSource,
  getPluginsForSource,
  getAllPlugins,
  getRemovedPlugins,
  findPlugin,
  upsertPlugin,
  addPlugins,
  removePlugin,
  getMarketplacePath
};

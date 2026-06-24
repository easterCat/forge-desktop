#!/usr/bin/env node
/**
 * Plugin scanner
 * Identifies Claude Code plugin directories by looking for:
 * 1. .claude-plugin/plugin.json
 * 2. README.md (fallback)
 */

import { readdirSync, readFileSync, existsSync, statSync } from 'fs';
import { join, basename } from 'path';
import { info, warn } from './utils.mjs';

/**
 * Check if a directory contains a valid plugin manifest
 */
function hasPluginManifest(dirPath) {
  const manifestPath = join(dirPath, '.claude-plugin', 'plugin.json');
  return existsSync(manifestPath);
}

/**
 * Check if a directory has a README (fallback indicator)
 */
function hasReadme(dirPath) {
  const readmePath = join(dirPath, 'README.md');
  return existsSync(readmePath);
}

/**
 * Extract description from README.md first paragraph
 */
function extractDescriptionFromReadme(dirPath) {
  try {
    const readmePath = join(dirPath, 'README.md');
    const content = readFileSync(readmePath, 'utf-8');
    
    // Get first paragraph (text before first empty line)
    const lines = content.split('\n');
    const firstParagraph = [];
    
    for (const line of lines) {
      if (line.trim() === '') break;
      firstParagraph.push(line);
    }
    
    const description = firstParagraph.join(' ').trim();
    // Remove markdown links and clean up
    return description.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').substring(0, 500);
  } catch {
    return null;
  }
}

/**
 * Extract plugin metadata from .claude-plugin/plugin.json
 */
function extractManifestMetadata(dirPath) {
  try {
    const manifestPath = join(dirPath, '.claude-plugin', 'plugin.json');
    const content = readFileSync(manifestPath, 'utf-8');
    const manifest = JSON.parse(content);
    
    return {
      name: manifest.name || basename(dirPath),
      description: manifest.description || null,
      version: manifest.version || '0.0.0',
      author: manifest.author || null,
      external: manifest.external || false,
      dependencies: manifest.dependencies || [],
      installMode: manifest.installMode || 'clone',
      manifest: {
        files: ['.claude-plugin/plugin.json']
      }
    };
  } catch (err) {
    warn(`Failed to parse plugin.json in ${dirPath}: ${err.message}`);
    return null;
  }
}

/**
 * Build plugin metadata from directory
 */
function buildPluginMetadata(dirPath, sourceName, repoUrl) {
  const pluginName = basename(dirPath);
  
  // Try to get metadata from manifest first
  const manifestMeta = extractManifestMetadata(dirPath);
  
  // Fallback to README
  let description = manifestMeta?.description || null;
  if (!description && hasReadme(dirPath)) {
    description = extractDescriptionFromReadme(dirPath);
  }
  
  // If still no description, use directory name
  if (!description) {
    description = `Claude Code plugin: ${pluginName}`;
  }
  
  return {
    name: pluginName,
    description: description,
    version: manifestMeta?.version || '0.0.0',
    author: manifestMeta?.author || sourceName,
    repoUrl: `${repoUrl}/tree/main/plugins/${pluginName}`,
    installedPath: `plugins/${sourceName}/${pluginName}`,
    external: manifestMeta?.external || false,
    dependencies: manifestMeta?.dependencies || [],
    installMode: manifestMeta?.installMode || 'clone',
    manifest: manifestMeta?.manifest || {
      files: hasReadme(dirPath) ? ['README.md'] : []
    }
  };
}

/**
 * Scan a directory for plugin subdirectories
 * @param {string} repoDir - Path to cloned repository
 * @param {string} sourceName - Source name (e.g., 'anthropics')
 * @param {string} repoUrl - Repository URL
 * @returns {Array} - Array of plugin metadata
 */
export function scanForPlugins(repoDir, sourceName, repoUrl) {
  const plugins = [];
  // Scan well-known plugin-bearing subdirectories only.
  // Scanning repo root causes false positives (e.g. anthropics repo has
  // external_plugins/, LICENSE/, README.md at root).
  const scanLocations = [
    join(repoDir, 'plugins'),
    join(repoDir, 'marketplace'),
  ];

  for (const scanPath of scanLocations) {
    if (!existsSync(scanPath)) {
      continue;
    }

    try {
      const entries = readdirSync(scanPath);
      
      for (const entry of entries) {
        const fullPath = join(scanPath, entry);
        
        // Skip files and hidden directories
        if (!statSync(fullPath).isDirectory() || entry.startsWith('.')) {
          continue;
        }
        
        // Check if this is a plugin directory
        const isPlugin = hasPluginManifest(fullPath) || hasReadme(fullPath);
        
        if (isPlugin) {
          const metadata = buildPluginMetadata(fullPath, sourceName, repoUrl);
          plugins.push(metadata);
          info(`  Found plugin: ${metadata.name}`);
        }
      }
    } catch (err) {
      warn(`Failed to scan ${scanPath}: ${err.message}`);
    }
  }

  return plugins;
}

/**
 * Get list of files in a plugin directory
 */
export function getPluginFiles(dirPath) {
  const files = [];
  
  function scanDir(dir, prefix = '') {
    try {
      const entries = readdirSync(dir, { withFileTypes: true });
      
      for (const entry of entries) {
        if (entry.name.startsWith('.')) continue; // Skip hidden files
        
        const fullPath = join(dir, entry.name);
        const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
        
        if (entry.isDirectory()) {
          scanDir(fullPath, relativePath);
        } else {
          files.push(relativePath);
        }
      }
    } catch (err) {
      warn(`Failed to scan directory ${dir}: ${err.message}`);
    }
  }
  
  scanDir(dirPath);
  return files;
}

export default {
  scanForPlugins,
  getPluginFiles,
  hasPluginManifest,
  hasReadme
};

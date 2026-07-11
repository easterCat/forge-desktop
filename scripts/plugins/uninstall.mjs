#!/usr/bin/env node
/**
 * Uninstall command
 * Removes a plugin from local marketplace (soft delete by default)
 */

import { getPath, info, success, error, warn } from './utils.mjs';
import { findPlugin, removePlugin, getAllPlugins } from './manifest.mjs';
import { rmSync, readdirSync, existsSync } from 'fs';

/**
 * Run uninstall command
 */
export async function run(args) {
  const options = parseArgs(args);
  
  // Validate plugin name
  if (!options.pluginName) {
    error('Plugin name is required');
    error('Usage: plugins uninstall <plugin_name> [--source <name>] [--force] [--purge]');
    error('');
    error('To list installed plugins, run: plugins list');
    process.exit(3);
  }
  
  info(`Uninstalling plugin: ${options.pluginName}`);
  
  // Find plugin
  let plugin = findPlugin(options.pluginName, options.source);
  
  if (!plugin) {
    error(`Plugin not found: ${options.pluginName}`);
    
    // Show suggestions
    const allPlugins = getAllPlugins();
    if (allPlugins.length > 0) {
      warn('');
      warn('Installed plugins:');
      for (const p of allPlugins) {
        console.log(`  - ${p.name} (source: ${p.source})`);
      }
    }
    
    error('');
    error('Usage: plugins uninstall <plugin_name> [--source <name>]');
    process.exit(3);
  }
  
  const sourceName = plugin.source;
  const pluginPath = getPath('plugins', sourceName, options.pluginName);
  
  info(`Found plugin in source: ${sourceName}`);
  info(`Local path: ${pluginPath}`);
  
  // Remove from marketplace.json
  const removed = removePlugin(options.pluginName, sourceName, {
    reason: options.force ? 'force uninstall' : 'manual uninstall',
    purge: options.purge
  });
  
  if (!removed) {
    error('Failed to remove plugin from marketplace');
    process.exit(1);
  }
  
  success(`Removed ${options.pluginName} from marketplace`);
  
  // Handle file deletion based on --force flag
  if (options.force) {
    if (existsSync(pluginPath)) {
      try {
        rmSync(pluginPath, { recursive: true, force: true });
        success(`Deleted local files: ${pluginPath}`);
      } catch (err) {
        warn(`Failed to delete local files: ${err.message}`);
      }
    }
  } else {
    info('Local files preserved (use --force to delete)');
    info(`Plugin files still at: ${pluginPath}`);
  }
  
  // Clean up empty directories
  const sourceDir = getPath('plugins', sourceName);
  if (existsSync(sourceDir)) {
    const entries = readdirSync(sourceDir);
    const visibleEntries = entries.filter(e => !e.startsWith('.'));
    
    if (visibleEntries.length === 0) {
      try {
        rmSync(sourceDir, { recursive: true, force: true });
        success(`Cleaned up empty directory: ${sourceDir}`);
      } catch (err) {
        warn(`Failed to clean up directory: ${err.message}`);
      }
    }
  }
  
  // Summary
  console.log('\n' + '='.repeat(50));
  success(`Successfully uninstalled ${options.pluginName}`);
  console.log('='.repeat(50));
  console.log('\nMarketplace updated: $FORGE_HOME/plugins/marketplace.json');
  
  if (options.purge) {
    info('Plugin completely purged from marketplace');
  } else {
    info('Plugin moved to removed list (can be restored)');
  }
}

/**
 * Parse command line arguments
 */
function parseArgs(args) {
  const options = {
    pluginName: null,
    source: null,
    force: false,
    purge: false
  };
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    switch (arg) {
      case '--source':
      case '-s':
        options.source = args[++i];
        break;
      case '--force':
      case '-f':
        options.force = true;
        break;
      case '--purge':
        options.purge = true;
        break;
      case '--help':
      /* falls through */
      case '-h':
        printHelp();
        process.exit(0);
        break;
      default:
        if (!arg.startsWith('-')) {
          options.pluginName = arg;
        }
    }
  }
  
  return options;
}

/**
 * Print help message
 */
function printHelp() {
  console.log(`
Claude Code Plugins Uninstaller
Usage: plugins uninstall <plugin_name> [options]

Options:
  --source <name>    Specify source to avoid ambiguity with same-named plugins
  --force            Also delete local plugin files (default: soft delete only)
  --purge            Completely remove from marketplace (including removed list)

Examples:
  plugins uninstall my-plugin
  plugins uninstall my-plugin --source anthropics
  plugins uninstall my-plugin --force
  plugins uninstall my-plugin --purge
`);
}

export default { run };

// Allow direct execution: `node uninstall.mjs <name> [options]`
import { fileURLToPath } from 'url';
const isDirect = process.argv[1] === fileURLToPath(import.meta.url);
if (isDirect) {
  run(process.argv.slice(2));
}

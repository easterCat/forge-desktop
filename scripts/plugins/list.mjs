#!/usr/bin/env node
/**
 * List command
 * Lists installed plugins from marketplace
 */

import { getAllPlugins, getRemovedPlugins, loadManifest } from './manifest.mjs';
import { log, info, success, colors } from './utils.mjs';

/**
 * Run list command
 */
export function run(args) {
  const options = parseArgs(args);
  
  if (options.showRemoved) {
    listRemoved();
  } else if (options.showSources) {
    listSources();
  } else if (options.json) {
    listPluginsJson();
  } else {
    listPlugins();
  }
}

/**
 * List all installed plugins
 */
function listPlugins() {
  const plugins = getAllPlugins();
  
  if (plugins.length === 0) {
    info('No plugins installed yet.');
    info('Run "plugins install <repo_url>" to install plugins.');
    return;
  }
  
  // Group by source
  const bySource = {};
  for (const plugin of plugins) {
    if (!bySource[plugin.source]) {
      bySource[plugin.source] = [];
    }
    bySource[plugin.source].push(plugin);
  }
  
  console.log('\n' + '='.repeat(60));
  console.log(`  Claude Code Plugins - ${plugins.length} installed`);
  console.log('='.repeat(60));
  
  for (const [source, sourcePlugins] of Object.entries(bySource)) {
    console.log(`\n${colors.bright}[${source}]${colors.reset} (${sourcePlugins.length} plugins)`);
    console.log('-'.repeat(60));
    
    for (const plugin of sourcePlugins) {
      console.log(`  ${colors.cyan}${plugin.name}${colors.reset}`);
      if (plugin.version) {
        console.log(`    Version: ${plugin.version}`);
      }
      if (plugin.description) {
        const shortDesc = plugin.description.substring(0, 70) + 
          (plugin.description.length > 70 ? '...' : '');
        console.log(`    ${shortDesc}`);
      }
      if (plugin.author) {
        console.log(`    Author: ${plugin.author}`);
      }
      if (plugin.installedPath) {
        console.log(`    Path: ${plugin.installedPath}`);
      }
      console.log('');
    }
  }
  
  console.log('='.repeat(60));
  console.log(`\nTotal: ${plugins.length} plugin(s) from ${Object.keys(bySource).length} source(s)`);
  console.log('\nRun "plugins list --removed" to see uninstalled plugins');
}

/**
 * List removed plugins
 */
function listRemoved() {
  const removed = getRemovedPlugins();
  
  if (removed.length === 0) {
    info('No removed plugins.');
    return;
  }
  
  console.log('\n' + '='.repeat(60));
  console.log(`  Removed Plugins - ${removed.length}`);
  console.log('='.repeat(60));
  
  for (const plugin of removed) {
    console.log(`\n${colors.yellow}${plugin.name}${colors.reset}`);
    console.log(`  Source: ${plugin.source}`);
    console.log(`  Removed: ${new Date(plugin.removedAt).toLocaleString()}`);
    if (plugin.reason) {
      console.log(`  Reason: ${plugin.reason}`);
    }
  }
  
  console.log('\n' + '='.repeat(60));
  console.log('\nRun "plugins list" to see installed plugins');
}

/**
 * List all sources
 */
function listSources() {
  const manifest = loadManifest();
  const sources = manifest.sources || {};
  const sourceEntries = Object.entries(sources);
  
  if (sourceEntries.length === 0) {
    info('No sources configured.');
    info('Run "plugins install <repo_url>" to add a source.');
    return;
  }
  
  console.log('\n' + '='.repeat(60));
  console.log(`  Plugin Sources - ${sourceEntries.length}`);
  console.log('='.repeat(60));
  
  for (const [name, source] of sourceEntries) {
    console.log(`\n${colors.bright}${name}${colors.reset}`);
    console.log(`  URL: ${source.repoUrl}`);
    console.log(`  Type: ${source.type}`);
    console.log(`  Plugins: ${source.pluginCount || 0}`);
    if (source.lastSyncAt) {
      console.log(`  Last sync: ${new Date(source.lastSyncAt).toLocaleString()}`);
    }
  }
  
  console.log('\n' + '='.repeat(60));
}

/**
 * List plugins in JSON format
 */
function listPluginsJson() {
  const manifest = loadManifest();
  console.log(JSON.stringify(manifest, null, 2));
}

/**
 * Parse command line arguments
 */
function parseArgs(args) {
  const options = {
    showRemoved: false,
    showSources: false,
    json: false
  };
  
  for (const arg of args) {
    switch (arg) {
      case '--removed':
      case '-r':
        options.showRemoved = true;
        break;
      case '--sources':
        options.showSources = true;
        break;
      case '--json':
        options.json = true;
        break;
      case '--help':
      case '-h':
        printHelp();
        process.exit(0);
    }
  }
  
  return options;
}

/**
 * Print help message
 */
function printHelp() {
  console.log(`
Claude Code Plugins Lister
Usage: plugins list [options]

Options:
  --removed     Show removed plugins
  --sources     Show all configured sources
  --json        Output in JSON format

Examples:
  plugins list
  plugins list --removed
  plugins list --sources
  plugins list --json
`);
}

export default { run };

// Allow direct execution: `node list.mjs [options]`
import { fileURLToPath } from 'url';
const isDirect = process.argv[1] === fileURLToPath(import.meta.url);
if (isDirect) {
  run(process.argv.slice(2));
}

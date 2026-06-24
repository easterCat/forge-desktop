#!/usr/bin/env node
/**
 * Install command
 *
 * Default behavior (FEAT-009): only sync the manifest — does NOT download any
 * plugin files to plugins/<source>/<name>/.  This keeps disk usage near zero.
 *
 * --all  Behaviour: full clone and copy (legacy, for dev/debug).
 * --only  Behaviour: same as --all but only copies listed plugin names.
 */

import {
  parseGitHubUrl,
  inferSourceName,
  getPath,
  ensureDir,
  exec,
  info,
  success,
  warn,
} from './utils.mjs';
import { clone, cloneSparse, cleanupTempDir, isGitAvailable } from './git.mjs';
import { scanForPlugins } from './scanner.mjs';
import { addPlugins, loadManifest } from './manifest.mjs';
import { existsSync, rmSync, cpSync } from 'fs';
import { join } from 'path';

/**
 * Copy plugin files to local plugins directory.
 * Used only when --all or --only is specified.
 */
function copyPluginToLocal(pluginDir, sourceName, pluginName) {
  const destDir = getPath('plugins', sourceName, pluginName);
  ensureDir(getPath('plugins', sourceName));

  if (existsSync(destDir)) {
    rmSync(destDir, { recursive: true, force: true });
  }

  try {
    cpSync(pluginDir, destDir, { recursive: true });
    return true;
  } catch (err) {
    warn(`Failed to copy plugin ${pluginName}: ${err.message}`);
    return false;
  }
}

/**
 * Find plugin source directory in cloned repo.
 */
function findPluginSourceDir(repoDir, pluginName) {
  const searchPaths = [
    join(repoDir, 'plugins', pluginName),
    join(repoDir, pluginName),
  ];

  for (const path of searchPaths) {
    if (existsSync(path)) {
      return path;
    }
  }
  return null;
}

/**
 * Get source type from source name.
 */
function getSourceType(sourceName) {
  const types = {
    'anthropics': 'official',
    'ccplugins': 'community-curated',
    'ananddtyagi': 'marketplace-index',
  };
  return types[sourceName] || 'unknown';
}

/**
 * Run install command.
 */
export async function run(args) {
  const options = parseArgs(args);

  // Validate repo URL
  if (!options.repoUrl) {
    warn('Repository URL is required');
    warn(
      'Usage: node scripts/plugins/install.mjs <repo_url> [--source <name>] [--branch <name>] [--only <plugin1,plugin2>] [--all]'
    );
    warn('');
    warn('  Default: sync manifest only (no plugin files downloaded)');
    warn('  --all:   clone entire repo and copy ALL plugin files to plugins/<source>/');
    warn('  --only:  clone entire repo and copy ONLY listed plugins (requires --all)');
    process.exit(1);
  }

  // Check git availability
  if (!await isGitAvailable()) {
    warn('Git is not available. Please install git first.');
    process.exit(2);
  }

  // Parse GitHub URL
  const parsed = parseGitHubUrl(options.repoUrl);
  if (!parsed) {
    warn(`Invalid GitHub URL: ${options.repoUrl}`);
    process.exit(1);
  }

  const sourceName = options.source || inferSourceName(options.repoUrl);
  if (!sourceName) {
    warn(`Could not infer source name from URL: ${options.repoUrl}`);
    warn('Use --source to specify a custom source name');
    process.exit(1);
  }

  // --only requires --all
  if (options.only.length > 0 && !options.all) {
    warn('--only can only be used together with --all');
    process.exit(1);
  }

  info(`Syncing plugins from ${options.repoUrl}...`);
  info(`Source: ${sourceName}`);

  let tmpRepoDir = null;

  try {
    if (options.all) {
      // ── Full clone + copy ────────────────────────────────────────────────
      info('Mode: full sync (--all) — cloning entire repository');
      tmpRepoDir = await clone(options.repoUrl, options.branch, { progress: true });

      info('Scanning for plugins...');
      let plugins = scanForPlugins(tmpRepoDir, sourceName, parsed.url);

      if (plugins.length === 0) {
        warn('No plugins found in repository');
        process.exit(0);
      }

      info(`Found ${plugins.length} plugin(s)`);

      // Filter by --only if specified
      if (options.only.length > 0) {
        const onlySet = new Set(options.only);
        plugins = plugins.filter((p) => onlySet.has(p.name));
        info(`Filtered to ${plugins.length} plugin(s) based on --only option`);
      }

      // Copy plugins to local directory
      info('Copying plugin files to plugins/...');
      let installedCount = 0;

      for (const plugin of plugins) {
        const sourceDir = findPluginSourceDir(tmpRepoDir, plugin.name);
        if (sourceDir) {
          const destDir = getPath('plugins', sourceName, plugin.name);
          ensureDir(getPath('plugins', sourceName));

          if (copyPluginToLocal(sourceDir, sourceName, plugin.name)) {
            plugin.installedPath = `plugins/${sourceName}/${plugin.name}`;
            installedCount++;
            success(`  Installed: ${plugin.name}`);
          } else {
            warn(`  Failed to install: ${plugin.name}`);
          }
        } else {
          warn(`  Could not find source for: ${plugin.name}`);
        }
      }

      info(`Copied ${installedCount}/${plugins.length} plugin(s) to plugins/`);
    } else {
      // ── Manifest-only sync (FEAT-009 default) ─────────────────────────────
      info('Mode: manifest-only sync — downloading plugin list only (no file copies)');

      // Clone with sparse-checkout so we only download directory metadata,
      // not file blobs.  This is fast even for large repos.
      tmpRepoDir = await cloneSparse(options.repoUrl, options.branch);

      info('Scanning for plugins...');
      let plugins = scanForPlugins(tmpRepoDir, sourceName, parsed.url);

      if (plugins.length === 0) {
        warn('No plugins found in repository');
        process.exit(0);
      }

      info(`Found ${plugins.length} plugin(s) — manifest updated, no files copied`);
      info('Tip: use --all to also copy plugin files to plugins/<source>/');
    }

    // ── Always update marketplace.json ────────────────────────────────────
    const sourceInfo = {
      repoUrl: parsed.url,
      type: getSourceType(sourceName),
      external: false,
      lastSyncAt: new Date().toISOString(),
    };

    // Load plugins from scanner output (they are in scope from the branch above)
    const plugins = scanForPlugins(tmpRepoDir, sourceName, parsed.url);
    addPlugins(sourceName, plugins, sourceInfo);

    console.log('\n' + '='.repeat(50));
    success(`Manifest synced: ${plugins.length} plugin(s) from ${sourceName}`);
    console.log('='.repeat(50));

    console.log(`\nMarketplace updated: $FORGE_HOME/plugins/marketplace.json`);
    if (!options.all) {
      console.log(`Disk usage: 0 bytes in $FORGE_HOME/plugins/ (use --all to download files)`);
    }
  } catch (err) {
    warn(`Sync failed: ${err.message}`);
    if (err.message.includes('Invalid GitHub URL') || err.message.includes('Could not infer source')) {
      process.exit(1);
    } else if (err.message.includes('not found') || err.message.includes('Authentication failed') || err.message.includes('network')) {
      process.exit(2);
    } else {
      process.exit(1);
    }
  } finally {
    if (tmpRepoDir) {
      cleanupTempDir(tmpRepoDir);
    }
  }
}

/**
 * Parse command line arguments.
 */
function parseArgs(args) {
  const options = {
    repoUrl: null,
    source: null,
    branch: 'main',
    only: [],
    all: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    switch (arg) {
      case '--source':
      case '-s':
        options.source = args[++i];
        break;
      case '--branch':
      case '-b':
        options.branch = args[++i];
        break;
      case '--only':
        options.only = args[++i].split(',').map((s) => s.trim());
        break;
      case '--all':
        options.all = true;
        break;
      case '--help':
      case '-h':
        printHelp();
        process.exit(0);
      default:
        if (!arg.startsWith('-')) {
          options.repoUrl = arg;
        }
    }
  }

  return options;
}

/**
 * Print help message.
 */
function printHelp() {
  console.log(`
Claude Code Plugins Installer
Usage: node scripts/plugins/install.mjs <repo_url> [options]

Options:
  --source <name>    Custom source name (default: inferred from URL)
  --branch <name>   Git branch to clone (default: main)
  --only <names>     Only install specified plugins (requires --all)
  --all              Clone entire repo and copy ALL plugin files
                     (default: manifest-only sync, 0 disk usage)

Examples:
  # Default: sync manifest only (no files downloaded)
  node scripts/plugins/install.mjs https://github.com/anthropics/claude-plugins-official

  # Full sync: clone and copy all plugins
  node scripts/plugins/install.mjs https://github.com/anthropics/claude-plugins-official --all

  # Only specific plugins
  node scripts/plugins/install.mjs https://github.com/anthropics/claude-plugins-official --all --only code-review,braand-search
`);
}

export default { run };

// Allow direct execution
import { fileURLToPath } from 'url';
const isDirect = process.argv[1] === fileURLToPath(import.meta.url);
if (isDirect) {
  run(process.argv.slice(2)).catch((err) => {
    console.error(`Fatal: ${err.message}`);
    process.exit(1);
  });
}

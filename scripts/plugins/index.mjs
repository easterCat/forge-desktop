#!/usr/bin/env node
/**
 * Claude Code Plugins CLI
 * Entry point for all plugin management commands
 * 
 * Usage:
 *   plugins install <repo_url> [options]
 *   plugins uninstall <plugin_name> [options]
 *   plugins list [options]
 *   plugins --help
 */

import { error } from './utils.mjs';

// CLI configuration
const COMMANDS = {
  install: {
    description: 'Install plugins from a GitHub repository',
    module: './install.mjs'
  },
  uninstall: {
    description: 'Uninstall a plugin',
    module: './uninstall.mjs'
  },
  list: {
    description: 'List installed plugins',
    module: './list.mjs'
  },
  init: {
    description: 'Scaffold a new plugin',
    module: './init.mjs'
  }
};

/**
 * Print main help
 */
function printHelp() {
  console.log(`
Claude Code Plugins Manager v1.0.0

A local plugin sync and management system for Claude Code.

Usage:
  plugins <command> [options]

Commands:
  install <repo_url>    Install plugins from a GitHub repository
  uninstall <name>       Uninstall a plugin
  list                  List installed plugins
  init <name>          Scaffold a new plugin

Options:
  --help, -h            Show this help message
  --version, -v         Show version

Examples:
  plugins install https://github.com/anthropics/claude-plugins-official
  plugins install https://github.com/ccplugins/awesome-claude-code-plugins
  plugins uninstall my-plugin
  plugins list
  plugins list --removed
  plugins list --sources
  plugins init my-plugin --description="My plugin"

For more details on a specific command, run:
  plugins <command> --help
`);
}

/**
 * Print version
 */
function printVersion() {
  console.log('Claude Code Plugins Manager v1.0.0');
}

// Main entry point
async function main() {
  const args = process.argv.slice(2);
  
  // Handle global flags
  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    process.exit(0);
  }
  
  if (args.includes('--version') || args.includes('-v')) {
    printVersion();
    process.exit(0);
  }
  
  // Get command
  const [command, ...commandArgs] = args;
  
  if (!command) {
    printHelp();
    process.exit(0);
  }
  
  // Route to command
  const cmd = COMMANDS[command];
  
  if (!cmd) {
    error(`Unknown command: ${command}`);
    error('');
    error('Available commands:');
    for (const [name, info] of Object.entries(COMMANDS)) {
      console.log(`  ${name.padEnd(15)} ${info.description}`);
    }
    error('');
    error('Run "plugins --help" for usage information');
    process.exit(1);
  }
  
  try {
    // Dynamic import of command module
    const module = await import(cmd.module);
    await module.run(commandArgs);
  } catch (err) {
    error(`Command failed: ${err.message}`);
    process.exit(1);
  }
}

// Run
main().catch(err => {
  error(`Fatal error: ${err.message}`);
  process.exit(1);
});

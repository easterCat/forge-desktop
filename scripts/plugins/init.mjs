#!/usr/bin/env node
/**
 * Claude Code Plugin Initializer
 * 
 * Scaffolds a new Claude Code plugin with the standard directory structure.
 * 
 * Usage:
 *   plugins init <name> [--description=<desc>] [--author=<author>]
 */

import { resolve, join } from 'path';
import { existsSync, mkdirSync, writeFileSync, chmodSync } from 'fs';
import { log, success, error } from './utils.mjs';

const VALID_NAME = /^[a-z0-9-]+$/;

function parseArgs(args) {
  const result = { name: null, description: '', author: '' };
  for (const arg of args) {
    if (arg.startsWith('--description=')) {
      result.description = arg.replace('--description=', '');
    } else if (arg.startsWith('--author=')) {
      result.author = arg.replace('--author=', '');
    } else if (!arg.startsWith('-')) {
      result.name = arg;
    }
  }
  return result;
}

function scaffoldPlugin(targetDir, name, opts) {
  if (!VALID_NAME.test(name)) {
    error(`Invalid plugin name "${name}". Use only lowercase letters, numbers, and hyphens.`);
    process.exit(1);
  }

  if (existsSync(targetDir)) {
    error(`Directory already exists: ${targetDir}`);
    process.exit(1);
  }

  log(`Scaffolding plugin "${name}" at ${targetDir}`);

  // Create directory structure
  const dirs = [
    '.claude-plugin',
    'skills',
    'commands',
    'hooks',
  ];
  for (const d of dirs) {
    mkdirSync(join(targetDir, d), { recursive: true });
  }

  // 1. plugin.json
  const pluginJson = {
    name,
    version: '0.1.0',
    description: opts.description || `A Claude Code plugin: ${name}`,
    author: opts.author || '',
    license: 'MIT',
    dependencies: [],
  };
  writeFileSync(
    join(targetDir, '.claude-plugin', 'plugin.json'),
    JSON.stringify(pluginJson, null, 2) + '\n'
  );

  // 2. SKILL.md scaffold (inside skills/<name>/SKILL.md)
  const skillDir = join(targetDir, 'skills', `${name}-skill`);
  mkdirSync(skillDir, { recursive: true });

  const skillMd = `---
name: ${name}-skill
description: ${opts.description || `A skill for the ${name} plugin.`}
---

# ${name}-skill

Write your skill instructions here.

## Usage

Describe how to use this skill.

## Examples

Provide example usage scenarios.
`;
  writeFileSync(join(skillDir, 'SKILL.md'), skillMd);

  // 3. Example command
  const commandMd = `---
name: ${name}
description: ${opts.description || `Execute the ${name} plugin command.`}
allowed-tools: Read, Edit, Bash
---

# ${name} Command

Describe what this command does.
`;
  writeFileSync(join(targetDir, 'commands', `${name}.md`), commandMd);

  // 4. hooks.json with example hook
  const hooksJson = {
    hooks: [
      {
        event: 'SessionStart',
        matcher: null,
        command: 'bash hooks/session-start.sh',
      },
    ],
  };
  writeFileSync(
    join(targetDir, 'hooks', 'hooks.json'),
    JSON.stringify(hooksJson, null, 2) + '\n'
  );

  // 5. Example hook script (executable)
  const hookScript = `#!/usr/bin/env bash
# ${name} plugin — SessionStart hook
echo "[${name}] Session started at $(date)"
`;
  writeFileSync(join(targetDir, 'hooks', 'session-start.sh'), hookScript);
  chmodSync(join(targetDir, 'hooks', 'session-start.sh'), 0o755);

  // 6. README.md
  const readme = `# ${name}

${opts.description || `A Claude Code plugin.`}

## Installation

Copy or link this plugin directory into your project's \`plugins/\` directory.

## Structure

\`\`\`
${name}/
  .claude-plugin/
    plugin.json      — Plugin manifest
  skills/
    ${name}-skill/   — Skill definitions
  commands/
    ${name}.md       — Slash commands
  hooks/
    hooks.json        — Hook declarations
    session-start.sh — Hook scripts
\`\`\`

## Author

${opts.author || 'Anonymous'}
`;
  writeFileSync(join(targetDir, 'README.md'), readme);

  success(`Plugin "${name}" scaffolded successfully at ${targetDir}`);
  log('');
  log('Next steps:');
  log(`  1. Edit .claude-plugin/plugin.json to set the correct version and author`);
  log(`  2. Write your skill in skills/${name}-skill/SKILL.md`);
  log(`  3. Add your slash commands in commands/`);
  log(`  4. Edit hooks/hooks.json and hooks/session-start.sh as needed`);
  log(`  5. Run: node scripts/plugins/install.mjs to install`);
}

async function run(args) {
  const opts = parseArgs(args);

  if (!opts.name) {
    error('Usage: plugins init <name> [--description=<desc>] [--author=<author>]');
    error('');
    error('Example:');
    error('  plugins init my-plugin --description="My custom plugin" --author="Your Name"');
    process.exit(1);
  }

  // Resolve target directory: <cwd>/plugins/<name>
  const targetDir = resolve('plugins', opts.name);
  scaffoldPlugin(targetDir, opts.name, opts);
}

export { run };

#!/usr/bin/env node
/**
 * Utility functions for Forge plugins management
 *
 * All installed content (marketplace manifests, plugin source files,
 * agents, rules, skills, prompts, logs, db) lives under the user-level
 * FORGE_HOME directory (default: ~/.forge). This can be overridden by
 * setting the FORGE_HOME environment variable — useful for tests and
 * portable dev setups.
 */

import { dirname, join, isAbsolute } from 'path';
import { fileURLToPath } from 'url';
import { homedir } from 'os';
import { existsSync, readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync, renameSync, copyFileSync, unlinkSync } from 'fs';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Resolve the Forge home directory. Order of precedence:
 *   1. FORGE_HOME environment variable (absolute or ~/-relative)
 *   2. ~/.forge
 *
 * Tilde (`~`) and `~/...` are expanded to the current user's home.
 */
export function getForgeHome() {
  const raw = process.env.FORGE_HOME;
  if (raw && raw.trim().length > 0) {
    let v = raw.trim();
    if (v === '~' || v.startsWith('~/') || v.startsWith('~\\')) {
      v = join(homedir(), v.slice(1));
    }
    if (!isAbsolute(v)) {
      v = join(process.cwd(), v);
    }
    return v;
  }
  return join(homedir(), '.forge');
}

/**
 * Backwards-compatible alias — the project root for plugin storage is the
 * Forge home directory. Kept as `PROJECT_ROOT` so existing call sites
 * (getPath, exec) continue to work without churn.
 */
export const PROJECT_ROOT = getForgeHome();

/**
 * Get the Forge home directory.
 * @deprecated prefer `getForgeHome()` — kept for callers that already
 * rely on this name.
 */
export function getProjectRoot() {
  return getForgeHome();
}

/**
 * Get a path under the Forge home directory.
 *
 *   getPath('plugins', 'anthropics', 'code-review')
 *     -> ~/.forge/plugins/anthropics/code-review
 *
 *   getPath('plugins', 'marketplace.json')
 *     -> ~/.forge/plugins/marketplace.json
 */
export function getPath(...segments) {
  return join(getForgeHome(), ...segments);
}

/**
 * Read JSON file safely
 */
export function readJson(filePath) {
  try {
    if (!existsSync(filePath)) {
      return null;
    }
    const content = readFileSync(filePath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    console.error(`Error reading JSON from ${filePath}:`, error.message);
    return null;
  }
}

/**
 * Write JSON file atomically (using rename for atomicity)
 */
export function writeJson(filePath, data) {
  const dir = dirname(filePath);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  
  const tmpPath = `${filePath}.tmp.${Date.now()}`;
  writeFileSync(tmpPath, JSON.stringify(data, null, 2), 'utf-8');
  
  try {
    renameSync(tmpPath, filePath);
  } catch (error) {
    try {
      copyFileSync(tmpPath, filePath);
      unlinkSync(tmpPath);
    } catch {
      try { unlinkSync(tmpPath); } catch { /* cleanup best-effort */ }
      throw error;
    }
  }
}

/**
 * Execute shell command and return promise
 */
export function exec(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, {
      cwd: options.cwd || PROJECT_ROOT,
      env: { ...process.env, ...options.env },
      shell: process.platform === 'win32',
      stdio: options.silent ? 'pipe' : 'inherit'
    });

    let stdout = '';
    let stderr = '';

    if (proc.stdout) {
      proc.stdout.on('data', (data) => {
        stdout += data.toString();
        if (options.onProgress) {
          options.onProgress(data.toString());
        }
      });
    }

    if (proc.stderr) {
      proc.stderr.on('data', (data) => {
        stderr += data.toString();
      });
    }

    proc.on('close', (code) => {
      if (code === 0 || options.ignoreExitCode) {
        resolve({ code, stdout, stderr });
      } else {
        const error = new Error(`Command failed with exit code ${code}: ${command} ${args.join(' ')}`);
        error.code = code;
        error.stdout = stdout;
        error.stderr = stderr;
        reject(error);
      }
    });

    proc.on('error', (error) => {
      reject(error);
    });

    if (options.timeout) {
      setTimeout(() => {
        proc.kill();
        reject(new Error(`Command timed out after ${options.timeout}ms`));
      }, options.timeout);
    }
  });
}

/**
 * Parse GitHub URL to extract owner, repo, and branch
 */
export function parseGitHubUrl(url) {
  const patterns = [
    new RegExp('github\\.com/([^/]+)/([^/]+?)(?:\\.git)?$'),
    new RegExp('github\\.com/([^/]+)/([^/]+?)(?:\\.git)?/tree/([^/]+)/'),
    new RegExp('github\\.com/([^/]+)/([^/]+?)(?:\\.git)?/pull/(\\d+)')
  ];

  for (const pattern of patterns) {
    const match = url.match(pattern);
    if (match) {
      return {
        owner: match[1],
        repo: match[2].replace(/\.git$/, ''),
        branch: match[3] || 'main',
        url: `https://github.com/${match[1]}/${match[2].replace(/\.git$/, '')}`
      };
    }
  }

  return null;
}

/**
 * Infer source name from GitHub URL
 */
export function inferSourceName(url) {
  const parsed = parseGitHubUrl(url);
  if (!parsed) return null;

  const knownRepos = {
    'anthropics/claude-plugins-official': 'anthropics',
    'ccplugins/awesome-claude-code-plugins': 'ccplugins',
    'ananddtyagi/cc-marketplace': 'ananddtyagi'
  };

  const repoKey = `${parsed.owner}/${parsed.repo}`;
  return knownRepos[repoKey] || parsed.owner;
}

/**
 * Ensure directory exists
 */
export function ensureDir(dirPath) {
  if (!existsSync(dirPath)) {
    mkdirSync(dirPath, { recursive: true });
  }
}

/**
 * Remove directory if empty
 */
export function removeEmptyDir(dirPath) {
  try {
    const entries = readdirSync(dirPath);
    const visibleEntries = entries.filter(e => !e.startsWith('.'));
    
    if (visibleEntries.length === 0) {
      rmSync(dirPath, { recursive: true, force: true });
      return true;
    }
    return false;
  } catch {
    return false;
  }
}

/**
 * Clean up empty parent directories
 */
export function cleanupEmptyDirs(baseDir, maxDepth = 3) {
  for (let depth = 0; depth < maxDepth; depth++) {
    const result = removeEmptyDir(baseDir);
    if (!result) break;
    baseDir = dirname(baseDir);
  }
}

/**
 * Print colored console output
 */
export const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m'
};

export function log(message, color = 'reset') {
  const c = colors[color] || '';
  console.log(`${c}${message}${colors.reset}`);
}

export function error(message) {
  log(`Error: ${message}`, 'red');
}

export function success(message) {
  log(message, 'green');
}

export function info(message) {
  log(message, 'cyan');
}

export function warn(message) {
  log(message, 'yellow');
}

/**
 * Format bytes to human readable
 */
export function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Get current ISO timestamp
 */
export function getTimestamp() {
  return new Date().toISOString();
}

/**
 * Simple file locking using fs.rename for atomicity
 */
export class FileLock {
  constructor(filePath) {
    this.lockPath = `${filePath}.lock`;
    this.filePath = filePath;
  }

  acquire(timeout = 5000) {
    const startTime = Date.now();
    while (existsSync(this.lockPath)) {
      if (Date.now() - startTime > timeout) {
        throw new Error(`Failed to acquire lock for ${this.filePath}: timeout`);
      }
      // Simple polling - in real production, use a proper lockfile library
    }
    
    writeFileSync(this.lockPath, JSON.stringify({ pid: process.pid, time: Date.now() }));
  }

  release() {
    try {
      if (existsSync(this.lockPath)) {
        rmSync(this.lockPath);
      }
    } catch {
      // Ignore errors on release
    }
  }
}

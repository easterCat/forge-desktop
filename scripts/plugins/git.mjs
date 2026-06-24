#!/usr/bin/env node
/**
 * Git operations wrapper
 * Handles clone, submodule operations with cross-platform support
 */

import { exec, info, warn, error } from './utils.mjs';
import { tmpdir } from 'os';
import { join } from 'path';
import { mkdirSync, rmSync, existsSync } from 'fs';

/**
 * Clone a repository to a temporary directory
 * @param {string} repoUrl - GitHub repository URL
 * @param {string} branch - Branch to clone (default: main)
 * @param {object} options - Additional options
 * @returns {Promise<string>} - Path to cloned repository
 */
export async function clone(repoUrl, branch = 'main', options = {}) {
  const tmpPath = options.tmpPath || join(tmpdir(), 'claude-plugins-sync');
  
  // Create temp directory if it doesn't exist
  if (!existsSync(tmpPath)) {
    mkdirSync(tmpPath, { recursive: true });
  }

  // Extract owner from URL for directory naming
  const ownerMatch = repoUrl.match(/github\.com\/([^\/]+)\/([^\/]+?)(?:\.git)?$/);
  if (!ownerMatch) {
    throw new Error(`Invalid GitHub URL: ${repoUrl}`);
  }
  
  const [, owner, repo] = ownerMatch;
  const destDir = join(tmpPath, owner);
  const repoDir = join(destDir, repo.replace(/\.git$/, ''));

  // Remove existing directory if it exists (for re-clone)
  if (existsSync(repoDir)) {
    try {
      rmSync(repoDir, { recursive: true, force: true });
    } catch (err) {
      warn(`Failed to remove existing directory: ${err.message}`);
    }
  }

  info(`Cloning ${repoUrl} (branch: ${branch})...`);
  
  try {
    // Use git clone with depth 1 for faster cloning
    const args = [
      'clone',
      '--depth', '1',
      '--branch', branch,
      '--single-branch',
      repoUrl,
      repoDir
    ];

    if (options.progress !== false) {
      info('This may take a moment depending on repository size...');
    }

    await exec('git', args, {
      timeout: options.timeout || 300000, // 5 minutes default
      silent: options.silent || false
    });

    return repoDir;
  } catch (err) {
    if (err.message.includes('Authentication failed') || 
        err.message.includes('could not find') ||
        err.message.includes('repository not found')) {
      throw new Error(`Failed to clone ${repoUrl}: Repository not found or access denied`);
    }
    throw err;
  }
}

/**
 * Clone as git submodule
 * @param {string} repoUrl - GitHub repository URL
 * @param {string} destPath - Destination path in plugins directory
 * @param {string} branch - Branch to clone
 */
export async function cloneAsSubmodule(repoUrl, destPath, branch = 'main') {
  info(`Adding as submodule: ${repoUrl}`);
  
  try {
    // Initialize submodules if not already
    await exec('git', ['submodule', 'update', '--init', '--recursive'], {
      ignoreExitCode: true
    });

    // Add submodule
    await exec('git', ['submodule', 'add', '-b', branch, repoUrl, destPath], {
      timeout: 120000
    });

    return true;
  } catch (err) {
    error(`Failed to add submodule: ${err.message}`);
    throw err;
  }
}

/**
 * Check if git is available
 */
export async function isGitAvailable() {
  try {
    await exec('git', ['--version']);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get current git version info
 */
export async function getGitVersion() {
  try {
    const result = await exec('git', ['--version']);
    return result.stdout.trim();
  } catch {
    return null;
  }
}

/**
 * Clone with sparse-checkout (no file blobs).
 * Fast for large repos — used for manifest-only sync.
 * @param {string} repoUrl - GitHub repository URL
 * @param {string} branch - Branch to clone (default: main)
 * @returns {Promise<string>} - Path to cloned repository
 */
export async function cloneSparse(repoUrl, branch = 'main') {
  const tmpPath = join(tmpdir(), 'claude-plugins-sync');
  if (!existsSync(tmpPath)) {
    mkdirSync(tmpPath, { recursive: true });
  }

  const ownerMatch = repoUrl.match(/github\.com\/([^\/]+)\/([^\/]+?)(?:\.git)?$/);
  if (!ownerMatch) {
    throw new Error(`Invalid GitHub URL: ${repoUrl}`);
  }

  const [, owner, repo] = ownerMatch;
  const destDir = join(tmpPath, owner);
  const repoDir = join(destDir, repo.replace(/\.git$/, ''));

  if (existsSync(repoDir)) {
    try {
      rmSync(repoDir, { recursive: true, force: true });
    } catch (err) {
      warn(`Failed to remove existing directory: ${err.message}`);
    }
  }

  info(`Sparse-cloning ${repoUrl} (branch: ${branch})...`);

  // git clone --depth 1 --filter=blob:none --sparse <url> <dir>
  await exec('git', [
    'clone',
    '--depth=1',
    '--filter=blob:none',
    '--sparse',
    '--branch',
    branch,
    repoUrl,
    repoDir,
  ], { timeout: 300000 });

  // git sparse-checkout set plugins/
  await exec('git', ['sparse-checkout', 'set', 'plugins/'], {
    cwd: repoDir,
    timeout: 60000,
  });

  return repoDir;
}

/**
 * Clean up temporary clone directory
 */
export function cleanupTempDir(tmpPath) {
  try {
    if (existsSync(tmpPath)) {
      rmSync(tmpPath, { recursive: true, force: true });
      return true;
    }
  } catch (err) {
    warn(`Failed to cleanup temp directory: ${err.message}`);
  }
  return false;
}

export default {
  clone,
  cloneSparse,
  cloneAsSubmodule,
  isGitAvailable,
  getGitVersion,
  cleanupTempDir
};

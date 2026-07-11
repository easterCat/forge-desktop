#!/usr/bin/env node

import { spawn, execSync } from 'child_process';
import { platform } from 'os';

const isWindows = platform() === 'win32';

// Kill any existing process on port 1420 before starting
function killPortProcess(port) {
  try {
    if (isWindows) {
      // Windows: find and kill process using the port
      const result = execSync(`netstat -aon | findstr :${port}`, { encoding: 'utf-8' });
      const lines = result.split('\n').filter(line => line.includes('LISTENING'));
      for (const line of lines) {
        const parts = line.trim().split(/\s+/);
        const pid = parts[parts.length - 1];
        if (pid && pid !== '0') {
          console.log(`Killing existing process on port ${port} (PID: ${pid})...`);
          try {
            execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
          } catch { /* process already gone */ }
        }
      }
    } else {
      // Unix/Mac: find and kill process using the port
      try {
        execSync(`lsof -ti:${port} | xargs kill -9 2>/dev/null || true`, { stdio: 'ignore' });
      } catch { /* no process found */ }
    }
  } catch { /* port already free */ }
}

killPortProcess(1420);

console.log(`Starting development on ${platform()}...`);

// Vite dev server
const web = spawn('npm', ['run', 'dev:web'], { shell: true, stdio: 'pipe' });

// Tauri dev - use platform-specific script
const tauriScript = isWindows ? 'tauri:dev:win' : 'tauri:dev:unix';
const tauri = spawn('npm', ['run', tauriScript], { shell: true, stdio: 'pipe' });

// Prefix output with labels
web.stdout?.on('data', d => process.stdout.write(`[web] ${d}`));
web.stderr?.on('data', d => process.stderr.write(`[web] ${d}`));

tauri.stdout?.on('data', d => process.stdout.write(`[tauri] ${d}`));
tauri.stderr?.on('data', d => process.stderr.write(`[tauri] ${d}`));

// Handle exit
let exited = false;
function cleanup() {
  if (exited) return;
  exited = true;
  web.kill();
  tauri.kill();
  process.exit();
}

web.on('exit', cleanup);
tauri.on('exit', cleanup);
process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

#!/usr/bin/env node
/**
 * Fetch Windsurf version metadata from Homebrew Cask (build-time).
 *
 * Windsurf is a closed-source editor distributed exclusively via the
 * Homebrew Cask tap `codeium/tap` (cask name: `windsurf`). It has no
 * npm package and no public GitHub releases, so the most authoritative
 * version source is `brew info --cask --json=v2 windsurf`.
 *
 * The script:
 *   1. Detects whether Homebrew is installed and on PATH.
 *   2. Calls `brew info --cask --json=v2 windsurf` (or falls back to
 *      `brew info --cask windsurf` text mode if JSON mode is unsupported).
 *   3. Parses out: version, tap, sha256, homepage, livecheck URL.
 *   4. Prints one of three outputs:
 *        - human report (default)         - colourised summary
 *        - JSON  (--json)                 - machine-readable, exits non-zero
 *                                          when the tool is missing
 *        - rust  (--rust)                 - a Rust seed patch snippet ready
 *                                          to be pasted into
 *                                          `seed_builtin_cli_tools()` in
 *                                          `src-tauri/src/services/cli_tools.rs`
 *
 * Design notes:
 *   - Pure build-time helper. Does NOT write to disk (no DB, no file
 *     patching). Callers decide how to consume the result. This keeps
 *     the script safe to run from CI and from local dev alike.
 *   - Does NOT introduce any new field on `CliToolConfig` or
 *     `custom_cli_tools` schema; the only fields the script can
 *     legitimately recommend updating today are `description`,
 *     `website_url`, and the existing `install_command` (which is
 *     already `brew install --cask windsurf` and is forward-compatible).
 *   - Tested via `scripts/cli-tools/__tests__/windsurf-version.spec.mjs`
 *     (mocks `execFile`, so it runs in any environment).
 */

import { execFile } from 'child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const execFileAsync = promisify(execFile);

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const REPO_ROOT = join(__dirname, '..', '..');

const WINDSURF_CASK = 'windsurf';
const EXPECTED_TAP = 'codeium/tap';

/* -------------------------------------------------------------------------- */
/*  Argument parsing                                                          */
/* -------------------------------------------------------------------------- */

function parseArgs(argv) {
  const opts = {
    mode: 'human', // 'human' | 'json' | 'rust'
    timeoutMs: 30_000,
    help: false,
  };
  for (const arg of argv) {
    switch (arg) {
      case '--json':
        opts.mode = 'json';
        break;
      case '--rust':
        opts.mode = 'rust';
        break;
      case '--help':
      case '-h':
        opts.help = true;
        break;
      default:
        if (arg.startsWith('--timeout=')) {
          const n = Number(arg.slice('--timeout='.length));
          if (Number.isFinite(n) && n > 0) opts.timeoutMs = n * 1000;
        } else {
          throw new Error(`Unknown argument: ${arg}`);
        }
    }
  }
  return opts;
}

function printHelp() {
  console.log(`Usage: node scripts/cli-tools/windsurf-version.mjs [options]

Fetch the latest Windsurf version metadata from Homebrew Cask so the
default (allagents) seed entry for 'windsurf' in
src-tauri/src/services/cli_tools.rs can be reviewed and updated.

Options:
  --json             Emit machine-readable JSON on stdout.
  --rust             Emit a Rust seed patch snippet on stdout.
  --timeout=<secs>   Override the brew call timeout (default 30).
  -h, --help         Show this help.

Exit codes:
  0  Brew found and returned Windsurf metadata.
  1  Brew is missing, the cask is not installed in the configured tap,
     or the JSON could not be parsed.
  2  Invalid CLI arguments.
`);
}

/* -------------------------------------------------------------------------- */
/*  Brew probing                                                              */
/* -------------------------------------------------------------------------- */

async function checkBrew() {
  try {
    const { stdout } = await execFileAsync('brew', ['--version'], {
      timeout: 5_000,
    });
    const firstLine = stdout.split('\n', 1)[0]?.trim() || 'brew (unknown version)';
    return { ok: true, version: firstLine };
  } catch (err) {
    return { ok: false, reason: err.message };
  }
}

// `WINDSURF_CASK` is the historical cask name. As of 2026 the upstream
// project has been folded into the "devin-desktop" cask and `windsurf`
// is only kept as an `old_tokens` alias. The probe order is therefore
// "windsurf first, then devin-desktop" — whichever Homebrew resolves is
// what we parse.
const PROBE_ORDER = ['windsurf', 'devin-desktop'];

async function fetchWindsurfCaskJson(timeoutMs) {
  // `brew info --cask --json=v2 <name>` returns a JSON array with one
  // entry per cask. We use execFile (not shell) so the cask name is
  // passed as a single argv entry — no quoting / injection risk.
  const tried = [];
  for (const cask of PROBE_ORDER) {
    tried.push(cask);
    // `brew info --cask --json=v2 <name>` returns a JSON object
    // `{ formulae: [], casks: [...] }`. We use execFile (not shell) so
    // the cask name is passed as a single argv entry — no quoting /
    // injection risk.
    //
    // We try each candidate in turn because the cask may have been
    // renamed upstream (e.g. `windsurf` → `devin-desktop`); the first
    // one that resolves is the one we parse. If all fail, the final
    // error message includes the tried list.
    let stdout;
    try {
      const result = await execFileAsync(
        'brew',
        ['info', '--cask', '--json=v2', cask],
        { timeout: timeoutMs, maxBuffer: 4 * 1024 * 1024 },
      );
      stdout = result.stdout;
    } catch {
      // Continue to the next candidate.
      continue;
    }
    return {
      ok: true,
      json: stdout,
      source: 'brew-info-json-v2',
      probedCask: cask,
      tried,
    };
  }
  return {
    ok: false,
    reason: `brew info --json=v2 failed for all candidates: ${tried.join(', ')}`,
    tried,
  };
}

/**
 * Parse the JSON object returned by `brew info --cask --json=v2 <name>`.
 * The schema is documented by Homebrew; the top level is an object
 * `{ formulae: [...], casks: [...] }` and we only look at the `casks`
 * array. We extract only the fields we actually consume, so a future
 * Homebrew schema change won't break us silently.
 *
 * Special case: the Windsurf cask has been renamed upstream (the cask
 * now installs as `devin-desktop` and reports `windsurf` only via
 * `old_tokens`). We still accept that JSON, but flag the rename in
 * the returned data so callers can surface it.
 */
function parseWindsurfCask(rawJson) {
  let parsed;
  try {
    parsed = JSON.parse(rawJson);
  } catch (e) {
    return { ok: false, reason: `JSON parse error: ${e.message}` };
  }
  // The top level is `{ formulae, casks }`, not a bare array.
  const casks = Array.isArray(parsed?.casks) ? parsed.casks : null;
  if (!casks || casks.length === 0) {
    return { ok: false, reason: 'brew returned an empty cask array' };
  }
  const cask = casks[0];
  const token = cask?.token;
  // `old_tokens` may be missing or absent in older brew versions.
  const oldTokens = Array.isArray(cask?.old_tokens) ? cask.old_tokens : [];
  const isWindsurfFamily = token === WINDSURF_CASK || oldTokens.includes(WINDSURF_CASK);
  if (!isWindsurfFamily) {
    return {
      ok: false,
      reason: `expected token "${WINDSURF_CASK}", got "${token ?? '<missing>'}" (old_tokens: ${oldTokens.join(',') || 'none'})`,
    };
  }

  // `version` is a string for normal casks. Some casks (including the
  // devin-desktop rebrand) report "<version>,<sha12>" — we want the
  // human-readable version prefix, not the comma-suffixed build id.
  const rawVersion = typeof cask.version === 'string' ? cask.version.trim() : null;
  if (!rawVersion) {
    return { ok: false, reason: 'cask.version is missing or not a string' };
  }
  const version = rawVersion.split(',')[0];

  const tap = typeof cask.tap === 'string' ? cask.tap : null;
  const sha256 = typeof cask.sha256 === 'string' ? cask.sha256 : null;
  const homepage = typeof cask.homepage === 'string' ? cask.homepage : null;
  // `livecheck` may be `null` (no livecheck strategy) or an object.
  const livecheckUrl =
    cask.livecheck && typeof cask.livecheck.url === 'string'
      ? cask.livecheck.url
      : null;

  // Detect upstream rename: the canonical cask token is no longer
  // "windsurf" but the old token is still in `old_tokens`.
  const renamed = token !== WINDSURF_CASK && oldTokens.includes(WINDSURF_CASK);

  return {
    ok: true,
    data: {
      token,
      version,
      tap,
      sha256,
      homepage,
      livecheckUrl,
      renamed,
      oldTokens,
    },
  };
}

/* -------------------------------------------------------------------------- */
/*  Output renderers                                                          */
/* -------------------------------------------------------------------------- */

const COLORS = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
};

function paint(text, color) {
  if (process.env.NO_COLOR) return text;
  return `${COLORS[color] ?? ''}${text}${COLORS.reset}`;
}

function renderHumanReport(ctx, payload) {
  const { data } = payload;
  const lines = [];
  lines.push(paint('Windsurf (Homebrew Cask) — version probe', 'bold'));
  lines.push(paint('─'.repeat(60), 'dim'));
  if (data.renamed) {
    lines.push(
      paint(
        `⚠  upstream rename detected: cask token is now "${data.token}" (old token "windsurf" in old_tokens)`,
        'yellow',
      ),
    );
  }
  lines.push(`${paint('cask          :', 'cyan')} ${data.token}`);
  if (data.oldTokens.length > 0) {
    lines.push(`${paint('old_tokens    :', 'cyan')} ${data.oldTokens.join(', ')}`);
  }
  lines.push(`${paint('version       :', 'cyan')} ${paint(data.version, 'green')}`);
  lines.push(
    `${paint('tap           :', 'cyan')} ${
      data.tap ? data.tap : paint('(unknown)', 'yellow')
    }`,
  );
  lines.push(
    `${paint('homepage      :', 'cyan')} ${
      data.homepage ? data.homepage : paint('(unknown)', 'yellow')
    }`,
  );
  lines.push(
    `${paint('livecheck URL :', 'cyan')} ${
      data.livecheckUrl ? data.livecheckUrl : paint('(none)', 'dim')
    }`,
  );
  lines.push(
    `${paint('sha256        :', 'cyan')} ${
      data.sha256 ? data.sha256 : paint('(none)', 'dim')
    }`,
  );
  lines.push(
    `${paint('brew          :', 'cyan')} ${ctx.brewVersion}${
      data.tap === EXPECTED_TAP ? '' : paint(`  (expected tap: ${EXPECTED_TAP})`, 'yellow')
    }`,
  );
  lines.push('');
  lines.push(paint('Suggested diff to apply in seed_builtin_cli_tools():', 'bold'));
  lines.push(renderRustSnippet(data));
  return lines.join('\n');
}

function renderRustSnippet(data) {
  // Keep the patch shape identical to the existing SeedEntry literal
  // in src-tauri/src/services/cli_tools.rs so a developer can paste
  // it directly over the windsurf entry. The fields we recommend
  // updating are description / website_url — install_command is
  // already correct because `brew install --cask windsurf` always
  // pulls the latest version. We do NOT introduce new columns.
  //
  // If the upstream cask has been renamed, the snippet still
  // references `key: "windsurf"` (we must not break the DB primary
  // key), but the description and the trailing comment warn the
  // developer that the brew cask name is now different.
  const tapNote = data.tap && data.tap !== EXPECTED_TAP
    ? `\n//  ! tap mismatch: expected ${EXPECTED_TAP}, got ${data.tap}`
    : '';
  const renameNote = data.renamed
    ? `\n//  ! upstream cask renamed: current token "${data.token}", old_tokens=[${data.oldTokens.join(', ')}]`
    : '';
  const description = data.homepage
    ? `Windsurf - AI coding assistant by Codeium (brew cask ${data.version})`
    : 'Windsurf - AI coding assistant by Codeium';
  return [
    '// --- begin snippet --------------------------------------------------',
    'SeedEntry {',
    '    key: "windsurf",',
    '    is_allagents: true,',
    '    name: "Windsurf",',
    '    icon: "windsurf",',
    `    description: "${description}",`,
    '    install_method: "brew",',
    '    install_command: "brew install --cask windsurf",',
    '    detect_command: "command -v windsurf || where windsurf",',
    `    website_url: Some("${data.homepage ?? 'https://codeium.com/windsurf'}"),`,
    '    plugin_dir: None,',
    '    timeout_secs: None,',
    '    npm_package: None,',
    '},',
    `// last verified: ${data.version}  (sha256: ${(data.sha256 ?? 'n/a').slice(0, 12)}…)${tapNote}${renameNote}`,
    '// --- end snippet ----------------------------------------------------',
  ].join('\n');
}

function renderRustMode(payload) {
  return renderRustSnippet(payload.data);
}

/* -------------------------------------------------------------------------- */
/*  Entry point                                                               */
/* -------------------------------------------------------------------------- */

async function main() {
  const opts = parseArgs(process.argv.slice(2));
  if (opts.help) {
    printHelp();
    return;
  }

  const brew = await checkBrew();
  if (!brew.ok) {
    const msg = `Homebrew is not available on PATH: ${brew.reason}`;
    if (opts.mode === 'json') {
      process.stdout.write(JSON.stringify({ ok: false, error: msg }) + '\n');
    } else {
      console.error(paint(`✗ ${msg}`, 'red'));
    }
    process.exit(1);
  }

  const fetched = await fetchWindsurfCaskJson(opts.timeoutMs);
  if (!fetched.ok) {
    if (opts.mode === 'json') {
      process.stdout.write(
        JSON.stringify({ ok: false, error: fetched.reason, brew: brew.version }) + '\n',
      );
    } else {
      console.error(paint(`✗ ${fetched.reason}`, 'red'));
    }
    process.exit(1);
  }

  const parsed = parseWindsurfCask(fetched.json);
  if (!parsed.ok) {
    if (opts.mode === 'json') {
      process.stdout.write(
        JSON.stringify({ ok: false, error: parsed.reason, brew: brew.version }) + '\n',
      );
    } else {
      console.error(paint(`✗ ${parsed.reason}`, 'red'));
    }
    process.exit(1);
  }

  const payload = {
    ok: true,
    source: fetched.source,
    brew: brew.version,
    data: parsed.data,
  };

  if (opts.mode === 'json') {
    process.stdout.write(JSON.stringify(payload, null, 2) + '\n');
  } else if (opts.mode === 'rust') {
    process.stdout.write(renderRustMode(payload) + '\n');
  } else {
    process.stdout.write(renderHumanReport({ brewVersion: brew.version }, payload) + '\n');
  }
  process.exit(0);
}

// Only run when invoked directly (not when imported by tests).
const isDirect = process.argv[1] === __filename;
if (isDirect) {
  main().catch((err) => {
    console.error(paint(`✗ unexpected error: ${err.message}`, 'red'));
    process.exit(1);
  });
}

// Re-export the pure functions so the test suite can call them
// without spawning a process.
export {
  parseWindsurfCask,
  renderRustSnippet,
  renderHumanReport,
  parseArgs,
  checkBrew,
  fetchWindsurfCaskJson,
  EXPECTED_TAP,
  WINDSURF_CASK,
  REPO_ROOT,
};

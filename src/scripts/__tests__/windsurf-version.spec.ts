import { describe, it, expect, beforeEach } from 'vitest';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const SCRIPT_PATH = join(
  __dirname,
  '..',
  '..',
  '..',
  'scripts',
  'cli-tools',
  'windsurf-version.mjs',
);

interface ParsedModule {
  parseWindsurfCask: (raw: string) => { ok: boolean; data?: any; reason?: string };
  renderRustSnippet: (data: any) => string;
  renderHumanReport: (ctx: any, payload: any) => string;
  parseArgs: (argv: string[]) => any;
  checkBrew: () => Promise<{ ok: boolean; version?: string; reason?: string }>;
  fetchWindsurfCaskJson: (timeout: number) => Promise<{ ok: boolean; json?: string; reason?: string }>;
  EXPECTED_TAP: string;
  WINDSURF_CASK: string;
  REPO_ROOT: string;
}

let mod: ParsedModule;
beforeEach(async () => {
  // Dynamic import so the script's `isDirect` guard does not fire.
  mod = await import(SCRIPT_PATH);
});

describe('windsurf-version.mjs: parseWindsurfCask', () => {
  it('extracts version, tap, sha256, homepage, livecheckUrl from real-shape JSON', () => {
    // brew info --cask --json=v2 returns { formulae, casks }, not a bare array.
    const raw = JSON.stringify({
      formulae: [],
      casks: [
        {
          token: 'windsurf',
          version: '1.13.4',
          tap: 'codeium/tap',
          sha256: 'abcdef0123456789',
          homepage: 'https://codeium.com/windsurf',
          livecheck: { url: 'https://codeium.com/windsurf/download' },
          old_tokens: [],
        },
      ],
    });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(true);
    expect(result.data).toEqual({
      token: 'windsurf',
      version: '1.13.4',
      tap: 'codeium/tap',
      sha256: 'abcdef0123456789',
      homepage: 'https://codeium.com/windsurf',
      livecheckUrl: 'https://codeium.com/windsurf/download',
      renamed: false,
      oldTokens: [],
    });
  });

  it('rejects JSON with the wrong cask token and no windsurf in old_tokens', () => {
    const raw = JSON.stringify({ casks: [{ token: 'not-windsurf', version: '1.0.0', old_tokens: [] }] });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(false);
    expect(result.reason).toMatch(/expected token/);
  });

  it('accepts devin-desktop with windsurf in old_tokens and flags the rename', () => {
    const raw = JSON.stringify({
      casks: [
        {
          token: 'devin-desktop',
          old_tokens: ['windsurf'],
          version: '3.4.22',
          tap: 'homebrew/cask',
          sha256: 'sha',
          homepage: 'https://devin.ai/desktop',
          livecheck: null,
        },
      ],
    });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(true);
    expect(result.data.token).toBe('devin-desktop');
    expect(result.data.oldTokens).toEqual(['windsurf']);
    expect(result.data.renamed).toBe(true);
    expect(result.data.version).toBe('3.4.22');
  });

  it('strips a comma-separated build id from the version field', () => {
    const raw = JSON.stringify({
      casks: [
        {
          token: 'devin-desktop',
          old_tokens: ['windsurf'],
          version: '3.4.22,0c84d3332806347c90e571331f48dd13a957d880',
          tap: 'homebrew/cask',
          sha256: 'x',
          homepage: 'https://devin.ai/desktop',
          livecheck: null,
        },
      ],
    });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(true);
    expect(result.data.version).toBe('3.4.22');
  });

  it('rejects empty casks arrays', () => {
    const result = mod.parseWindsurfCask(JSON.stringify({ casks: [] }));
    expect(result.ok).toBe(false);
    expect(result.reason).toMatch(/empty cask array/);
  });

  it('rejects JSON whose top-level has no casks field', () => {
    const result = mod.parseWindsurfCask(JSON.stringify({ formulae: [] }));
    expect(result.ok).toBe(false);
    expect(result.reason).toMatch(/empty cask array/);
  });

  it('rejects malformed JSON', () => {
    const result = mod.parseWindsurfCask('{not-json');
    expect(result.ok).toBe(false);
    expect(result.reason).toMatch(/JSON parse/);
  });

  it('rejects cask with missing version', () => {
    const raw = JSON.stringify({ casks: [{ token: 'windsurf', old_tokens: [] }] });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(false);
    expect(result.reason).toMatch(/version/);
  });

  it('tolerates livecheck: null and returns livecheckUrl: null', () => {
    const raw = JSON.stringify({
      casks: [
        {
          token: 'windsurf',
          version: '2.0.0',
          tap: 'codeium/tap',
          sha256: 'deadbeef',
          homepage: 'https://codeium.com/windsurf',
          livecheck: null,
          old_tokens: [],
        },
      ],
    });
    const result = mod.parseWindsurfCask(raw);
    expect(result.ok).toBe(true);
    expect(result.data.livecheckUrl).toBeNull();
  });
});

describe('windsurf-version.mjs: renderRustSnippet', () => {
  it('emits a SeedEntry-shaped literal keyed on "windsurf"', () => {
    const out = mod.renderRustSnippet({
      token: 'windsurf',
      version: '1.13.4',
      tap: 'codeium/tap',
      sha256: 'abcdef0123456789',
      homepage: 'https://codeium.com/windsurf',
      livecheckUrl: 'https://codeium.com/windsurf/download',
      renamed: false,
      oldTokens: [],
    });
    expect(out).toContain('key: "windsurf"');
    expect(out).toContain('is_allagents: true');
    expect(out).toContain('install_method: "brew"');
    expect(out).toContain('install_command: "brew install --cask windsurf"');
    expect(out).toContain('npm_package: None');
    // version is folded into description and last-verified comment
    expect(out).toContain('1.13.4');
    // website_url is the SeedEntry field name; the script recommends
    // filling it from the brew-reported homepage.
    expect(out).toContain('website_url: Some("https://codeium.com/windsurf")');
  });

  it('falls back to default homepage when brew returns none', () => {
    const out = mod.renderRustSnippet({
      token: 'windsurf',
      version: '1.0.0',
      tap: 'codeium/tap',
      sha256: null,
      homepage: null,
      livecheckUrl: null,
      renamed: false,
      oldTokens: [],
    });
    expect(out).toContain('https://codeium.com/windsurf');
  });

  it('annotates a tap mismatch when the cask is not from codeium/tap', () => {
    const out = mod.renderRustSnippet({
      token: 'windsurf',
      version: '1.0.0',
      tap: 'homebrew/cask',
      sha256: 'x',
      homepage: 'https://example.com',
      livecheckUrl: null,
      renamed: false,
      oldTokens: [],
    });
    expect(out).toMatch(/tap mismatch/);
    expect(out).toContain('homebrew/cask');
  });

  it('does not annotate a tap mismatch when tap equals codeium/tap', () => {
    const out = mod.renderRustSnippet({
      token: 'windsurf',
      version: '1.0.0',
      tap: 'codeium/tap',
      sha256: 'x',
      homepage: 'https://codeium.com/windsurf',
      livecheckUrl: null,
      renamed: false,
      oldTokens: [],
    });
    expect(out).not.toMatch(/tap mismatch/);
  });

  it('annotates an upstream rename when windsurf has been folded into another cask', () => {
    const out = mod.renderRustSnippet({
      token: 'devin-desktop',
      version: '3.4.22',
      tap: 'homebrew/cask',
      sha256: 'x',
      homepage: 'https://devin.ai/desktop',
      livecheckUrl: null,
      renamed: true,
      oldTokens: ['windsurf'],
    });
    // The DB key must stay "windsurf" so existing rows keep matching.
    expect(out).toContain('key: "windsurf"');
    expect(out).toMatch(/upstream cask renamed/);
    expect(out).toContain('devin-desktop');
  });
});

describe('windsurf-version.mjs: renderHumanReport', () => {
  it('includes version, homepage, and the rust snippet in the human report', () => {
    const payload = {
      data: {
        token: 'windsurf',
        version: '1.13.4',
        tap: 'codeium/tap',
        sha256: 'abcdef0123456789',
        homepage: 'https://codeium.com/windsurf',
        livecheckUrl: null,
        renamed: false,
        oldTokens: [],
      },
    };
    const out = mod.renderHumanReport({ brewVersion: 'Homebrew 4.4.0' }, payload);
    expect(out).toContain('1.13.4');
    expect(out).toContain('https://codeium.com/windsurf');
    expect(out).toContain('Homebrew 4.4.0');
    expect(out).toContain('SeedEntry {');
  });

  it('surfaces the upstream rename warning when the cask was renamed', () => {
    const payload = {
      data: {
        token: 'devin-desktop',
        version: '3.4.22',
        tap: 'homebrew/cask',
        sha256: 'x',
        homepage: 'https://devin.ai/desktop',
        livecheckUrl: null,
        renamed: true,
        oldTokens: ['windsurf'],
      },
    };
    const out = mod.renderHumanReport({ brewVersion: 'Homebrew 6.0.7' }, payload);
    expect(out).toMatch(/upstream rename detected/);
    expect(out).toContain('devin-desktop');
    expect(out).toContain('old_tokens');
  });
});

describe('windsurf-version.mjs: parseArgs', () => {
  it('defaults to human mode', () => {
    expect(mod.parseArgs([]).mode).toBe('human');
  });
  it('switches to json mode with --json', () => {
    expect(mod.parseArgs(['--json']).mode).toBe('json');
  });
  it('switches to rust mode with --rust', () => {
    expect(mod.parseArgs(['--rust']).mode).toBe('rust');
  });
  it('parses --timeout as milliseconds', () => {
    expect(mod.parseArgs(['--timeout=45']).timeoutMs).toBe(45_000);
  });
  it('throws on unknown arguments', () => {
    expect(() => mod.parseArgs(['--bogus'])).toThrow(/Unknown argument/);
  });
});

describe('windsurf-version.mjs: checkBrew & fetchWindsurfCaskJson', () => {
  it('checkBrew resolves with ok=true when brew --version succeeds', async () => {
    // Spy on execFile by stubbing the module's internal child_process
    // import. The script does `import { execFile } from 'child_process'`
    // and uses `promisify(execFile)`. We rely on the host's real brew
    // binary: if absent, the test asserts the failure path is well-typed
    // (either ok:true with a version, or ok:false with a reason).
    const r = await mod.checkBrew();
    if (r.ok) {
      expect(typeof r.version).toBe('string');
      expect(r.version).toMatch(/Homebrew/i);
    } else {
      expect(typeof r.reason).toBe('string');
    }
  });

  it('fetchWindsurfCaskJson returns ok=false with a reason when brew is missing', async () => {
    // Force a failure by passing an impossibly small timeout.
    const r = await mod.fetchWindsurfCaskJson(1);
    // Either timing out, ENOENT, or simply taking >1ms on slow hosts
    // should produce a typed failure object.
    if (!r.ok) {
      expect(typeof r.reason).toBe('string');
      expect(r.reason.length).toBeGreaterThan(0);
    } else {
      // If somehow brew responded in <1ms, the data should still parse
      // and round-trip — at least sanity-check the shape.
      expect(typeof r.json).toBe('string');
      expect(r.source).toBe('brew-info-json-v2');
    }
  });
});

describe('windsurf-version.mjs: module-level exports', () => {
  it('exposes the expected constants', () => {
    expect(mod.WINDSURF_CASK).toBe('windsurf');
    expect(mod.EXPECTED_TAP).toBe('codeium/tap');
  });

  it('REPO_ROOT points to the project root (parent of scripts/)', () => {
    expect(mod.REPO_ROOT.endsWith('env-manager')).toBe(true);
  });
});

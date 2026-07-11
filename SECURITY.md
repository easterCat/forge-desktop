# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Forge Desktop, please report it privately
to keep user data safe while we investigate and ship a fix.

**How to report:**

1. Open a private security advisory on GitHub: **Repository → Security → Advisories → New draft security advisory**
2. **Do not** open a public issue for security-sensitive bugs.
3. Include as much detail as possible:
   - Affected component (e.g. `src-tauri/src/services/cli_tools.rs`, `src/utils/markdown.ts`)
   - Reproduction steps (preferably a minimal PoC)
   - Impact assessment (data exposure, RCE, XSS, etc.)
   - Environment (OS, Tauri version, allagents version)

## Response Targets

| Stage          | SLA                              |
| -------------- | -------------------------------- |
| Acknowledgement | within 72 hours                  |
| Initial triage | within 7 days                    |
| Patch release  | severity-dependent (see below)   |

### Severity classification

- **Critical** (RCE, arbitrary file write, credential leak): patch within 7 days
- **High** (XSS in default UI, privilege escalation): patch within 30 days
- **Medium** (info disclosure, DoS): patch within 90 days
- **Low** (theoretical, requires user action): next regular release

## Security Architecture Notes

Forge Desktop ships as a local desktop app (Tauri 2.0) backed by SQLite and the
`allagents` CLI. The following design choices affect the threat model:

- **Local SQLite is the source of truth** — UI state never depends on remote services.
- **`allagents` runs as a child process** — commands are constructed via `shell_words`
  or explicit arg arrays; never via shell concatenation of user input.
- **GitHub tokens** are stored at `<app_data_dir>/github_token` with mode `0o600`,
  outside the SQLite DB to avoid leakage via `VACUUM INTO`.
- **Subprocess commands** on Windows use `CREATE_NO_WINDOW (0x08000000)` to avoid
  console pop-ups; this is cosmetic, not a security boundary.
- **All Tauri commands return `Result<T, String>`** — errors are surfaced to the UI
  via the `CommandResult` envelope; nothing is silently swallowed.

## Out-of-Scope

- Vulnerabilities in upstream dependencies (`@tauri-apps/*`, `allagents`, `pinia`,
  `vue`). Please report those to the respective maintainers.
- Issues that require physical access to the user's machine.
- Social-engineering attacks against the user.

## Acknowledgements

We appreciate responsible disclosure. Reporters of valid, previously-unknown
vulnerabilities will be credited in the release notes (unless they prefer to remain
anonymous).
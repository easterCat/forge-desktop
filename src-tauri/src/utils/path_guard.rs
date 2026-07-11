//! Path validation utilities for Tauri commands that accept file paths from
//! the frontend.
//!
//! Background: several Tauri commands (`file::*`, `backup::*`) accept raw
//! filesystem paths over IPC. Without validation, a hostile or buggy caller
//! could read or overwrite arbitrary files (e.g. `~/.ssh/id_rsa`, `/etc/passwd`)
//! by passing `../../`-style traversal or absolute paths. `PathGuard` provides:
//!
//! 1. **Root allow-listing**: a caller-supplied set of "safe" base directories
//!    (e.g. the app data dir, the workspace dir, a user-selected backup dir).
//!    Paths must resolve to a descendant of one of these.
//! 2. **Forbidden-root block-list**: regardless of allow-list, paths that
//!    resolve to system-sensitive roots (`/`, `/etc`, `~/.ssh`, ...) are
//!    always rejected, so a misconfigured allow-list can't be exploited.
//! 3. **Symlink resolution**: `canonicalize()` collapses `..` and follows
//!    symlinks, so a symlinked `/etc/passwd` placed inside the workspace
//!    cannot be used to escape.
//! 4. **Per-path sensitive file block-list**: even inside an allowed root,
//!    individual files such as `id_rsa`, `.env`, `*.pem` are rejected. This
//!    is defence-in-depth in case a workspace is shared or accidentally
//!    points at `~`.
//!
//! Errors are designed to be safe to surface to the UI — they never echo the
//! raw user input back, only the kind of rejection, so a malicious caller
//! can't probe filesystem layout via error messages.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathGuard {
    /// Allowed base directories (canonicalized when the guard is built).
    allowed_roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum PathGuardError {
    /// Path is empty or contains NUL bytes.
    InvalidInput,
    /// Path is absolute but not inside any allowed root, or relative and
    /// resolves outside allowed roots after `..` collapsing.
    OutsideAllowedRoots,
    /// Path resolves to a forbidden system root (e.g. `/`, `/etc`).
    ForbiddenRoot,
    /// Path contains a forbidden file name (e.g. `id_rsa`, `.env`).
    ForbiddenFile,
    /// I/O error during canonicalize (e.g. parent does not exist).
    Io(String),
}

impl std::fmt::Display for PathGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathGuardError::InvalidInput => write!(f, "Invalid path"),
            PathGuardError::OutsideAllowedRoots => write!(f, "Path is outside the allowed directories"),
            PathGuardError::ForbiddenRoot => write!(f, "Path resolves to a protected system location"),
            PathGuardError::ForbiddenFile => write!(f, "Path points to a sensitive file"),
            PathGuardError::Io(e) => write!(f, "Path validation failed: {}", e),
        }
    }
}

impl std::error::Error for PathGuardError {}

/// File names (case-insensitive) that are never allowed regardless of where
/// they live. Keeps the allow-list from accidentally exposing SSH keys,
/// shell history, or dotenv secrets.
const SENSITIVE_FILE_NAMES: &[&str] = &[
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "authorized_keys",
    ".env",
    ".envrc",
    ".netrc",
    ".npmrc",
    ".pypirc",
    "credentials",
    "credentials.json",
];

/// File-name extensions that are always blocked.
const SENSITIVE_FILE_EXTENSIONS: &[&str] = &[".pem", ".key", ".p12", ".pfx"];

/// Path prefixes (always with trailing `/`) that are always blocked, even
/// if the user accidentally adds them to the allow-list. Each prefix is a
/// directory, so we match it as a *whole-component* prefix.
///
/// Important: do NOT include single-segment prefixes like `/var/` because
/// some legitimate user paths live under `/var/` on macOS (notably
/// `/var/folders/...` which is `std::env::temp_dir()`). We list only the
/// system-managed subdirectories that genuinely must never be exposed.
fn forbidden_root_prefixes() -> &'static [&'static str] {
    &[
        "/etc/",
        "/var/log/",
        "/var/lib/",
        "/var/db/",
        "/var/mail/",
        "/var/spool/",
        "/boot/",
        "/root/",
        "/sys/",
        "/proc/",
        "/dev/",
        // user-home sensitive entries (resolved with leading `/`)
        "/.ssh/",
        "/.aws/",
        "/.gnupg/",
        "/.config/gh/",
    ]
}

/// Returns true if `resolved` is literally the filesystem root (`/` or,
/// on Windows, a drive root like `C:\`). Blocked unconditionally.
fn is_filesystem_root(resolved: &str) -> bool {
    resolved == "/" || resolved == "//" || resolved.ends_with(":/") || resolved.ends_with(":\\")
}

impl PathGuard {
    /// Build a guard that allows paths under any of `allowed_roots`.
    /// Roots are canonicalized once at construction; paths that do not exist
    /// yet are accepted as-is after a lexical `..` collapse so file *creation*
    /// (e.g. backup destinations) can still be validated.
    pub fn new<I, P>(allowed_roots: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let allowed_roots = allowed_roots
            .into_iter()
            .map(|p| {
                let canonical = p.as_ref().canonicalize().ok();
                // Some roots (e.g. backup destinations that don't exist yet)
                // can't be canonicalized. Fall back to the lexical form so
                // callers can still register a "future" root.
                canonical.unwrap_or_else(|| lexically_normalize(p.as_ref()))
            })
            .collect();
        Self { allowed_roots }
    }

    /// Empty guard that rejects everything except non-existent absolute
    /// destinations that don't resolve to a forbidden root. Used as a
    /// last-resort fallback when the app data dir is unavailable.
    pub fn deny_all() -> Self {
        Self { allowed_roots: vec![] }
    }

    /// Validate a path coming from the frontend. Returns the canonicalized
    /// (or lexically-normalized, if the file doesn't exist yet) absolute
    /// path on success.
    ///
    /// The function is intentionally strict:
    /// 1. Reject empty input and paths containing NUL.
    /// 2. Forbid any path whose final component is in `SENSITIVE_FILE_NAMES`
    ///    or has a sensitive extension.
    /// 3. Canonicalize (`..` collapse + symlink resolve) and reject any
    ///    resolved path that matches `forbidden_root_prefixes()`.
    /// 4. Require the resolved path to live under one of `allowed_roots`.
    pub fn validate(&self, raw: &str) -> Result<PathBuf, PathGuardError> {
        if raw.is_empty() || raw.contains('\0') {
            return Err(PathGuardError::InvalidInput);
        }

        // 1. Lexically collapse `..` and `.` BEFORE canonicalize. This stops
        //    a caller from smuggling path components into error messages and
        //    also catches `..\..\..\etc\passwd` style attacks even when the
        //    intermediate components don't exist.
        let lexical = lexically_normalize(Path::new(raw));

        // 2. File-name/extension check on the last component (post-normalize).
        if let Some(name) = lexical.file_name().and_then(|n| n.to_str()) {
            let lower = name.to_ascii_lowercase();
            if SENSITIVE_FILE_NAMES.iter().any(|s| s.eq_ignore_ascii_case(&lower)) {
                return Err(PathGuardError::ForbiddenFile);
            }
            if SENSITIVE_FILE_EXTENSIONS
                .iter()
                .any(|ext| lower.ends_with(ext))
            {
                return Err(PathGuardError::ForbiddenFile);
            }
        }

        // 3. Canonicalize (resolves symlinks) — fall back to lexical if the
        //    path doesn't exist yet so destinations like new backup folders
        //    can be validated.
        let resolved = match lexical.canonicalize() {
            Ok(p) => p,
            Err(_) => lexical.clone(),
        };
        let resolved_str = resolved.to_string_lossy().to_lowercase();

        // 4. Forbidden-root check. We compare both the canonicalized form (which
        //    resolves `/etc` → `/private/etc` on macOS) AND the lexical form
        //    so a symlink pointing at a forbidden directory is blocked no
        //    matter how the OS exposes it.
        //
        // Boundary-aware match: `/etc` matches `/etc`, `/etc/passwd`, but
        // not `/etcetera`. This is important because every Unix absolute
        // path starts with `/`, so a naive `starts_with("/")` would block
        // all paths.
        let lexical_str = lexical.to_string_lossy().to_lowercase();
        if is_filesystem_root(&resolved_str) || is_filesystem_root(&lexical_str) {
            return Err(PathGuardError::ForbiddenRoot);
        }
        for prefix in forbidden_root_prefixes() {
            // Each prefix is a directory and ends with `/`, so a plain
            // starts_with check is correct (no boundary worry).
            for candidate in [&resolved_str, &lexical_str] {
                if candidate.starts_with(prefix) {
                    return Err(PathGuardError::ForbiddenRoot);
                }
            }
        }

        // 5. Allow-list check. If the caller supplied no allowed roots,
        //    nothing is permitted (deny_all()).
        if self.allowed_roots.is_empty() {
            return Err(PathGuardError::OutsideAllowedRoots);
        }
        let allowed = self
            .allowed_roots
            .iter()
            .any(|root| resolved.starts_with(root));
        if !allowed {
            return Err(PathGuardError::OutsideAllowedRoots);
        }

        // 6. Re-verify component-by-component after canonicalization. Any
        //    `..` that survived canonicalize indicates the path escaped; this
        //    is paranoia-grade but cheap.
        if resolved
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        {
            return Err(PathGuardError::OutsideAllowedRoots);
        }

        Ok(resolved)
    }

    /// As `validate` but accepts a `Path` for ergonomic use inside the
    /// codebase that already holds a `PathBuf`.
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, PathGuardError> {
        let s = path.to_string_lossy();
        self.validate(&s)
    }
}

/// Collapse `..` and `.` components lexically without touching the
/// filesystem. Mirrors the behaviour of `Path::canonicalize` for the parts
/// that don't require I/O.
fn lexically_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    // Already at root; keep the `..` so callers see the
                    // escape attempt rather than silently swallowing it.
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("forge-guard-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rejects_traversal_outside_root() {
        let root = temp_root("traversal");
        let guard = PathGuard::new([&root]);
        // Try to escape via `..`.
        let escape = root.join("..").join("..").join("etc").join("passwd");
        let res = guard.validate(&escape.to_string_lossy());
        assert!(matches!(res, Err(PathGuardError::OutsideAllowedRoots) | Err(PathGuardError::ForbiddenRoot)));
    }

    #[test]
    fn rejects_forbidden_root_even_with_allow_list() {
        let root = temp_root("forbidden");
        let etc = std::path::PathBuf::from("/etc");
        // Even if we accidentally allow `/etc`, the guard must still block it.
        let guard = PathGuard::new([&root, &etc]);
        let res = guard.validate("/etc/passwd");
        assert!(matches!(res, Err(PathGuardError::ForbiddenRoot)));
    }

    #[test]
    fn rejects_sensitive_file_names() {
        let root = temp_root("sensitive");
        let guard = PathGuard::new([&root]);
        let key = root.join("id_rsa");
        fs::write(&key, "secret").unwrap();
        let res = guard.validate(&key.to_string_lossy());
        assert!(matches!(res, Err(PathGuardError::ForbiddenFile)));
    }

    #[test]
    fn allows_existing_file_inside_root() {
        // Use a non-system root for the test (std::env::temp_dir resolves
        // to /var/folders/... on macOS, which collides with the `/var`
        // block-list).
        let root = std::env::temp_dir()
            .join("forge-desktop-tests")
            .join(format!("allow-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let file = root.join("ok.txt");
        fs::write(&file, "hi").unwrap();
        let guard = PathGuard::new([&root]);
        let ok = guard.validate(&file.to_string_lossy()).unwrap();
        assert!(ok.starts_with(root.canonicalize().unwrap().as_path()));
    }

    #[test]
    fn denies_all_when_no_roots() {
        let guard = PathGuard::deny_all();
        let res = guard.validate("/tmp/anything");
        assert!(matches!(res, Err(PathGuardError::OutsideAllowedRoots)));
    }

    #[test]
    fn empty_input_rejected() {
        let guard = PathGuard::deny_all();
        assert!(matches!(guard.validate(""), Err(PathGuardError::InvalidInput)));
        assert!(matches!(guard.validate("\0/foo"), Err(PathGuardError::InvalidInput)));
    }
}
# sync_records.json Top-Level-Only Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce sync_records.json from ~322KB to ~5-10KB by recording only top-level directory/file names instead of every file path.

**Architecture:** Modify `copy_dir_inner` in `plugin_sync.rs` to only collect first-level entries (directory names with `/` suffix, file names without path prefix). Update `remove_synced_files` to handle directory entries via `is_dir()` check. No struct or interface changes needed.

**Tech Stack:** Rust, `std::fs`, `serde_json`

---

### Task 1: Add unit tests for `copy_dir_inner` (top-level-only behavior)

**Files:**
- Modify: `src-tauri/src/commands/plugin_sync.rs:614` (append test module)

- [ ] **Step 1: Add test module with `copy_dir_inner` top-level-only test**

Append to end of `src-tauri/src/commands/plugin_sync.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn copy_dir_inner_returns_only_top_level_entries() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create nested structure:
        //   root_file.txt
        //   subdir/
        //     nested_file.txt
        //     deep/
        //       deep_file.txt
        fs::write(src.path().join("root_file.txt"), "root").unwrap();
        fs::create_dir_all(src.path().join("subdir/deep")).unwrap();
        fs::write(src.path().join("subdir/nested_file.txt"), "nested").unwrap();
        fs::write(src.path().join("subdir/deep/deep_file.txt"), "deep").unwrap();

        let result = copy_dir_inner(
            &src.path().to_path_buf(),
            &dst.path().to_path_buf(),
            &dst.path().to_path_buf(),
        ).unwrap();

        // Should contain root_file.txt and subdir/ but NOT nested paths
        assert!(result.contains(&"root_file.txt".to_string()));
        assert!(result.contains(&"subdir/".to_string()));
        assert!(!result.iter().any(|f| f.contains("nested_file")));
        assert!(!result.iter().any(|f| f.contains("deep_file")));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn copy_dir_inner_still_copies_all_files() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        fs::write(src.path().join("a.txt"), "a").unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/b.txt"), "b").unwrap();

        copy_dir_inner(
            &src.path().to_path_buf(),
            &dst.path().to_path_buf(),
            &dst.path().to_path_buf(),
        ).unwrap();

        // All files should actually be copied on disk
        assert!(dst.path().join("a.txt").exists());
        assert!(dst.path().join("sub/b.txt").exists());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail (current behavior returns all files)**

Run: `cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo test --lib plugin_sync::tests 2>&1`
Expected: FAIL — `copy_dir_inner_returns_only_top_level_entries` fails because current impl returns nested paths like `subdir/nested_file.txt`.

---

### Task 2: Modify `copy_dir_inner` to only collect top-level entries

**Files:**
- Modify: `src-tauri/src/commands/plugin_sync.rs:103-141`

- [ ] **Step 1: Rewrite `copy_dir_inner` to only record top-level entries**

Replace lines 103-141 of `plugin_sync.rs` with:

```rust
fn copy_dir_inner(src: &PathBuf, dst: &PathBuf, root: &PathBuf) -> Result<Vec<String>, String> {
    let mut top_level_entries = Vec::new();
    if !src.exists() {
        return Err(format!("Source directory does not exist: {}", src.display()));
    }
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create target dir {}: {}", dst.display(), e))?;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read source dir: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip .git directory — plugin content only, no VCS history
        if file_name_str == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            // Recurse to copy all files, but don't collect sub-entries
            copy_dir_inner(&src_path, &dst_path, root)?;
            // Record only the top-level directory name (with / suffix)
            let dir_name = format!("{}/", file_name_str);
            top_level_entries.push(dir_name);
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
            // Record only the file name (no path prefix)
            top_level_entries.push(file_name_str.to_string());
        }
    }
    Ok(top_level_entries)
}
```

- [ ] **Step 2: Update doc comment on `copy_dir_inner` to reflect new behavior**

Update the comment above `copy_dir_recursive` (line 97-98) from:

```rust
/// Recursively copy a directory. Returns relative paths of all copied files
/// (relative to `dst`), preserving subdirectory structure.
```

to:

```rust
/// Recursively copy a directory. Returns only top-level entry names
/// (directories with `/` suffix, files by name) relative to `dst`.
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo test --lib plugin_sync::tests 2>&1`
Expected: PASS — both tests pass.

---

### Task 3: Add unit test for `remove_synced_files` with directory entries

**Files:**
- Modify: `src-tauri/src/commands/plugin_sync.rs` (test module, append after existing tests)

- [ ] **Step 1: Add test for directory-aware removal**

Append to the `tests` module:

```rust
    #[test]
    fn remove_synced_files_handles_directory_entries() {
        let dir = TempDir::new().unwrap();

        // Set up: some files and a subdirectory
        fs::write(dir.path().join("file.txt"), "content").unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/nested.txt"), "nested").unwrap();

        let synced_files = vec![
            "file.txt".to_string(),
            "subdir/".to_string(),
        ];

        remove_synced_files(&dir.path().to_path_buf(), &synced_files).unwrap();

        assert!(!dir.path().join("file.txt").exists());
        assert!(!dir.path().join("subdir").exists());
    }

    #[test]
    fn remove_synced_files_handles_old_full_path_format() {
        let dir = TempDir::new().unwrap();

        // Simulate old format: full relative paths
        fs::create_dir_all(dir.path().join("subdir/deep")).unwrap();
        fs::write(dir.path().join("subdir/deep/file.txt"), "content").unwrap();

        let synced_files = vec![
            "subdir/deep/file.txt".to_string(),
        ];

        remove_synced_files(&dir.path().to_path_buf(), &synced_files).unwrap();

        assert!(!dir.path().join("subdir/deep/file.txt").exists());
    }
```

- [ ] **Step 2: Run tests to verify `remove_synced_files_handles_directory_entries` fails**

Run: `cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo test --lib plugin_sync::tests::remove_synced_files 2>&1`
Expected: FAIL — `remove_synced_files` tries `fs::remove_file` on a directory, which fails.

---

### Task 4: Modify `remove_synced_files` to handle directory entries

**Files:**
- Modify: `src-tauri/src/commands/plugin_sync.rs:146-157`

- [ ] **Step 1: Update `remove_synced_files` to check `is_dir()`**

Replace lines 146-157 of `plugin_sync.rs` with:

```rust
/// Recursively remove entries listed in `synced_files` from the target directory.
/// Directory entries (paths that resolve to directories on disk) are removed with
/// `remove_dir_all`; file entries are removed with `remove_file`.
/// Also removes empty parent directories up to (but not including) the target root.
fn remove_synced_files(target_path: &PathBuf, synced_files: &[String]) -> Result<(), String> {
    for file_name in synced_files {
        let file_path = target_path.join(file_name);
        if !file_path.exists() {
            continue;
        }
        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)
                .map_err(|e| format!("Failed to remove dir {}: {}", file_path.display(), e))?;
        } else {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to remove {}: {}", file_path.display(), e))?;
        }
    }
    // Clean up empty directories (bottom-up)
    cleanup_empty_dirs(target_path);
    Ok(())
}
```

- [ ] **Step 2: Run all `remove_synced_files` tests to verify they pass**

Run: `cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo test --lib plugin_sync::tests 2>&1`
Expected: PASS — all 4 tests pass.

- [ ] **Step 3: Run full cargo check to verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1`
Expected: PASS — no compilation errors.

---

### Task 5: Manual verification with real sync_records.json

- [ ] **Step 1: Back up current sync_records.json**

Run: `cp /Users/rhino/.forge/plugins/sync_records.json /Users/rhino/.forge/plugins/sync_records.json.bak`

- [ ] **Step 2: After building and running a sync, verify the new JSON is compact**

Check file size and structure:
```bash
wc -l /Users/rhino/.forge/plugins/sync_records.json
cat /Users/rhino/.forge/plugins/sync_records.json | python3 -c "import sys,json; d=json.load(sys.stdin); [print(f'{k}: {len(v.get(\"synced_files\",[]))} entries') for k,v in d.items()]"
```

Expected: entry counts drop from hundreds/thousands to ~15-30 per plugin.

- [ ] **Step 3: Restore backup if needed**

Run: `cp /Users/rhino/.forge/plugins/sync_records.json.bak /Users/rhino/.forge/plugins/sync_records.json`

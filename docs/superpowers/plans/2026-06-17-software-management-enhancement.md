# Software Management Module Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enhance the Software Management module with version management UI feedback, platform filtering, official website links, style unification, and MCP naming changes.

**Architecture:** Incremental enhancement — extend existing Rust models with `website_url` and `platform` fields, add platform filter state to the Pinia store, enhance the Vue view with website links and loading states, and apply consistent styling via shared CSS.

**Tech Stack:** Tauri 2 (Rust backend), Vue 3 + TypeScript (frontend), Pinia (state), SQLite (database)

---

## File Map

### Rust Backend (create/modify)
- `src-tauri/src/models/mod.rs` — add `website_url`, `platform` to `Software` struct
- `src-tauri/src/db/schema.rs` — add `website_url`, `platform` columns to `software` table
- `src-tauri/src/db/connection.rs` — update SQL queries to include new columns
- `src-tauri/src/services/software_scanner.rs` — add `website_url`, `platform` to `SoftwareConfig` and `DetectedSoftware`, increase timeout
- `src-tauri/src/commands/software.rs` — add `update_software` command

### Frontend (modify)
- `src/types/software.ts` — add `websiteUrl`, `platform` fields
- `src/stores/software.ts` — add `selectedPlatform` state, `updateSoftware` action
- `src/views/SoftwareManagementView.vue` — platform filter, website link, loading states
- `src/assets/main.css` — shared `.search-box` style
- `src/App.vue` — MCP naming (line 43)
- `src/components/layout/Sidebar.vue` — MCP naming (line 126)
- `src/views/MCPView.vue` — MCP naming (line 6)
- `src/components/mcp/MCPExportDialog.vue` — MCP naming (line 6)
- `src/components/mcp/MCPImportDialog.vue` — MCP naming (line 6)
- `src/components/plugins/PluginDetailsDialog.vue` — MCP naming (line 452)

---

## Task 1: Rust Model — Add `website_url` and `platform` to Software

**Files:**
- Modify: `src-tauri/src/models/mod.rs:43-59`

- [ ] **Step 1: Add fields to Software struct**

In `src-tauri/src/models/mod.rs`, add two new fields to the `Software` struct after `status`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: String,
    pub name: String,
    pub key: String,
    pub version: Option<String>,
    pub install_path: Option<String>,
    pub config_path: String,
    pub is_installed: bool,
    pub last_checked: Option<String>,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub is_upgradable: bool,
    #[serde(default)]
    pub status: SoftwareStatus,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: Compilation errors in files that construct `Software` without the new fields.

- [ ] **Step 3: Fix DetectedSoftware::from impl**

In `src-tauri/src/services/software_scanner.rs`, update the `From<DetectedSoftware> for Software` impl (around line 38-58) to include the new fields:

```rust
impl From<DetectedSoftware> for Software {
    fn from(detected: DetectedSoftware) -> Self {
        let status = SoftwareScanner::new().compute_status(
            detected.is_installed,
            detected.version.as_deref(),
            None,
        );
        Software {
            id: uuid_from_key(&detected.key),
            name: detected.name,
            key: detected.key,
            version: detected.version,
            install_path: None,
            config_path: detected.config_path,
            is_installed: detected.is_installed,
            last_checked: Some(chrono_lite_now()),
            latest_version: None,
            is_upgradable: false,
            status,
            website_url: detected.website_url,
            platform: Some(detected.platform),
        }
    }
}
```

- [ ] **Step 4: Add fields to DetectedSoftware struct**

In `src-tauri/src/services/software_scanner.rs`, add `website_url` to `DetectedSoftware`:

```rust
#[derive(Debug, Clone)]
pub struct DetectedSoftware {
    pub key: String,
    pub name: String,
    pub tier: u8,
    pub platform: String,
    pub version: Option<String>,
    pub config_path: String,
    pub is_installed: bool,
    pub website_url: Option<String>,
}
```

- [ ] **Step 5: Add fields to SoftwareConfig and populate**

In `src-tauri/src/services/software_scanner.rs`, add `website_url` to `SoftwareConfig`:

```rust
#[derive(Debug, Clone)]
pub struct SoftwareConfig {
    pub key: String,
    pub name: String,
    pub tier: u8,
    pub platform: String,
    pub default_config_paths: Vec<PathBuf>,
    pub website_url: Option<String>,
}
```

Then update every `SoftwareConfig` entry in `get_supported_software()` to include `website_url`. Example for the first few:

```rust
SoftwareConfig {
    key: "homebrew".to_string(),
    name: "Homebrew".to_string(),
    tier: 1,
    platform: "macOS".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://brew.sh".to_string()),
},
SoftwareConfig {
    key: "chocolatey".to_string(),
    name: "Chocolatey".to_string(),
    tier: 1,
    platform: "Windows".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://chocolatey.org".to_string()),
},
SoftwareConfig {
    key: "scoop".to_string(),
    name: "Scoop".to_string(),
    tier: 1,
    platform: "Windows".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://scoop.sh".to_string()),
},
SoftwareConfig {
    key: "git".to_string(),
    name: "Git".to_string(),
    tier: 1,
    platform: "Cross-platform".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://git-scm.com".to_string()),
},
SoftwareConfig {
    key: "ssh".to_string(),
    name: "SSH Config".to_string(),
    tier: 1,
    platform: "Cross-platform".to_string(),
    default_config_paths: vec![...],
    website_url: None,
},
SoftwareConfig {
    key: "windows-terminal".to_string(),
    name: "Windows Terminal".to_string(),
    tier: 1,
    platform: "Windows".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://github.com/microsoft/terminal".to_string()),
},
SoftwareConfig {
    key: "iterm2".to_string(),
    name: "iTerm2".to_string(),
    tier: 1,
    platform: "macOS".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://iterm2.com".to_string()),
},
SoftwareConfig {
    key: "oh-my-posh".to_string(),
    name: "Oh My Posh".to_string(),
    tier: 1,
    platform: "Cross-platform".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://ohmyposh.dev".to_string()),
},
SoftwareConfig {
    key: "vscode".to_string(),
    name: "VS Code".to_string(),
    tier: 1,
    platform: "Cross-platform".to_string(),
    default_config_paths: vec![...],
    website_url: Some("https://code.visualstudio.com".to_string()),
},
```

Continue for all Tier 2-5 entries:
- `nvm` → `https://github.com/nvm-sh/nvm`
- `pyenv` → `https://github.com/pyenv/pyenv`
- `goenv` → `https://github.com/syndbg/goenv`
- `jenv` → `https://github.com/jenv/jenv`
- `asdf` → `https://asdf-vm.com`
- `docker` → `https://www.docker.com`
- `docker-compose` → `https://docs.docker.com/compose`
- `ffmpeg` → `https://ffmpeg.org`
- `apifox` → `https://apifox.com`
- `postman` → `https://www.postman.com`
- `charles` → `https://www.charlesproxy.com`
- `cyberduck` → `https://cyberduck.io`
- `filezilla` → `https://filezilla-project.org`
- `7zip` → `https://www.7-zip.org`
- `everything` → `https://www.voidtools.com`
- `powertoys` → `https://learn.microsoft.com/en-us/windows/powertoys`
- `snipaste` → `https://www.snipaste.com`
- `obsidian` → `https://obsidian.md`
- `excalidraw` → `https://excalidraw.com`
- `drawio` → `https://www.drawio.com`

- [ ] **Step 6: Propagate website_url through detection**

In `src-tauri/src/services/software_scanner.rs`, find where `DetectedSoftware` is constructed in the detection methods and ensure `website_url` is passed from `SoftwareConfig`. Look for the pattern where config fields are copied to detected and add:

```rust
website_url: config.website_url.clone(),
```

- [ ] **Step 6b: Increase version detection timeout**

In `src-tauri/src/services/software_scanner.rs`, find the timeout constant (likely `Duration::from_millis(300)`) and change it to:

```rust
Duration::from_millis(1000)
```

This gives version detection commands more time to complete, reducing false "unknown" statuses.

- [ ] **Step 7: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: Compilation succeeds.

- [ ] **Step 8: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src-tauri/src/models/mod.rs src-tauri/src/services/software_scanner.rs
git commit -m "feat(backend): add website_url and platform to Software model"
```

---

## Task 2: Database Schema — Add Columns and Migrate

**Files:**
- Modify: `src-tauri/src/db/schema.rs:7-16`
- Modify: `src-tauri/src/db/connection.rs:235-308`

- [ ] **Step 1: Add columns to schema**

In `src-tauri/src/db/schema.rs`, update the `software` table creation:

```sql
CREATE TABLE IF NOT EXISTS software (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    key TEXT UNIQUE NOT NULL,
    version TEXT,
    install_path TEXT,
    config_path TEXT NOT NULL,
    is_installed INTEGER DEFAULT 0,
    last_checked TEXT,
    website_url TEXT,
    platform TEXT
);
```

- [ ] **Step 2: Add migration for existing databases**

After the `CREATE TABLE` block in `schema.rs`, add an `ALTER TABLE` migration that runs safely (SQLite ignores ALTER if column exists via try/catch pattern, or use `PRAGMA table_info` check):

```rust
// Migration: add website_url and platform columns if they don't exist
let _ = conn.execute("ALTER TABLE software ADD COLUMN website_url TEXT", []);
let _ = conn.execute("ALTER TABLE software ADD COLUMN platform TEXT", []);
```

The `let _ =` pattern is safe because SQLite returns an error if the column already exists, and we silently ignore it.

- [ ] **Step 3: Update get_all_software query**

In `src-tauri/src/db/connection.rs`, update `get_all_software` (line 235) to select the new columns:

```rust
pub fn get_all_software(&self) -> Result<Vec<Software>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform FROM software"
    )?;

    let software_iter = stmt.query_map([], |row| {
        Ok(Software {
            id: row.get(0)?,
            name: row.get(1)?,
            key: row.get(2)?,
            version: row.get(3)?,
            install_path: row.get(4)?,
            config_path: row.get(5)?,
            is_installed: row.get::<_, i32>(6)? != 0,
            last_checked: row.get(7)?,
            latest_version: None,
            is_upgradable: false,
            status: crate::models::SoftwareStatus::default(),
            website_url: row.get(8)?,
            platform: row.get(9)?,
        })
    })?;

    let mut results = Vec::new();
    for software in software_iter {
        results.push(software?);
    }
    Ok(results)
}
```

- [ ] **Step 4: Update get_software_by_key query**

In `src-tauri/src/db/connection.rs`, update `get_software_by_key` (line 264) similarly:

```rust
pub fn get_software_by_key(&self, key: &str) -> Result<Option<Software>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform FROM software WHERE key = ?"
    )?;

    let mut rows = stmt.query(params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Software {
            id: row.get(0)?,
            name: row.get(1)?,
            key: row.get(2)?,
            version: row.get(3)?,
            install_path: row.get(4)?,
            config_path: row.get(5)?,
            is_installed: row.get::<_, i32>(6)? != 0,
            last_checked: row.get(7)?,
            latest_version: None,
            is_upgradable: false,
            status: crate::models::SoftwareStatus::default(),
            website_url: row.get(8)?,
            platform: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}
```

- [ ] **Step 5: Update upsert_software**

In `src-tauri/src/db/connection.rs`, update `upsert_software` (line 291):

```rust
pub fn upsert_software(&self, software: &Software) -> Result<()> {
    let conn = self.conn.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO software (id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            software.id,
            software.name,
            software.key,
            software.version,
            software.install_path,
            software.config_path,
            if software.is_installed { 1 } else { 0 },
            software.last_checked,
            software.website_url,
            software.platform,
        ],
    )?;
    Ok(())
}
```

- [ ] **Step 6: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: Compilation succeeds.

- [ ] **Step 7: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src-tauri/src/db/schema.rs src-tauri/src/db/connection.rs
git commit -m "feat(db): add website_url and platform columns to software table"
```

---

## Task 3: Frontend Types and Store — Add Fields and Platform State

**Files:**
- Modify: `src/types/software.ts:3-15`
- Modify: `src/stores/software.ts:62-242`

- [ ] **Step 1: Update Software type**

In `src/types/software.ts`, add `websiteUrl` and `platform` to the `Software` interface:

```typescript
export interface Software {
  id: string;
  name: string;
  key: string;
  version: string | null;
  installPath: string | null;
  configPath: string;
  isInstalled: boolean;
  lastChecked: string | null;
  latestVersion?: string | null;
  isUpgradable?: boolean;
  status?: SoftwareStatus;
  websiteUrl?: string | null;
  platform?: string;
}
```

- [ ] **Step 2: Add platform filter state to store**

In `src/stores/software.ts`, add after the existing `ref` declarations (around line 67):

```typescript
// Platform filter state
const selectedPlatform = ref<string>(
  localStorage.getItem('forge-selected-platform') || 'auto'
);

function setSelectedPlatform(platform: string) {
  selectedPlatform.value = platform;
  localStorage.setItem('forge-selected-platform', platform);
}
```

- [ ] **Step 3: Add updateSoftware action to store**

In `src/stores/software.ts`, add a new interface and function before the `return` statement:

```typescript
export interface UpdateCheckResult {
  success: boolean;
  hasUpdate: boolean;
  message: string;
  newVersion: string | null;
}

async function updateSoftware(softwareKey: string): Promise<UpdateCheckResult> {
  try {
    isInstalling.value = true;
    error.value = null;
    const result = await invoke<UpdateCheckResult>('update_software', { softwareKey });
    if (result.success) {
      runWhenIdle(() => detectSoftware());
    }
    return result;
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to update software';
    throw e;
  } finally {
    isInstalling.value = false;
  }
}
```

- [ ] **Step 4: Export new state and actions**

In `src/stores/software.ts`, add to the `return` statement:

```typescript
return {
  // ... existing exports
  selectedPlatform,
  setSelectedPlatform,
  updateSoftware,
};
```

- [ ] **Step 5: Verify TypeScript compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: May show errors in SoftwareManagementView.vue (expected, will fix in later tasks).

- [ ] **Step 6: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/types/software.ts src/stores/software.ts
git commit -m "feat(store): add websiteUrl, platform fields and platform filter state"
```

---

## Task 4: View — Platform Filter UI

**Files:**
- Modify: `src/views/SoftwareManagementView.vue:33-61`

- [ ] **Step 1: Add platform filter state and options**

In `src/views/SoftwareManagementView.vue`, add after the `statusFilters` declaration (around line 221):

```typescript
// Platform filter
const platformOptions = [
  { value: 'all', label: 'All' },
  { value: 'macOS', label: 'macOS' },
  { value: 'Windows', label: 'Windows' },
  { value: 'Linux', label: 'Linux' },
];

// Detect current platform on mount
const detectedPlatform = ref('all');
const detectPlatform = () => {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes('mac')) detectedPlatform.value = 'macOS';
  else if (ua.includes('win')) detectedPlatform.value = 'Windows';
  else if (ua.includes('linux')) detectedPlatform.value = 'Linux';
};

const selectedPlatform = computed({
  get: () => softwareStore.selectedPlatform === 'auto' ? detectedPlatform.value : softwareStore.selectedPlatform,
  set: (val: string) => softwareStore.setSelectedPlatform(val),
});
```

- [ ] **Step 2: Call detectPlatform on mount**

In the `onMounted` callback (around line 322), add `detectPlatform()` before the software detection:

```typescript
onMounted(async () => {
  detectPlatform();
  try {
    isLoading.value = true;
    await softwareStore.detectSoftware();
    updateSoftwareFromStore();
  } catch (e) {
    console.error('Failed to detect software:', e);
  } finally {
    isLoading.value = false;
  }
});
```

- [ ] **Step 3: Update getFilteredSoftware to apply platform filter**

In `src/views/SoftwareManagementView.vue`, update `getFilteredSoftware` (around line 394):

```typescript
const getFilteredSoftware = (tierId: number) => {
  const tierSoftware = softwareByTier.value.get(tierId) || [];
  let result = tierSoftware;

  // Platform filter
  if (selectedPlatform.value !== 'all') {
    result = result.filter(s =>
      s.platform === selectedPlatform.value || s.platform === 'Cross-platform'
    );
  }

  // Status filter
  if (statusFilter.value !== 'all') {
    const wanted = statusFilter.value;
    result = result.filter(s => getStatus(s) === wanted);
  }

  // Search filter
  if (debouncedSearch.value) {
    const query = debouncedSearch.value.toLowerCase();
    result = result.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.platform.toLowerCase().includes(query)
    );
  }

  return result;
};
```

- [ ] **Step 4: Add platform filter UI to template**

In `src/views/SoftwareManagementView.vue`, wrap the existing `status-filter` div in a `filter-bar` and add the platform filter. Replace lines 49-61:

```html
<!-- Filter Bar -->
<div class="filter-bar">
  <div class="status-filter">
    <span class="filter-label">Filter:</span>
    <button
      v-for="f in statusFilters"
      :key="f.value"
      class="filter-pill"
      :class="{ active: statusFilter === f.value }"
      @click="statusFilter = f.value"
    >
      {{ f.label }}
    </button>
  </div>
  <div class="platform-filter">
    <button
      v-for="p in platformOptions"
      :key="p.value"
      class="filter-pill"
      :class="{ active: selectedPlatform === p.value }"
      @click="selectedPlatform = p.value"
    >
      {{ p.label }}
    </button>
  </div>
</div>
```

- [ ] **Step 5: Add filter-bar CSS**

In the `<style scoped>` section of `SoftwareManagementView.vue`, add:

```css
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
}

.platform-filter {
  display: flex;
  align-items: center;
  gap: 4px;
}
```

- [ ] **Step 6: Verify dev server**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -5`
Expected: Build succeeds.

- [ ] **Step 7: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/views/SoftwareManagementView.vue
git commit -m "feat(software): add platform filter UI with auto-detection"
```

---

## Task 5: View — Website Link Display

**Files:**
- Modify: `src/views/SoftwareManagementView.vue:110-126` (template), `src/views/SoftwareManagementView.vue:~400` (script)

- [ ] **Step 1: Add copyUrl and openUrl functions**

In `src/views/SoftwareManagementView.vue`, add after the existing handler functions (around line 523):

```typescript
const copyUrl = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url);
    showNotification?.('Link copied!', 'success');
  } catch {
    showNotification?.('Failed to copy link', 'error');
  }
};

const openUrl = async (url: string) => {
  try {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open(url);
  } catch {
    // Fallback: open in current window
    window.open(url, '_blank');
  }
};
```

- [ ] **Step 2: Add website link to software card template**

In `src/views/SoftwareManagementView.vue`, add a new `meta-item` after the config/install path meta-item (after line 122, before the `meta-status` div):

```html
<div class="meta-item" v-if="sw.websiteUrl">
  <span class="meta-label">Website</span>
  <span class="meta-value website-link" :title="sw.websiteUrl">
    {{ sw.websiteUrl }}
  </span>
  <button class="btn-icon-tiny" @click="copyUrl(sw.websiteUrl)" title="Copy URL">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
    </svg>
  </button>
  <button class="btn-icon-tiny" @click="openUrl(sw.websiteUrl)" title="Open in browser">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
      <polyline points="15 3 21 3 21 9"/>
      <line x1="10" y1="14" x2="21" y2="3"/>
    </svg>
  </button>
</div>
```

- [ ] **Step 3: Add CSS for website link and tiny button**

In the `<style scoped>` section of `SoftwareManagementView.vue`:

```css
.website-link {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-muted);
}

.meta-value {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg);
}

.btn-icon-tiny {
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  background: transparent;
  border: none;
  color: var(--fg-ghost);
  cursor: pointer;
  padding: 0;
  transition: all 0.15s ease;
}

.btn-icon-tiny:hover {
  color: var(--accent);
  background: var(--bg-input);
}
```

- [ ] **Step 4: Update mock data to include websiteUrl and platform**

In `src/views/SoftwareManagementView.vue`, update the hardcoded `software` ref (lines 277-320) to include `websiteUrl` and `platform` fields for each entry. Example for the first entry:

```typescript
{ key: 'homebrew', name: 'Homebrew', version: '4.3.0', configPath: '~/.brew/', installed: true, lastChecked: '2 min ago', platform: 'macOS', tier: 1, websiteUrl: 'https://brew.sh' },
```

Add `websiteUrl` to all entries that have known websites. The `platform` field already exists in the mock data.

- [ ] **Step 5: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/views/SoftwareManagementView.vue
git commit -m "feat(software): add website link display with copy and open actions"
```

---

## Task 6: View — Version Management UI Feedback (Loading States)

**Files:**
- Modify: `src/views/SoftwareManagementView.vue:128-164` (template), script section

- [ ] **Step 1: Add loading state refs**

In `src/views/SoftwareManagementView.vue`, add after the existing state declarations (around line 220):

```typescript
const installingKeys = ref(new Set<string>());
const updatingKeys = ref(new Set<string>());
const uninstallingKeys = ref(new Set<string>());
```

- [ ] **Step 2: Update Install button with loading state**

Replace the Install button (around lines 157-163) in the template:

```html
<button
  v-else-if="!sw.installed"
  class="btn btn-primary btn-sm"
  :disabled="installingKeys.has(sw.key)"
  @click="handleInstall(sw)"
>
  <span v-if="installingKeys.has(sw.key)" class="btn-spinner"></span>
  {{ installingKeys.has(sw.key) ? 'Installing...' : 'Install' }}
</button>
```

- [ ] **Step 3: Update Update button with loading state**

Replace the Update button (around lines 137-142):

```html
<button
  v-if="sw.installed"
  class="btn btn-ghost btn-sm"
  :disabled="updatingKeys.has(sw.key)"
  @click="handleUpdate(sw)"
>
  <span v-if="updatingKeys.has(sw.key)" class="btn-spinner"></span>
  {{ updatingKeys.has(sw.key) ? 'Checking...' : 'Update' }}
</button>
```

- [ ] **Step 4: Update Uninstall button with loading state**

Replace the Uninstall button (around lines 143-149):

```html
<button
  v-if="sw.installed"
  class="btn btn-ghost btn-sm text-danger"
  :disabled="uninstallingKeys.has(sw.key)"
  @click="handleUninstall(sw)"
>
  <span v-if="uninstallingKeys.has(sw.key)" class="btn-spinner"></span>
  {{ uninstallingKeys.has(sw.key) ? 'Removing...' : 'Uninstall' }}
</button>
```

- [ ] **Step 5: Update handleInstall to use loading state**

Replace the `handleInstall` function:

```typescript
const handleInstall = async (sw: Software) => {
  installingKeys.value.add(sw.key);
  try {
    const result = await softwareStore.installSoftware(sw.key);
    if (result.success) {
      updateSoftwareFromStore();
      showNotification?.(`${sw.name} installed successfully`, 'success');
    } else {
      showNotification?.(result.message, 'error');
    }
  } catch {
    showNotification?.(`Failed to install ${sw.name}`, 'error');
  } finally {
    installingKeys.value.delete(sw.key);
  }
};
```

- [ ] **Step 6: Update handleUpdate to check for updates**

Replace the `handleUpdate` function:

```typescript
const handleUpdate = async (sw: Software) => {
  updatingKeys.value.add(sw.key);
  try {
    const result = await softwareStore.updateSoftware(sw.key);
    if (result.hasUpdate) {
      showNotification?.(`Updated ${sw.name} to ${result.newVersion}`, 'success');
    } else {
      showNotification?.(`${sw.name} is already up to date`, 'info');
    }
    updateSoftwareFromStore();
  } catch {
    showNotification?.(`Failed to check update for ${sw.name}`, 'error');
  } finally {
    updatingKeys.value.delete(sw.key);
  }
};
```

- [ ] **Step 7: Update handleUninstall with confirmation**

Replace the `handleUninstall` function:

```typescript
const handleUninstall = async (sw: Software) => {
  if (!window.confirm(`Uninstall ${sw.name}?`)) return;
  uninstallingKeys.value.add(sw.key);
  try {
    const result = await softwareStore.uninstallSoftware(sw.key);
    if (result.success) {
      updateSoftwareFromStore();
      showNotification?.(`${sw.name} uninstalled successfully`, 'success');
    } else {
      showNotification?.(result.message, 'error');
    }
  } catch {
    showNotification?.(`Failed to uninstall ${sw.name}`, 'error');
  } finally {
    uninstallingKeys.value.delete(sw.key);
  }
};
```

- [ ] **Step 8: Add btn-spinner CSS**

In the `<style scoped>` section:

```css
.btn-spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  margin-right: 4px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  pointer-events: none;
}
```

- [ ] **Step 9: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/views/SoftwareManagementView.vue
git commit -m "feat(software): add loading states for install/update/uninstall actions"
```

---

## Task 7: Backend — Add update_software Command

**Files:**
- Modify: `src-tauri/src/commands/software.rs`
- Modify: `src-tauri/src/main.rs` (or wherever commands are registered)

- [ ] **Step 1: Add UpdateCheckResult struct**

In `src-tauri/src/commands/software.rs`, add at the top:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub success: bool,
    pub has_update: bool,
    pub message: String,
    pub new_version: Option<String>,
}
```

- [ ] **Step 2: Add update_software command**

In `src-tauri/src/commands/software.rs`, add:

```rust
#[tauri::command]
pub async fn update_software(software_key: String) -> Result<UpdateCheckResult, String> {
    log::info!("Checking update for software: {}", software_key);
    let scanner = SoftwareScanner::new();

    // Detect current version
    let detected = scanner.detect_software_parallel();
    let current = detected.iter().find(|s| s.key == software_key);

    match current {
        Some(sw) if sw.is_installed => {
            // For now, reinstall to update (same as install which upgrades)
            let installer = SoftwareInstaller::new();
            let result = installer.install(&software_key).map_err(|e| e.to_string())?;
            Ok(UpdateCheckResult {
                success: result.success,
                has_update: result.success,
                message: result.message,
                new_version: result.installed_version,
            })
        }
        Some(_) => Ok(UpdateCheckResult {
            success: true,
            has_update: false,
            message: "Software is not installed".to_string(),
            new_version: None,
        }),
        None => Err(format!("Unknown software key: {}", software_key)),
    }
}
```

- [ ] **Step 3: Register the command**

Find where Tauri commands are registered (likely in `src-tauri/src/main.rs` or `src-tauri/src/lib.rs`) and add `update_software` to the invoke handler list.

- [ ] **Step 4: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: Compilation succeeds.

- [ ] **Step 5: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src-tauri/src/commands/software.rs
git commit -m "feat(backend): add update_software Tauri command"
```

---

## Task 8: Style Unification — Shared Search Box

**Files:**
- Modify: `src/assets/main.css`
- Modify: `src/views/SoftwareManagementView.vue:539-556`

- [ ] **Step 1: Add shared search-box style to main.css**

In `src/assets/main.css`, add after the existing styles:

```css
/* Shared Search Box */
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-input);
  transition: border-color var(--transition-fast, 150ms ease), box-shadow var(--transition-fast, 150ms ease);
}

.search-box:focus-within {
  border-color: var(--accent);
  box-shadow: var(--focus-ring, 0 0 0 2px rgba(217, 119, 6, 0.12));
}

.search-box input {
  border: none;
  background: transparent;
  font-size: 13px;
  outline: none;
  width: 200px;
  color: var(--fg);
}

.search-box input::placeholder {
  color: var(--fg-muted);
}
```

- [ ] **Step 2: Remove scoped search-box styles from SoftwareManagementView**

In `src/views/SoftwareManagementView.vue`, remove the scoped `.search-box` and `.search-box input` styles (lines 539-556) since they're now in the shared stylesheet.

- [ ] **Step 3: Verify build**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -5`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/assets/main.css src/views/SoftwareManagementView.vue
git commit -m "style: unify search-box styling via shared CSS"
```

---

## Task 9: MCP Naming — Replace Display Text

**Files:**
- Modify: `src/App.vue:43`
- Modify: `src/components/layout/Sidebar.vue:126`
- Modify: `src/views/MCPView.vue:6`
- Modify: `src/components/mcp/MCPExportDialog.vue:6`
- Modify: `src/components/mcp/MCPImportDialog.vue:6`
- Modify: `src/components/plugins/PluginDetailsDialog.vue:452`

- [ ] **Step 1: Update App.vue page title**

In `src/App.vue`, line 43:
```
'/mcp': 'MCP Servers',
```
→
```
'/mcp': 'Mcps',
```

- [ ] **Step 2: Update Sidebar nav label**

In `src/components/layout/Sidebar.vue`, line 126:
```
          MCP Servers
```
→
```
          Mcps
```

- [ ] **Step 3: Update MCPView heading**

In `src/views/MCPView.vue`, line 6:
```
        <h2>MCP Servers</h2>
```
→
```
        <h2>Mcps</h2>
```

- [ ] **Step 4: Update MCPExportDialog heading**

In `src/components/mcp/MCPExportDialog.vue`, line 6:
```
        <h3 id="export-dialog-title">Export MCP Servers</h3>
```
→
```
        <h3 id="export-dialog-title">Export Mcps</h3>
```

- [ ] **Step 5: Update MCPImportDialog heading**

In `src/components/mcp/MCPImportDialog.vue`, line 6:
```
        <h3 id="import-dialog-title">Import MCP Servers</h3>
```
→
```
        <h3 id="import-dialog-title">Import Mcps</h3>
```

- [ ] **Step 6: Update PluginDetailsDialog text**

In `src/components/plugins/PluginDetailsDialog.vue`, line 452:
```
                <span>MCP Servers: {{ validationReport.capabilities.mcpServers }}</span>
```
→
```
                <span>Mcps: {{ validationReport.capabilities.mcpServers }}</span>
```

- [ ] **Step 7: Verify build**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -5`
Expected: Build succeeds.

- [ ] **Step 8: Commit**

```bash
cd /Users/rhino/Desktop/AI/env-manager
git add src/App.vue src/components/layout/Sidebar.vue src/views/MCPView.vue src/components/mcp/MCPExportDialog.vue src/components/mcp/MCPImportDialog.vue src/components/plugins/PluginDetailsDialog.vue
git commit -m "rename: MCP Servers → Mcps in UI display text"
```

---

## Task 10: Final Integration — Verify Full Build

- [ ] **Step 1: Run Rust compilation check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10`
Expected: No errors.

- [ ] **Step 2: Run frontend build**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vite build 2>&1 | tail -10`
Expected: Build succeeds with no errors.

- [ ] **Step 3: Run TypeScript check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | tail -20`
Expected: No type errors (or only pre-existing errors unrelated to our changes).

- [ ] **Step 4: Final commit if needed**

If any fixes were needed:
```bash
cd /Users/rhino/Desktop/AI/env-manager
git add -A
git commit -m "fix: integration fixes for software management enhancement"
```

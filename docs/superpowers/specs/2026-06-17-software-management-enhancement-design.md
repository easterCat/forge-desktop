# Software Management Module Enhancement Design

**Date:** 2026-06-17
**Status:** Draft
**Scope:** Feature enhancements to the Software Management module in Forge

---

## Overview

Enhance the Software Management module with five improvements: version management UI feedback, platform filtering, official website links, style unification, and MCP naming adjustment.

**Approach:** Incremental Enhancement (extend existing types/store, minimal disruption).

---

## 1. Data Model Changes

### Frontend Type Extension

**File:** `src/types/software.ts`

Add `websiteUrl` field to the `Software` interface:

```typescript
export interface Software {
  // ... existing fields unchanged
  websiteUrl?: string | null;  // Official website URL
}
```

### Rust Backend Extension

**File:** `src-tauri/src/services/software_scanner.rs`

Extend `SoftwareConfig`:
```rust
pub struct SoftwareConfig {
    // ... existing fields
    pub website_url: Option<String>,
}
```

Extend `DetectedSoftware` and the `Software` model (in `src-tauri/src/models/`):
```rust
pub struct DetectedSoftware {
    // ... existing fields
    pub website_url: Option<String>,
}
```

Add `website_url` to each software entry in `get_supported_software()`. Example values:
- `homebrew` → `"https://brew.sh"`
- `git` → `"https://git-scm.com"`
- `docker` → `"https://www.docker.com"`
- `vscode` → `"https://code.visualstudio.com"`
- `nvm` → `"https://github.com/nvm-sh/nvm"`
- (all 32+ entries)

### Platform Filter State

**File:** `src/stores/software.ts`

Add to `useSoftwareStore`:
```typescript
const selectedPlatform = ref<string>(
  localStorage.getItem('forge-selected-platform') || 'auto'
);
// 'auto' = detect current OS, 'all' = show everything, 'macOS'/'Windows'/'Linux' = filter
```

Persist selection to `localStorage` on change.

---

## 2. Platform Filter UI

### Layout

Add a platform filter control in the `SoftwareManagementView.vue`, next to the existing `status-filter`:

```
┌─────────────────────────────────────────────────────────────────┐
│ [Filter: All | Installed | Not Installed | Update Available]    │
│                                    [🌐 All ▾ | macOS | Windows] │
└─────────────────────────────────────────────────────────────────┘
```

Wrap both in a `filter-bar` container:
```html
<div class="filter-bar">
  <div class="status-filter">...</div>
  <div class="platform-filter">
    <button v-for="p in platformOptions" ...>{{ p.label }}</button>
  </div>
</div>
```

### CSS
```css
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.platform-filter {
  display: flex;
  gap: 4px;
}
```

### Filtering Logic

Update `getFilteredSoftware()` to apply platform filter:
```typescript
const filteredByPlatform = (sw: Software[]) => {
  if (selectedPlatform.value === 'all') return sw;
  const target = selectedPlatform.value === 'auto' ? detectedPlatform.value : selectedPlatform.value;
  return sw.filter(s => s.platform === target || s.platform === 'Cross-platform');
};
```

### Auto-Detection

On mount, detect the current OS using frontend `navigator.platform` or `navigator.userAgent` (no new Tauri command needed). Map to `'macOS'` | `'Windows'` | `'Linux'`. Default to `'all'` if detection fails.

---

## 3. Software Card Enhancements

### Website Link Display

Add a new `meta-item` row in the software card template:

```html
<div class="meta-item" v-if="sw.websiteUrl">
  <span class="meta-label">Website</span>
  <span class="meta-value website-link" :title="sw.websiteUrl">
    {{ sw.websiteUrl }}
  </span>
  <button class="btn-icon-tiny" @click="copyUrl(sw.websiteUrl)" title="Copy URL">
    <!-- clipboard icon -->
  </button>
  <button class="btn-icon-tiny" @click="openUrl(sw.websiteUrl)" title="Open in browser">
    <!-- external link icon -->
  </button>
</div>
```

### CSS for Website Link
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
}

.btn-icon-tiny:hover {
  color: var(--accent);
  background: var(--bg-input);
}
```

### Copy & Open Actions
```typescript
const copyUrl = async (url: string) => {
  await navigator.clipboard.writeText(url);
  showNotification?.('Link copied!', 'success');
};

const openUrl = async (url: string) => {
  // Use Tauri shell plugin to open in default browser
  const { open } = await import('@tauri-apps/plugin-shell');
  await open(url);
};
```

Note: `@tauri-apps/plugin-shell` must be configured in `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` permissions. If already configured, use the existing import pattern from the project.

### Version Management UI Feedback

**Install button:**
```html
<button
  v-if="!sw.installed"
  class="btn btn-primary btn-sm"
  :class="{ 'btn-loading': installingKeys.has(sw.key) }"
  :disabled="installingKeys.has(sw.key)"
  @click="handleInstall(sw)"
>
  <span v-if="installingKeys.has(sw.key)" class="btn-spinner"></span>
  {{ installingKeys.has(sw.key) ? 'Installing...' : 'Install' }}
</button>
```

**Uninstall button — add confirmation:**
```typescript
const handleUninstall = async (sw: Software) => {
  if (!confirm(`Uninstall ${sw.name}?`)) return;
  // ... existing uninstall logic with loading state
};
```

**Update button — check for updates:**
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
  } catch (e) {
    showNotification?.(`Update failed for ${sw.name}`, 'error');
  } finally {
    updatingKeys.value.delete(sw.key);
  }
};
```

### Version Detection Enhancement (Rust Backend)

**File:** `src-tauri/src/services/software_scanner.rs`

- Increase version detection timeout from 300ms to 1000ms
- Add `check_latest_version(key: &str)` method that queries each software's official source:
  - Initially implemented for key software only: `git`, `docker`, `nvm`, `node`, `python`
  - Others return `None` (no latest version check available)
  - Can be extended incrementally for more software later
  - Uses GitHub releases API, Docker Hub API, or official version endpoints
- Rate limiting: cache latest version results for 1 hour to avoid API abuse

**New Tauri command:** `update_software` that:
1. Calls `check_latest_version()` for the given key
2. If update available, runs the install command (which typically upgrades)
3. Returns `UpdateResult { success, message, new_version }`

---

## 4. Style Unification

### Search Box Standard

Apply consistent `.search-box` styling across all views:

```css
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-input);
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.search-box:focus-within {
  border-color: var(--accent);
  box-shadow: var(--focus-ring);
}

.search-box input {
  border: none;
  background: transparent;
  font-size: 13px;
  outline: none;
  width: 200px;
}
```

Add this to `src/assets/main.css` as a shared utility class. Update all views that use `.search-box` to reference this shared style.

### Typography & Spacing Tokens

Ensure all text in the Software Management page uses global CSS variables:
- Card body text: `font-size: 13px` (matches `data-table`)
- Labels/badges: `font-size: 11px`
- Spacing: use `--card-padding-md`, `--section-header-gap`, `--radius` tokens

---

## 5. MCP Naming Change

Replace display text "MCP Servers" → "Mcps" in these 6 files (UI text only, code identifiers unchanged):

| File | Line | Change |
|------|------|--------|
| `src/App.vue` | 43 | `'/mcp': 'MCP Servers'` → `'/mcp': 'Mcps'` |
| `src/components/layout/Sidebar.vue` | 126 | `MCP Servers` → `Mcps` |
| `src/views/MCPView.vue` | 6 | `<h2>MCP Servers</h2>` → `<h2>Mcps</h2>` |
| `src/components/mcp/MCPExportDialog.vue` | 6 | `<h3>Export MCP Servers</h3>` → `<h3>Export Mcps</h3>` |
| `src/components/mcp/MCPImportDialog.vue` | 6 | `<h3>Import MCP Servers</h3>` → `<h3>Import Mcps</h3>` |
| `src/components/plugins/PluginDetailsDialog.vue` | 452 | `MCP Servers:` → `Mcps:` |

---

## Files to Modify

### Frontend
1. `src/types/software.ts` — add `websiteUrl` field
2. `src/stores/software.ts` — add `selectedPlatform` state and `updateSoftware` action
3. `src/views/SoftwareManagementView.vue` — platform filter UI, website link, version feedback
4. `src/assets/main.css` — shared `.search-box` style
5. `src/App.vue` — MCP naming
6. `src/components/layout/Sidebar.vue` — MCP naming
7. `src/views/MCPView.vue` — MCP naming
8. `src/components/mcp/MCPExportDialog.vue` — MCP naming
9. `src/components/mcp/MCPImportDialog.vue` — MCP naming
10. `src/components/plugins/PluginDetailsDialog.vue` — MCP naming

### Rust Backend
1. `src-tauri/src/models/` — add `website_url` to Software model
2. `src-tauri/src/services/software_scanner.rs` — add `website_url` to configs, increase timeout, add `check_latest_version`
3. `src-tauri/src/commands/software.rs` — add `update_software` command
4. `src-tauri/src/services/software_installer.rs` — enhance install/uninstall with progress feedback
5. `src-tauri/Cargo.toml` — ensure `tauri-plugin-shell` is included (for `open` URL functionality)

---

## Out of Scope

- Language version switching (e.g., `nvm use 18`) — separate feature
- Frontend mock data removal — deferred to a later refactor
- Search box component extraction — deferred to a later refactor

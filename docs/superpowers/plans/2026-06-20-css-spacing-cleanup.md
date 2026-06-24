# CSS Spacing Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate CSS padding stacking between `.view` and `.content`, and remove unwanted `margin-bottom` from `.tab-bar` and `.section-header` elements.

**Architecture:** Modify global theme CSS variables and rules, then update scoped styles in Vue components to ensure consistent spacing behavior.

**Tech Stack:** CSS, Vue 3, Tailwind CSS

---

## File Structure

### Files to modify:

1. **`src/assets/theme.css`** — Global CSS variables and component rules
   - `.content` padding (unchanged)
   - `.tab-bar` margin-bottom → 0 (2 locations)
   - `.section-header` margin-bottom → 0
   - CSS variable `--section-header-margin-bottom` → 0

2. **`src/components/layout/AppFrame.vue`** — App frame layout component
   - Remove `.content` padding declaration (inherits global)

3. **`src/views/PluginsView.vue`** — Plugins management view
   - Remove `.view` padding declaration
   - `.tab-bar` margin-bottom → 0

4. **`src/views/BackupView.vue`** — Backup management view
   - Remove `.view` padding declaration
   - `.section-header` margin-bottom → 0

5. **`src/views/SoftwareManagementView.vue`** — Software management view
   - `.tab-bar` margin-bottom → 0
   - `.section-header` margin-bottom → 0

6. **`src/views/RulesView.vue`** — Rules management view
   - `.section-header` margin-bottom → 0

7. **`src/views/DashboardView.vue`** — Dashboard view
   - `.section-header` margin-bottom → 0

8. **`src/views/AgentsView.vue`** — Agents management view
   - `.section-header` margin-bottom → 0

9. **`src/views/SettingsView.vue`** — Settings view
   - `.section-header` margin-bottom → 0

10. **`src/views/CliToolsView.vue`** — CLI tools view
    - `.tab-bar` margin-bottom → 0
    - `.section-header` margin-bottom → 0

---

## Task 1: Update Global Theme CSS

**Files:**
- Modify: `src/assets/theme.css:83-88` (CSS variables)
- Modify: `src/assets/theme.css:1851-1856` (.tab-bar rule)
- Modify: `src/assets/theme.css:2719-2725` (.tab-bar rule, FEAT-024-B section)

- [ ] **Step 1: Update CSS variable for section-header margin-bottom**

```css
/* src/assets/theme.css - around line 83-88 */
:root {
  --section-header-padding: 16px 0;
  --section-header-margin-bottom: 0;  /* Changed from 24px */
  --section-header-gap: 12px;
  --section-header-title-size: 18px;
  --section-header-title-weight: 600;
}
```

- [ ] **Step 2: Update first .tab-bar rule**

```css
/* src/assets/theme.css - around line 1851-1856 */
.tab-bar {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);
  margin-bottom: 0;  /* Changed from 20px */
}
```

- [ ] **Step 3: Update second .tab-bar rule (FEAT-024-B section)**

```css
/* src/assets/theme.css - around line 2719-2725 */
.tab-bar {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);
  margin-bottom: 0;  /* Changed from 20px */
  overflow-x: auto;
}
```

- [ ] **Step 4: Commit global theme changes**

```bash
git add src/assets/theme.css
git commit -m "fix(css): update global theme spacing variables and rules

- Set --section-header-margin-bottom to 0
- Set .tab-bar margin-bottom to 0 (both rule locations)
- Aligns with design reference (forge.css)"
```

---

## Task 2: Update AppFrame Component

**Files:**
- Modify: `src/components/layout/AppFrame.vue:161-166`

- [ ] **Step 1: Remove .content padding declaration**

```vue
<!-- src/components/layout/AppFrame.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.content {
  flex: 1;
  overflow-y: auto;
  /* padding: 24px 32px 40px; */  /* Removed - inherits from global theme.css */
  position: relative;
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Commit AppFrame changes**

```bash
git add src/components/layout/AppFrame.vue
git commit -m "fix(css): remove .content padding from AppFrame

- Inherits global padding from theme.css
- Eliminates potential padding stacking with .view containers"
```

---

## Task 3: Update PluginsView Component

**Files:**
- Modify: `src/views/PluginsView.vue:577` (.view padding)
- Modify: `src/views/PluginsView.vue:622` (.tab-bar margin-bottom)

- [ ] **Step 1: Remove .view padding declaration**

```vue
<!-- src/views/PluginsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.view {
  /* padding: 24px 32px 40px; */  /* Removed - no padding on .view */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Update .tab-bar margin-bottom**

```vue
<!-- src/views/PluginsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.tab-bar {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 3: Commit PluginsView changes**

```bash
git add src/views/PluginsView.vue
git commit -m "fix(css): clean up PluginsView spacing

- Remove .view padding to prevent stacking with .content
- Set .tab-bar margin-bottom to 0"
```

---

## Task 4: Update BackupView Component

**Files:**
- Modify: `src/views/BackupView.vue:331` (.view padding)
- Modify: `src/views/BackupView.vue:335` (.section-header margin-bottom)

- [ ] **Step 1: Remove .view padding declaration**

```vue
<!-- src/views/BackupView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.view {
  /* padding: var(--spacing-lg); */  /* Removed - no padding on .view */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Update .section-header margin-bottom**

```vue
<!-- src/views/BackupView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from var(--spacing-lg) */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 3: Commit BackupView changes**

```bash
git add src/views/BackupView.vue
git commit -m "fix(css): clean up BackupView spacing

- Remove .view padding to prevent stacking with .content
- Set .section-header margin-bottom to 0"
```

---

## Task 5: Update SoftwareManagementView Component

**Files:**
- Modify: `src/views/SoftwareManagementView.vue:334` (.tab-bar margin-bottom)
- Modify: `src/views/SoftwareManagementView.vue:304` (.section-header margin-bottom)

- [ ] **Step 1: Update .tab-bar margin-bottom**

```vue
<!-- src/views/SoftwareManagementView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.tab-bar {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Update .section-header margin-bottom**

```vue
<!-- src/views/SoftwareManagementView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 3: Commit SoftwareManagementView changes**

```bash
git add src/views/SoftwareManagementView.vue
git commit -m "fix(css): clean up SoftwareManagementView spacing

- Set .tab-bar margin-bottom to 0
- Set .section-header margin-bottom to 0"
```

---

## Task 6: Update RulesView Component

**Files:**
- Modify: `src/views/RulesView.vue:307` (.section-header margin-bottom)

- [ ] **Step 1: Update .section-header margin-bottom**

```vue
<!-- src/views/RulesView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Commit RulesView changes**

```bash
git add src/views/RulesView.vue
git commit -m "fix(css): clean up RulesView spacing

- Set .section-header margin-bottom to 0"
```

---

## Task 7: Update DashboardView Component

**Files:**
- Modify: `src/views/DashboardView.vue:162` (.section-header margin-bottom)

- [ ] **Step 1: Update .section-header margin-bottom**

```vue
<!-- src/views/DashboardView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Commit DashboardView changes**

```bash
git add src/views/DashboardView.vue
git commit -m "fix(css): clean up DashboardView spacing

- Set .section-header margin-bottom to 0"
```

---

## Task 8: Update AgentsView Component

**Files:**
- Modify: `src/views/AgentsView.vue:192` (.section-header margin-bottom)

- [ ] **Step 1: Update .section-header margin-bottom**

```vue
<!-- src/views/AgentsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Commit AgentsView changes**

```bash
git add src/views/AgentsView.vue
git commit -m "fix(css): clean up AgentsView spacing

- Set .section-header margin-bottom to 0"
```

---

## Task 9: Update SettingsView Component

**Files:**
- Modify: `src/views/SettingsView.vue:123` (.section-header margin-bottom)

- [ ] **Step 1: Update .section-header margin-bottom**

```vue
<!-- src/views/SettingsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 8px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Commit SettingsView changes**

```bash
git add src/views/SettingsView.vue
git commit -m "fix(css): clean up SettingsView spacing

- Set .section-header margin-bottom to 0"
```

---

## Task 10: Update CliToolsView Component

**Files:**
- Modify: `src/views/CliToolsView.vue:745` (.tab-bar margin-bottom)
- Modify: `src/views/CliToolsView.vue:627` (.section-header margin-bottom)

- [ ] **Step 1: Update .tab-bar margin-bottom**

```vue
<!-- src/views/CliToolsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.tab-bar {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 2: Update .section-header margin-bottom**

```vue
<!-- src/views/CliToolsView.vue - scoped style section -->
<style scoped>
/* ... existing styles ... */

.section-header {
  /* ... other properties ... */
  margin-bottom: 0;  /* Changed from 20px */
  /* ... other properties ... */
}

/* ... existing styles ... */
</style>
```

- [ ] **Step 3: Commit CliToolsView changes**

```bash
git add src/views/CliToolsView.vue
git commit -m "fix(css): clean up CliToolsView spacing

- Set .tab-bar margin-bottom to 0
- Set .section-header margin-bottom to 0"
```

---

## Task 11: Verification and Final Commit

**Files:**
- Test: Manual visual verification in browser

- [ ] **Step 1: Run development server**

```bash
npm run dev
```

Expected: Application starts without errors

- [ ] **Step 2: Verify visual spacing**

Check the following views in the browser:
1. DashboardView — .section-header should have no bottom margin
2. PluginsView — .view should have no padding, .tab-bar no bottom margin
3. BackupView — .view should have no padding, .section-header no bottom margin
4. SoftwareManagementView — .tab-bar and .section-header no bottom margin
5. RulesView — .section-header no bottom margin
6. AgentsView — .section-header no bottom margin
7. SettingsView — .section-header no bottom margin
8. CliToolsView — .tab-bar and .section-header no bottom margin

Expected: No visual padding stacking between .view and .content containers

- [ ] **Step 3: Run linting**

```bash
npm run lint
```

Expected: No linting errors

- [ ] **Step 4: Run tests**

```bash
npm test
```

Expected: All tests pass

- [ ] **Step 5: Final commit with summary**

```bash
git add -A
git commit -m "fix(css): complete CSS spacing cleanup

Addresses:
1. Padding stacking between .view and .content containers
2. Unwanted margin-bottom on .tab-bar and .section-header elements

Changes:
- Global: Updated CSS variables and rules in theme.css
- AppFrame: Removed .content padding (inherits global)
- PluginsView: Removed .view padding, updated .tab-bar margin
- BackupView: Removed .view padding, updated .section-header margin
- SoftwareManagementView: Updated .tab-bar and .section-header margins
- RulesView: Updated .section-header margin
- DashboardView: Updated .section-header margin
- AgentsView: Updated .section-header margin
- SettingsView: Updated .section-header margin
- CliToolsView: Updated .tab-bar and .section-header margins

Result:
- Vertical padding only from .content (24px top, 40px bottom)
- No extra spacing from .tab-bar or .section-header
- Responsive behavior unchanged"
```

---

## Summary

**Total tasks:** 11
**Estimated time:** 15-20 minutes
**Files modified:** 10
**Commits:** 11 (one per task)

**Key changes:**
1. CSS variables aligned with design reference
2. Eliminated padding stacking by removing .view padding
3. Removed all margin-bottom from .tab-bar and .section-header
4. Maintained responsive behavior through CSS variables

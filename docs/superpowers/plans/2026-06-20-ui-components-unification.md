# UI Components Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify dropdown, progress, and tab components across all views — fix uninstall error handling, add viewport-aware dropdown positioning, extract ProgressSlot and SourceTabs as shared components.

**Architecture:** Extract repeated inline patterns into reusable Vue components (`DropdownMenu`, `ProgressSlot`, `SourceTabs`) with a `useDropdown` composable for viewport-aware positioning. A shared `extractError` utility handles Tauri error normalization.

**Tech Stack:** Vue 3 `<script setup>`, TypeScript, Composition API, existing `useOperationProgress` composable.

---

## File Map

| File | Action | Purpose |
|------|--------|---------|
| `src/utils/error.ts` | Create | `extractError()` for safe error message extraction |
| `src/composables/useDropdown.ts` | Create | Viewport-aware dropdown positioning logic |
| `src/components/common/DropdownMenu.vue` | Create | Reusable dropdown with auto-positioning |
| `src/components/common/ProgressSlot.vue` | Create | Reusable progress bar + status text |
| `src/components/common/SourceTabs.vue` | Create | Reusable secondary tab navigation |
| `src/assets/theme.css` | Modify | Add global dropdown/progress/source-tabs styles |
| `src/views/CliToolsView.vue` | Modify | Use DropdownMenu, fix error handling, remove scoped dropdown CSS |
| `src/views/MCPView.vue` | Modify | Use DropdownMenu, TabBar, remove scoped dropdown CSS |
| `src/views/SkillsView.vue` | Modify | Use DropdownMenu, SourceTabs, remove scoped dropdown/source-tabs CSS |
| `src/views/SoftwareManagementView.vue` | Modify | Use DropdownMenu, TabBar, fix error handling, remove scoped dropdown CSS |
| `src/views/PluginsView.vue` | Modify | Use SourceTabs, remove scoped source-tabs CSS |
| `src/components/plugins/PluginCard.vue` | Modify | Use ProgressSlot for install progress |

---

### Task 1: Create `src/utils/error.ts`

**Files:**
- Create: `src/utils/error.ts`

- [ ] **Step 1: Create the error utility**

```typescript
// src/utils/error.ts
/**
 * Safely extract a human-readable error message from any thrown value.
 * Tauri invoke() may throw strings, plain objects, or non-standard Errors.
 */
export function extractError(e: unknown): string {
  if (e instanceof Error) return e.message
  if (typeof e === 'string') return e
  try { return JSON.stringify(e) } catch { return String(e) }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit src/utils/error.ts`
Expected: No errors (or only module resolution errors for standalone check — acceptable)

- [ ] **Step 3: Commit**

```bash
git add src/utils/error.ts
git commit -m "feat(utils): add extractError for safe Tauri error message extraction"
```

---

### Task 2: Create `src/composables/useDropdown.ts`

**Files:**
- Create: `src/composables/useDropdown.ts`

- [ ] **Step 1: Create the dropdown positioning composable**

```typescript
// src/composables/useDropdown.ts
import { ref, nextTick, type Ref, type CSSProperties } from 'vue'

export type DropdownPlacement = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'

export function useDropdown(triggerRef: Ref<HTMLElement | null>) {
  const positionStyle = ref<CSSProperties>({})

  /**
   * Calculate menu position based on trigger element's viewport rect.
   * Default: bottom-right (opens upward, right-aligned with trigger).
   * Falls back to other quadrants if the default overflows the viewport.
   */
  async function computePosition(menuWidth = 160, menuHeight = 200) {
    await nextTick()

    const trigger = triggerRef.value
    if (!trigger) {
      positionStyle.value = {}
      return
    }

    const rect = trigger.getBoundingClientRect()
    const vw = window.innerWidth
    const vh = window.innerHeight
    const gap = 6

    // Default: open upward from trigger, right-aligned
    let top: number
    let left: number

    // Try bottom-right (opens upward): top = trigger.top - gap - menuHeight
    if (rect.top - gap - menuHeight > 0) {
      // Enough space above
      top = rect.top - gap - menuHeight
      left = rect.right - menuWidth
    } else {
      // Not enough space above, open downward
      top = rect.bottom + gap
      left = rect.right - menuWidth
    }

    // Clamp horizontal: ensure menu stays within viewport
    if (left < 8) left = 8
    if (left + menuWidth > vw - 8) left = vw - menuWidth - 8

    // Clamp vertical
    if (top < 8) top = 8
    if (top + menuHeight > vh - 8) top = vh - menuHeight - 8

    positionStyle.value = {
      position: 'fixed',
      top: `${top}px`,
      left: `${left}px`,
    }
  }

  return { positionStyle, computePosition }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit src/composables/useDropdown.ts`
Expected: No type errors

- [ ] **Step 3: Commit**

```bash
git add src/composables/useDropdown.ts
git commit -m "feat(composables): add useDropdown with viewport-aware positioning"
```

---

### Task 3: Create `src/components/common/DropdownMenu.vue`

**Files:**
- Create: `src/components/common/DropdownMenu.vue`

- [ ] **Step 1: Create the DropdownMenu component**

```vue
<!-- src/components/common/DropdownMenu.vue -->
<script setup lang="ts">
import { ref, watch, nextTick, onBeforeUnmount } from 'vue'
import { useDropdown } from '@/composables/useDropdown'

interface Props {
  modelValue: boolean
  minWidth?: number
}

const props = withDefaults(defineProps<Props>(), {
  minWidth: 160,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const triggerRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const { positionStyle, computePosition } = useDropdown(triggerRef)

// Recompute position when menu opens
watch(() => props.modelValue, async (open) => {
  if (open) {
    await nextTick()
    const menuHeight = menuRef.value?.offsetHeight || 200
    computePosition(props.minWidth, menuHeight)
  }
})

// Close on click outside
function handleClickOutside(e: MouseEvent) {
  if (props.modelValue && menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('update:modelValue', false)
  }
}

// Close on Escape
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.modelValue) {
    emit('update:modelValue', false)
  }
}

// Close on scroll
function handleScroll() {
  if (props.modelValue) {
    emit('update:modelValue', false)
  }
}

watch(() => props.modelValue, (open) => {
  if (open) {
    document.addEventListener('click', handleClickOutside, true)
    document.addEventListener('keydown', handleKeydown)
    window.addEventListener('scroll', handleScroll, true)
  } else {
    document.removeEventListener('click', handleClickOutside, true)
    document.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('scroll', handleScroll, true)
  }
}, { immediate: true })

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside, true)
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('scroll', handleScroll, true)
})
</script>

<template>
  <div class="dropdown-wrapper" @click.stop ref="triggerRef">
    <slot name="trigger" />
    <Transition name="dropdown">
      <div
        v-if="modelValue"
        ref="menuRef"
        class="dropdown-menu"
        :style="[positionStyle, { minWidth: minWidth + 'px' }]"
      >
        <slot />
      </div>
    </Transition>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/common/DropdownMenu.vue
git commit -m "feat(components): add DropdownMenu with viewport-aware positioning"
```

---

### Task 4: Create `src/components/common/ProgressSlot.vue`

**Files:**
- Create: `src/components/common/ProgressSlot.vue`

- [ ] **Step 1: Create the ProgressSlot component**

```vue
<!-- src/components/common/ProgressSlot.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import type { OperationStage } from '@/composables/useOperationProgress'
import { STAGE_CONFIG } from '@/composables/useOperationProgress'

interface Props {
  stage: OperationStage
  progress: number
  message?: string
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  message: '',
  compact: false,
})

const stageLabel = computed(() => STAGE_CONFIG[props.stage]?.label || props.stage)

const progressColor = computed(() => {
  switch (props.stage) {
    case 'completed': return 'var(--success)'
    case 'failed': return 'var(--error)'
    case 'cancelled': return 'var(--fg-ghost)'
    default: return 'var(--accent)'
  }
})
</script>

<template>
  <div class="progress-slot" v-if="stage !== 'idle'">
    <div class="progress-bar-wrap">
      <div
        class="progress-bar-fill"
        :class="stage"
        :style="{ width: progress + '%', background: progressColor }"
      />
    </div>
    <span class="progress-msg" v-if="!compact">
      {{ message || stageLabel }} {{ stage !== 'completed' && stage !== 'failed' && stage !== 'cancelled' ? progress + '%' : '' }}
    </span>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/common/ProgressSlot.vue
git commit -m "feat(components): add ProgressSlot for card-footer progress display"
```

---

### Task 5: Create `src/components/common/SourceTabs.vue`

**Files:**
- Create: `src/components/common/SourceTabs.vue`

- [ ] **Step 1: Create the SourceTabs component**

```vue
<!-- src/components/common/SourceTabs.vue -->
<script setup lang="ts">
interface TabItem {
  id: string
  label: string
  count?: number | string
}

interface Props {
  tabs: TabItem[]
  modelValue?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

function selectTab(id: string) {
  emit('update:modelValue', id)
}
</script>

<template>
  <div class="source-tabs" role="tablist">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="source-tab"
      :class="{ active: modelValue === tab.id }"
      role="tab"
      :tabindex="modelValue === tab.id ? 0 : -1"
      :aria-selected="modelValue === tab.id"
      @click="selectTab(tab.id)"
      @keydown.enter.space.prevent="selectTab(tab.id)"
    >
      {{ tab.label }}
      <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/common/SourceTabs.vue
git commit -m "feat(components): add SourceTabs for secondary tab navigation"
```

---

### Task 6: Add global dropdown/progress/source-tabs styles to `theme.css`

**Files:**
- Modify: `src/assets/theme.css`

- [ ] **Step 1: Add global dropdown styles**

Append to `src/assets/theme.css` (after the existing dropdown-menu at line 334):

```css
/* Global Dropdown Component */
.dropdown-wrapper { position: relative; }
.dropdown-menu {
  position: absolute;
  background: rgba(255,255,255,0.52);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255,255,255,0.35);
  border-radius: var(--radius);
  box-shadow: 0 8px 32px rgba(0,0,0,0.12), inset 0 1px 0 rgba(255,255,255,0.50);
  padding: 4px;
  z-index: var(--z-dropdown);
}
.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
  font-size: 12px;
  color: var(--fg-muted);
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 150ms ease;
  text-align: left;
}
.dropdown-item:hover { background: rgba(255,255,255,0.40); color: var(--fg); }
.dropdown-item.danger { color: var(--error); }
.dropdown-item.danger:hover { background: rgba(184,90,66,0.12); }
.dropdown-divider { height: 1px; background: rgba(255,255,255,0.20); margin: 4px 0; }

/* Dropdown Transition */
.dropdown-enter-active, .dropdown-leave-active {
  transition: opacity 150ms ease, transform 150ms ease;
}
.dropdown-enter-from, .dropdown-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

/* Global Progress Slot */
.progress-slot {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}
.progress-slot .progress-bar-wrap { flex: 1; min-width: 0; }
.progress-slot .progress-msg {
  font-size: 11px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
  white-space: nowrap;
  flex-shrink: 0;
}
.progress-bar-wrap {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.32);
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 400ms var(--ease);
}
.progress-bar-fill.completed { background: var(--success); }
.progress-bar-fill.failed { background: var(--error); }

/* Global Source Tabs */
.source-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.10);
  margin-bottom: 16px;
  overflow-x: auto;
}
.source-tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--fg-ghost);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--t-fast);
  white-space: nowrap;
}
.source-tab:hover {
  background: rgba(255, 255, 255, 0.10);
  color: var(--fg);
}
.source-tab.active {
  border-bottom-color: var(--accent);
  color: var(--accent);
}
.source-tab .tab-count {
  margin-left: 4px;
  font-family: var(--font-mono);
  font-size: 10px;
  opacity: 0.5;
}
.source-tab.active .tab-count {
  opacity: 0.7;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/assets/theme.css
git commit -m "feat(theme): add global dropdown, progress-slot, and source-tabs styles"
```

---

### Task 7: Migrate CliToolsView to DropdownMenu + extractError

**Files:**
- Modify: `src/views/CliToolsView.vue`

- [ ] **Step 1: Import new components**

In `<script setup>`, add these imports:

```typescript
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import { extractError } from '@/utils/error'
```

- [ ] **Step 2: Replace dropdown HTML with DropdownMenu**

Replace the inline dropdown block (lines 184-207) with:

```html
<DropdownMenu v-model="openDropdown" :trigger="tool.key === openDropdown">
  <template #trigger>
    <button class="btn-icon btn-sm" @click.stop="handleMoreOptions(tool)" title="More options" aria-label="More options">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
      </svg>
    </button>
  </template>
  <button class="dropdown-item" @click.stop="handleOpenConfig(tool)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
    Open Config
  </button>
  <button class="dropdown-item" @click.stop="handleResetOptions(tool)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
    Reset Options
  </button>
  <div class="dropdown-divider"></div>
  <button class="dropdown-item danger" @click.stop="handleUninstall(tool)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
    Uninstall
  </button>
</DropdownMenu>
```

- [ ] **Step 3: Fix handleUninstall error handling**

Replace the `.catch` block in `handleUninstall` (line 583):

```typescript
function handleUninstall(tool: CliToolInfo) {
  closeDropdown();
  if (confirm(`确认卸载 ${tool.name}？`)) {
    softwareStore.uninstallSoftware(tool.key).then((res) => {
      if (showNotification) showNotification(res.message || `${tool.name} 已卸载`, 'success');
    }).catch((e) => {
      if (showNotification) showNotification(`卸载失败: ${extractError(e)}`, 'error');
    });
  }
}
```

- [ ] **Step 4: Remove scoped dropdown CSS**

Remove these lines from `<style scoped>` (approximately lines 1143-1151):

```
.dropdown-wrapper { position: relative; }
.dropdown-menu { position: absolute; bottom: 100%; ... }
.dropdown-item { ... }
.dropdown-item:hover { ... }
.dropdown-item.danger { ... }
.dropdown-item.danger:hover { ... }
.dropdown-divider { ... }
.dropdown-enter-active, .dropdown-leave-active { ... }
.dropdown-enter-from, .dropdown-leave-to { ... }
```

- [ ] **Step 5: Remove scoped progress CSS**

Remove the progress-related CSS from `<style scoped>` (lines 1017-1060):

```
.progress-slot { ... }
.progress-slot .progress-bar-wrap { ... }
.progress-slot .progress-msg { ... }
.progress-bar-wrap { ... }
.progress-bar-fill { ... }
.progress-bar-fill.success { ... }
.progress-bar-fill.error { ... }
```

- [ ] **Step 6: Commit**

```bash
git add src/views/CliToolsView.vue
git commit -m "refactor(cli-tools): use DropdownMenu component and extractError"
```

---

### Task 8: Migrate MCPView to DropdownMenu + TabBar

**Files:**
- Modify: `src/views/MCPView.vue`

- [ ] **Step 1: Import new components**

In `<script setup>`, add:

```typescript
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import TabBar from '@/components/common/TabBar.vue'
```

- [ ] **Step 2: Define tabItems for TabBar**

Add to `<script setup>`:

```typescript
const tabItems = [
  { id: 'services', label: 'Services', count: servers.value.length },
  { id: 'groups', label: 'Groups' },
  { id: 'audit', label: 'Audit Log' },
]
```

- [ ] **Step 3: Replace tab-bar HTML with TabBar**

Replace the inline tab-bar (lines 10-26) with:

```html
<TabBar v-model="activeTab" :tabs="tabItems" />
```

- [ ] **Step 4: Replace dropdown HTML with DropdownMenu**

Replace the inline dropdown block (lines 116-139) with:

```html
<DropdownMenu v-model="openDropdown" :trigger="server.name === openDropdown">
  <template #trigger>
    <button class="btn-icon btn-sm" @click.stop="toggleDropdown(server.name)" aria-label="More actions">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
      </svg>
    </button>
  </template>
  <button class="dropdown-item" @click.stop="handleViewLogs(server)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
    View Logs
  </button>
  <button class="dropdown-item" @click.stop="handleRestart(server)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
    Restart
  </button>
  <div class="dropdown-divider"></div>
  <button class="dropdown-item danger" @click.stop="handleDisconnect(server)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    Disconnect
  </button>
</DropdownMenu>
```

- [ ] **Step 5: Remove scoped dropdown CSS**

Remove from `<style scoped>` (lines 642-651):

```
.dropdown-wrapper { position: relative; }
.dropdown-menu { ... }
.dropdown-item { ... }
.dropdown-item:hover { ... }
.dropdown-item.danger { ... }
.dropdown-item.danger:hover { ... }
.dropdown-divider { ... }
.dropdown-enter-active, .dropdown-leave-active { ... }
.dropdown-enter-from, .dropdown-leave-to { ... }
```

- [ ] **Step 6: Commit**

```bash
git add src/views/MCPView.vue
git commit -m "refactor(mcp): use DropdownMenu and TabBar components"
```

---

### Task 9: Migrate SkillsView to DropdownMenu + SourceTabs

**Files:**
- Modify: `src/views/SkillsView.vue`

- [ ] **Step 1: Import new components**

In `<script setup>`, add:

```typescript
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import SourceTabs from '@/components/common/SourceTabs.vue'
```

- [ ] **Step 2: Replace source-tabs HTML with SourceTabs**

Replace the inline source-tabs block (lines 10-25) with:

```html
<SourceTabs v-model="activeSource" :tabs="sourceTabs" />
```

- [ ] **Step 3: Replace dropdown HTML with DropdownMenu**

Replace the inline dropdown block (lines 141-164) with:

```html
<DropdownMenu v-model="openDropdown" :trigger="skill.name === openDropdown">
  <template #trigger>
    <button class="btn-icon btn-sm" @click.stop="toggleDropdown(skill.name)" aria-label="More actions">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
      </svg>
    </button>
  </template>
  <button class="dropdown-item" @click.stop="handleDuplicate(skill)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
    Duplicate
  </button>
  <button class="dropdown-item" @click.stop="handleExport(skill)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
    Export
  </button>
  <div class="dropdown-divider"></div>
  <button class="dropdown-item danger" @click.stop="handleDeleteSkill(skill)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
    Delete
  </button>
</DropdownMenu>
```

- [ ] **Step 4: Remove scoped source-tabs CSS**

Remove from `<style scoped>` (lines 382-418):

```
.source-tabs { ... }
.source-tab { ... }
.source-tab:hover { ... }
.source-tab.active { ... }
.tab-count { ... }
```

- [ ] **Step 5: Remove scoped dropdown CSS**

Remove from `<style scoped>` (lines 762-771):

```
.dropdown-wrapper { position: relative; }
.dropdown-menu { ... }
.dropdown-item { ... }
.dropdown-item:hover { ... }
.dropdown-item.danger { ... }
.dropdown-item.danger:hover { ... }
.dropdown-divider { ... }
.dropdown-enter-active, .dropdown-leave-active { ... }
.dropdown-enter-from, .dropdown-leave-to { ... }
```

- [ ] **Step 6: Commit**

```bash
git add src/views/SkillsView.vue
git commit -m "refactor(skills): use DropdownMenu and SourceTabs components"
```

---

### Task 10: Migrate SoftwareManagementView to DropdownMenu + TabBar + extractError

**Files:**
- Modify: `src/views/SoftwareManagementView.vue`

- [ ] **Step 1: Import new components**

In `<script setup>`, add:

```typescript
import DropdownMenu from '@/components/common/DropdownMenu.vue'
import TabBar from '@/components/common/TabBar.vue'
import { extractError } from '@/utils/error'
```

- [ ] **Step 2: Define tabItems for TabBar**

Add to `<script setup>`:

```typescript
const tabItems = [
  { id: 'all', label: 'All' },
  { id: 'detected', label: 'Detected' },
  { id: 'not-found', label: 'Not Found' },
]
```

- [ ] **Step 3: Replace tab-bar HTML with TabBar**

Replace the inline tab-bar (lines 208-224) with:

```html
<TabBar v-model="selectedStatus" :tabs="tabItems" />
```

- [ ] **Step 4: Replace dropdown HTML with DropdownMenu**

Replace the inline dropdown block (lines 317-340) with:

```html
<DropdownMenu v-model="openDropdown" :trigger="sw.key === openDropdown">
  <template #trigger>
    <button class="btn-icon btn-sm" @click.stop="toggleDropdown(sw.key)" title="More actions" aria-label="More actions">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>
      </svg>
    </button>
  </template>
  <button class="dropdown-item" @click.stop="handleViewLogs(sw)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
    View Logs
  </button>
  <button class="dropdown-item" @click.stop="handleCheckUpdate(sw)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
    Check Update
  </button>
  <div class="dropdown-divider"></div>
  <button class="dropdown-item danger" @click.stop="handleUninstall(sw)">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
    Uninstall
  </button>
</DropdownMenu>
```

- [ ] **Step 5: Fix handleUninstall error handling**

Replace the `.catch` block in `handleUninstall` (line 193):

```typescript
function handleUninstall(sw: Software) {
  openDropdown.value = null
  if (confirm(`确认卸载 ${sw.name}？`)) {
    softwareStore.uninstallSoftware(sw.key).then((res) => {
      console.log(res.message || `${sw.name} 已卸载`)
    }).catch((e) => {
      console.error('卸载失败:', extractError(e))
    })
  }
}
```

- [ ] **Step 6: Remove scoped dropdown CSS**

Remove from `<style scoped>` (lines 611-620):

```
.dropdown-wrapper { position: relative; }
.dropdown-menu { ... }
.dropdown-item { ... }
.dropdown-item:hover { ... }
.dropdown-item.danger { ... }
.dropdown-item.danger:hover { ... }
.dropdown-divider { ... }
.dropdown-enter-active, .dropdown-leave-active { ... }
.dropdown-enter-from, .dropdown-leave-to { ... }
```

- [ ] **Step 7: Commit**

```bash
git add src/views/SoftwareManagementView.vue
git commit -m "refactor(software): use DropdownMenu, TabBar, and extractError"
```

---

### Task 11: Migrate PluginsView to SourceTabs

**Files:**
- Modify: `src/views/PluginsView.vue`

- [ ] **Step 1: Import SourceTabs component**

In `<script setup>`, add:

```typescript
import SourceTabs from '@/components/common/SourceTabs.vue'
```

- [ ] **Step 2: Create a computed for SourceTabs tabs data**

The existing `installedSources` and `totalMarketplacePlugins` need to be mapped to the SourceTabs format. Add a computed:

```typescript
const sourceTabItems = computed(() => [
  ...installedSources.value.map(s => ({ id: s.id, label: s.name, count: s.pluginCount ?? 0 })),
  { id: 'all', label: 'All Sources', count: totalMarketplacePlugins.value },
])
```

- [ ] **Step 3: Replace inline source-tabs with SourceTabs**

Replace the inline source-tabs block (lines 115-136) with:

```html
<div class="source-tabs-wrap">
  <SourceTabs v-model="currentSourceId" :tabs="sourceTabItems" />
</div>
```

Note: The `selectSource` function call is replaced by v-model binding. The existing `selectSource` method should be replaced with direct v-model or kept for backward compatibility. Since `currentSourceId` is already a ref, v-model will update it directly.

- [ ] **Step 4: Remove scoped source-tabs CSS**

Remove from `<style scoped>` (lines 762-819):

```
.source-tabs-wrap { ... }
.source-tabs { ... }
.source-tab { ... }
.source-tab:hover { ... }
.source-tab.active { ... }
.source-tab-name { ... }
.source-tab-count { ... }
.source-tab.active .source-tab-count { ... }
```

- [ ] **Step 5: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "refactor(plugins): use SourceTabs component"
```

---

### Task 12: Migrate PluginCard to ProgressSlot

**Files:**
- Modify: `src/components/plugins/PluginCard.vue`

- [ ] **Step 1: Import ProgressSlot component**

In `<script setup>`, add:

```typescript
import ProgressSlot from '@/components/common/ProgressSlot.vue'
```

- [ ] **Step 2: Map PluginInstallProgress to OperationStage**

The `progress` prop uses `PluginInstallProgress` which has different stage names (`pending`, `downloading`, `installing`, `success`, `failed`). Map to `OperationStage`:

```typescript
const mappedStage = computed(() => {
  if (!props.progress) return 'idle' as OperationStage
  const map: Record<string, OperationStage> = {
    pending: 'preparing',
    downloading: 'downloading',
    installing: 'installing',
    success: 'completed',
    failed: 'failed',
  }
  return map[props.progress.stage] || 'preparing'
})
```

- [ ] **Step 3: Replace install-progress HTML with ProgressSlot**

Replace the inline progress block (lines 53-66) with:

```html
<div v-if="progress" class="card-progress-slot">
  <ProgressSlot
    :stage="mappedStage"
    :progress="progress.progress"
    :message="progress.message"
  />
</div>
```

- [ ] **Step 4: Add scoped style for card-progress-slot**

Add to `<style scoped>`:

```css
.card-progress-slot {
  margin-bottom: 12px;
}
```

- [ ] **Step 5: Commit**

```bash
git add src/components/plugins/PluginCard.vue
git commit -m "refactor(plugin-card): use ProgressSlot component"
```

---

### Task 13: Final verification

- [ ] **Step 1: Run type check**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -50
```

Expected: No new type errors

- [ ] **Step 2: Run linter**

```bash
npm run lint 2>&1 | head -50
```

Expected: No new lint errors

- [ ] **Step 3: Verify dev server starts**

```bash
npm run dev:web 2>&1 &
sleep 5
# Check for compilation errors in output
kill %1
```

Expected: No compilation errors

- [ ] **Step 4: Final commit with all changes**

If any fixes were needed during verification:

```bash
git add -A
git commit -m "fix: resolve type/lint issues from component unification"
```

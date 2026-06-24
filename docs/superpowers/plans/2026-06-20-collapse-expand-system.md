# Collapse/Expand System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a full collapse/expand system with sidebar toggle, reusable CollapsePanel and ToggleSwitch components, and localStorage persistence.

**Architecture:** Pinia UI store manages sidebar collapse state with localStorage persistence. CollapsePanel and ToggleSwitch are reusable common components. AppFrame wires the Topbar toggle event to the store, and Sidebar receives the collapsed state as a prop.

**Tech Stack:** Vue 3 + TypeScript + Pinia + Vitest

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `src/stores/ui.ts` | Create | UI state store — sidebar collapsed state + localStorage |
| `src/stores/__tests__/ui.spec.ts` | Create | Unit tests for useUiStore |
| `src/components/common/ToggleSwitch.vue` | Create | Reusable toggle switch (v-model) |
| `src/components/common/CollapsePanel.vue` | Create | Generic collapsible panel with transition |
| `src/components/common/__tests__/ToggleSwitch.spec.ts` | Create | ToggleSwitch component tests |
| `src/components/common/__tests__/CollapsePanel.spec.ts` | Create | CollapsePanel component tests |
| `src/components/layout/AppFrame.vue` | Modify | Wire toggle-sidebar event to uiStore |
| `src/components/layout/Sidebar.vue` | Modify | Accept collapsed prop, add collapse CSS |
| `src/views/SettingsView.vue` | Modify | Replace inline toggle with ToggleSwitch |

---

### Task 1: Create `useUiStore` — sidebar collapse state with localStorage

**Files:**
- Create: `src/stores/ui.ts`
- Create: `src/stores/__tests__/ui.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/stores/__tests__/ui.spec.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUiStore } from '@/stores/ui';

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it('defaults sidebarCollapsed to false', () => {
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(false);
  });

  it('toggleSidebar flips the state', () => {
    const store = useUiStore();
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(true);
    store.toggleSidebar();
    expect(store.sidebarCollapsed).toBe(false);
  });

  it('persists sidebarCollapsed to localStorage', () => {
    const store = useUiStore();
    store.toggleSidebar();
    expect(localStorage.getItem('forge-sidebar-collapsed')).toBe('true');
  });

  it('restores sidebarCollapsed from localStorage on init', () => {
    localStorage.setItem('forge-sidebar-collapsed', 'true');
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(true);
  });

  it('handles invalid localStorage value gracefully', () => {
    localStorage.setItem('forge-sidebar-collapsed', 'invalid');
    const store = useUiStore();
    expect(store.sidebarCollapsed).toBe(false);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/stores/__tests__/ui.spec.ts`
Expected: FAIL — `useUiStore` module not found

- [ ] **Step 3: Write minimal implementation**

```typescript
// src/stores/ui.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';

const STORAGE_KEY = 'forge-sidebar-collapsed';

export const useUiStore = defineStore('ui', () => {
  // --- State ---
  const sidebarCollapsed = ref(false);

  // --- Actions ---
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
    try {
      localStorage.setItem(STORAGE_KEY, String(sidebarCollapsed.value));
    } catch {
      // Silently ignore localStorage write failures
    }
  }

  // --- Init: restore from localStorage ---
  function initUi() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved === 'true') {
        sidebarCollapsed.value = true;
      }
    } catch {
      // Silently ignore localStorage read failures
    }
  }

  // Auto-init on store creation
  initUi();

  return {
    sidebarCollapsed,
    toggleSidebar,
    initUi,
  };
});
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/stores/__tests__/ui.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/stores/ui.ts src/stores/__tests__/ui.spec.ts
git commit -m "feat(store): add useUiStore with sidebar collapse + localStorage persistence"
```

---

### Task 2: Create `ToggleSwitch` component

**Files:**
- Create: `src/components/common/ToggleSwitch.vue`
- Create: `src/components/common/__tests__/ToggleSwitch.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/common/__tests__/ToggleSwitch.spec.ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ToggleSwitch from '@/components/common/ToggleSwitch.vue';

describe('ToggleSwitch', () => {
  it('renders with modelValue false by default', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    expect(wrapper.find('.toggle').classes()).not.toContain('on');
  });

  it('renders with modelValue true', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: true } });
    expect(wrapper.find('.toggle').classes()).toContain('on');
  });

  it('emits update:modelValue on click', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    await wrapper.find('.toggle').trigger('click');
    expect(wrapper.emitted('update:modelValue')).toEqual([[true]]);
  });

  it('does not emit when disabled', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, disabled: true } });
    await wrapper.find('.toggle').trigger('click');
    expect(wrapper.emitted('update:modelValue')).toBeUndefined();
  });

  it('renders label when provided', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false, label: 'Enable' } });
    expect(wrapper.find('.toggle-label').text()).toBe('Enable');
  });

  it('does not render label when not provided', () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: false } });
    expect(wrapper.find('.toggle-label').exists()).toBe(false);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/components/common/__tests__/ToggleSwitch.spec.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write implementation**

```vue
<!-- src/components/common/ToggleSwitch.vue -->
<script setup lang="ts">
interface Props {
  modelValue: boolean
  label?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
}
</script>

<template>
  <div class="toggle-wrap" :class="{ disabled }">
    <div
      class="toggle"
      :class="{ on: modelValue }"
      role="switch"
      :aria-checked="modelValue"
      :aria-label="label"
      @click="toggle"
    />
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.toggle-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toggle-wrap.disabled {
  opacity: 0.5;
  pointer-events: none;
}

.toggle {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.22);
  border: 1px solid rgba(255, 255, 255, 0.32);
  position: relative;
  cursor: pointer;
  transition: background var(--t-base);
}

.toggle.on {
  background: var(--accent);
}

.toggle::after {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: white;
  top: 2px;
  left: 2px;
  transition: transform var(--t-base);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.toggle.on::after {
  transform: translateX(16px);
}

.toggle-label {
  font-size: 12px;
  color: var(--fg-muted);
}
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/components/common/__tests__/ToggleSwitch.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/common/ToggleSwitch.vue src/components/common/__tests__/ToggleSwitch.spec.ts
git commit -m "feat(ui): add reusable ToggleSwitch component with v-model support"
```

---

### Task 3: Create `CollapsePanel` component

**Files:**
- Create: `src/components/common/CollapsePanel.vue`
- Create: `src/components/common/__tests__/CollapsePanel.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/common/__tests__/CollapsePanel.spec.ts
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import CollapsePanel from '@/components/common/CollapsePanel.vue';

describe('CollapsePanel', () => {
  it('renders with expanded=true by default', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    expect(wrapper.find('.panel-body').isVisible()).toBe(true);
  });

  it('renders with expanded=false', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test', expanded: false } });
    expect(wrapper.find('.panel-body').isVisible()).toBe(false);
  });

  it('toggles on header click', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    await wrapper.find('.panel-header').trigger('click');
    expect(wrapper.emitted('update:expanded')).toEqual([[false]]);
  });

  it('displays title text', () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'My Panel' } });
    expect(wrapper.find('.panel-title').text()).toBe('My Panel');
  });

  it('renders header slot content', () => {
    const wrapper = mount(CollapsePanel, {
      props: { title: 'Test' },
      slots: { header: '<span class="custom">Custom</span>' },
    });
    expect(wrapper.find('.custom').exists()).toBe(true);
  });

  it('renders default slot content', () => {
    const wrapper = mount(CollapsePanel, {
      props: { title: 'Test' },
      slots: { default: '<p>Content</p>' },
    });
    expect(wrapper.find('.panel-body').html()).toContain('Content');
  });

  it('shows chevron icon that rotates when collapsed', async () => {
    const wrapper = mount(CollapsePanel, { props: { title: 'Test' } });
    expect(wrapper.find('.chevron').classes()).not.toContain('rotated');
    await wrapper.setProps({ expanded: false });
    expect(wrapper.find('.chevron').classes()).toContain('rotated');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/components/common/__tests__/CollapsePanel.spec.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write implementation**

```vue
<!-- src/components/common/CollapsePanel.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  title: string
  expanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  expanded: true,
})

const emit = defineEmits<{
  'update:expanded': [value: boolean]
}>()

const isExpanded = ref(props.expanded)

watch(() => props.expanded, (val) => {
  isExpanded.value = val
})

function toggle() {
  isExpanded.value = !isExpanded.value
  emit('update:expanded', isExpanded.value)
}
</script>

<template>
  <div class="collapse-panel">
    <div class="panel-header" @click="toggle">
      <slot name="header">
        <span class="panel-title">{{ title }}</span>
      </slot>
      <svg
        class="chevron"
        :class="{ rotated: !isExpanded }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </div>
    <Transition name="collapse">
      <div v-show="isExpanded" class="panel-body">
        <slot />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.collapse-panel {
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: pointer;
  transition: background var(--t-fast);
  user-select: none;
}

.panel-header:hover {
  background: rgba(255, 255, 255, 0.20);
}

.panel-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
}

.chevron {
  color: var(--fg-muted);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.chevron.rotated {
  transform: rotate(-90deg);
}

.panel-body {
  padding: 0 16px 12px;
}

/* Collapse transition */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  max-height: 500px;
}
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/components/common/__tests__/CollapsePanel.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/common/CollapsePanel.vue src/components/common/__tests__/CollapsePanel.spec.ts
git commit -m "feat(ui): add reusable CollapsePanel component with transition"
```

---

### Task 4: Wire sidebar collapse in AppFrame

**Files:**
- Modify: `src/components/layout/AppFrame.vue`

- [ ] **Step 1: Read current AppFrame.vue**

Confirm the file content matches what was read earlier: `Topbar` emits `toggle-sidebar`, but `AppFrame` does not handle it.

- [ ] **Step 2: Add `useUiStore` import and `toggle-sidebar` handler**

In `AppFrame.vue`, add the import and handler. The full `<script setup>` section becomes:

```vue
<script setup lang="ts">
import { ref, computed, provide, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Sidebar from '@/components/layout/Sidebar.vue';
import Topbar from '@/components/layout/Topbar.vue';
import MobileTabbar from '@/components/layout/MobileTabbar.vue';
import { useThemeStore } from '@/stores/theme';
import { useUiStore } from '@/stores/ui';
import '@/assets/theme.css';

const router = useRouter();
const themeStore = useThemeStore();
const uiStore = useUiStore();

onMounted(() => {
  themeStore.initTheme();
});

const currentRoute = computed(() => router.currentRoute.value.path);

const pageTitles: Record<string, string> = {
  '/': 'Dashboard',
  '/cli-tools': 'CLI Tools',
  '/software': 'Software',
  '/plugins': 'Plugins',
  '/skills': 'Skills',
  '/mcp': 'MCP Servers',
  '/rules': 'Rules',
  '/backup': 'Backup & Restore',
  '/import-export': 'Import / Export',
  '/settings': 'Settings',
  '/prompts': 'Prompt Manager',
  '/agents': 'Agents',
};

const currentTitle = computed(() => {
  return pageTitles[currentRoute.value] || 'Dashboard';
});

const toast = ref<{
  visible: boolean;
  message: string;
  type: string;
}>({
  visible: false,
  message: '',
  type: ''
});

let toastTimeout: ReturnType<typeof setTimeout> | null = null;

const toastIcon = computed(() => {
  switch (toast.value.type) {
    case 'success': return '✓';
    case 'error': return '✕';
    case 'info': return 'ℹ';
    default: return 'ℹ';
  }
});

const toastIconColor = computed(() => {
  switch (toast.value.type) {
    case 'success': return 'var(--success)';
    case 'error': return 'var(--error)';
    case 'info': return 'var(--info)';
    default: return 'var(--info)';
  }
});

const showNotification = (message: string, type: string = 'info') => {
  if (toastTimeout) {
    clearTimeout(toastTimeout);
  }
  toast.value = {
    visible: true,
    message,
    type
  };
  toastTimeout = setTimeout(() => {
    toast.value.visible = false;
  }, 3000);
};

provide('showNotification', showNotification);

const handleRefresh = () => {
  showNotification('Refresh complete · 0 changes detected', 'success');
};

const handleSettings = () => {
  showNotification('Settings opened', 'info');
};

const handleToggleSidebar = () => {
  uiStore.toggleSidebar();
};
</script>
```

- [ ] **Step 3: Update template to pass collapsed prop and wire event**

In the template, change the `<Sidebar />` and `<Topbar>` lines:

```vue
<template>
  <div class="window-frame">
    <!-- Main shell container -->
    <div class="shell">
      <!-- Sidebar navigation -->
      <Sidebar :collapsed="uiStore.sidebarCollapsed" />

      <!-- Main content area -->
      <div class="main">
        <!-- Top navigation bar -->
        <Topbar
          :title="currentTitle"
          @refresh="handleRefresh"
          @settings="handleSettings"
          @toggle-sidebar="handleToggleSidebar"
        />

        <!-- Content area with router -->
        <main class="content">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </main>
      </div>
    </div>

    <!-- Mobile bottom tab bar -->
    <MobileTabbar />

    <!-- Toast notifications -->
    <Transition name="toast">
      <div v-if="toast.visible" class="toast" :class="toast.type">
        <span class="toast-icon" :style="{ color: toastIconColor }">{{ toastIcon }}</span>
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>
```

- [ ] **Step 4: Run dev build to check for errors**

Run: `pnpm run build` (or `pnpm run dev` to check in browser)
Expected: No compilation errors

- [ ] **Step 5: Commit**

```bash
git add src/components/layout/AppFrame.vue
git commit -m "feat(layout): wire sidebar toggle event to useUiStore"
```

---

### Task 5: Add collapse support to Sidebar

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

- [ ] **Step 1: Read current Sidebar.vue**

Confirm the file content: `Sidebar.vue` has no `collapsed` prop and no collapse CSS.

- [ ] **Step 2: Add `collapsed` prop**

Update the `<script setup>` section to add the prop:

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useSoftwareStore } from '@/stores/software';
import { usePluginStore } from '@/stores/plugin';
import { useSkillStore } from '@/stores/skill';
import { useMCPStore } from '@/stores/mcp';
import { useRuleStore } from '@/stores/rule';
import { useAgentStore } from '@/stores/agent';

defineProps<{
  collapsed?: boolean
}>();

const softwareStore = useSoftwareStore();
const pluginStore = usePluginStore();
const skillStore = useSkillStore();
const mcpStore = useMCPStore();
const ruleStore = useRuleStore();
const agentStore = useAgentStore();

const route = useRoute();

const appVersion = '0.1.0';

const isActive = (path: string) => {
  return route.path === path;
};

const cliToolsCount = computed(() => softwareStore.cliTools.length);
const softwareCount = computed(() => softwareStore.softwareList.length);
const pluginsCount = computed(() => pluginStore.plugins.length);
const skillsCount = computed(() => skillStore.skills.length);
const mcpCount = computed(() => mcpStore.services.length);
const rulesCount = computed(() => ruleStore.rules.length);
const agentsCount = computed(() => agentStore.agents.length);
</script>
```

- [ ] **Step 3: Update template to apply collapsed class**

Update the `<aside>` tag to bind the collapsed class:

```vue
<template>
  <aside class="sidebar" :class="{ collapsed }">
    <!-- ... rest of template unchanged ... -->
  </aside>
</template>
```

- [ ] **Step 4: Add collapse CSS**

Add the following CSS at the end of the `<style scoped>` section, before the closing `</style>`:

```css
/* Sidebar collapse state — matches prototype */
.sidebar.collapsed {
  width: 60px;
  min-width: 60px;
  overflow: hidden;
}

.sidebar.collapsed .sidebar-brand span {
  display: none;
}

.sidebar.collapsed .sidebar-nav {
  padding: 0 6px;
}

.sidebar.collapsed .nav-section-title {
  display: none;
}

.sidebar.collapsed .nav-item {
  justify-content: center;
  padding: 10px 0;
  border-left: none;
}

.sidebar.collapsed .nav-label {
  display: none;
}

.sidebar.collapsed .nav-item svg {
  margin-right: 0;
}

.sidebar.collapsed .nav-badge {
  display: none;
}

.sidebar.collapsed .sidebar-footer {
  padding: 12px 0;
  justify-content: center;
}

.sidebar.collapsed .sidebar-footer > div:not(.avatar) {
  display: none;
}

/* Responsive: at tablet width, auto-collapse */
@media (max-width: 1024px) {
  .sidebar {
    width: 60px;
    min-width: 60px;
    overflow: hidden;
  }

  .sidebar .sidebar-brand span {
    display: none;
  }

  .sidebar .sidebar-nav {
    padding: 0 6px;
  }

  .sidebar .nav-section-title,
  .sidebar .nav-label {
    display: none;
  }

  .sidebar .nav-item {
    justify-content: center;
    padding: 10px 0;
    border-left: none;
  }

  .sidebar .nav-item svg {
    margin-right: 0;
  }

  .sidebar .nav-badge {
    display: none;
  }

  .sidebar .sidebar-footer {
    padding: 12px 0;
    justify-content: center;
  }

  .sidebar .sidebar-footer > div:not(.avatar) {
    display: none;
  }

  /* Fully hide when collapsed at tablet */
  .sidebar.collapsed {
    width: 0;
    min-width: 0;
    padding: 0;
    border: none;
  }
}
```

- [ ] **Step 5: Run dev build to check for errors**

Run: `pnpm run build`
Expected: No compilation errors

- [ ] **Step 6: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "feat(layout): add collapse CSS and collapsed prop to Sidebar"
```

---

### Task 6: Replace inline toggle in SettingsView with ToggleSwitch

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: Read current SettingsView.vue**

Confirm the file has the inline toggle at line 57: `<div class="toggle on" data-setting="autoScan" @click="$event.target.classList.toggle('on')">`.

- [ ] **Step 2: Import ToggleSwitch and add state**

Update the `<script setup>` section:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useThemeStore } from '@/stores/theme'
import type { ThemeId } from '@/stores/theme'
import ToggleSwitch from '@/components/common/ToggleSwitch.vue'

const themeStore = useThemeStore()

// Settings state
const launchOnStartup = ref(true)

// Available themes — 20 themes matching prototype THEMES array
const availableThemes = [
  { id: 'warm' as ThemeId, name: 'Warm Glass', previewColors: ['#F5F3F0', '#F0EDE8', '#D2CAB8', '#2D2D2D', '#5A8A64', '#B8944A'] },
  { id: 'cool-mist' as ThemeId, name: 'Cool Mist', previewColors: ['#EEF1F5', '#DDE2EA', '#B8C4D4', '#1E2A3A', '#4A8A6A', '#6B7FAA'] },
  { id: 'midnight' as ThemeId, name: 'Midnight', previewColors: ['#1A1D24', '#242830', '#2E3340', '#E0E4EC', '#5A9A7A', '#7A8AAA'] },
  { id: 'sakura' as ThemeId, name: 'Sakura', previewColors: ['#F8F0F2', '#F0DDE2', '#E4B8C4', '#3A2028', '#8A5A6A', '#C47A8A'] },
  { id: 'sage' as ThemeId, name: 'Sage', previewColors: ['#EFF3EE', '#DAE4D8', '#B8CCB4', '#1E2A1E', '#5A8A5A', '#8AAA6A'] },
  { id: 'lavender' as ThemeId, name: 'Lavender', previewColors: ['#F2EFF8', '#E0DAF0', '#C4B8E0', '#2A2038', '#7A5AAA', '#A070C0'] },
  { id: 'ocean' as ThemeId, name: 'Ocean', previewColors: ['#ECF4F4', '#D4E8E8', '#A8D4D4', '#0A2828', '#2A8A8A', '#4A9AAA'] },
  { id: 'ember' as ThemeId, name: 'Ember', previewColors: ['#F8F2EC', '#F0E0CC', '#E0C8A0', '#382010', '#B87A3A', '#D49A4A'] },
  { id: 'slate' as ThemeId, name: 'Slate', previewColors: ['#F0F0EE', '#E0E0DC', '#C8C8C0', '#2A2A28', '#6A7A6A', '#8A8A7A'] },
  { id: 'aurora' as ThemeId, name: 'Aurora', previewColors: ['#F0F4F8', '#DAE4F0', '#A8C4E0', '#14202C', '#4A8ACC', '#8A6ACC'] },
  { id: 'cream' as ThemeId, name: 'Cream', previewColors: ['#FAF8F5', '#F2EDE6', '#E8DFD0', '#2C2820', '#7A8A5A', '#AA9A60'] },
  { id: 'arctic' as ThemeId, name: 'Arctic', previewColors: ['#F4F8FA', '#E4EEF4', '#C8DAE8', '#142430', '#3A7AAA', '#5A9ACC'] },
  { id: 'rose-gold' as ThemeId, name: 'Rose Gold', previewColors: ['#F8F2F4', '#F0DDE2', '#E0C0C8', '#301820', '#B85A70', '#D48A6A'] },
  { id: 'cyberpunk' as ThemeId, name: 'Cyberpunk', previewColors: ['#0E0E18', '#1A1A2E', '#2A2A40', '#E0E0F0', '#FF2E63', '#08D9D6'] },
  { id: 'forest' as ThemeId, name: 'Forest', previewColors: ['#F0F4EE', '#D8E4D0', '#B0C8A0', '#14200E', '#3A6A2A', '#5A8A4A'] },
  { id: 'desert' as ThemeId, name: 'Desert Sand', previewColors: ['#F5EEE4', '#E8D8C4', '#D0BC98', '#2C2014', '#A07848', '#C49A58'] },
  { id: 'cotton-candy' as ThemeId, name: 'Cotton Candy', previewColors: ['#F8F0F8', '#F0DAF0', '#E0C0E8', '#282030', '#C060A0', '#60A0C0'] },
  { id: 'charcoal' as ThemeId, name: 'Charcoal', previewColors: ['#181818', '#252525', '#333333', '#E8E8E8', '#6A8A6A', '#8A6A6A'] },
  { id: 'peach' as ThemeId, name: 'Peach Fuzz', previewColors: ['#FBF0E8', '#F5DCC8', '#E8C0A0', '#302018', '#D48A58', '#E8A070'] },
  { id: 'nordic' as ThemeId, name: 'Nordic', previewColors: ['#F4F6F8', '#E2E8EE', '#C8D4E0', '#1C2430', '#5A7A9A', '#8A6A5A'] },
]

function selectTheme(themeId: ThemeId) {
  themeStore.setTheme(themeId)
}
</script>
```

- [ ] **Step 3: Replace inline toggle in template**

Replace the "Launch on startup" setting row in the template:

```vue
<!-- Launch on startup -->
<div class="setting-row">
  <span class="setting-label">Launch on startup</span>
  <ToggleSwitch v-model="launchOnStartup" />
</div>
```

- [ ] **Step 4: Remove duplicate toggle CSS from SettingsView**

Remove the `.toggle`, `.toggle.on`, `.toggle::after`, and `.toggle.on::after` CSS rules from SettingsView's `<style scoped>` section (lines 278-309). The ToggleSwitch component now owns these styles.

- [ ] **Step 5: Run dev build to check for errors**

Run: `pnpm run build`
Expected: No compilation errors

- [ ] **Step 6: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "refactor(settings): replace inline toggle with ToggleSwitch component"
```

---

### Task 7: Run all tests and verify

- [ ] **Step 1: Run the full test suite**

Run: `pnpm test`
Expected: All tests pass

- [ ] **Step 2: Run lint**

Run: `pnpm run lint`
Expected: No errors

- [ ] **Step 3: Run build**

Run: `pnpm run build`
Expected: Build succeeds

- [ ] **Step 4: Manual verification checklist**

Open `pnpm run dev` and verify:
- [ ] Click hamburger icon in topbar → sidebar collapses to 60px
- [ ] Click hamburger icon again → sidebar expands to 240px
- [ ] Refresh page → sidebar state is preserved
- [ ] At <1024px viewport → sidebar auto-collapses
- [ ] Settings page → toggle switch works with v-model
- [ ] All existing navigation still works in collapsed state

- [ ] **Step 5: Final commit if needed**

```bash
git add -A
git commit -m "chore: collapse/expand system complete"
```

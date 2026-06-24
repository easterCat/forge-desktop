# Forge Glass UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Warm theme with the Cross-Platform Glass style from the prototype, implement a custom titlebar, rewrite all 11 view components and key sub-components with glassmorphism effects, add plugin CLI sync chips and agent install target chip interactions, and implement 5-8 theme variants.

**Architecture:** The redesign touches three layers: (1) CSS variables in `theme.css` for the Warm theme overhaul + 7 new glass variants, (2) layout components (App.vue shell, Titlebar, Sidebar, Topbar) for the new window frame structure, and (3) view components for glass-styled cards, sync chips, and target selectors. All Pinia stores, TypeScript types, router config, and Tauri backend code remain untouched.

**Tech Stack:** Vue 3.5 + TypeScript + Composition API, Tauri 2.0, CSS custom properties, Pinia 3.x

**Spec:** `docs/superpowers/specs/2026-06-18-forge-glass-ui-redesign-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/components/layout/Titlebar.vue` | Custom cross-platform titlebar with Mac traffic lights / Windows buttons |
| `src/composables/useTitlebar.ts` | Window minimize/maximize/close/drag logic via Tauri API |
| `src/composables/useGlassTheme.ts` | Glass theme variant management (8 variants), replaces useTheme for warm themes |

### Modified Files (grouped by layer)

**Theme Layer:**
| File | Change |
|------|--------|
| `src/assets/theme.css` | Replace `[data-theme="warm"]` variables + add glass global styles + add 7 variant selectors |
| `src/composables/useTheme.ts` | Extend Theme type to include glass variants, update THEME_ORDER |

**Layout Layer:**
| File | Change |
|------|--------|
| `src/App.vue` | Rewrite shell to window-frame structure, add Titlebar import |
| `src/components/layout/Sidebar.vue` | Rewrite template + add scoped styles for glass sidebar |
| `src/components/layout/Topbar.vue` | Rewrite template + add scoped styles for glass topbar |

**View Layer:**
| File | Change |
|------|--------|
| `src/views/DashboardView.vue` | Rewrite stat cards with tint variants + drift/sweep animations |
| `src/views/CliToolsView.vue` | Rewrite tool cards with glass layout |
| `src/views/SoftwareManagementView.vue` | Rewrite with glass card styles |
| `src/views/PluginsView.vue` | Rewrite with CLI sync chips in installed tab |
| `src/views/SkillsView.vue` | Rewrite with glass styles |
| `src/views/AgentsView.vue` | Rewrite with install target chip selector |
| `src/views/MCPView.vue` | Rewrite with glass styles |
| `src/views/RulesView.vue` | Rewrite with glass styles |
| `src/views/BackupView.vue` | Rewrite with glass styles |
| `src/views/SettingsView.vue` | Rewrite + add theme variant selector grid |
| `src/views/PromptManagerView.vue` | Update scoped styles for glass appearance |

**Component Layer:**
| File | Change |
|------|--------|
| `src/components/plugins/PluginCard.vue` | Rewrite with CLI sync chip row |
| `src/components/plugins/PluginDetailsDialog.vue` | Update scoped styles |
| `src/components/plugins/AddRepoSourceDialog.vue` | Update scoped styles |
| `src/components/plugins/SourceNoteDialog.vue` | Update scoped styles |
| `src/components/skills/SkillCard.vue` | Update scoped styles |
| `src/components/skills/SkillDetailsDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPServerCard.vue` | Update scoped styles |
| `src/components/mcp/MCPDetailsDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPInstallDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPServerFormDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPExportDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPImportDialog.vue` | Update scoped styles |
| `src/components/mcp/MCPAuditLogTable.vue` | Update scoped styles |
| `src/components/mcp/MCPGroupsPanel.vue` | Update scoped styles |
| `src/components/mcp/MCPInvocationDialog.vue` | Update scoped styles |
| `src/components/agents/AgentCard.vue` | Rewrite with install target chips |
| `src/components/agents/AgentDetailsDialog.vue` | Update scoped styles |
| `src/components/agents/AgentImportDialog.vue` | Update scoped styles |
| `src/components/common/VirtualGrid.vue` | Update scoped styles |

**Config:**
| File | Change |
|------|--------|
| `src-tauri/tauri.conf.json` | Add `"decorations": false` |

---

## Task 1: Theme CSS Variables Overhaul

**Files:**
- Modify: `src/assets/theme.css:179-293`

**Context:** The `[data-theme="warm"]` section starts at line 179 and ends around line 293. It defines all CSS custom properties for the Warm theme. We need to replace these values with the prototype's Cross-Platform Glass values and add new glass-specific variables.

- [ ] **Step 1: Read current warm theme section**

Run: Read `src/assets/theme.css` lines 179-293 to get the exact current content.

- [ ] **Step 2: Replace warm theme CSS variables**

Replace the entire `[data-theme="warm"]` block with the new Cross-Platform Glass values. The new block should contain:

```css
[data-theme="warm"] {
  /* === Background Colors === */
  --bg: #E8E4DE;
  --bg-card: rgba(255, 255, 255, 0.42);
  --bg-card-hover: rgba(255, 255, 255, 0.58);
  --bg-input: rgba(255, 255, 255, 0.25);
  --bg-input-focus: rgba(255, 255, 255, 0.45);
  --bg-sidebar: rgba(255, 255, 255, 0.15);
  --bg-topbar: rgba(255, 255, 255, 0.15);

  /* Legacy aliases */
  --bg-primary: #E8E4DE;
  --bg-secondary: #EDEAE6;
  --bg-tertiary: #E2DED6;

  /* === Border Colors === */
  --border: rgba(255, 255, 255, 0.15);
  --border-hover: rgba(255, 255, 255, 0.28);

  /* === Text Colors === */
  --fg: #1A1A1A;
  --fg-title: #111111;
  --fg-muted: #5C5C5C;
  --fg-ghost: #9A9A9A;
  --fg-white: #FFFFFF;

  /* === Accent Colors === */
  --accent: #2D2D2D;
  --accent-hover: #1A1A1A;
  --accent-press: #0D0D0D;
  --accent-bg: rgba(45, 45, 45, 0.08);

  /* === Semantic Colors === */
  --success: #5A8A64;
  --success-bg: rgba(90, 138, 100, 0.12);
  --error: #B85A42;
  --error-bg: rgba(184, 90, 66, 0.12);
  --info: #5A6B7A;
  --info-bg: rgba(90, 107, 122, 0.12);
  --warn: #B8944A;
  --warn-bg: rgba(184, 148, 74, 0.12);

  /* === Border Radius === */
  --radius: 18px;
  --radius-xs: 6px;
  --radius-sm: 12px;
  --radius-md: 16px;
  --radius-lg: 24px;
  --radius-xl: 28px;

  /* === Layout === */
  --sidebar-w: 240px;
  --topbar-h: 64px;
  --titlebar-h: 38px;
  --content-padding: 24px 32px;
  --card-padding: 20px;

  /* === Card Tokens === */
  --card-padding-sm: 12px;
  --card-padding-md: 16px;
  --card-padding-lg: 24px;
  --card-radius: 18px;

  /* === Shadows === */
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.04);
  --shadow: 0 2px 16px rgba(0, 0, 0, 0.04);
  --shadow-md: 0 4px 20px rgba(0, 0, 0, 0.05);
  --shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.07);
  --shadow-lg: 0 16px 48px rgba(0, 0, 0, 0.10), 0 4px 16px rgba(0, 0, 0, 0.04);

  /* === Glass-Specific Variables === */
  --glass-bg: rgba(255, 255, 255, 0.28);
  --glass-bg-hover: rgba(255, 255, 255, 0.42);
  --glass-sidebar: rgba(255, 255, 255, 0.18);
  --glass-topbar: rgba(255, 255, 255, 0.20);
  --glass-input: rgba(255, 255, 255, 0.25);
  --glass-input-focus: rgba(255, 255, 255, 0.45);
  --glass-window: rgba(255, 255, 255, 0.12);
  --glass-inner-glow: rgba(255, 255, 255, 0.70);
  --glass-highlight: rgba(255, 255, 255, 0.85);
  --border-window: rgba(255, 255, 255, 0.12);
  --border-outer-glow: rgba(255, 255, 255, 0.10);
  --shadow-inner: inset 0 1px 2px rgba(0, 0, 0, 0.03);
  --tint-warm: rgba(200, 190, 175, 0.15);
  --tint-cool: rgba(180, 185, 195, 0.12);
  --tint-soft: rgba(220, 215, 208, 0.18);
  --tint-amber: rgba(184, 148, 74, 0.12);

  /* === Button Heights === */
  --btn-height-sm: 32px;
  --btn-height-md: 40px;
  --btn-height-lg: 48px;
  --btn-height-icon: 34px;
}
```

- [ ] **Step 3: Add warm theme body background and glass global styles**

After the `[data-theme="warm"]` variable block, add the body background system and glass component overrides:

```css
/* === Warm Glass: Body Background === */
[data-theme="warm"] body {
  background: var(--bg);
  background-image:
    linear-gradient(160deg, #DDD8D0 0%, #E5E1DA 25%, #EBE7E0 50%, #E2DED6 75%, #D9D4CC 100%),
    radial-gradient(ellipse 120% 80% at 0% 0%, rgba(190, 175, 155, 0.20) 0%, transparent 50%),
    radial-gradient(ellipse 100% 80% at 100% 100%, rgba(160, 170, 190, 0.18) 0%, transparent 50%),
    radial-gradient(ellipse 80% 60% at 60% 30%, rgba(200, 195, 185, 0.12) 0%, transparent 45%),
    linear-gradient(rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.025) 1px, transparent 1px),
    linear-gradient(rgba(0, 0, 0, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 0, 0, 0.015) 1px, transparent 1px);
  background-size: 100% 100%, 100% 100%, 100% 100%, 100% 100%, 24px 24px, 24px 24px, 96px 96px, 96px 96px;
  background-attachment: fixed;
}

[data-theme="warm"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background:
    radial-gradient(ellipse 70% 50% at 10% 15%, rgba(185, 170, 150, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 50% at 90% 80%, rgba(155, 165, 185, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse 50% 40% at 50% 50%, rgba(200, 195, 185, 0.08) 0%, transparent 45%);
}

[data-theme="warm"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.30;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.75' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.06'/%3E%3C/svg%3E");
  background-size: 180px 180px;
}

/* === Warm Glass: Window Frame === */
[data-theme="warm"] .window-frame {
  position: fixed;
  inset: 0;
  background: rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(40px) saturate(1.3);
  -webkit-backdrop-filter: blur(40px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 16px;
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* === Warm Glass: Card === */
[data-theme="warm"] .card {
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
  transition: all var(--transition-base);
}

[data-theme="warm"] .card:hover {
  background: var(--bg-card-hover);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

/* === Warm Glass: Sidebar === */
[data-theme="warm"] .sidebar {
  background: var(--glass-sidebar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-right: 1px solid rgba(255, 255, 255, 0.22);
}

/* === Warm Glass: Topbar === */
[data-theme="warm"] .topbar {
  height: var(--topbar-h);
  min-height: var(--topbar-h);
  background: var(--glass-topbar);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-bottom: 1px solid rgba(255, 255, 255, 0.22);
}

/* === Warm Glass: Inputs === */
[data-theme="warm"] input,
[data-theme="warm"] textarea,
[data-theme="warm"] select {
  background: var(--glass-input);
  backdrop-filter: blur(16px) saturate(1.2);
  -webkit-backdrop-filter: blur(16px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: var(--radius-sm);
}

[data-theme="warm"] input:focus,
[data-theme="warm"] textarea:focus,
[data-theme="warm"] select:focus {
  background: var(--glass-input-focus);
  border-color: rgba(255, 255, 255, 0.30);
  box-shadow: 0 0 0 2px rgba(45, 45, 45, 0.12);
}

/* === Warm Glass: Modal === */
[data-theme="warm"] .modal-overlay {
  background: rgba(0, 0, 0, 0.22);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .modal {
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
}

/* === Warm Glass: Toast === */
[data-theme="warm"] .toast {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(30px) saturate(1.3);
  -webkit-backdrop-filter: blur(30px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.30);
  border-radius: var(--radius);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.10), inset 0 1px 0 rgba(255, 255, 255, 0.40);
}

/* === Warm Glass: Buttons === */
[data-theme="warm"] .btn-primary {
  background: rgba(45, 45, 45, 0.85);
  color: #fff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

[data-theme="warm"] .btn-primary:hover {
  background: rgba(26, 26, 26, 0.90);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

[data-theme="warm"] .btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

[data-theme="warm"] .btn-secondary:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

/* === Warm Glass: Badges === */
[data-theme="warm"] .badge.success {
  background: rgba(90, 138, 100, 0.15);
  border: 1px solid rgba(90, 138, 100, 0.20);
  color: var(--success);
}

[data-theme="warm"] .badge.error {
  background: rgba(184, 90, 66, 0.15);
  border: 1px solid rgba(184, 90, 66, 0.20);
  color: var(--error);
}

[data-theme="warm"] .badge.warn {
  background: rgba(184, 148, 74, 0.15);
  border: 1px solid rgba(184, 148, 74, 0.20);
  color: var(--warn);
}

[data-theme="warm"] .badge.info {
  background: rgba(90, 107, 122, 0.15);
  border: 1px solid rgba(90, 107, 122, 0.20);
  color: var(--info);
}

/* === Warm Glass: Toggle === */
[data-theme="warm"] .toggle {
  background: rgba(255, 255, 255, 0.22);
  border: 1px solid rgba(255, 255, 255, 0.12);
}

[data-theme="warm"] .toggle.on {
  background: var(--accent);
}

/* === Warm Glass: Stat Cards with Tint Animations === */
@keyframes tint-drift {
  0% { transform: translate(0, 0) }
  33% { transform: translate(8%, 5%) }
  66% { transform: translate(-5%, 8%) }
  100% { transform: translate(0, 0) }
}

@keyframes tint-sweep {
  0% { transform: translateX(0) }
  50% { transform: translateX(33%) }
  100% { transform: translateX(0) }
}

[data-theme="warm"] .stat-card {
  background: rgba(255, 255, 255, 0.62);
  backdrop-filter: blur(24px) saturate(1.3);
  -webkit-backdrop-filter: blur(24px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.55);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 16px rgba(0, 0, 0, 0.03), inset 0 1px 0 rgba(255, 255, 255, 0.60);
  position: relative;
  overflow: hidden;
}

[data-theme="warm"] .stat-card:hover {
  background: rgba(255, 255, 255, 0.78);
  border-color: rgba(255, 255, 255, 0.70);
  transform: translateY(-1px);
}

[data-theme="warm"] .stat-card.tint-warm::before {
  content: '';
  position: absolute;
  top: -80%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(ellipse at center, rgba(210, 190, 160, 0.12) 0%, rgba(210, 190, 160, 0.04) 30%, transparent 60%);
  animation: tint-drift 8s ease-in-out infinite;
  pointer-events: none;
}

[data-theme="warm"] .stat-card.tint-cool::before {
  content: '';
  position: absolute;
  top: -80%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(ellipse at center, rgba(155, 170, 210, 0.12) 0%, rgba(155, 170, 210, 0.04) 30%, transparent 60%);
  animation: tint-drift 10s ease-in-out infinite;
  animation-delay: -3s;
  pointer-events: none;
}

[data-theme="warm"] .stat-card.tint-soft::before {
  content: '';
  position: absolute;
  top: -80%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(ellipse at center, rgba(195, 185, 170, 0.12) 0%, rgba(195, 185, 170, 0.04) 30%, transparent 60%);
  animation: tint-drift 9s ease-in-out infinite;
  animation-delay: -5s;
  pointer-events: none;
}

[data-theme="warm"] .stat-card.tint-amber::before {
  content: '';
  position: absolute;
  top: -80%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(ellipse at center, rgba(210, 175, 75, 0.10) 0%, rgba(210, 175, 75, 0.03) 30%, transparent 60%);
  animation: tint-drift 7s ease-in-out infinite;
  animation-delay: -2s;
  pointer-events: none;
}

[data-theme="warm"] .stat-card.tint-warm::after,
[data-theme="warm"] .stat-card.tint-cool::after,
[data-theme="warm"] .stat-card.tint-soft::after,
[data-theme="warm"] .stat-card.tint-amber::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 300%;
  height: 2px;
  background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0) 10%, rgba(255, 255, 255, 0.40) 30%, rgba(255, 255, 255, 0.60) 50%, rgba(255, 255, 255, 0.40) 70%, rgba(255, 255, 255, 0) 90%, transparent 100%);
  animation: tint-sweep 5s ease-in-out infinite;
  pointer-events: none;
}

[data-theme="warm"] .stat-card.tint-cool::after { animation-duration: 6s; animation-delay: -2s }
[data-theme="warm"] .stat-card.tint-soft::after { animation-duration: 7s; animation-delay: -4s }
[data-theme="warm"] .stat-card.tint-amber::after { animation-duration: 4.5s; animation-delay: -1s }

/* === Warm Glass: Dropdown === */
[data-theme="warm"] .dropdown-menu {
  background: rgba(255, 255, 255, 0.45);
  backdrop-filter: blur(30px) saturate(1.3);
  -webkit-backdrop-filter: blur(30px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.30);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.10), inset 0 1px 0 rgba(255, 255, 255, 0.40);
}

/* === Warm Glass: Scrollbar === */
[data-theme="warm"] ::-webkit-scrollbar { width: 6px; height: 6px }
[data-theme="warm"] ::-webkit-scrollbar-track { background: transparent }
[data-theme="warm"] ::-webkit-scrollbar-thumb { background: rgba(0, 0, 0, 0.10); border-radius: 3px }
[data-theme="warm"] ::-webkit-scrollbar-thumb:hover { background: rgba(0, 0, 0, 0.18) }

/* === Warm Glass: Selection === */
[data-theme="warm"] ::selection { background: rgba(45, 45, 45, 0.10) }
```

- [ ] **Step 4: Verify theme compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors related to theme.css (CSS files don't cause TS errors, but verify no import issues)

- [ ] **Step 5: Commit**

```bash
git add src/assets/theme.css
git commit -m "feat(theme): replace Warm theme with Cross-Platform Glass style

Replace all CSS variables in [data-theme='warm'] with prototype values.
Add body background system with multi-layer gradients, grid texture, and noise overlay.
Add glass component overrides for cards, sidebar, topbar, inputs, modals, buttons.
Add stat card tint-drift and tint-sweep animations."
```

---

## Task 2: New Composables (useTitlebar + useGlassTheme)

**Files:**
- Create: `src/composables/useTitlebar.ts`
- Create: `src/composables/useGlassTheme.ts`
- Modify: `src/composables/useTheme.ts`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Create useTitlebar.ts**

```typescript
// src/composables/useTitlebar.ts
import { ref, onMounted } from 'vue'

export type Platform = 'macos' | 'windows' | 'linux'

const currentPlatform = ref<Platform>('macos')

export function useTitlebar() {
  async function detectPlatform() {
    try {
      const { platform } = await import('@tauri-apps/plugin-os')
      const p = await platform()
      if (p === 'macos') currentPlatform.value = 'macos'
      else if (p === 'windows') currentPlatform.value = 'windows'
      else currentPlatform.value = 'linux'
    } catch {
      // Fallback for dev mode without Tauri
      const ua = navigator.userAgent.toLowerCase()
      if (ua.includes('mac')) currentPlatform.value = 'macos'
      else if (ua.includes('win')) currentPlatform.value = 'windows'
      else currentPlatform.value = 'linux'
    }
  }

  async function minimize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().minimize()
    } catch { /* dev mode */ }
  }

  async function toggleMaximize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().toggleMaximize()
    } catch { /* dev mode */ }
  }

  async function close() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch { /* dev mode */ }
  }

  onMounted(() => {
    detectPlatform()
  })

  return {
    platform: currentPlatform,
    minimize,
    toggleMaximize,
    close,
  }
}
```

- [ ] **Step 2: Create useGlassTheme.ts**

```typescript
// src/composables/useGlassTheme.ts
import { computed } from 'vue'
import { currentTheme, setTheme, type Theme } from './useTheme'

export type GlassVariant =
  | 'warm'
  | 'warm-glass'
  | 'cool-mist'
  | 'sage'
  | 'lavender'
  | 'ember'
  | 'slate'
  | 'arctic'
  | 'rose-gold'

export interface ThemeVariantInfo {
  id: GlassVariant
  name: string
  desc: string
  colors: [string, string, string, string, string, string] // bg, surface, border, fg, accent, warn
}

export const GLASS_VARIANTS: ThemeVariantInfo[] = [
  { id: 'warm', name: 'Warm Glass', desc: 'Default · Frosted warm', colors: ['#E8E4DE', '#DDD8D0', '#D2CAB8', '#1A1A1A', '#2D2D2D', '#B8944A'] },
  { id: 'cool-mist', name: 'Cool Mist', desc: 'Blue-grey frost', colors: ['#EEF1F5', '#DDE2EA', '#B8C4D4', '#1E2A3A', '#4A8A6A', '#6B7FAA'] },
  { id: 'sage', name: 'Sage', desc: 'Muted green glass', colors: ['#EFF3EE', '#DAE4D8', '#B8CCB4', '#1E2A1E', '#5A8A5A', '#8AAA6A'] },
  { id: 'lavender', name: 'Lavender', desc: 'Purple frosted', colors: ['#F2EFF8', '#E0DAF0', '#C4B8E0', '#2A2038', '#7A5AAA', '#A070C0'] },
  { id: 'ember', name: 'Ember', desc: 'Warm amber glow', colors: ['#F8F2EC', '#F0E0CC', '#E0C8A0', '#382010', '#B87A3A', '#D49A4A'] },
  { id: 'slate', name: 'Slate', desc: 'Neutral stone', colors: ['#F0F0EE', '#E0E0DC', '#C8C8C0', '#2A2A28', '#6A7A6A', '#8A8A7A'] },
  { id: 'arctic', name: 'Arctic', desc: 'Cool blue-white', colors: ['#F4F8FA', '#E4EEF4', '#C8DAE8', '#142430', '#3A7AAA', '#5A9ACC'] },
  { id: 'rose-gold', name: 'Rose Gold', desc: 'Metallic warm pink', colors: ['#F8F2F4', '#F0DDE2', '#E0C0C8', '#301820', '#B85A70', '#D48A6A'] },
]

export function useGlassTheme() {
  const isGlassTheme = computed(() => {
    const t = currentTheme.value
    return t === 'warm' || t === 'glass'
  })

  const isWindowRounded = computed(() => currentTheme.value === 'warm')

  function selectGlassVariant(variantId: GlassVariant) {
    if (variantId === 'warm') {
      setTheme('warm' as Theme)
      return
    }
    // For non-warm variants, apply via dynamic CSS variable overrides
    const variant = GLASS_VARIANTS.find(v => v.id === variantId)
    if (!variant) return

    setTheme('warm' as Theme)
    // Apply variant color overrides on top of warm theme
    const [bg, surface, border, fg, accent, warn] = variant.colors
    const r = document.documentElement.style
    r.setProperty('--bg', bg)
    r.setProperty('--bg-primary', bg)
    r.setProperty('--bg-secondary', surface)
    r.setProperty('--fg', fg)
    r.setProperty('--fg-title', fg)
    r.setProperty('--fg-muted', fg === '#1A1A1A' ? '#5C5C5C' : 'rgba(255,255,255,0.6)')
    r.setProperty('--fg-ghost', fg === '#1A1A1A' ? '#9A9A9A' : 'rgba(255,255,255,0.35)')
    r.setProperty('--accent', accent)
    r.setProperty('--accent-hover', accent + 'dd')
    r.setProperty('--accent-press', accent + 'bb')
    r.setProperty('--accent-bg', accent + '22')
    r.setProperty('--warn', warn)

    // Update body background gradient
    const isDark = parseInt(fg.slice(1, 3), 16) > parseInt(bg.slice(1, 3), 16)
    document.body.style.background = isDark
      ? `linear-gradient(180deg, ${bg} 0%, ${bg} 50%, ${bg} 100%)`
      : `linear-gradient(160deg, ${surface} 0%, ${bg} 25%, ${bg}ee 50%, ${surface} 75%, ${bg} 100%)`

    localStorage.setItem('aem-glass-variant', variantId)
  }

  function restoreVariant() {
    const saved = localStorage.getItem('aem-glass-variant') as GlassVariant | null
    if (saved && saved !== 'warm') {
      // Defer until after theme is applied
      setTimeout(() => selectGlassVariant(saved), 50)
    }
  }

  return {
    isGlassTheme,
    isWindowRounded,
    selectGlassVariant,
    restoreVariant,
    variants: GLASS_VARIANTS,
  }
}
```

- [ ] **Step 3: Update useTheme.ts to extend Theme type**

Read the current file, then update the Theme type to include glass variants (the variants are applied via dynamic CSS overrides, so the base Theme type stays as-is for the data-theme attribute, but we need to add the glass variant names to THEME_LABELS):

```typescript
// src/composables/useTheme.ts
export type Theme = 'light' | 'dark' | 'warm' | 'glass' | 'yellow'

export const THEME_ORDER: Theme[] = ['light', 'dark', 'warm', 'glass', 'yellow']

export const THEME_LABELS: Record<Theme, string> = {
  light: 'Light',
  dark: 'Dark',
  warm: 'Warm Glass',
  glass: 'Glass',
  yellow: 'Yellow',
}
```

The only change is updating the label for `warm` from whatever it currently is to `'Warm Glass'`. Keep all other logic identical.

- [ ] **Step 4: Update tauri.conf.json**

Read `src-tauri/tauri.conf.json`, then add `"decorations": false` to the window configuration. The window section should become:

```json
"windows": [
  {
    "title": "Forge",
    "width": 1200,
    "height": 800,
    "minWidth": 900,
    "minHeight": 600,
    "resizable": true,
    "fullscreen": false,
    "center": true,
    "decorations": false
  }
]
```

- [ ] **Step 5: Verify TypeScript compiles**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/composables/useTitlebar.ts src/composables/useGlassTheme.ts src/composables/useTheme.ts src-tauri/tauri.conf.json
git commit -m "feat: add useTitlebar and useGlassTheme composables

Add useTitlebar for custom window controls (minimize/maximize/close).
Add useGlassTheme for 8 glass theme variant management.
Set decorations: false in tauri.conf.json for custom titlebar."
```

---

## Task 3: Titlebar Component

**Files:**
- Create: `src/components/layout/Titlebar.vue`

- [ ] **Step 1: Create Titlebar.vue**

```vue
<!-- src/components/layout/Titlebar.vue -->
<script setup lang="ts">
import { useTitlebar } from '@/composables/useTitlebar'
import { useGlassTheme } from '@/composables/useGlassTheme'

const { platform, minimize, toggleMaximize, close } = useTitlebar()
const { isWindowRounded } = useGlassTheme()
</script>

<template>
  <div class="titlebar" :class="{ rounded: isWindowRounded }">
    <!-- Mac traffic lights -->
    <div v-if="platform === 'macos'" class="titlebar-mac">
      <div class="traffic-light close" title="Close" @click="close" />
      <div class="traffic-light minimize" title="Minimize" @click="minimize" />
      <div class="traffic-light maximize" title="Maximize" @click="toggleMaximize" />
    </div>

    <!-- Center: app icon + name -->
    <div class="titlebar-center">
      <div class="app-icon">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
          <rect x="4" y="4" width="16" height="16" rx="3" fill="rgba(255,255,255,0.8)" />
          <rect x="8" y="8" width="8" height="8" rx="1.5" fill="#F5F3F0" />
        </svg>
      </div>
      <span>Forge</span>
    </div>

    <!-- Windows/Linux buttons -->
    <div v-if="platform !== 'macos'" class="titlebar-win">
      <div class="win-btn minimize" title="Minimize" @click="minimize">
        <svg viewBox="0 0 10 1" fill="currentColor"><rect width="10" height="1" /></svg>
      </div>
      <div class="win-btn maximize" title="Maximize" @click="toggleMaximize">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1"><rect x="0.5" y="0.5" width="9" height="9" rx="1" /></svg>
      </div>
      <div class="win-btn close" title="Close" @click="close">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"><line x1="1" y1="1" x2="9" y2="9" /><line x1="9" y1="1" x2="1" y2="9" /></svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: var(--titlebar-h, 38px);
  min-height: var(--titlebar-h, 38px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  -webkit-app-region: drag;
  user-select: none;
  position: relative;
  z-index: 20;
}

/* Glass styles only apply in warm theme */
[data-theme="warm"] .titlebar {
  background: rgba(255, 255, 255, 0.10);
  border-bottom: 1px solid rgba(255, 255, 255, 0.22);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
}

.titlebar.rounded {
  /* Extra rounding for window frame */
}

.titlebar-mac {
  display: flex;
  align-items: center;
  gap: 8px;
}

.traffic-light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  position: relative;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.traffic-light::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 50%;
  opacity: 0;
  transition: opacity 150ms ease;
}

.traffic-light:hover::after {
  opacity: 1;
}

.traffic-light.close {
  background: #FF5F57;
}

.traffic-light.close::after {
  background: linear-gradient(180deg, #E0443E 0%, #C73E38 100%);
}

.traffic-light.minimize {
  background: #FEBC2E;
}

.traffic-light.minimize::after {
  background: linear-gradient(180deg, #DFA123 0%, #C49019 100%);
}

.traffic-light.maximize {
  background: #28C840;
}

.traffic-light.maximize::after {
  background: linear-gradient(180deg, #1AAB29 0%, #14982A 100%);
}

.titlebar-center {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted, #5C5C5C);
  display: flex;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.app-icon {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  background: var(--accent, #2D2D2D);
  display: flex;
  align-items: center;
  justify-content: center;
}

.titlebar-win {
  display: flex;
  align-items: center;
  gap: 2px;
}

.win-btn {
  width: 46px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  cursor: pointer;
  -webkit-app-region: no-drag;
  transition: background 150ms ease;
  color: var(--fg-muted, #5C5C5C);
}

.win-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

.win-btn.close:hover {
  background: rgba(184, 90, 66, 0.18);
  color: var(--error, #B85A42);
}

.win-btn svg {
  width: 10px;
  height: 10px;
}
</style>
```

- [ ] **Step 2: Verify component renders**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/Titlebar.vue
git commit -m "feat(layout): add custom cross-platform Titlebar component

Mac traffic lights + Windows/Linux buttons with glass styling.
Platform auto-detection via @tauri-apps/plugin-os."
```

---

## Task 4: App.vue Shell Rewrite

**Files:**
- Modify: `src/App.vue`

**Context:** Current App.vue has a simple `<div class="shell">` with sidebar + main. We need to wrap it in a `.window-frame` div and add the Titlebar component. The toast notification and provide/inject logic must be preserved.

- [ ] **Step 1: Read current App.vue**

Read the full file to understand the exact structure.

- [ ] **Step 2: Rewrite App.vue template**

The template should become:

```vue
<template>
  <div class="window-frame" :class="{ 'window-rounded': isWindowRounded }">
    <AppTitlebar />
    <div class="shell">
      <AppSidebar />
      <div class="main">
        <AppTopbar
          :title="currentTitle"
          :subtitle="currentSubtitle"
          @refresh="handleRefresh"
          @settings="router.push('/settings')"
        />
        <div class="content">
          <router-view v-slot="{ Component }">
            <Transition name="fade" mode="out-in">
              <component :is="Component" />
            </Transition>
          </router-view>
        </div>
      </div>
    </div>

    <!-- Toast Notification -->
    <Transition name="toast">
      <div
        v-if="toastVisible"
        class="toast"
        :class="toastType"
      >
        <span v-html="toastIcon"></span>
        <span>{{ toastMessage }}</span>
      </div>
    </Transition>
  </div>
</template>
```

- [ ] **Step 3: Update script section**

Add the Titlebar import and glass theme composable:

```vue
<script setup lang="ts">
import { ref, computed, provide } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import AppSidebar from '@/components/layout/Sidebar.vue'
import AppTopbar from '@/components/layout/Topbar.vue'
import AppTitlebar from '@/components/layout/Titlebar.vue'
import { useGlassTheme } from '@/composables/useGlassTheme'
import '@/assets/theme.css'

const router = useRouter()
const route = useRoute()
const { isWindowRounded, restoreVariant } = useGlassTheme()

// ... keep existing pageTitles, currentTitle, currentSubtitle, toast logic, showNotification, handleRefresh ...
// ... keep existing provide('showNotification', showNotification) ...

// Restore glass variant on mount
restoreVariant()
</script>
```

- [ ] **Step 4: Update scoped styles**

Add window-frame styles to the scoped style block:

```vue
<style scoped>
.window-frame {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.window-frame.window-rounded {
  border-radius: 16px;
  /* Glass background and border are handled by theme.css */
}

.shell {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: var(--content-padding);
  position: relative;
}

.content > * {
  position: relative;
  z-index: 1;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.toast-enter-active {
  transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-leave-active {
  transition: all 200ms ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/App.vue
git commit -m "feat(layout): rewrite App.vue shell with window-frame structure

Add Titlebar component. Wrap layout in .window-frame with optional
rounded corners for warm theme. Preserve toast and provide/inject."
```

---

## Task 5: Sidebar Rewrite

**Files:**
- Modify: `src/components/layout/Sidebar.vue`

**Context:** Current Sidebar has no scoped styles (relies entirely on theme.css global classes). The prototype adds a sidebar search input, updates nav item styles, and changes the footer. We need to rewrite the template and add scoped styles.

- [ ] **Step 1: Read current Sidebar.vue**

Read the full file to understand all nav items, badge counts, and theme toggle logic.

- [ ] **Step 2: Rewrite Sidebar.vue template**

Key changes:
- Keep `.sidebar-brand` with SVG logo (update colors for warm theme)
- Keep `.sidebar-search` input (already exists)
- Keep all nav items with their badge counts (all 11 routes)
- Remove theme toggle button from footer (moved to Settings)
- Keep footer with user avatar + version info
- All existing store imports and badge count logic must be preserved

The template structure should match the prototype:

```vue
<template>
  <aside class="sidebar">
    <div class="sidebar-brand">
      <svg><!-- logo --></svg>
      <span>Forge</span>
    </div>

    <div class="sidebar-search">
      <div class="sidebar-search-wrap">
        <svg><!-- search icon --></svg>
        <input v-model="searchQuery" type="text" placeholder="Search…" />
      </div>
    </div>

    <nav class="sidebar-nav">
      <div class="nav-section">
        <div class="nav-section-title">Overview</div>
        <router-link to="/" class="nav-item" active-class="active" :class="{ active: $route.path === '/' }">
          <svg><!-- dashboard icon --></svg>
          Dashboard
        </router-link>
      </div>

      <div class="nav-section">
        <div class="nav-section-title">Manage</div>
        <!-- All manage nav items with badges -->
      </div>

      <div class="nav-section">
        <div class="nav-section-title">Data</div>
        <!-- Backup & Import/Export nav items -->
      </div>
    </nav>

    <div class="sidebar-footer">
      <div class="avatar">R</div>
      <div>
        <div class="user-name">rhino</div>
        <div class="user-status">Local only · v0.1.0</div>
      </div>
    </div>
  </aside>
</template>
```

- [ ] **Step 3: Add scoped styles for glass sidebar**

```vue
<style scoped>
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  display: flex;
  flex-direction: column;
  z-index: 10;
}

/* Warm theme glass styles are in theme.css global overrides */

.sidebar-brand {
  height: var(--topbar-h, 64px);
  min-height: var(--topbar-h, 64px);
  padding: 0 16px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.sidebar-brand span {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--fg-title, var(--fg));
}

.sidebar-search {
  padding: 0 12px 12px;
}

.sidebar-search-wrap {
  position: relative;
}

.sidebar-search-wrap svg {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.sidebar-search input {
  width: 100%;
  padding: 8px 10px 8px 34px;
  font-size: 13px;
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
}

.nav-section {
  margin-bottom: 4px;
}

.nav-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--fg-ghost);
  padding: 16px 12px 6px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  color: var(--fg-muted);
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: all 150ms ease;
  text-decoration: none;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.18);
  color: var(--fg);
}

.nav-item.active {
  background: rgba(255, 255, 255, 0.18);
  color: var(--fg-title, var(--fg));
  border-left-color: var(--accent);
}

.nav-item svg {
  flex-shrink: 0;
  opacity: 0.5;
}

.nav-item.active svg {
  opacity: 1;
}

.nav-badge {
  margin-left: auto;
  font-size: 11px;
  font-weight: 600;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.sidebar-footer {
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: var(--fg-ghost);
}

.avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 11px;
}

[data-theme="warm"] .avatar {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: var(--accent);
}

.user-name {
  font-size: 12px;
  color: var(--fg-muted);
  font-weight: 500;
}

.user-status {
  font-size: 10px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}
</style>
```

- [ ] **Step 4: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/components/layout/Sidebar.vue
git commit -m "feat(layout): rewrite Sidebar with glass styling

Add sidebar search input styling, update nav items with 3px active
border, add user avatar footer. Remove theme toggle (moved to Settings)."
```

---

## Task 6: Topbar Rewrite

**Files:**
- Modify: `src/components/layout/Topbar.vue`

**Context:** Current Topbar is 43 lines with no scoped styles. The prototype changes height to 64px, adds a global search input, and updates button styles.

- [ ] **Step 1: Rewrite Topbar.vue**

```vue
<!-- src/components/layout/Topbar.vue -->
<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()

defineEmits<{
  refresh: []
  settings: []
}>()
</script>

<template>
  <div class="topbar">
    <span class="topbar-title">{{ title }}</span>
    <span v-if="subtitle" class="topbar-subtitle">{{ subtitle }}</span>
    <div class="topbar-spacer" />
    <div class="topbar-search">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#9A9A9A" stroke-width="2" stroke-linecap="round">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input type="text" placeholder="Search configs, plugins, rules…" />
    </div>
    <button class="topbar-btn secondary" @click="$emit('settings')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    </button>
    <button class="topbar-btn primary" @click="$emit('refresh')">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
        <polyline points="23 4 23 10 17 10" />
        <polyline points="1 20 1 14 7 14" />
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
      </svg>
      Refresh
    </button>
  </div>
</template>

<style scoped>
.topbar {
  height: var(--topbar-h, 64px);
  min-height: var(--topbar-h, 64px);
  display: flex;
  align-items: center;
  padding: 0 24px;
  gap: 16px;
}

.topbar-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--fg-title, var(--fg));
  letter-spacing: -0.02em;
}

.topbar-subtitle {
  font-size: 12px;
  color: var(--fg-ghost);
  font-family: var(--font-mono);
}

.topbar-spacer {
  flex: 1;
}

.topbar-search {
  position: relative;
  flex: 1;
  max-width: 320px;
}

.topbar-search svg {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.topbar-search input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  font-size: 13px;
}

.topbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  transition: all 150ms ease;
  cursor: pointer;
  border: none;
  background: none;
  font-family: inherit;
}

.topbar-btn.secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.topbar-btn.secondary:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

.topbar-btn.primary {
  background: rgba(45, 45, 45, 0.85);
  color: #fff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.topbar-btn.primary:hover {
  background: rgba(26, 26, 26, 0.90);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.topbar-btn.primary:active {
  background: rgba(13, 13, 13, 0.95);
  transform: translateY(0);
}
</style>
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/components/layout/Topbar.vue
git commit -m "feat(layout): rewrite Topbar with glass styling and search input

Height 56px -> 64px. Add global search input with icon. Update button
styles with glass backgrounds. Add subtitle display."
```

---

## Task 7: DashboardView Rewrite

**Files:**
- Modify: `src/views/DashboardView.vue`

**Context:** This view has stat cards and a CLI tools grid. We need to rewrite the stat cards with tint variants (tint-warm, tint-cool, tint-soft, tint-amber) and update the tool card styles. All existing Tauri invoke calls and data logic must be preserved.

- [ ] **Step 1: Read current DashboardView.vue**

Read the full file to understand the data fetching logic, stat card rendering, and tool card rendering.

- [ ] **Step 2: Update stat card template**

Replace the `.stats-row` section. Each stat card should use the new tint classes:

```vue
<div class="stats-row">
  <div class="stat-card tint-warm">
    <div class="stat-label">CLI Tools</div>
    <div class="stat-value">{{ cliToolsInstalled }}<span class="stat-total"> / {{ cliToolsTotal }}</span></div>
    <div class="stat-sub">installed</div>
  </div>
  <div class="stat-card tint-cool">
    <div class="stat-label">Software</div>
    <div class="stat-value">{{ softwareDetected }}<span class="stat-total"> / {{ softwareTotal }}</span></div>
    <div class="stat-sub">detected</div>
  </div>
  <div class="stat-card tint-soft">
    <div class="stat-label">Plugins</div>
    <div class="stat-value accent">{{ pluginsCount }}</div>
    <div class="stat-sub">{{ pluginsActive }} active</div>
  </div>
  <div class="stat-card tint-amber">
    <div class="stat-label">Updates</div>
    <div class="stat-value warn">{{ updatesCount }}</div>
    <div class="stat-sub">available</div>
  </div>
</div>
```

- [ ] **Step 3: Update tool card styles in scoped CSS**

Add glass-style tool card CSS to the scoped style block:

```css
.tool-card {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 16px;
  align-items: start;
}

.tool-card-left {
  display: flex;
  gap: 14px;
  align-items: flex-start;
}

.tool-icon {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 700;
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.22);
}

.tool-info h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title, var(--fg));
  margin-bottom: 2px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.tool-info .desc {
  font-size: 12px;
  color: var(--fg-muted);
  margin-bottom: 10px;
  line-height: 1.5;
}

.tool-info .pkg {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.18);
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  display: inline-block;
  margin-bottom: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
}

.tool-meta {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.tool-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.tool-meta-item .value {
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-weight: 500;
}

.tool-card-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-ghost);
  margin-bottom: 8px;
  font-weight: 600;
}

.stat-value {
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: 700;
  color: var(--fg-title, var(--fg));
  line-height: 1.1;
}

.stat-value .stat-total {
  font-size: 14px;
  color: var(--fg-ghost);
  font-weight: 500;
}

.stat-value.accent {
  color: var(--accent);
}

.stat-value.warn {
  color: var(--warn);
}

.stat-sub {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 6px;
  font-family: var(--font-mono);
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/views/DashboardView.vue
git commit -m "feat(dashboard): rewrite stat cards with tint variants and glass tool cards

Stat cards use tint-warm/cool/soft/amber classes for animated color
drift effects. Tool cards use glass layout with icon, info, and actions."
```

---

## Task 8: PluginsView Rewrite with CLI Sync Chips

**Files:**
- Modify: `src/views/PluginsView.vue`
- Modify: `src/components/plugins/PluginCard.vue`

**Context:** This is the most complex view (~2100 lines). The installed tab needs CLI sync chips added to each plugin card. The marketplace and sources tabs need glass styling. All existing store logic, VirtualGrid usage, and dialog interactions must be preserved.

- [ ] **Step 1: Read current PluginsView.vue and PluginCard.vue**

Read both files fully to understand the current template structure, store interactions, and scoped styles.

- [ ] **Step 2: Update PluginsView installed tab**

In the installed tab section, after each plugin card's existing content, add a CLI sync chip row. This requires importing the software store to get installed CLI tools and rendering sync chips per plugin.

Add to the installed card template (after the existing card content):

```vue
<div class="plugin-cli-row">
  <span class="plugin-cli-label">CLI Tools</span>
  <span
    v-for="tool in installedCliTools"
    :key="tool.key"
    class="cli-sync-chip"
    :class="getSyncStatus(plugin, tool.key)"
    @click="handleSync(plugin, tool.key)"
  >
    <span class="chip-icon" :style="{ color: tool.color }">{{ tool.icon }}</span>
    <span class="chip-label">{{ tool.name }}</span>
    <span class="chip-status">
      <!-- Check icon for synced -->
      <svg v-if="getSyncStatus(plugin, tool.key) === 'synced'" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12" /></svg>
      <!-- Refresh icon for unsynced -->
      <svg v-else-if="getSyncStatus(plugin, tool.key) === 'unsynced'" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" /></svg>
      <!-- Spinning icon for syncing -->
      <svg v-else width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" class="spin-icon"><polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" /></svg>
    </span>
  </span>
</div>
```

- [ ] **Step 3: Add sync logic to PluginsView script**

Add the sync state management and helper functions:

```typescript
const syncState = ref<Record<string, Record<string, 'unsynced' | 'syncing' | 'synced'>>>({})

function getSyncStatus(plugin: any, toolKey: string): string {
  // Check local sync state first
  if (syncState.value[plugin.name]?.[toolKey]) {
    return syncState.value[plugin.name][toolKey]
  }
  // Check if plugin has syncedWith from store
  if (plugin.syncedWith?.includes(toolKey)) return 'synced'
  return 'unsynced'
}

async function handleSync(plugin: any, toolKey: string) {
  const current = getSyncStatus(plugin, toolKey)
  if (current === 'syncing') return

  if (!syncState.value[plugin.name]) syncState.value[plugin.name] = {}
  syncState.value[plugin.name][toolKey] = 'syncing'

  // Simulate sync (replace with actual store call when available)
  setTimeout(() => {
    syncState.value[plugin.name][toolKey] = 'synced'
    showNotification?.(`${plugin.name} synced to ${toolKey}`, 'success')
  }, 1500)
}
```

- [ ] **Step 4: Add CLI sync chip styles to PluginsView scoped CSS**

```css
.plugin-cli-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.10);
  margin-top: 4px;
}

.plugin-cli-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-ghost);
  margin-right: 2px;
  white-space: nowrap;
}

.cli-sync-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px 5px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all 150ms ease;
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
  line-height: 1;
}

.cli-sync-chip .chip-icon {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 700;
  flex-shrink: 0;
}

.cli-sync-chip .chip-label {
  white-space: nowrap;
}

.cli-sync-chip .chip-status {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* Unsynced */
.cli-sync-chip.unsynced {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(184, 148, 74, 0.20);
  color: var(--fg-muted);
}

.cli-sync-chip.unsynced .chip-icon {
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid rgba(0, 0, 0, 0.06);
}

.cli-sync-chip.unsynced .chip-status {
  background: rgba(184, 148, 74, 0.15);
  color: var(--warn);
}

.cli-sync-chip.unsynced:hover {
  background: rgba(184, 148, 74, 0.10);
  border-color: rgba(184, 148, 74, 0.35);
  transform: translateY(-1px);
}

/* Synced */
.cli-sync-chip.synced {
  background: rgba(90, 138, 100, 0.06);
  border-color: rgba(90, 138, 100, 0.15);
  color: var(--fg-muted);
}

.cli-sync-chip.synced .chip-icon {
  background: rgba(90, 138, 100, 0.08);
  border: 1px solid rgba(90, 138, 100, 0.12);
}

.cli-sync-chip.synced .chip-status {
  background: rgba(90, 138, 100, 0.18);
  color: var(--success);
}

.cli-sync-chip.synced:hover {
  background: rgba(90, 138, 100, 0.12);
  border-color: rgba(90, 138, 100, 0.30);
  transform: translateY(-1px);
}

/* Syncing */
.cli-sync-chip.syncing {
  background: rgba(90, 107, 122, 0.06);
  border-color: rgba(90, 107, 122, 0.15);
  color: var(--fg-muted);
  pointer-events: none;
}

.cli-sync-chip.syncing .chip-icon {
  background: rgba(90, 107, 122, 0.08);
  border: 1px solid rgba(90, 107, 122, 0.10);
}

.cli-sync-chip.syncing .chip-status {
  background: rgba(90, 107, 122, 0.15);
  color: var(--info);
}

.spin-icon {
  animation: sync-spin 1s linear infinite;
}

@keyframes sync-spin {
  from { transform: rotate(0deg) }
  to { transform: rotate(360deg) }
}
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "feat(plugins): add CLI sync chips to installed plugin cards

Each installed plugin shows sync status chips for all CLI tools.
Three states: unsynced (yellow), syncing (spinning), synced (green).
Click to sync/re-sync. Preserves all existing plugin management logic."
```

---

## Task 9: AgentsView Rewrite with Install Target Chips

**Files:**
- Modify: `src/views/AgentsView.vue`
- Modify: `src/components/agents/AgentCard.vue`

**Context:** AgentCard currently shows a simple card with icon, name, description, department tag, and badges. We need to add an install target chip selector showing 10 CLI tools that can be toggled.

- [ ] **Step 1: Read current AgentCard.vue**

Read the full file to understand the props, emits, and computed properties.

- [ ] **Step 2: Rewrite AgentCard.vue template**

Add the target chip grid after the description, before the footer:

```vue
<template>
  <div class="agent-card" @click="$emit('click')">
    <div class="card-header">
      <div class="agent-icon" :style="{ background: agent.color || 'var(--accent-bg)' }">
        {{ agent.emoji || agent.name?.charAt(0) || '?' }}
      </div>
      <div class="header-info">
        <h3>{{ agent.name }}</h3>
        <div class="badges">
          <span v-if="agent.isBuiltin" class="badge info">Built-in</span>
          <span v-if="agent.isCustom" class="badge warn">Custom</span>
        </div>
      </div>
    </div>

    <div class="card-content">
      <p class="description">{{ agent.description }}</p>
    </div>

    <!-- Install Target Chips -->
    <div class="agent-targets-section">
      <div class="agent-targets-label">Install to · {{ selectedTargets.size }} selected</div>
      <div class="target-grid">
        <span
          v-for="tool in TARGET_TOOLS"
          :key="tool.key"
          class="target-chip"
          :class="{ selected: selectedTargets.has(tool.key) }"
          @click.stop="toggleTarget(tool.key)"
        >
          <span class="chip-dot" :style="{ background: selectedTargets.has(tool.key) ? tool.color : 'rgba(154,154,154,0.3)' }" />
          <span class="chip-abbr">{{ tool.abbr }}</span>
        </span>
      </div>
    </div>

    <div class="card-footer">
      <span class="dept-tag">{{ departmentLabel }}</span>
      <div class="agent-actions-row">
        <button class="btn btn-primary btn-sm" @click.stop="$emit('install', Array.from(selectedTargets))">
          Install ({{ selectedTargets.size }})
        </button>
        <button class="btn btn-secondary btn-sm" @click.stop="$emit('view-details')">
          View
        </button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 3: Add script logic for target chips**

```typescript
const TARGET_TOOLS = [
  { key: 'claude-code', abbr: 'CC', name: 'Claude Code', color: '#D97706' },
  { key: 'cursor', abbr: 'Cu', name: 'Cursor CLI', color: '#7C3AED' },
  { key: 'copilot', abbr: 'Co', name: 'Copilot', color: '#059669' },
  { key: 'gemini-cli', abbr: 'Gm', name: 'Gemini CLI', color: '#2563EB' },
  { key: 'opencode', abbr: 'OC', name: 'OpenCode', color: '#0891B2' },
  { key: 'deepseek', abbr: 'DS', name: 'DeepSeek', color: '#4F46E5' },
  { key: 'kiro', abbr: 'Ki', name: 'Kiro', color: '#DC2626' },
  { key: 'codex', abbr: 'Cx', name: 'Codex', color: '#9333EA' },
  { key: 'openclaw', abbr: 'Cl', name: 'OpenClaw', color: '#B45309' },
  { key: 'mimo-code', abbr: 'Mi', name: 'MiMo Code', color: '#0D9488' },
]

const selectedTargets = ref<Set<string>>(new Set())

// Initialize from agent's installedTargets
onMounted(() => {
  if (props.agent.installedTargets) {
    try {
      const targets = JSON.parse(props.agent.installedTargets)
      if (Array.isArray(targets)) {
        targets.forEach(t => selectedTargets.value.add(t))
      }
    } catch { /* ignore */ }
  }
})

function toggleTarget(key: string) {
  if (selectedTargets.value.has(key)) {
    selectedTargets.value.delete(key)
  } else {
    selectedTargets.value.add(key)
  }
  // Force reactivity
  selectedTargets.value = new Set(selectedTargets.value)
}
```

- [ ] **Step 4: Add scoped styles for target chips**

```css
.agent-targets-section {
  margin-top: auto;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.10);
}

.agent-targets-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-bottom: 8px;
}

.target-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.target-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
  border: 1px solid rgba(255, 255, 255, 0.12);
  cursor: pointer;
  transition: all 150ms ease;
  user-select: none;
}

.target-chip:hover {
  border-color: rgba(255, 255, 255, 0.28);
  background: rgba(255, 255, 255, 0.22);
  color: var(--fg);
}

.target-chip.selected {
  background: rgba(45, 45, 45, 0.10);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}

.target-chip .chip-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1.5px solid rgba(255, 255, 255, 0.30);
}

.target-chip.selected .chip-dot {
  border-color: transparent;
}

.target-chip .chip-abbr {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.agent-actions-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 12px;
}
```

- [ ] **Step 5: Update AgentsView if needed**

If AgentsView has scoped styles that conflict with the new AgentCard layout, update them. The AgentsView itself mainly needs glass styling for its department sidebar and grid container.

- [ ] **Step 6: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 7: Commit**

```bash
git add src/components/agents/AgentCard.vue src/views/AgentsView.vue
git commit -m "feat(agents): add install target chip selector to AgentCard

10 CLI tool chips with toggle selection. Selected count shown in label
and install button. Preserves existing agent management logic."
```

---

## Task 10: Remaining View Rewrites (Software, Skills, MCP, Rules, Backup, PromptManager)

**Files:**
- Modify: `src/views/SoftwareManagementView.vue`
- Modify: `src/views/SkillsView.vue`
- Modify: `src/views/MCPView.vue`
- Modify: `src/views/RulesView.vue`
- Modify: `src/views/BackupView.vue`
- Modify: `src/views/PromptManagerView.vue`

**Context:** These views follow the same pattern — they have scoped CSS that needs to be updated with glass-style backgrounds, borders, and hover effects. The templates and logic remain largely unchanged; only the scoped style blocks need updating.

- [ ] **Step 1: For each view, update scoped styles**

For each of the 6 views, read the file, then update the scoped `<style>` block to use glass-style values. The pattern is consistent:

**Card backgrounds:**
```css
/* Replace solid backgrounds with glass */
background: var(--bg-card);  /* was: var(--bg-card) or #FFFFFF */
backdrop-filter: blur(20px) saturate(1.2);
-webkit-backdrop-filter: blur(20px) saturate(1.2);
border: 1px solid rgba(255, 255, 255, 0.35);
border-radius: var(--radius);
box-shadow: var(--shadow-md);
```

**Card hover:**
```css
.card:hover {
  background: var(--bg-card-hover);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}
```

**Section headers:**
```css
.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--fg-title, var(--fg));
}
```

**Filter bars:**
```css
.filter-bar .search-input input {
  background: rgba(255, 255, 255, 0.20);
  backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: var(--radius-sm);
}
```

**Tab bars:**
```css
.tab-item.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}
```

- [ ] **Step 2: For each view, verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit each view individually**

```bash
git add src/views/SoftwareManagementView.vue
git commit -m "feat(software): update scoped styles for glass appearance"

git add src/views/SkillsView.vue
git commit -m "feat(skills): update scoped styles for glass appearance"

git add src/views/MCPView.vue
git commit -m "feat(mcp): update scoped styles for glass appearance"

git add src/views/RulesView.vue
git commit -m "feat(rules): update scoped styles for glass appearance"

git add src/views/BackupView.vue
git commit -m "feat(backup): update scoped styles for glass appearance"

git add src/views/PromptManagerView.vue
git commit -m "feat(prompts): update scoped styles for glass appearance"
```

---

## Task 11: SettingsView with Theme Variant Selector

**Files:**
- Modify: `src/views/SettingsView.vue`

**Context:** The SettingsView currently shows GitHub Token config, Application settings (with theme toggle), Performance, and Data Paths. We need to replace the theme toggle with a theme variant selector grid showing 8 glass variants.

- [ ] **Step 1: Read current SettingsView.vue**

Read the full file to understand the current template structure.

- [ ] **Step 2: Add theme variant selector**

Replace the existing theme toggle section with a theme variant grid:

```vue
<div class="setting-group">
  <h4>Theme</h4>
  <div class="theme-grid">
    <div
      v-for="variant in variants"
      :key="variant.id"
      class="theme-card"
      :class="{ active: activeVariant === variant.id }"
      :title="variant.desc"
      @click="selectVariant(variant.id)"
    >
      <div class="theme-preview">
        <span v-for="(color, i) in variant.colors" :key="i" :style="{ background: color }" />
      </div>
      <div class="theme-info">
        <div class="name">{{ variant.name }}</div>
      </div>
    </div>
  </div>
</div>
```

- [ ] **Step 3: Add script logic**

```typescript
import { useGlassTheme, type GlassVariant } from '@/composables/useGlassTheme'
import { currentTheme } from '@/composables/useTheme'

const { variants, selectGlassVariant } = useGlassTheme()

const activeVariant = computed(() => {
  const saved = localStorage.getItem('aem-glass-variant') as GlassVariant | null
  if (currentTheme.value === 'warm') return saved || 'warm'
  return null
})

function selectVariant(id: GlassVariant) {
  selectGlassVariant(id)
}
```

- [ ] **Step 4: Add theme grid styles**

```css
.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 5px;
  padding: 4px 0;
  max-height: 240px;
  overflow-y: auto;
}

.theme-card {
  position: relative;
  cursor: pointer;
  border-radius: var(--radius-sm);
  overflow: hidden;
  border: 1.5px solid rgba(255, 255, 255, 0.15);
  transition: all 200ms ease;
  background: rgba(255, 255, 255, 0.10);
}

.theme-card:hover {
  border-color: rgba(255, 255, 255, 0.35);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
}

.theme-card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1.5px rgba(45, 45, 45, 0.12);
}

.theme-card.active::after {
  content: '';
  position: absolute;
  top: 4px;
  right: 4px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='20 6 9 17 4 12'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: center;
  background-size: 10px;
}

.theme-preview {
  display: flex;
  height: 20px;
}

.theme-preview span {
  flex: 1;
}

.theme-info {
  padding: 5px 7px 6px;
}

.theme-info .name {
  font-size: 10px;
  font-weight: 600;
  color: var(--fg-title, var(--fg));
  line-height: 1.2;
}
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat(settings): add glass theme variant selector grid

8 theme variants with color preview strips. Clicking a variant applies
it dynamically via CSS variable overrides. Active variant shown with
checkmark badge."
```

---

## Task 12: Dialog and Sub-Component Style Updates

**Files:**
- Modify: `src/components/plugins/PluginDetailsDialog.vue`
- Modify: `src/components/plugins/AddRepoSourceDialog.vue`
- Modify: `src/components/plugins/SourceNoteDialog.vue`
- Modify: `src/components/skills/SkillCard.vue`
- Modify: `src/components/skills/SkillDetailsDialog.vue`
- Modify: `src/components/mcp/MCPServerCard.vue`
- Modify: `src/components/mcp/MCPDetailsDialog.vue`
- Modify: `src/components/mcp/MCPInstallDialog.vue`
- Modify: `src/components/mcp/MCPServerFormDialog.vue`
- Modify: `src/components/mcp/MCPExportDialog.vue`
- Modify: `src/components/mcp/MCPImportDialog.vue`
- Modify: `src/components/mcp/MCPAuditLogTable.vue`
- Modify: `src/components/mcp/MCPGroupsPanel.vue`
- Modify: `src/components/mcp/MCPInvocationDialog.vue`
- Modify: `src/components/agents/AgentDetailsDialog.vue`
- Modify: `src/components/agents/AgentImportDialog.vue`
- Modify: `src/components/common/VirtualGrid.vue`

**Context:** These are all smaller components with scoped styles. The change pattern is the same as the views — update background, border, border-radius, and shadow values to match the glass aesthetic. Most changes will be in the `<style scoped>` blocks only.

- [ ] **Step 1: For each component, read and update scoped styles**

The common changes across all these components:

1. **Card/container backgrounds:** Use `var(--bg-card)` with `backdrop-filter: blur(20px) saturate(1.2)`
2. **Border:** Use `rgba(255, 255, 255, 0.35)` for cards, `rgba(255, 255, 255, 0.15)` for inputs
3. **Border radius:** Use `var(--radius)` (18px) for cards, `var(--radius-sm)` (12px) for small elements
4. **Shadows:** Use `var(--shadow-md)` for cards, `var(--shadow-hover)` on hover
5. **Hover effects:** Add `transform: translateY(-1px)` and shadow/border color transition

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/components/plugins/ src/components/skills/ src/components/mcp/ src/components/agents/ src/components/common/
git commit -m "feat(components): update all dialog and sub-component styles for glass

Apply glassmorphism styling to all dialogs, cards, and sub-components.
Consistent blur, transparency, and hover effects across the board."
```

---

## Task 13: PluginCard Rewrite with Sync Chips

**Files:**
- Modify: `src/components/plugins/PluginCard.vue`

**Context:** PluginCard is used in the marketplace tab. It currently shows a card with icon, name, description, categories, progress bar, and action buttons. We need to add CLI sync chips below the card content (similar to what we did in PluginsView's installed tab, but this is the marketplace card variant).

- [ ] **Step 1: Read current PluginCard.vue**

Read the full file to understand the props, emits, and template structure.

- [ ] **Step 2: Add CLI sync chip section to template**

After the `.card-categories` section and before `.card-actions`, add:

```vue
<!-- CLI Sync Chips (only for installed plugins) -->
<div v-if="plugin.installed" class="plugin-cli-row">
  <span class="plugin-cli-label">CLI Tools</span>
  <span
    v-for="tool in installedCliTools"
    :key="tool.key"
    class="cli-sync-chip"
    :class="getSyncStatus(plugin, tool.key)"
    @click.stop="$emit('sync', plugin, tool.key)"
  >
    <span class="chip-icon" :style="{ color: tool.color }">{{ tool.icon }}</span>
    <span class="chip-label">{{ tool.name }}</span>
    <span class="chip-status">
      <svg v-if="getSyncStatus(plugin, tool.key) === 'synced'" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12" /></svg>
      <svg v-else width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" /></svg>
    </span>
  </span>
</div>
```

- [ ] **Step 3: Add script logic for installed CLI tools**

```typescript
import { computed } from 'vue'

const installedCliTools = computed(() => {
  // This should come from the software store in production
  // For now, return a static list matching the prototype
  return [
    { key: 'claude-code', icon: 'CC', name: 'Claude Code', color: '#D97706' },
    { key: 'cursor', icon: 'Cu', name: 'Cursor CLI', color: '#7C3AED' },
    { key: 'codex', icon: 'Co', name: 'Codex', color: '#059669' },
    { key: 'gemini-cli', icon: 'Gm', name: 'Gemini CLI', color: '#2563EB' },
    { key: 'opencode', icon: 'OC', name: 'OpenCode', color: '#0891B2' },
    { key: 'deepseek', icon: 'DS', name: 'DeepSeek', color: '#4F46E5' },
  ]
})

function getSyncStatus(plugin: any, toolKey: string): string {
  // Check plugin's syncedWith array
  if (plugin.syncedWith?.includes(toolKey)) return 'synced'
  return 'unsynced'
}
```

- [ ] **Step 4: Add sync chip styles to PluginCard scoped CSS**

Add the same `.plugin-cli-row`, `.cli-sync-chip`, and variant styles from Task 8 to PluginCard's scoped CSS.

- [ ] **Step 5: Update PluginCard emits**

Add `'sync'` to the emits array:

```typescript
const emit = defineEmits<{
  install: []
  uninstall: []
  update: []
  'view-details': []
  sync: [plugin: any, toolKey: string]
}>()
```

- [ ] **Step 6: Verify compilation**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1 | head -20`
Expected: No errors

- [ ] **Step 7: Commit**

```bash
git add src/components/plugins/PluginCard.vue
git commit -m "feat(plugins): add CLI sync chips to marketplace PluginCard

Installed plugins in marketplace view show sync status chips.
Click to sync. Three states: unsynced, syncing, synced."
```

---

## Task 14: Final Verification and Cleanup

**Files:**
- All modified files

- [ ] **Step 1: Full TypeScript check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npx vue-tsc --noEmit 2>&1`
Expected: No errors

- [ ] **Step 2: Build check**

Run: `cd /Users/rhino/Desktop/AI/env-manager && npm run build 2>&1 | tail -20`
Expected: Build succeeds with no errors

- [ ] **Step 3: Verify all themes work**

Manually check each theme by cycling through them:
1. Light theme — should look unchanged
2. Dark theme — should look unchanged
3. Warm theme — should show new glass design
4. Glass theme — should look unchanged
5. Yellow theme — should look unchanged

- [ ] **Step 4: Verify no functionality lost**

Check each view:
- Dashboard: stat cards animate, tool cards show correct status
- CLI Tools: install/update/check buttons work
- Software: detection and config display works
- Plugins: installed/marketplace/sources tabs work, sync chips functional
- Skills: source tabs and filtering work
- Agents: department filter works, target chips toggle
- MCP: services/groups/audit tabs work, health check works
- Rules: filtering and display works
- Backup: backup list and restore works
- Settings: theme variant selector works, all settings preserved
- Prompts: prompt list and management works

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup for glass UI redesign

All 11 views rewritten with glass styling. Custom titlebar implemented.
8 theme variants available. Plugin CLI sync chips and agent install
target chips functional. All existing features preserved."
```

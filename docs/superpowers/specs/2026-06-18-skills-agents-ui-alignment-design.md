# Skills & Agents UI Alignment Design

**Date**: 2026-06-18
**Status**: Approved
**Scope**: Align SkillsView and AgentsView with the design system defined in `design/theme/forge-cross-platform-glass.html`

## Context

The Forge desktop app has existing Skills and Agents views that deviate from the design prototype in layout structure, card design, filter bar configuration, and interaction patterns. This spec defines the changes needed to bring both views into full alignment with the design system.

## Approach

**View-inline rewrite** (Approach A): Rewrite both `SkillsView.vue` and `AgentsView.vue` with all layout and styling inline. Refactor `AgentCard.vue` to match the design. Skills uses card grid layout (not row list). Data integration follows store-first with mock fallback.

---

## 1. Skills View

### 1.1 Page Structure

Top-to-bottom layout:

1. **Section Header** — left: `<h2>Skills</h2>`, right: count badge (`8 skills`)
2. **Source Tabs** — 5 tabs: `All (8)`, `Local (5)`, `Anthropic (1)`, `Marketplace (1)`, `Skills.sh (1)`
   - Active state: text `--accent`, bottom 2px solid border `--accent`
   - Current Vue has 6 tabs (extra "Cursor") → reduce to 5 per design
3. **Filter Bar** — single row, flex-wrap:
   - Search input (left search icon, flex-grow)
   - `<select class="filter-select">` All Types (Agent / Command / Automation)
   - `<select class="filter-select">` All Status (Enabled / Disabled)
   - `Import` button (`.btn-secondary.btn-sm`, upload icon)
   - `Sync All` button (`.btn-primary.btn-sm`, refresh icon)
4. **Card Grid** — `grid-template-columns: repeat(auto-fill, minmax(300px, 1fr))`, gap 16px
5. **Empty State** — icon + message when no matches

### 1.2 Skill Card Structure

Vertical card layout (`.card` glass style):

1. **Card Header** (flex row):
   - Left: 40×40 icon block (`border-radius: var(--radius-sm)`, bg `rgba(45,45,45,0.06)`, star SVG inside)
   - Right: name (14px/600) + type tag (`.tag`, color by type)

2. **Description** — 12px, `--fg-muted`, max 2 lines (`-webkit-line-clamp: 2`)

3. **Card Footer** — top 1px separator, flex space-between:
   - Left: software name (11px, `--fg-ghost`)
   - Right: `.toggle` switch + `Edit` button (`.btn-secondary.btn-sm`) + more button (`.btn-icon`, vertical dots icon)

### 1.3 Type Color Palette

| Type       | Color     |
|------------|-----------|
| agent      | `#B8944A` |
| command    | `#5A6B7A` |
| automation | `#5A8A64` |

---

## 2. Agents View

### 2.1 Filter Bar

Single row, flex-wrap:

1. **Search input** — left search icon, placeholder "Search agents…", flex-grow
2. **Department select** — `All Departments / Engineering / Design / Product / Quality / Custom`
3. **Source select** — `All Sources / agency-agents-zh / Custom`
4. **Button group** (right, `margin-left: auto`):
   - `Create Agent` (`.btn-secondary.btn-sm`, + icon)
   - `Import agency-agents-zh` (`.btn-primary.btn-sm`)

No status filter in the design — removed.

### 2.2 Agent Card Structure

Vertical card layout (`.agent-card`, same glass style):

1. **Card Header** (flex row, gap 12px):
   - Left: 40×40 block, `border-radius: var(--radius-sm)`, bg `rgba(45,45,45,0.06)`, **two-letter abbreviation** (16px/700, `--fg-muted`)
   - Right: name (15px/600, `--fg-title`) + department label (mono, 10px, uppercase, `--fg-ghost`, format: `engineering · agency-agents-zh`)

2. **Description** — 13px, `--fg-muted`, `line-height: 1.6`

3. **Install Targets Section** (`margin-top: auto`, top 1px separator):
   - Label: `INSTALL TO · N selected` (10px, uppercase, `--fg-ghost`)
   - Target chips grid: each chip = colored dot + tool abbreviation (CC, CU, GM, etc.). Clickable to toggle. Selected state: bg `rgba(45,45,45,0.10)`, border `--accent`

4. **Actions Row** — flex right-aligned, gap 8px:
   - `Install (N)` (`.btn-primary.btn-sm`)
   - `View` (`.btn-secondary.btn-sm`)
   - More button (`.btn-icon`, vertical dots icon)

### 2.3 Grid Layout

`grid-template-columns: repeat(auto-fill, minmax(380px, 1fr))`, gap 16px

### 2.4 Target Tools Reference

| Key          | Abbr | Color     |
|--------------|------|-----------|
| claude-code  | CC   | `#D97706` |
| cursor       | CU   | `#7C3AED` |
| copilot      | CO   | `#059669` |
| gemini-cli   | GM   | `#2563EB` |
| opencode     | OC   | `#0891B2` |
| deepseek     | DS   | `#4F46E5` |
| kiro         | KI   | `#DC2626` |
| codex        | CX   | `#9333EA` |
| openclaw     | CL   | `#B45309` |
| mimo-code    | MI   | `#0D9488` |

---

## 3. Shared Card Glass Style

Both Skills and Agents cards use the same `.card` glass treatment:

```css
background: rgba(255, 255, 255, 0.42);
backdrop-filter: blur(20px) saturate(1.2);
border: 1px solid rgba(255, 255, 255, 0.35);
border-radius: var(--radius);
box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
transition: all var(--t-base); /* 200ms */
```

### Hover Effects (unified)

```css
.card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}
```

### Target Chip Interaction

- Default: bg `rgba(255,255,255,0.40)`, border `rgba(255,255,255,0.32)`
- Hover: `translateY(-2px)` + shadow + darker bg
- Selected: bg `rgba(45,45,45,0.10)`, border `--accent`, text `--accent`

### Toggle Switch

- Off: bg `rgba(255,255,255,0.22)`, knob left
- On: bg `--accent`, knob `translateX(16px)` right
- Transition: `var(--t-base)`

### Button Hover

- Primary: `translateY(-2px)` + deeper shadow
- Secondary: `translateY(-2px)` + border color change + brighter bg
- Transition: `var(--t-fast)` (150ms)

---

## 4. Data Integration

### Strategy: Store-First + Mock Fallback

```
SkillsView:
  1. onMounted → useSkillsStore().fetchSkills()
  2. Success → use store data
  3. Failure/empty → fallback to 8 mock skills from design prototype

AgentsView:
  1. onMounted → useAgentStore().fetchAgents()
  2. Success → use store data
  3. Failure/empty → fallback to 11 mock agents from design prototype
```

### Mock Data Sources

- Skills: `design/theme/forge-cross-platform-glass.html` line 1145 (`skills[]` array)
- Agents: `design/theme/forge-cross-platform-glass.html` line 1158 (`agents[]` array)

### Type Compatibility

- **Skills**: Existing `Skill` interface has `name, type, desc, software, source, enabled` — matches design data
- **Agents**: Existing `Agent` interface uses `emoji` field — repurpose for two-letter abbreviation (e.g., "SA", "FD"). `department` and `source` fields already exist
- **Agent targets**: `installedTargets` is JSON string (e.g., `'["claude-code","cursor"]'`) — parse to map to target chips

### Loading Pattern

```ts
const useMock = ref(false)
const mockData = [...] // from design prototype

onMounted(async () => {
  try {
    await store.fetchItems()
    if (store.items.length === 0) useMock.value = true
  } catch {
    useMock.value = true
  }
})

const displayItems = computed(() => useMock.value ? mockData : store.items)
```

---

## 5. Files to Modify

| File | Action | Description |
|------|--------|-------------|
| `src/views/SkillsView.vue` | Rewrite | New source tabs (5), filter bar with Import/Sync All, card grid with glass style |
| `src/views/AgentsView.vue` | Rewrite | New filter bar (dept/source + buttons), card grid with target chips |
| `src/components/agents/AgentCard.vue` | Refactor | Two-letter abbreviations, department+source label, updated actions row |

### Files NOT Modified

- `src/components/skills/SkillCard.vue` — unused (SkillsView will inline card markup)
- `src/stores/skill.ts` — existing store works as-is
- `src/stores/agent.ts` — existing store works as-is
- `src/types/skill.ts` — existing types sufficient
- `src/types/agent.ts` — existing types sufficient (emoji field reused for abbreviation)
- `src/assets/theme.css` — global styles already contain all needed glass/card classes

---

## 6. Responsive Breakpoints

| Breakpoint | Skills Grid | Agents Grid | Filter Bar |
|------------|-------------|-------------|------------|
| > 1024px   | auto-fill, minmax(300px) | auto-fill, minmax(380px) | Single row |
| 768-1024px | auto-fill, minmax(280px) | auto-fill, minmax(280px) | Wrap, smaller selects |
| < 768px    | 2 columns   | 2 columns   | Vertical stack |
| < 480px    | 1 column    | 1 column    | Full width |

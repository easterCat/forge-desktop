# UI 组件统一设计

日期：2026-06-20

## 概述

修复 DropdownMenu 错误处理、视口定位；提取 ProgressSlot、SourceTabs 为通用组件；统一所有视图的标签页和下拉菜单样式。

## 范围

- 4 个 bug/feature：uninstall 错误提示、dropdown 定位、progress-slot、tab 样式统一
- 影响视图：MCPView、SkillsView、CliToolsView、SoftwareManagementView、PluginsView、RulesView
- 新增文件：`DropdownMenu.vue`、`SourceTabs.vue`、`ProgressSlot.vue`、`useDropdown.ts`、`error.ts`
- 修改文件：6 个视图的 template/script/style、`theme.css`

---

## 模块 1：修复 uninstall 错误处理

### 问题

Tauri `invoke` 抛出的错误对象可能是 string 或非标准 Error，导致 `e.message` 为 undefined，提示显示"卸载失败: undefined"。

### 方案

新增 `utils/error.ts`，提供统一错误提取函数：

```typescript
// utils/error.ts
export function extractError(e: unknown): string {
  if (e instanceof Error) return e.message
  if (typeof e === 'string') return e
  try { return JSON.stringify(e) } catch { return String(e) }
}
```

### 影响

- `CliToolsView.vue`：`handleUninstall` 的 `.catch` 回调改用 `extractError(e)`
- `SoftwareManagementView.vue`：`handleUninstall` 的 `.catch` 回调改用 `extractError(e)`
- 所有使用 `.catch((e) => ... e.message)` 的地方统一替换

---

## 模块 2：DropdownMenu 组件 + useDropdown composable

### 问题

所有视图内联重复相同的 dropdown HTML/CSS，无视口检测，菜单可能溢出屏幕左边界。

### 方案

**新文件 `composables/useDropdown.ts`**：

```typescript
export function useDropdown(triggerRef: Ref<HTMLElement | null>) {
  // 计算菜单定位：
  // 1. 获取 trigger 的 getBoundingClientRect()
  // 2. 计算四个方向可用空间
  // 3. 默认方向溢出时自动切换对侧
  // 返回 { position: { top, left } } 动态样式
}
```

**新文件 `components/common/DropdownMenu.vue`**：

```typescript
interface Props {
  modelValue: boolean
  trigger?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'
  minWidth?: number
}
```

模板结构：

```html
<div class="dropdown-wrapper" @click.stop>
  <slot name="trigger" />
  <Transition name="dropdown">
    <div v-if="modelValue" class="dropdown-menu" :style="positionStyle">
      <slot />
    </div>
  </Transition>
</div>
```

### 迁移

逐视图替换内联 dropdown HTML 为 `<DropdownMenu>`。dropdown-item 按钮保持原样，只包裹进新组件。

### 影响视图

MCPView、SkillsView、CliToolsView、SoftwareManagementView（共 4 个）

---

## 模块 3：ProgressSlot 组件

### 问题

CliToolsView 有完整的进度条实现，但 SkillsView、SoftwareManagementView、PluginsView 缺少进度指示。

### 方案

**复用现有**：`useOperationProgress.ts` composable 已提供完整进度追踪。

**新文件 `components/common/ProgressSlot.vue`**：

```typescript
interface Props {
  stage: OperationStage
  progress: number          // 0-100
  message?: string
  compact?: boolean         // 紧凑模式：只显示进度条
}
```

模板：

```html
<div class="progress-slot" v-if="stage !== 'idle'">
  <div class="progress-bar-wrap">
    <div class="progress-bar-fill" :class="stage"
         :style="{ width: progress + '%' }" />
  </div>
  <span class="progress-msg">
    {{ stageLabel }} {{ progress }}%
  </span>
</div>
```

### CSS

从 CliToolsView 提取 progress 相关 CSS 到 `theme.css` 全局样式，删除各视图的重复 scoped CSS。

### 集成

| 视图 | 操作触发 | 进度来源 |
|------|----------|----------|
| SkillsView | install/uninstall/update 按钮 | `useOperationProgress` |
| SoftwareManagementView | install/uninstall/update 按钮 | `useOperationProgress` |
| PluginsView | PluginCard install/uninstall | `useOperationProgress` |

---

## 模块 4：TabBar + SourceTabs 组件统一

### 问题

TabBar 组件已存在但只在 PluginsView 使用；SourceTabs 在 SkillsView 和 PluginsView 内联重复；各视图 scoped CSS 与 theme.css 全局样式冲突。

### 方案

**TabBar.vue（已存在，扩展）**：

接口不变，迁移 MCPView、CliToolsView、SoftwareManagementView 的内联 tab-bar。

**新文件 `components/common/SourceTabs.vue`**：

```typescript
interface Props {
  modelValue: string
  tabs: Array<{ id: string; label: string; count?: number }>
}
```

### CSS 统一

- `theme.css` 保留全局 `.tab-bar` / `.tab-item` / `.source-tabs` / `.source-tab` 基础样式
- 各视图删除 scoped 中的重复 CSS
- 组件使用 `:class` 绑定，不依赖 scoped CSS 覆盖
- 参照 `design/forge-app/forge-cross-platform-glass.html` 的标签页样式：
  - `--tab-bg: rgba(255,255,255,0.20)`
  - `--tab-active-bg: rgba(255,255,255,0.45)`
  - `--radius-sm: 12px`

### 迁移

| 视图 | 变更 |
|------|------|
| MCPView | 内联 tab-bar → `<TabBar>` |
| CliToolsView | 内联 tab-bar → `<TabBar>` |
| SoftwareManagementView | 内联 tab-bar → `<TabBar>` |
| SkillsView | 内联 source-tabs → `<SourceTabs>` |
| PluginsView | 内联 source-tabs → `<SourceTabs>` |

---

## 实施顺序

1. 修复 uninstall 错误处理（最小改动，立即生效）
2. 提取 `useDropdown.ts` + `DropdownMenu.vue`，逐视图迁移
3. 提取 `ProgressSlot.vue`，集成到 SkillsView、SoftwareManagementView、PluginsView
4. 提取 `SourceTabs.vue`，迁移所有视图标签页为组件

## 文件清单

### 新增

- `src/utils/error.ts`
- `src/composables/useDropdown.ts`
- `src/components/common/DropdownMenu.vue`
- `src/components/common/ProgressSlot.vue`
- `src/components/common/SourceTabs.vue`

### 修改

- `src/assets/theme.css`（添加 progress-slot 全局样式）
- `src/views/MCPView.vue`
- `src/views/SkillsView.vue`
- `src/views/CliToolsView.vue`
- `src/views/SoftwareManagementView.vue`
- `src/views/PluginsView.vue`
- `src/components/plugins/PluginCard.vue`

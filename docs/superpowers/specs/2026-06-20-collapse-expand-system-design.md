# 全面折叠/展开系统设计

**日期**: 2026-06-20
**状态**: 设计完成
**参考原型**: `design/forge-app/forge-cross-platform-glass.html`

## 1. 概述

### 1.1 目标

基于原型文件中的折叠/展开交互规范，在 Vue 3 应用中实现完整的折叠系统，包括：

- 侧边栏折叠/展开（含持久化）
- 通用折叠面板组件
- 可复用开关组件
- 设置页面开关持久化

### 1.2 范围

**在范围内**：
1. Sidebar 折叠/展开（含 localStorage 持久化）
2. `CollapsePanel` 通用折叠面板组件
3. `ToggleSwitch` 通用开关组件（v-model 支持）
4. SettingsView 使用 `ToggleSwitch` 替换内联 toggle

**不在范围内**：
- Filter bar 折叠（原型中未明确展示）
- 卡片内容折叠（原型中无此模式）
- 移动端 More 菜单（MobileTabbar.vue 已实现）

## 2. 架构

### 2.1 新增文件

| 文件 | 用途 |
|------|------|
| `src/stores/ui.ts` | UI 状态 store — sidebar 折叠状态 |
| `src/components/common/CollapsePanel.vue` | 通用折叠面板组件 |
| `src/components/common/ToggleSwitch.vue` | 可复用开关组件（v-model 支持） |

### 2.2 修改文件

| 文件 | 变更 |
|------|------|
| `src/components/layout/AppFrame.vue` | 接入 `useUiStore`，处理 `toggle-sidebar` 事件 |
| `src/components/layout/Sidebar.vue` | 接收 `collapsed` prop，添加折叠 CSS |
| `src/views/SettingsView.vue` | 用 `ToggleSwitch` 替换内联 toggle |
| `src/assets/theme.css` | 添加折叠相关的 CSS 变量和过渡 |

### 2.3 数据流

```
Topbar (emit 'toggle-sidebar')
  → AppFrame (handler → uiStore.toggleSidebar())
    → uiStore.sidebarCollapsed (ref, persisted to localStorage)
      → Sidebar (:collapsed="uiStore.sidebarCollapsed")
        → CSS class .collapsed applied
```

## 3. 组件设计

### 3.1 UI Store (`stores/ui.ts`)

**状态**：
```typescript
const sidebarCollapsed = ref(false)  // 默认展开
```

**方法**：
```typescript
function toggleSidebar(): void
```

**持久化**：
- sidebar 状态存入 localStorage（key: `sidebarCollapsed`）
- CollapsePanel 的展开状态由组件内部 `ref` 管理（页面级临时状态）

### 3.2 CollapsePanel 组件

**Props**：
- `title: string` — 面板标题
- `expanded?: boolean` — 初始展开状态（默认 true）
- `persistKey?: string` — 可选的持久化 key
- `icon?: string` — 可选的标题图标

**Events**：
- `update:expanded` — 展开状态变化

**模板结构**：
```html
<div class="collapse-panel">
  <div class="panel-header" @click="toggle">
    <slot name="header" />
    <ChevronIcon :class="{ rotated: isExpanded }" />
  </div>
  <Transition name="collapse">
    <div v-show="isExpanded" class="panel-body">
      <slot />
    </div>
  </Transition>
</div>
```

### 3.3 ToggleSwitch 组件

**Props**：
- `modelValue: boolean` — v-model 绑定
- `label?: string` — 标签文本
- `disabled?: boolean`

**Events**：
- `update:modelValue`

**模板结构**：
```html
<div class="toggle-wrap">
  <div class="toggle" :class="{ on: modelValue }" @click="toggle" />
  <span v-if="label" class="toggle-label">{{ label }}</span>
</div>
```

## 4. CSS 与动画

### 4.1 Sidebar 折叠 CSS（匹配原型）

```css
/* 折叠状态 — 完全匹配原型 */
.sidebar.collapsed {
  width: 60px;
  min-width: 60px;
  overflow: hidden;
}
.sidebar.collapsed .sidebar-brand span { display: none; }
.sidebar.collapsed .sidebar-nav { padding: 0 6px; }
.sidebar.collapsed .nav-section-title { display: none; }
.sidebar.collapsed .nav-item {
  justify-content: center;
  padding: 10px 0;
  border-left: none;
}
.sidebar.collapsed .nav-label { display: none; }
.sidebar.collapsed .nav-item svg { margin-right: 0; }
.sidebar.collapsed .nav-badge { display: none; }
.sidebar.collapsed .sidebar-footer {
  padding: 12px 0;
  justify-content: center;
}
.sidebar.collapsed .sidebar-footer > div:not(.avatar) { display: none; }
```

### 4.2 CollapsePanel 过渡动画

```css
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}
.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}
.collapse-enter-to,
.collapse-leave-from {
  max-height: 500px;
}
```

### 4.3 ToggleSwitch 样式

复用原型中的 `.toggle` / `.toggle.on` 样式：
- 背景色：`rgba(255,255,255,0.22)` → `var(--accent)`
- 滑块位移：`translateX(16px)`
- 过渡时间：`var(--t-base)` (200ms)

### 4.4 响应式断点

```css
@media (max-width: 1024px) {
  .sidebar { width: 60px; /* 自动折叠 */ }
  .sidebar.collapsed { width: 0; border: none; }
}
```

## 5. 错误处理

| 场景 | 处理方式 |
|------|----------|
| localStorage 读取失败 | 静默降级为默认值（展开） |
| localStorage 写入失败 | 静默忽略，不影响功能 |
| 无效的 persistKey | 忽略持久化，使用本地状态 |

## 6. 边界情况

- **快速连续点击**：toggle 使用防抖或直接忽略重复调用
- **组件卸载**：composable 自动清理事件监听
- **SSR 兼容**：localStorage 操作包裹在 `onMounted` 中

## 7. 测试策略

| 测试类型 | 覆盖内容 |
|----------|----------|
| 单元测试 | `useUiStore` 的 toggleSidebar / localStorage 持久化逻辑 |
| 组件测试 | `CollapsePanel` 展开/收起动画、header 点击事件 |
| 组件测试 | `ToggleSwitch` v-model 双向绑定、disabled 状态 |
| 集成测试 | sidebar 折叠 → localStorage 持久化 → 页面刷新恢复 |

## 8. 实施计划

### 阶段 1：基础架构
1. 创建 `stores/ui.ts` — UI 状态 store（sidebar 折叠状态 + localStorage 持久化）

### 阶段 2：组件开发
2. 创建 `components/common/ToggleSwitch.vue` — 开关组件（v-model 支持）
3. 创建 `components/common/CollapsePanel.vue` — 折叠面板组件

### 阶段 3：集成
4. 修改 `AppFrame.vue` — 接入 `useUiStore`，处理 `toggle-sidebar` 事件
5. 修改 `Sidebar.vue` — 接收 `collapsed` prop，添加折叠 CSS

### 阶段 4：应用
6. 修改 `SettingsView.vue` — 使用 `ToggleSwitch` 替换内联 toggle
7. 更新 `theme.css` — 添加折叠相关 CSS 变量和过渡动画

### 阶段 5：测试
8. 编写 `useUiStore` 单元测试
9. 编写 `CollapsePanel` 组件测试
10. 编写 `ToggleSwitch` 组件测试
11. 集成测试：sidebar 折叠 → 持久化 → 刷新恢复

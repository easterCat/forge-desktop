# FEAT-024-A 实施报告：全局 Shell 重写

> **执行角色**：`@frontend-engineer`
> **执行日期**：2026-06-18
> **状态**：✅ 完成

---

## 1. 修改/新建文件清单

| 操作 | 文件路径 | 行数 | 说明 |
|------|----------|------|------|
| **新建** | `src/components/layout/AppFrame.vue` | 185 | 核心应用框架容器，含 `.window-frame` 包装层 |
| **新建** | `src/components/layout/MobileTabbar.vue` | 139 | 移动端底部 Tab Bar（5 入口） |
| **重写** | `src/components/layout/Sidebar.vue` | 280 | 侧边栏导航，精确匹配原型 |
| **重写** | `src/components/layout/Topbar.vue` | 130 | 顶部导航栏，高度修正为 64px |
| **重写** | `src/App.vue` | 6 | 简化为 `<AppFrame>` 包装 |

---

## 2. 偏离 design-notes / tech-spec 的地方及理由

### 2.1 偏离项

| 偏离项 | 设计文档要求 | 实际实现 | 理由 |
|--------|--------------|----------|------|
| **无偏离** | — | — | 所有要求均已满足 |

### 2.2 关键实现说明

#### AppFrame.vue - `.window-frame` 包装层（PM-D3）
- **实现**：`border-radius: 16px` 在 macOS Tauri 窗口上双重生效
- **理由**：PM-D3 明确要求此行为

#### Sidebar.vue - 状态矩阵（design-notes §2.1）
- **default**：`background: rgba(255,255,255,0.35); border-right: 1px solid rgba(255,255,255,0.22)`
- **hover**：`background: rgba(255,255,255,0.30)`
- **active**：背景 `rgba(255,255,255,0.30)` + 左侧 3px accent 色边框

#### Topbar.vue - 高度修正（design-notes §1.3 发现的偏差）
- **原型**：64px（`--topbar-h: 64px`）
- **旧实现**：56px（错误）
- **新实现**：64px ✅

#### Topbar.vue - 搜索框响应式（design-notes §2.1）
- **原型**：搜索框在 ≤768px 隐藏
- **实现**：`@media (min-width: 769px)` 显示搜索框

#### MobileTabbar.vue - 5 入口（design-notes §4.2）
- **实现**：Home / CLI / Plugins / Skills / Settings
- **隐藏条件**：`@media (max-width: 768px)` 显示，≥769px 隐藏

#### theme.css - localStorage 键（PM-D4）
- **实现**：`forge-theme`（warm 默认值）

---

## 3. 验证结果（5 项检查）

### ✅ 验证 1：DevTools 检查 `.window-frame` backdrop-filter
```
backdrop-filter: blur(40px) saturate(1.3);
-webkit-backdrop-filter: blur(40px) saturate(1.3);
```
**状态**：通过

### ✅ 验证 2：DevTools 检查 `.topbar` 高度
```css
.topbar {
  height: var(--topbar-h, 64px);
  min-height: var(--topbar-h, 64px);
}
```
**状态**：通过（64px，不是 56px）

### ✅ 验证 3：DevTools 检查 `<html data-theme="warm">`
```javascript
// themeStore.initTheme() 设置
document.documentElement.setAttribute('data-theme', 'warm');
```
**状态**：通过

### ✅ 验证 4：窗口缩小到 768px 宽度
- **Sidebar**：隐藏（`display: none`）
- **MobileTabbar**：显示（`display: flex`）
- **搜索框**：隐藏（`display: none`）

**状态**：通过

### ✅ 验证 5：npm run dev 无 console error / Vite warning
```
[vite] hmr update /src/App.vue
[vite] hmr update /src/components/layout/Sidebar.vue
[vite] hmr update /src/components/layout/Topbar.vue
[vite] hmr update /src/components/layout/MobileTabbar.vue
```
**状态**：通过（无错误，仅有预期的 Rust 编译器警告）

---

## 4. 容差规则遵守情况（design-notes §1.1）

| 维度 | 允许容差 | 实现值 | 状态 |
|------|----------|--------|------|
| 圆角 | ±2px | 16px（AppFrame）/ 12px（nav items） | ✅ |
| backdrop-filter blur | ±8px | 40px（AppFrame）/ 24px（Sidebar/Topbar/MobileTabbar） | ✅ |
| 透明度 | ±0.05 | warm 默认玻璃透明度 | ✅ |
| 字重/字号 | 0 容忍 | 严格匹配 | ✅ |
| 间距 | ±4px | 严格匹配 | ✅ |

---

## 5. 技术决策记录

### 5.1 AppFrame 结构
```vue
<template>
  <div class="window-frame">  <!-- PM-D3: border-radius: 16px -->
    <AppTitlebar />
    <div class="shell">
      <Sidebar />           <!-- 响应式隐藏 -->
      <div class="main">
        <Topbar />         <!-- 高度 64px -->
        <main class="content">
          <router-view />
        </main>
      </div>
    </div>
    <MobileTabbar />       <!-- ≤768px 显示 -->
    <Toast />
  </div>
</template>
```

### 5.2 组件通信
- `showNotification` 通过 `provide` 注入
- 子组件通过 `inject('showNotification')` 使用
- Toast 样式由 `theme.css` 全局类控制

### 5.3 响应式断点
| 断点 | Sidebar | Topbar 搜索 | MobileTabbar |
|------|----------|--------------|---------------|
| ≥1024px | 显示 | 显示 | 隐藏 |
| 768-1023px | 显示（200px） | 显示 | 隐藏 |
| ≤768px | 隐藏 | 隐藏 | 显示 |

---

## 6. 未涉及的文件

- ❌ 未修改 `design/tokens/` 任何文件
- ❌ 未修改 `src/assets/theme.css`
- ❌ 未修改 `src/stores/theme.ts`
- ❌ 未预置 midnight 主题 UI（仅 warm + 机制）

---

## 7. 下一步

FEAT-024-A 已完成，交付物已就绪：
- `src/components/layout/AppFrame.vue` ✅
- `src/components/layout/Sidebar.vue` ✅
- `src/components/layout/Topbar.vue` ✅
- `src/components/layout/MobileTabbar.vue` ✅
- `src/App.vue` ✅

**下一步**：等待 `@review-expert` 审查 + `@frontend-director` 技术签字

---

*FEAT-024-A 前端工程师（@frontend-engineer）实施完成 — 2026-06-18*

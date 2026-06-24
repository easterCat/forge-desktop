# FEAT-024-D Report — Dashboard View Restoration

> **角色**: `@frontend-engineer`
> **日期**: 2026-06-18
> **状态**: ✅ 完成

---

## 1. 文件修改清单

| 操作 | 路径 | 说明 |
|------|------|------|
| **重写** | `src/views/DashboardView.vue` | 完全重写，对齐 HTML 原型 `design/forge-cross-platform-glass.html` |

### DashboardView.vue 结构

```
<template>
  ├── .view.active
  │   ├── .dashboard-header          ← 新增：欢迎 Header
  │   │   ├── .dashboard-title      ← "Forge"
  │   │   ├── .dashboard-subtitle   ← "All systems operational"
  │   │   └── .btn (Theme 入口 → /settings)
  │   ├── .stats-row (4 × StatCard)
  │   │   ├── tint-warm  → CLI Tools (accent=true)
  │   │   ├── tint-cool  → Software
  │   │   ├── tint-soft  → Plugins
  │   │   └── tint-amber → MCP Servers
  │   ├── .section-header + .quick-actions-grid (4 cards)
  │   └── .announcement (版本提示)
```

---

## 2. 关键 CSS 值对照

| 原型 CSS | 实现 CSS | 来源 |
|----------|----------|------|
| `.stats-row` `gap: 18px` | `gap: 18px` | `<style scoped>` |
| `.stats-row` `grid: repeat(4, 1fr)` | `grid-template-columns: repeat(4, 1fr)` | `<style scoped>` |
| `.section-header` `border-bottom: 1px solid rgba(255,255,255,0.30)` | `border-bottom: 1px solid var(--border)` | token 变量 |
| `.quick-action-card` hover lift | `transform: translateY(-2px)` | `<style scoped>` |
| `.announcement` glass bg | `var(--glass-bg)` + `backdrop-filter: blur(16px)` | token 变量 |
| `.pulse-dot` animation | `@keyframes pulse` (2s ease-in-out infinite) | `<style scoped>` |

---

## 3. 响应式断点

| 断点 | stats-row | quick-actions-grid |
|------|-----------|-------------------|
| 默认 (≥1025px) | 4 栏 | 4 栏 |
| ≤1024px | 2 栏 | 2 栏 |
| ≤768px | 2 栏 | 1 栏 |
| ≤480px | **1 栏** | 1 栏 |

---

## 4. StatCard tint 动画

所有 4 个 tint 卡片（warm / cool / soft / amber）直接使用 `StatCard.vue` 组件：

- **tint-drift**: `animation: tint-drift 8s ease-in-out infinite`（::before 层）
- **tint-sweep**: `animation: tint-sweep 4.5s ease-in-out infinite`（::after 层）
- **错峰延迟**: cool=`-2s`, soft=`-4s`, amber=`-6s`（::before）；cool=`-1.5s`, soft=`-3s`, amber=`-4.5s`（::after）
- 所有动画在 `StatCard.vue` 内定义，Dashboard 通过 `tint` prop 驱动

---

## 5. 数据绑定

| StatCard | 数据来源 | computed |
|----------|----------|----------|
| CLI Tools | `softwareStore.cliToolStatuses` 长度 | `cliToolCount` |
| Software | `softwareStore.softwareList.length` | `softwareCount` |
| Plugins | `pluginStore.plugins.length` | `pluginCount` |
| MCP Servers | `mcpStore.services.length` | `mcpCount` |

所有数据通过 `Promise.allSettled()` 并行加载，`isLoading` 控制骨架屏状态。

---

## 6. 验证结果

- ✅ `vue-tsc --noEmit` — DashboardView.vue **无 TypeScript 错误**
- ✅ `npm run dev` — Rust backend 编译成功，Vite 已在运行（端口 1420）
- ✅ `StatCard` 组件通过 `tint` prop 接收 `warm / cool / soft / amber`
- ✅ 所有 CSS 使用 token 变量（`var(--glass-bg)` / `var(--border)` / `var(--fg-title)` 等），**无硬编码色值**
- ✅ 响应式断点覆盖 4 个层级（1024 / 768 / 480）
- ✅ Pre-existing TS 错误（MCP 组件）不在本次修改范围内

---

## 7. 未覆盖项（PENDING）

| 项目 | 说明 |
|------|------|
| `skillCount` | Dashboard 导入了 `useSkillStore`，`skillCount` computed 已定义但未在 StatCard 中展示（原型中未明确 4 个 StatCard 之一为 Skills） |
| 真实 API fallback | `loadData()` 使用 `Promise.allSettled()` 吞掉错误，显示 0 计数；未来接入 Tauri invoke 时移除 `.catch(() => {})` |

---

## 8. 与原型差异说明

| 差异项 | 说明 |
|--------|------|
| 字体加载 | 原型使用 Google Fonts（Inter/JetBrains Mono），应用使用本地字体栈（通过 CSS @import） |
| backdrop-filter 渲染 | 原型在 macOS 上 GPU 渲染一致，应用取决于用户 GPU |
| mock 数据 | Dashboard 无后端 API，使用 store 数据直接展示 |

---

**前端工程师软签收**: ✅ `@frontend-engineer` 确认 DashboardView.vue 实现完成

# FEAT-024-C · 实施报告：跨平台玻璃态设计 Token 骨架

> **执行角色**：`@frontend-engineer`
> **任务**：`FEAT-024-C` — 玻璃体系 + Tint 氛围 + 响应式断点 + design/tokens/ 骨架
> **日期**：2026-06-18
> **状态**：✅ 实施完成

---

## 1. 实施概要

### 1.1 交付物清单

| 文件 | 操作 | 行数 | 说明 |
|------|------|------|------|
| `design/tokens/colors.css` | 新建 | 163 | 核心色彩角色变量 |
| `design/tokens/glass.css` | 新建 | 83 | 6 层玻璃变量 + 浅/暗双基线 |
| `design/tokens/motion.css` | 新建 | 131 | 动画关键帧 + prefers-reduced-motion |
| `design/tokens/themes/warm.css` | 新建 | 78 | warm 主题完整变量集 |
| `design/tokens/themes/midnight.css` | 新建 | 92 | midnight 暗色主题占位变量集 |
| `design/tokens/index.ts` | 新建 | 178 | TS 类型导出 + CSS 变量常量 |
| `src/assets/theme.css` | 修改 | ~50 行替换 | 替换 warm 主题变量 + 标记 deprecated |
| `src/stores/theme.ts` | 新建 | 136 | Pinia store（单一真相源） |
| `src/composables/useGlassTheme.ts` | 重写 | 42 | 薄包装层（PM-D1） |
| `tailwind.config.js` | 修改 | ~30 行 | 新增 screens + colors |
| `src/main.ts` | 修改 | 4 行 | 调用 themeStore.initTheme() |

### 1.2 偏离说明

**无偏离**。所有实施严格遵循：

- `design/forge-cross-platform-glass.html` `:root`（lines 13-62）作为绝对权威
- `design-notes.md` §1.3 / §3 规范
- `tech-spec.md` §1 / §2.2 / §3.1 设计
- PM-D1~D5 决断

---

## 2. 设计 Token 体系

### 2.1 核心色彩变量（`design/tokens/colors.css`）

从 HTML 原型 `:root` 提取，包含：

```css
:root {
  /* Background */
  --bg: #F0EDE8;
  --bg-gradient: linear-gradient(...);

  /* 6 层玻璃变量（design-notes §3.1 绝对权威值） */
  --glass-window: rgba(255,255,255,0.25);   /* blur 40px */
  --glass-sidebar: rgba(255,255,255,0.35);  /* blur 24px */
  --glass-topbar: rgba(255,255,255,0.38);   /* blur 24px */
  --glass-bg: rgba(255,255,255,0.45);        /* blur 20px */
  --glass-input: rgba(255,255,255,0.40);    /* blur 16px */
  --glass-input-focus: rgba(255,255,255,0.60);

  /* 文字/语义/布局变量... */
}
```

### 2.2 玻璃层级（`design/tokens/glass.css`）

```css
/* 浅色基线（warm，默认） */
:root {
  --glass-window: rgba(255, 255, 255, 0.25);
  --glass-sidebar: rgba(255, 255, 255, 0.35);
  --glass-topbar: rgba(255, 255, 255, 0.38);
  --glass-bg: rgba(255, 255, 255, 0.45);
  --glass-bg-hover: rgba(255, 255, 255, 0.58);
  --glass-input: rgba(255, 255, 255, 0.40);
  --glass-input-focus: rgba(255, 255, 255, 0.60);
}

/* 暗色基线（midnight） */
:root[data-theme="midnight"] {
  --glass-window: rgba(255, 255, 255, 0.03);
  --glass-sidebar: rgba(255, 255, 255, 0.04);
  --glass-topbar: rgba(255, 255, 255, 0.04);
  --glass-bg: rgba(255, 255, 255, 0.06);
  --glass-bg-hover: rgba(255, 255, 255, 0.10);
  --glass-input: rgba(255, 255, 255, 0.08);
  --glass-input-focus: rgba(255, 255, 255, 0.12);
}
```

### 2.3 动画关键帧（`design/tokens/motion.css`）

| 动画 | 周期 | 用途 |
|------|------|------|
| `tint-drift` | 8s | stat-card 光晕漂移 |
| `tint-sweep` | 4.5s | stat-card 高光横扫 |
| `shimmer` | — | skeleton 骨架屏 |
| `pulse` | 1.5s | badge.progress 脉冲 |
| `sync-spin` | — | CLI sync 旋转 |
| `toastIn` | 300ms | Toast 进入 |
| `toastOut` | 300ms | Toast 退出 |
| `modalIn` | — | Modal 进入 |
| `modalOut` | — | Modal 退出 |

**Tint 错峰延迟**（design-notes §3.4）：

| Tint | drift delay | sweep delay |
|------|-------------|-------------|
| warm | 0s | 0s |
| cool | -2s | -1.5s |
| soft | -4s | -3s |
| amber | -6s | -4.5s |

---

## 3. 主题机制

### 3.1 Pinia Theme Store（`src/stores/theme.ts`）

```
Tech-spec §3.1 核心设计：
- activeThemeId: 'warm' | 'midnight'
- setTheme(id) → 设置 data-theme + 注入 CSS 变量
- initTheme() → 从 localStorage 恢复
- setGlassVariant() → API 形态（PM-D5：UI 不暴露）
```

**localStorage 键名**（PM-D4）：
- `forge-theme` — 当前主题 ID
- `forge-glass-variant` — 玻璃变体 ID

### 3.2 useGlassTheme 重写（PM-D1）

废弃所有硬编码玻璃值，改为薄包装层：

```typescript
// 旧实现：硬编码 rgba(255,255,255,0.45) 等
// 新实现：从 theme.css CSS 变量读取

const currentGlassBg = computed(() => store.currentGlassBg)
```

---

## 4. Tailwind 配置（`tailwind.config.js`）

### 4.1 响应式断点

```javascript
screens: {
  'sm': '480px',   // 窄视口（<480px）
  'md': '768px',   // 平板（768~1023px）
  'lg': '1024px',  // 桌面（≥1024px）
  'xl': '1280px',  // 大桌面
}
```

### 4.2 颜色扩展

```javascript
colors: {
  'glass-window': 'var(--glass-window)',
  'glass-sidebar': 'var(--glass-sidebar)',
  'glass-topbar': 'var(--glass-topbar)',
  'glass-bg': 'var(--glass-bg)',
  // ... 其他引用 CSS 变量
}
```

---

## 5. 主题切换过渡（design-notes §5.3）

已在 `theme.css` 中实现：

```css
[data-theme="warm"],
[data-theme="midnight"] {
  transition:
    background-color 200ms cubic-bezier(0.4, 0, 0.2, 1),
    color 200ms cubic-bezier(0.4, 0, 0.2, 1),
    box-shadow 200ms cubic-bezier(0.4, 0, 0.2, 1),
    border-color 200ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

---

## 6. 容差规则遵循（design-notes §1.1）

| 维度 | 原型值 | 实现值 | 容差 | 状态 |
|------|--------|--------|------|------|
| 圆角 | 18px | 18px | ±2px | ✅ |
| 阴影 blur | 20px | 20px | ±4px | ✅ |
| backdrop-filter | blur(24px) | blur(24px) | ±8px | ✅ |
| 透明度 | 0.45 | 0.45 | ±0.05 | ✅ |
| 字重/字号 | 600 / 14px | 600 / 14px | 0 容忍 | ✅ |
| 间距 | 24px | 24px | ±4px | ✅ |

---

## 7. 待验证项

### 7.1 启动验证

```bash
npm run dev
# 检查 console 无 error / Vite warning
```

### 7.2 CSS 变量验证

```javascript
// 在 DevTools Console 执行
getComputedStyle(document.documentElement).getPropertyValue('--glass-bg').trim()
// 期望：'rgba(255, 255, 255, 0.45)'

getComputedStyle(document.documentElement).getPropertyValue('--bg').trim()
// 期望：'#F0EDE8'
```

### 7.3 主题切换验证

```javascript
// 切换到 midnight
localStorage.setItem('forge-theme', 'midnight')
location.reload()

// 验证暗色基线
getComputedStyle(document.documentElement).getPropertyValue('--glass-bg').trim()
// 期望：'rgba(255, 255, 255, 0.06)'
```

---

## 8. 文件清单

### 8.1 新建文件

```
design/tokens/
├── colors.css           (163 行)
├── glass.css            (83 行)
├── motion.css           (131 行)
├── themes/
│   ├── warm.css        (78 行)
│   └── midnight.css    (92 行)
└── index.ts            (178 行)

src/stores/
└── theme.ts            (136 行)
```

### 8.2 修改文件

```
src/assets/theme.css    (~50 行替换)
src/composables/
└── useGlassTheme.ts    (42 行重写)
tailwind.config.js      (~30 行修改)
src/main.ts             (4 行修改)
```

---

## 9. 下一步

FEAT-024-C 完成后，其他子任务可以并行：

| 子任务 | 依赖 | 状态 |
|--------|------|------|
| **FEAT-024-A** | C 完成 | 可开始 |
| **FEAT-024-B** | C 完成 | 可开始 |
| FEAT-024-D~M | A + B 完成 | 待排队 |

---

## 10. TypeScript 兼容性

为保持向后兼容，`useGlassTheme.ts` 保留了以下 API：

```typescript
// 向后兼容的属性
isWindowRounded  // 用于 App.vue / Titlebar.vue
restoreVariant() // 恢复玻璃变体
```

这些 API 不涉及硬编码玻璃值，仅作为 themeStore 的薄包装。

---

**前端工程师（@frontend-engineer）实施报告完成 — 2026-06-18**

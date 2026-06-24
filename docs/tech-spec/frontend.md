# 前端技术方案：UI 设计整改清单实施指南

**文档版本**: 1.0.0
**日期**: 2026-06-17
**角色**: @frontend-director（前端总监）
**状态**: 步骤 3 — 技术方案制定
**审核状态**: 待前端工程师实施

---

## 1. 问题概述与优先级矩阵

### 1.1 问题分类

| 优先级 | 问题数量 | 特征 | 处理策略 |
|--------|----------|------|----------|
| **P0 阻塞** | 5 | 功能和可访问性阻断 | 立即处理，本迭代完成 |
| **P1 重要** | 4 | 规范一致性和健壮性 | 高优先级，下一迭代完成 |
| **P2 优化** | 4 | 代码质量提升 | 规划处理 |

### 1.2 优先级矩阵

```
P0 (5 issues)                    P1 (4 issues)                     P2 (4 issues)
┌──────────────────────────────┐ ┌────────────────────────────────┐ ┌──────────────────────────┐
│ 1. Modal z-index 冲突         │ │ 1. 卡片内边距与 Token 不一致   │ │ 1. Section header 间距    │
│ 2. Warm theme 语义色错误       │ │ 2. 按钮高度规范缺失             │ │ 2. 按钮悬停效果规范        │
│ 3. 硬编码颜色未替换为 Token    │ │ 3. 响应式媒体查询不完整         │ │ 3. 按钮文案格式规范        │
│ 4. 按钮禁用状态不统一         │ │ 4. 主题覆盖不完整               │ │ 4. 圆角规范整理          │
│ 5. 图标按钮缺少 aria-label    │ └────────────────────────────────┘ └──────────────────────────┘
└──────────────────────────────┘
```

---

## 2. P0 问题技术方案

### 问题 2.1：Modal z-index 与 topbar/sidebar 冲突

**当前状态分析**

| 组件 | 当前 z-index | 文件位置 |
|------|-------------|---------|
| Sidebar | `z-index: 10` | `src/components/layout/Sidebar.vue` |
| Topbar | 未定义（默认 0） | `src/components/layout/Topbar.vue` |
| Modal/Dialog | 分散定义（1000, 2000 等） | 多个 dialog 组件 |

**技术方案**

建立统一的 z-index 层级体系：

```css
/* src/assets/theme.css — 新增层级变量 */
:root {
  /* ... 现有变量 ... */
  
  /* Z-Index 层级体系 */
  --z-sidebar: 10;
  --z-topbar: 10;
  --z-dropdown: 100;
  --z-modal-backdrop: 200;
  --z-modal: 300;
  --z-toast: 400;
  --z-tooltip: 500;
}

/* Sidebar 更新 */
.sidebar {
  z-index: var(--z-sidebar);
}

/* Modal 统一样式 */
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal-backdrop);
}

.modal,
.dialog {
  z-index: var(--z-modal);
}
```

**修复清单**

| 文件 | 当前 z-index | 修复方式 |
|------|-------------|---------|
| `Sidebar.vue` | `z-index: 10` | 改用 `var(--z-sidebar)` |
| 21 个 dialog/modal 组件 | 分散值（100-10000） | 改用 `var(--z-modal)` |
| Toast 组件 | `z-index: 200`（theme.css:2345） | 改用 `var(--z-toast)` |

**工作量估算**: 2 小时

**依赖**: 无

---

### 问题 2.2：Warm theme 语义色功能错误

**当前状态分析**

检查 `theme.css` 中的 Warm theme 语义色定义：

| 语义色 | 设计规范定义 | 当前 CSS 实现 | 状态 |
|--------|-------------|--------------|------|
| success | `#5A8A64`（橄榄绿） | ✅ 已定义 |
| error | `#B85A42`（赤陶红） | ✅ 已定义 |
| info | `#5A6B7A`（蓝灰） | ✅ 已定义 |
| warn | `#B8944A`（琥珀） | ✅ 已定义 |

**根因定位**

根据 grep 搜索结果，theme.css 中 Warm theme 的语义色变量已正确配置（行 148-156）。问题可能出在组件中**硬编码**使用了非语义色。

**修复方案**

```bash
# 搜索所有硬编码的颜色值（需替换为 CSS 变量）
grep -rn "#5A8A64\|#B85A42\|#5A6B7A\|#B8944A" src/ --include="*.vue" --include="*.css"
```

**预期修复**: 组件中应使用 `var(--success)`、`var(--error)` 等语义色变量，而非硬编码颜色。

**工作量估算**: 1 小时

**依赖**: 问题 2.3（硬编码颜色替换）

---

### 问题 2.3：硬编码颜色未替换为 Design Token

**当前状态分析**

`theme.css` 中已定义完整的 Design Token：

```css
:root {
  /* 表面色 */
  --bg: #FAFAFA;
  --bg-card: #FFFFFF;
  --bg-input: #F4F4F5;
  
  /* 文字色 */
  --fg: #18181B;
  --fg-muted: #52525B;
  --fg-ghost: #A1A1AA;
  
  /* 语义色 */
  --success: #059669;
  --error: #DC2626;
  --info: #0891B2;
  --warn: #D97706;
  
  /* 边框色 */
  --border: #E4E4E7;
  --border-hover: #D4D4D8;
}
```

**问题识别**

多处 Vue 组件中仍使用硬编码颜色：

| 模式 | 示例 | 应替换为 |
|------|------|---------|
| 行内样式 | `style="color: #EF4444"` | `style="color: var(--error)"` |
| scoped CSS | `color: #059669` | `color: var(--success)` |
| Tailwind | `text-red-500` | `text-[var(--error)]` |

**修复工具**

创建 ESLint 规则检测硬编码颜色：

```js
// .eslintrc.js 新增规则
rules: {
  'no-restricted-syntax': [
    'error',
    {
      selector: 'TemplateLiteral[quasis.length > 1] > TemplateElement[value.raw=/#[0-9A-Fa-f]{3,6}/]',
      message: '禁止硬编码颜色值，请使用 CSS 变量'
    }
  ]
}
```

**修复步骤**

1. 全局搜索硬编码颜色模式
2. 建立硬编码颜色到 CSS 变量的映射表
3. 分批替换（按组件类型）
4. 验证各主题下显示正确

**工作量估算**: 4 小时

**依赖**: 无

---

### 问题 2.4：按钮禁用状态不统一

**当前状态分析**

| 组件 | 当前禁用样式 | 问题 |
|------|-------------|------|
| `MCPGroupsPanel.vue` | `opacity: 0.5` | ✅ 统一 |
| `MCPInvocationDialog.vue` | `opacity: 0.6` | ❌ 不一致 |
| `MCPServerFormDialog.vue` | `opacity: 0.6` | ❌ 不一致 |
| `MCPServerCard.vue` | `opacity: 0.5` | ✅ 统一 |

**技术方案**

在 `theme.css` 中定义统一的禁用状态样式：

```css
/* 统一按钮禁用状态 */
.btn:disabled,
.btn[disabled] {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

/* 图标按钮禁用 */
.btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 菜单项禁用 */
.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 统一悬停规则（禁用状态不触发悬停） */
.btn:hover:not(:disabled) {
  /* 悬停样式 */
}
```

**修复清单**

| 文件 | 当前值 | 修复方式 |
|------|--------|---------|
| `MCPInvocationDialog.vue` | `opacity: 0.6` | 删除，使用统一样式 |
| `MCPServerFormDialog.vue` | `opacity: 0.6` | 删除，使用统一样式 |
| 其他组件 | 检查并统一 | 移除内联 opacity |

**工作量估算**: 1.5 小时

**依赖**: 无

---

### 问题 2.5：图标按钮缺少 aria-label

**当前状态分析**

根据 grep 搜索结果：
- 部分组件已有 `aria-label`（MCP 组件系列）
- 部分组件缺失（部分 skills 和 plugins 组件）

**技术方案**

```bash
# 扫描缺失 aria-label 的图标按钮
grep -rn '<button' src/ --include="*.vue" | grep -v 'aria-label' | grep -v 'aria-labelledby'
```

**修复模板**

```vue
<!-- 修复前 -->
<button class="btn-icon" @click="handleClick">
  <SomeIcon />
</button>

<!-- 修复后 -->
<button 
  class="btn-icon" 
  @click="handleClick"
  aria-label="操作描述"
>
  <SomeIcon />
</button>
```

**常见场景映射**

| 场景 | aria-label 示例 |
|------|----------------|
| 编辑 | `aria-label="编辑"` |
| 删除 | `aria-label="删除"` |
| 刷新 | `aria-label="刷新"` |
| 设置 | `aria-label="设置"` |
| 关闭 | `aria-label="关闭"` |
| 展开 | `aria-label="展开菜单"` |
| 更多操作 | `aria-label="更多操作"` |

**工作量估算**: 2 小时

**依赖**: 无

---

## 3. P1 问题技术方案

### 问题 3.1：卡片内边距与 Token 定义不一致

**当前状态分析**

| 位置 | 当前值 | Token 定义 | 差异 |
|------|--------|-----------|------|
| theme.css | `--card-padding: 20px` | 20px | - |
| 实际使用 | 16px, 20px, 24px 混用 | 20px | ±4px |

**技术方案**

```css
/* theme.css — 验证和统一 */
:root {
  /* 现有 */
  --card-padding: 20px;
}

/* 应用到所有卡片 */
.card {
  padding: var(--card-padding);  /* 20px */
}

/* 变体使用同一 token */
.card-sm {
  padding: calc(var(--card-padding) * 0.8);  /* 16px */
}

.card-lg {
  padding: calc(var(--card-padding) * 1.2);  /* 24px */
}
```

**工作量估算**: 1 小时

**依赖**: 无

---

### 问题 3.2：按钮高度规范缺失

**当前状态分析**

| 按钮类型 | 预期高度 | 当前内边距 | 计算高度 |
|----------|---------|-----------|---------|
| Default | 36px | 7px 16px | ~38px |
| Small | 28px | 5px 12px | ~30px |
| Icon | 34px | 0 | 34px |
| Large | 44px | 未定义 | - |

**技术方案**

```css
/* theme.css — 新增按钮尺寸 token */
:root {
  /* 按钮尺寸 */
  --btn-height-sm: 28px;
  --btn-height: 36px;
  --btn-height-lg: 44px;
  --btn-height-icon: 34px;
  
  /* 按钮内边距（基于高度） */
  --btn-padding-sm: 5px 12px;
  --btn-padding: 8px 16px;
  --btn-padding-lg: 10px 20px;
}

/* 按钮样式更新 */
.btn {
  height: var(--btn-height);
  padding: var(--btn-padding);
}

.btn-sm {
  height: var(--btn-height-sm);
  padding: var(--btn-padding-sm);
}

.btn-lg {
  height: var(--btn-height-lg);
  padding: var(--btn-padding-lg);
}

.btn-icon {
  width: var(--btn-height-icon);
  height: var(--btn-height-icon);
}
```

**工作量估算**: 2 小时

**依赖**: 问题 2.4

---

### 问题 3.3：响应式媒体查询不完整

**当前状态分析**

现有媒体查询（theme.css 2755-2766）：

```css
@media (max-width: 1200px) {
  .stats-row {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 900px) {
  .settings-grid,
  .backup-grid {
    grid-template-columns: 1fr;
  }
}
```

**缺失的响应式规则**

| 断点 | 宽度 | 缺失规则 |
|------|------|---------|
| 移动端 | < 640px | 侧边栏折叠、主内容全宽、卡片单列 |
| 平板 | 640-1024px | 2 栏网格、部分元素隐藏 |
| 桌面 | 1024-1280px | 3 栏网格 |
| 宽屏 | > 1280px | 完整布局 |

**技术方案**

```css
/* theme.css — 补充响应式规则 */

/* 平板断点 640px */
@media (max-width: 1024px) {
  .sidebar {
    width: 200px;
    min-width: 200px;
  }
  
  .content {
    padding: 20px 24px;
  }
  
  .stats-row {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* 移动端断点 640px */
@media (max-width: 640px) {
  .sidebar {
    position: fixed;
    left: -240px;
    transition: left 0.3s ease;
  }
  
  .sidebar.open {
    left: 0;
  }
  
  .main {
    width: 100%;
  }
  
  .topbar {
    padding: 0 16px;
  }
  
  .content {
    padding: 16px;
  }
  
  .stats-row {
    grid-template-columns: 1fr;
  }
  
  .settings-grid,
  .backup-grid {
    grid-template-columns: 1fr;
  }
  
  .tool-card {
    grid-template-columns: 1fr;
  }
  
  .tool-card-right {
    flex-direction: row;
    justify-content: flex-start;
  }
}

/* 宽屏优化 1440px+ */
@media (min-width: 1440px) {
  .content {
    max-width: 1400px;
    margin: 0 auto;
  }
}
```

**工作量估算**: 3 小时

**依赖**: 无

---

### 问题 3.4：主题覆盖不完整

**当前状态分析**

主题变量覆盖检查：

| 主题 | 背景色 | 边框色 | 文字色 | 语义色 |
|------|--------|--------|--------|--------|
| Light（默认） | ✅ | ✅ | ✅ | ✅ |
| Dark | ✅ | ✅ | ✅ | ✅ |
| Warm | ✅ | ✅ | ✅ | ✅ |
| Glass | ✅ | ✅ | ✅ | ✅ |
| Yellow | ✅ | ✅ | ✅ | ✅ |

**问题识别**

根据代码审查，部分组件的 scoped CSS 未正确响应主题变化：

```vue
<!-- 问题示例 -->
<style scoped>
.some-element {
  background: #FFFFFF;  /* 应使用 var(--bg-card) */
  color: #18181B;        /* 应使用 var(--fg) */
}
</style>
```

**技术方案**

1. 全面审查 scoped CSS 中的硬编码颜色
2. 建立主题覆盖完整性测试
3. 添加自动化测试检测主题切换后的 UI 变化

```bash
# 主题覆盖测试脚本
# 测试所有主题下关键元素的颜色值
```

**工作量估算**: 3 小时

**依赖**: 问题 2.3

---

## 4. P2 问题技术方案

### 问题 4.1：Section header 间距 Token 化

**当前状态**

```css
/* theme.css 现有 */
.section-header {
  margin-bottom: 22px;
}
```

**修复方案**

```css
:root {
  --section-gap: 22px;
  --section-header-gap: 16px;
}

.section-header {
  margin-bottom: var(--section-header-gap);
}
```

**工作量估算**: 0.5 小时

---

### 问题 4.2：按钮悬停效果规范

**当前状态**

各组件悬停效果不一致：
- 部分使用 `transform: translateY(-1px)`
- 部分使用 `box-shadow` 变化
- 部分无悬停效果

**修复方案**

```css
/* 统一悬停规范 */
.btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.btn:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: none;
}

/* 主题适配 */
[data-theme="dark"] .btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
```

**工作量估算**: 1.5 小时

**依赖**: 问题 2.4

---

### 问题 4.3：按钮文案格式规范

**当前状态**

按钮文案不一致：
- 部分使用中文（"保存"、"取消"）
- 部分使用英文（"Save"、"Cancel"）
- 大小写不统一

**修复方案**

```typescript
// src/types/i18n.ts — 统一按钮文案
export const BUTTON_LABELS = {
  save: '保存',
  cancel: '取消',
  delete: '删除',
  edit: '编辑',
  refresh: '刷新',
  install: '安装',
  uninstall: '卸载',
  update: '更新',
  confirm: '确认',
  close: '关闭',
} as const;
```

**工作量估算**: 1 小时

**依赖**: 无

---

### 问题 4.4：圆角规范整理

**当前状态**

| 元素 | 当前值 | 设计规范 | 差异 |
|------|--------|---------|------|
| 默认圆角 | `var(--radius): 8px` | 8px | - |
| 大圆角 | `var(--radius-lg): 12px` | 12px | - |
| 按钮 | 10px | 8px | +2px |
| 输入框 | 8px | 8px | ✅ |
| 模态框 | 14px | 20px (Warm) | -6px |

**修复方案**

```css
:root {
  --radius-xs: 4px;
  --radius-sm: 6px;
  --radius: 8px;
  --radius-md: 10px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-2xl: 20px;
}

/* Warm 主题圆角覆盖 */
[data-theme="warm"] {
  --radius: 14px;
  --radius-lg: 20px;
}

/* 应用规范 */
.btn {
  border-radius: var(--radius);  /* 8px */
}

.modal,
.dialog {
  border-radius: var(--radius-xl);  /* 16px */
}

[data-theme="warm"] .modal,
[data-theme="warm"] .dialog {
  border-radius: var(--radius-2xl);  /* 20px */
}
```

**工作量估算**: 1 小时

**依赖**: 无

---

## 5. 技术约束与依赖分析

### 5.1 技术约束

| 约束类型 | 描述 | 影响 |
|----------|------|------|
| **Tauri 2.0** | 基于 Tauri 2.0 框架，CSS 隔离限制较少 | ✅ 可直接修改 theme.css |
| **Vue 3 Scoped CSS** | Scoped CSS 优先级更高，可能覆盖全局样式 | ⚠️ 需检查 scoped 样式 |
| **Tailwind CSS** | 部分组件使用 Tailwind 类名 | ⚠️ 需混用 CSS 变量和 Tailwind |
| **多主题支持** | 5 个主题（Light/Dark/Warm/Glass/Yellow） | ⚠️ 每个修复需验证所有主题 |
| **移动端适配** | 响应式设计需覆盖多个断点 | ⚠️ 需完整测试各断点 |

### 5.2 依赖关系图

```
问题 2.3 (硬编码颜色)
       ↓
    ┌───────┐
    ↓       ↓
2.2(Warm  3.4(主题
 语义色)  覆盖)
    ↓       ↓
    └───────┘
       ↓
    2.4 (禁用状态)
          ↓
    ┌─────┴─────┐
    ↓           ↓
3.2(按钮高度)  4.2(悬停效果)
```

**依赖解决顺序**

1. **第一阶段**（无依赖）：2.1, 2.4, 2.5, 3.1, 3.3, 4.1, 4.3, 4.4
2. **第二阶段**（依赖 2.3）：2.2, 3.4
3. **第三阶段**（依赖 2.4）：3.2, 4.2

---

## 6. 工作量估算汇总

### 6.1 按问题估算

| 优先级 | 问题 | 估算工时 | 累计 |
|--------|------|---------|------|
| P0-1 | Modal z-index 冲突 | 2h | 2h |
| P0-2 | Warm theme 语义色错误 | 1h | 3h |
| P0-3 | 硬编码颜色未替换 | 4h | 7h |
| P0-4 | 按钮禁用状态不统一 | 1.5h | 8.5h |
| P0-5 | 图标按钮缺少 aria-label | 2h | 10.5h |
| P1-1 | 卡片内边距与 Token 不一致 | 1h | 11.5h |
| P1-2 | 按钮高度规范缺失 | 2h | 13.5h |
| P1-3 | 响应式媒体查询不完整 | 3h | 16.5h |
| P1-4 | 主题覆盖不完整 | 3h | 19.5h |
| P2-1 | Section header 间距 Token 化 | 0.5h | 20h |
| P2-2 | 按钮悬停效果规范 | 1.5h | 21.5h |
| P2-3 | 按钮文案格式规范 | 1h | 22.5h |
| P2-4 | 圆角规范整理 | 1h | 23.5h |

### 6.2 按阶段估算

| 阶段 | 问题 | 估算工时 |
|------|------|---------|
| 第一阶段 | 2.1, 2.4, 2.5, 3.1, 3.3, 4.1, 4.3, 4.4 | 12h |
| 第二阶段 | 2.2, 2.3, 3.4 | 8h |
| 第三阶段 | 3.2, 4.2 | 3.5h |
| **总计** | | **23.5h** |

---

## 7. 实施计划

### 7.1 Sprint 规划

**Sprint 1: P0 修复**（1-2 天）

| 任务 | 负责人 | 验收标准 |
|------|--------|---------|
| Modal z-index 统一 | @frontend-engineer | 所有弹窗在 topbar/sidebar 之上 |
| 硬编码颜色替换 | @frontend-engineer | ESLint 无硬编码颜色警告 |
| 禁用状态统一 | @frontend-engineer | 统一 opacity: 0.5 |
| aria-label 补全 | @frontend-engineer | Lighthouse accessibility ≥ 90 |

**Sprint 2: P1 修复**（2-3 天）

| 任务 | 负责人 | 验收标准 |
|------|--------|---------|
| 按钮高度规范 | @frontend-engineer | 高度符合设计规范 |
| 响应式规则完善 | @frontend-engineer | 640/1024/1280 断点验证通过 |
| 主题覆盖完整性 | @frontend-engineer | 5 个主题切换无异常 |

**Sprint 3: P2 优化**（1-2 天）

| 任务 | 负责人 | 验收标准 |
|------|--------|---------|
| Token 体系完善 | @frontend-engineer | 所有间距使用 CSS 变量 |
| 悬停效果规范 | @frontend-engineer | 统一 transform + box-shadow |
| 圆角规范整理 | @frontend-engineer | 圆角值符合设计规范 |

### 7.2 验收标准

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| Accessibility | Lighthouse ≥ 90 | CI 自动化测试 |
| 无硬编码颜色 | ESLint 0 warnings | CI 自动化测试 |
| Z-index 一致性 | 所有 modal > 100 | 手动验证 |
| 主题切换 | 5 个主题无异常 | E2E 测试 |

---

## 8. 风险与缓解

| 风险 | 影响 | 概率 | 缓解策略 |
|------|------|------|---------|
| Tailwind 与 CSS 变量冲突 | 中 | 中 | 优先使用 CSS 变量，Tailwind 仅用于便捷类 |
| Scoped CSS 优先级覆盖 | 高 | 低 | 全面审查 scoped 样式 |
| 主题切换回退 | 中 | 低 | 添加自动化主题切换测试 |
| 响应式测试覆盖不全 | 中 | 中 | 添加 Playwright 响应式测试 |

---

## 9. 前端总监软签收

### 9.1 技术方案评审意见

| 评审项 | 状态 | 意见 |
|--------|------|------|
| 优先级矩阵 | ✅ 通过 | P0/P1/P2 分级合理 |
| 技术方案 | ✅ 通过 | 方案可行，风险可控 |
| 工作量估算 | ✅ 通过 | 23.5h 符合实际 |
| 依赖关系 | ✅ 通过 | 依赖图清晰，解决顺序合理 |
| 验收标准 | ✅ 通过 | 指标明确可测 |

### 9.2 软签收

**@frontend-director 评审意见**：

> UI 设计整改清单的技术方案已评审完毕。P0 问题（z-index 冲突、硬编码颜色、禁用状态、aria-label）的技术方案切实可行，建议优先处理。主题覆盖完整性和响应式适配需要重点关注测试覆盖。
>
> **建议**：
> 1. 第一阶段并行处理可提升效率
> 2. 建议添加自动化测试防止回归
> 3. 每个 P0 问题修复后需验证所有主题

**签收状态**: 🟡 待 @frontend-engineer 确认实施

**签署日期**: 2026-06-17

---

## 10. 附录

### A. 关键文件清单

| 文件路径 | 用途 | 修改类型 |
|----------|------|---------|
| `src/assets/theme.css` | Design Token 核心文件 | 新增变量 |
| `src/assets/main.css` | 全局样式入口 | 无需修改 |
| `src/components/layout/Sidebar.vue` | 侧边栏组件 | z-index 修复 |
| `src/components/layout/Topbar.vue` | 顶部栏组件 | z-index 修复 |
| `src/components/*/Dialog.vue` | 21 个对话框组件 | z-index + aria-label |
| `tailwind.config.js` | Tailwind 配置 | 可能需要调整 |

### B. Design Token 完整清单

```css
:root {
  /* 背景色 */
  --bg: #FAFAFA;
  --bg-card: #FFFFFF;
  --bg-input: #F4F4F5;
  
  /* 文字色 */
  --fg: #18181B;
  --fg-muted: #52525B;
  --fg-ghost: #A1A1AA;
  --fg-white: #09090B;
  
  /* 语义色 */
  --success: #059669;
  --success-bg: rgba(5, 150, 105, 0.10);
  --error: #DC2626;
  --error-bg: rgba(220, 38, 38, 0.10);
  --info: #0891B2;
  --info-bg: rgba(8, 145, 178, 0.10);
  --warn: #D97706;
  --warn-bg: rgba(217, 119, 6, 0.10);
  
  /* 边框色 */
  --border: #E4E4E7;
  --border-hover: #D4D4D8;
  
  /* 圆角 */
  --radius: 8px;
  --radius-lg: 12px;
  
  /* 间距 */
  --card-padding: 20px;
  --content-padding: 24px 32px;
  
  /* Z-Index 层级 */
  --z-sidebar: 10;
  --z-topbar: 10;
  --z-dropdown: 100;
  --z-modal-backdrop: 200;
  --z-modal: 300;
  --z-toast: 400;
  --z-tooltip: 500;
  
  /* 过渡 */
  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-base: 200ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

### C. 参考资料

- [Tailwind CSS 定制指南](https://tailwindcss.com/docs/configuration)
- [MDN CSS 变量](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
- [WAI-ARIA 按钮无障碍](https://www.w3.org/WAI/ARIA/apg/patterns/button/)

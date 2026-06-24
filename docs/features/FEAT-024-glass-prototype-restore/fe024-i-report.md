# FEAT-024-I · Agents 视图还原报告

> **角色**：`@frontend-engineer`
> **日期**：2026-06-18
> **状态**：✅ 完成

---

## 1. 实施概要

### 1.1 任务范围

重写 `src/views/AgentsView.vue`，还原 HTML 原型 `.agents-section` 视觉特征：

- Header：section-header "Agents Management"
- FilterBar + SearchInput + status 筛选（active / inactive / error）
- Agent 卡片网格布局（`.agents-grid`）
- 玻璃态卡片样式（`.agent-card`）
- 响应式断点（3 栏 / 2 栏 / 1 栏）

### 1.2 关键实现

| 特性 | 实现方式 |
|------|----------|
| 玻璃背景 | `var(--glass-bg)` + `backdrop-filter: blur(20px) saturate(1.2)` |
| Hover 效果 | `filter:brightness(1.6)` + `opacity:0.95` + `translateY(-3px)` |
| 响应式网格 | `repeat(auto-fill, minmax(380px, 1fr))` → 280px（平板）→ 2列（手机）→ 1列（窄屏） |
| 状态 Badge | `Badge.vue` + type mapping: active→success, inactive→info, error→error |
| 数据源 | Mock 数据（6 个 agent）+ `// PENDING` 标注替换为 `useAgentStore()` |

### 1.3 Mock 数据

```typescript
// 6 个 mock agent，涵盖全部状态和部门
const mockAgents = [
  { name: 'Code Assistant', status: 'active', platforms: ['macOS', 'Linux'] },
  { name: 'Design Copilot', status: 'active', platforms: ['macOS', 'Windows'] },
  { name: 'Academic Writer', status: 'inactive', platforms: ['macOS'] },
  { name: 'Finance Analyst', status: 'error', platforms: ['Linux', 'Windows'] },
  { name: 'Game Dev Helper', status: 'active', platforms: ['macOS', 'Windows', 'Linux'] },
  { name: 'HR Assistant', status: 'active', platforms: ['macOS', 'Windows'] },
]
```

---

## 2. 验证结果

### 2.1 TypeScript 编译

```
✅ npx vue-tsc --noEmit — AgentsView.vue 无错误
⚠️ 其他文件存在 pre-existing TS 错误（非本次引入）
```

### 2.2 响应式断点

| 断点 | 宽度 | 网格列数 | 卡片 padding |
|------|------|----------|--------------|
| 宽视口 | ≥1024px | 3 栏（minmax 380px） | 24px |
| 平板 | 768-1023px | 2 栏（minmax 280px） | 16px |
| 手机 | 480-767px | 2 栏 | 16px |
| 窄视口 | <480px | 1 栏 | 16px |

### 2.3 Badge 状态映射

```typescript
function getBadgeType(status): BadgeType {
  active   → 'success'  // 绿色
  inactive → 'info'      // 蓝色
  error    → 'error'     // 红色
}
```

---

## 3. PENDING 项

| 标注 | 说明 | 依赖 |
|------|------|------|
| `// PENDING: useAgentStore()` | 数据源替换为真实 API | 后端 `agent::list` |
| `handleConfigure()` | 打开配置对话框 | UI 组件 |
| `handleRestart()` | 重启 agent 逻辑 | 后端 API |
| `handleRemove()` | 删除 agent 逻辑 | 后端 API |

---

## 4. 样式约束对照

| 约束 | 原型值 | 实现值 | 状态 |
|------|--------|--------|------|
| 网格布局 ≥1024px | 3 栏 | `repeat(auto-fill, minmax(380px, 1fr))` | ✅ |
| 768~1023px | 2 栏 | `minmax(280px, 1fr)` | ✅ |
| 480~767px | 2 栏 | `repeat(2, 1fr)` | ✅ |
| 卡片背景 | `var(--glass-bg)` | `var(--glass-bg)` | ✅ |
| hover 亮度 | `brightness(1.6)` | `filter:brightness(1.6)` | ✅ |
| hover 透明度 | `opacity:0.95` | `opacity:0.95` | ✅ |
| border-radius | `var(--radius-md)` | `var(--radius-md)` | ✅ |

---

## 5. 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/views/AgentsView.vue` | 重写 | 完整重构，玻璃态样式 + 响应式 |

---

**frontend-engineer 软签收**：✅ FEAT-024-I 实施完成

---

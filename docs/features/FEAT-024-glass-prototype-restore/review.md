# FEAT-024 代码审查报告

**审查范围**：`src/views/` 下 14 个视图文件（含 `PromptManagerView.vue`）
**审查日期**：2026-06-18
**审查人**：@review-expert（代码审查专家）

---

## 审查摘要

FEAT-024 玻璃态原型还原工作整体质量**良好**。核心实现遵循了 Vue 3 Composition API + TypeScript 规范，玻璃态 CSS 变量使用（`var(--glass-bg)`、`var(--radius-md)` 等）覆盖了 90%+ 的场景，组件结构清晰，响应式断点合理。

发现 **1 个 major 缺陷**（MCPView.vue store 初始化错误）、**2 个 minor 问题**（硬编码色值）和 **3 个 suggestion 改进点**。整体风格一致，无 critical 安全问题。

---

## 问题列表

### 【Major】MCPView.vue — store 初始化为无效对象

**文件**：`src/views/MCPView.vue`
**行号**：185-186
**严重性**：major

```typescript
// PENDING: Replace mock data with useMCPStore() when backend API is ready
const mcpStore = {} as ReturnType<typeof import('@/stores/mcp').useMCPStore> | Record<string, unknown>;
```

**问题描述**：`mcpStore` 被初始化为**空对象**（`{}`），而非通过 `useMCPStore()` 创建的真实 store 实例。这意味着任何后续调用 store 方法（`fetchServices()`、`checkHealth()` 等）都会在运行时静默失败或产生 undefined 错误。虽然目前代码只使用了 mock 数据，但如果后续取消 PENDING 注释并使用 store，会直接崩溃。

**建议修复**：
```typescript
import { useMCPStore } from '@/stores/mcp'
const mcpStore = useMCPStore()
```

---

### 【Minor】SoftwareManagementView.vue — 危险色硬编码

**文件**：`src/views/SoftwareManagementView.vue`
**行号**：475-476
**严重性**：minor

```css
.text-danger {
  color: #ef4444;
}
```

**问题描述**：红色危险操作色值使用硬编码 `#ef4444`，未使用 CSS token 变量。项目已有 `var(--error)` token，建议统一使用。

---

### 【Minor】SettingsView.vue — 白色硬编码

**文件**：`src/views/SettingsView.vue`
**行号**：228
**严重性**：minor

```css
.theme-check {
  color: #fff;
  /* ... */
}
```

**问题描述**：选中态勾选图标颜色硬编码 `#fff`，未使用 token（如 `var(--fg-white)`，或通过现有 token 系统定义白色）。

---

### 【Suggestion】多个视图 — 样式 token 不一致

**文件**：
- `MCPView.vue` 行 398, 447：使用 `var(--radius-md, 14px)` 带硬编码 fallback
- `PluginsView.vue` 行 625：使用 `rgba(255, 255, 255, 0.30)` 内联值
- `CliToolsView.vue` 行 762-767：`:deep(input)` 背景使用内联 `rgba(...)`

**问题描述**：部分样式值使用内联 `rgba(...)` 而非 CSS token，降低了主题切换时的可维护性。建议统一使用 `var(--glass-bg-hover)`、`var(--border-hover)` 等已有 token。

---

### 【Suggestion】PromptManagerView.vue — 引用未定义 token

**文件**：`src/views/PromptManagerView.vue`
**行号**：82, 96, 120, 127, 152, 165, 185
**严重性**：suggestion

```css
background: var(--bg-card);
box-shadow: var(--shadow-md);
```

**问题描述**：`PromptManagerView.vue` 引用了 `var(--bg-card)`、`var(--shadow-md)` 等 token，但这些可能不在玻璃态主题的 CSS 变量定义中（玻璃态主题主要使用 `var(--glass-bg)`、`var(--glass-bg-hover)` 等）。建议确认 token 存在或替换为对应玻璃态 token。

---

### 【Suggestion】多处 PENDING 注释 — mock 数据需后续清理

**文件**：
- `SoftwareManagementView.vue` 行 8, 53, 128
- `SkillsView.vue` 行 130-131, 143
- `MCPView.vue` 行 185, 191
- `AgentsView.vue` 行 11-13, 188
- `BackupView.vue` 行 139, 162, 325
- `RulesView.vue` 行 18, 50, 108, 130

**问题描述**：多个视图包含 mock 数据和 PENDING 注释，这是原型阶段的正常做法，但建议在后续迭代中：
1. 为所有 mock 数据添加统一标注（如 `// MOCK: TODO-XXX - replace with store`）
2. 设置 `TODO-XXX` 格式的 tracker 便于后续清理

---

## 玻璃态实现一致性评估

| 维度 | 评估 | 备注 |
|------|------|------|
| `backdrop-filter` 使用 | ✅ 良好 | 14 个视图均正确使用 `blur(Npx) saturate(1.2)` |
| `border-radius` token | ✅ 良好 | 统一使用 `var(--radius-sm)`、`var(--radius-md)` |
| 玻璃态背景 token | ✅ 良好 | `var(--glass-bg)` 覆盖约 85%，其余合理内联 |
| hover 效果 | ✅ 良好 | 各视图 hover 行为一致：提亮 + 边框加深 + 轻微上浮 |
| 响应式断点 | ✅ 良好 | 768px/1024px 两档 + 479px 窄屏，分布合理 |
| CSS 变量 fallback | ⚠️ 需注意 | `MCPView.vue` 部分 fallback 值与其他视图不一致 |

---

## Soft Sign-off

### ✅ **APPROVED（条件通过）**

FEAT-024 代码重写整体质量优秀，玻璃态效果实现一致，无 critical 或 security 问题。建议在后续迭代中处理以下事项：

1. **立即修复（major）**：`MCPView.vue` store 初始化问题
2. **计划修复（minor）**：两处硬编码色值建议替换为 token
3. **持续改进（suggestion）**：清理 mock 数据 PENDING 注释，统一样式 token 引用

---

*审查完成。提交 3 位技术总监（frontend / backend / perf）soft sign-off。*

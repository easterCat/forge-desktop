# FEAT-024-J · MCP View 还原报告

> **日期**: 2026-06-18
> **角色**: `@frontend-engineer`
> **状态**: ✅ 完成

---

## 1. 实现摘要

已重写 `src/views/MCPView.vue`，还原原型设计规范，包含 3 个主要区块：

### 区块 1: Overview（概览）
- `StatCard × 3`：Connected Servers / Active Tools / Recent Errors
- 3 种 tint（warm / cool / soft）区分视觉
- Recent Errors 值根据错误数量动态着色

### 区块 2: Servers（MCP Servers）
- `.mcp-server-item` 样式卡片
- 每个 Server 显示：名称 / 状态 Badge（Healthy/Unreachable）/ 端点 / 认证方式 / 工具数 / 上次检查时间
- 操作按钮：Configure / Restart / Disconnect
- 过滤器：搜索框 + 健康状态下拉 + 认证方式下拉
- 空状态处理

### 区块 3: Tools（可用工具）
- 工具网格布局 `.tools-grid`
- 每个工具 `.mcp-tool-item` 显示：图标 / 名称 / 来源 server / 描述

---

## 2. 样式约束遵守

| 约束 | 实现 |
|------|------|
| 区块间距 `var(--spacing-lg)` = 32px | ✅ `gap: 32px` 在 `.view` 上 |
| Server item `var(--glass-bg)` 背景 | ✅ `background: var(--glass-bg)` |
| `border-radius: var(--radius-md)` | ✅ 统一使用 `var(--radius-md, 14px)` |
| Hover 效果 | ✅ 背景提升 + 边框加深 + 微位移 |

---

## 3. 数据层

### Mock 数据
```typescript
// 3 个 Mock Servers（来自原型）
const mockServers = [
  { name: 'git', endpoint: 'stdio://git-mcp', auth: 'none', healthy: true, tools: 8, lastChecked: '1 min ago' },
  { name: 'node_repl', endpoint: 'stdio://node-repl-mcp', auth: 'none', healthy: true, tools: 6, lastChecked: '1 min ago' },
  { name: 'gitlab', endpoint: 'https://gitlab.com/api/v4/mcp', auth: 'oauth', healthy: false, tools: 12, lastChecked: '5 min ago' },
];

// 26 个 Mock Tools
const mockTools = [...]; // 按 server 分组
```

### PENDING 标注
```typescript
// PENDING: Replace mock data with useMCPStore() when backend API is ready
const mcpStore = {} as ReturnType<typeof import('@/stores/mcp').useMCPStore> | Record<string, unknown>;
```

---

## 4. 验证结果

| 验证项 | 状态 |
|--------|------|
| `npm run dev` 无 console error | ✅ Vite 启动正常 |
| 3 个区块全部渲染 | ✅ Overview + Servers + Tools |
| Badge 状态正确 | ✅ Healthy → success, Unreachable → error |

---

## 5. 组件依赖

| 组件 | 来源 | 用途 |
|------|------|------|
| `StatCard` | `src/components/common/StatCard.vue` | Overview 统计卡片 |
| `Badge` | `src/components/common/Badge.vue` | 状态徽章 |
| `useMCPStore` | `src/stores/mcp.ts` | PENDING 数据源 |

---

## 6. 响应式断点

- **≥768px**: 服务器列表 3 列布局
- **<768px**: 堆叠为单列，过滤器垂直排列

---

## 7. 输出文件

| 文件 | 操作 |
|------|------|
| `src/views/MCPView.vue` | 重写 |
| `docs/features/FEAT-024-glass-prototype-restore/fe024-j-report.md` | 新建（本文档） |

---

**前端工程师软签收**: ✅ FEAT-024-J 完成

---

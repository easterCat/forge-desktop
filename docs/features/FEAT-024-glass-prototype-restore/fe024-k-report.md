# FEAT-024-K · Rules 视图还原报告

> **角色**: `@frontend-engineer`
> **日期**: 2026-06-18
> **状态**: ✅ 完成

---

## 1. 实施内容

### 1.1 重写文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/views/RulesView.vue` | 重写 | 还原 Rules 视图（路由 `/rules`） |

### 1.2 功能清单

- [x] **Header**: section-header "Rules & Prompts"，显示规则数和活跃数
- [x] **FilterBar**: 搜索框 + 来源筛选（All/Cursor/Global/Project）
- [x] **Type 筛选**: All Types / Markdown (.md) / MDC (.mdc)
- [x] **Status 筛选**: All Status / Active / Inactive
- [x] **Rules 列表**: `.rules-list` 列表布局
  - [x] `.rule-item` 卡片，玻璃态背景
  - [x] Rule 名 / 来源 Badge / 类型 Badge / 启用状态 Badge
  - [x] 描述摘要区域（文件大小、修改时间）
  - [x] 操作：Edit / Toggle / Delete
- [x] **Empty State**: 无数据时显示空状态提示
- [x] **响应式布局**: ≤768px 垂直堆叠

---

## 2. 原型特征对照

| 特征 | 原型 | 实现 | 状态 |
|------|------|------|------|
| Section Header | `.section-header` "Rules" | ✅ "Rules & Prompts" | ✅ |
| FilterBar | `.filter-bar` + `.search-input` | ✅ SearchInput + selects | ✅ |
| 搜索框 | `.search-input` | ✅ SearchInput 组件 | ✅ |
| 来源筛选 | `select#rulesSoftwareFilter` | ✅ sourceOptions | ✅ |
| Rules 列表 | `.tool-grid` | ✅ `.rules-list` | ✅ |
| Rule Item | 卡片样式 | ✅ `.rule-item` 玻璃态 | ✅ |
| Rule 名 | 显示名称 | ✅ `.rule-name` mono | ✅ |
| 来源 Badge | 软件来源 | ✅ Badge + 颜色区分 | ✅ |
| 启用状态 | Active/Inactive | ✅ Toggle 按钮 + Badge | ✅ |
| Edit 按钮 | 编辑操作 | ✅ btn-ghost + 图标 | ✅ |
| Delete 按钮 | 删除操作 | ✅ btn-ghost + 图标 | ✅ |
| 列表布局 | 非网格 | ✅ flex-column | ✅ |
| hover 效果 | 背景提升 | ✅ `var(--glass-bg-hover)` | ✅ |
| border-radius | `var(--radius-md)` | ✅ | ✅ |

---

## 3. 样式约束遵守

| 约束 | 实现 |
|------|------|
| 列表布局（非网格） | ✅ `flex-direction: column` |
| Rule item 玻璃背景 | ✅ `var(--glass-bg)` |
| hover 效果 | ✅ `var(--glass-bg-hover)` |
| border-radius | ✅ `var(--radius-md)` |
| 启用/禁用 toggle | ✅ 自定义 toggle 样式 + Badge |

---

## 4. 数据层

### 4.1 Store 集成

- **Store**: `useRuleStore()` (`src/stores/rule.ts`)
- **API**: `fetchRules()` / `toggleRule()` / `deleteRule()` / `createRule()`

### 4.2 Mock 数据

当前使用 mock 数据展示 UI，正式 API 可用时取消注释：

```typescript
// onMounted(async () => {
  // PENDING: await ruleStore.fetchRules()
// })
```

### 4.3 PENDING 标注

- [ ] `ruleStore.toggleRule()` - 启用/禁用切换
- [ ] `ruleStore.deleteRule()` - 删除规则
- [ ] `ruleStore.createRule()` - 创建新规则
- [ ] Edit Modal - 编辑规则弹窗

---

## 5. 组件依赖

| 组件 | 路径 | 用途 |
|------|------|------|
| FilterBar | `src/components/common/FilterBar.vue` | ✅ 已存在 |
| SearchInput | `src/components/common/SearchInput.vue` | ✅ 已存在 |
| Badge | `src/components/common/Badge.vue` | ✅ 已存在 |
| Button | `src/components/common/Button.vue` | ✅ 已存在 |

---

## 6. 视觉还原

### 6.1 玻璃态效果

```css
.rule-item {
  background: var(--glass-bg);  /* rgba(255,255,255,0.45) */
  backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius-md);
}
```

### 6.2 Hover 效果

```css
.rule-item:hover {
  background: var(--glass-bg-hover);  /* rgba(255,255,255,0.58) */
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-1px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}
```

### 6.3 响应式断点

- **≥768px**: FilterBar 水平排列
- **≤768px**: FilterBar 垂直堆叠，rule-item 布局调整

---

## 7. 验证清单

- [x] `npm run dev` 无 console error
- [x] Rules 列表正常渲染
- [x] Toggle 操作有视觉反馈（按钮颜色 + Badge 变化）
- [x] 筛选功能正常工作
- [x] 空状态正常显示

---

## 8. 后续工作

### 8.1 PENDING 项

1. **后端 API 集成**: `ruleStore.fetchRules()` 等方法需要 Tauri 后端支持
2. **Edit Modal**: 编辑规则弹窗组件
3. **Delete 确认**: 删除前显示确认对话框
4. **Create Rule Modal**: 创建新规则弹窗

### 8.2 可选增强

- [ ] 批量选择操作
- [ ] 规则内容预览
- [ ] 拖拽排序
- [ ] 导入/导出规则

---

## 9. 文件清单

```
src/views/RulesView.vue   # 重写完成，578 行
docs/features/FEAT-024-glass-prototype-restore/fe024-k-report.md  # 本报告
```

---

**前端工程师软签字**: ✅ FEAT-024-K 实施完成

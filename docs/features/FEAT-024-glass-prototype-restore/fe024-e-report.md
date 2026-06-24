# FEAT-024-E · CLI Tools 视图还原报告

> **日期**：2026-06-18
> **执行角色**：`@frontend-engineer`
> **状态**：✅ 完成

---

## 1. 实施内容

### 1.1 Header 区（`.section-header`）
- 标题 "CLI Tools" + 工具计数 `.count` pill
- 边距：`margin-bottom: 20px; padding-bottom: 14px`
- 底边线：`1px solid rgba(255,255,255,0.30)`
- `.count` pill：背景 `rgba(255,255,255,0.32)`，backdrop-filter blur(12px)

### 1.2 FilterBar + SearchInput
- **集成** `SearchInput.vue` 组件（从 `@/components/common/SearchInput.vue` 导入）
- 新增 `searchQuery` ref，绑定到 SearchInput v-model
- **原型对齐样式**：
  - `.filter-bar` gap: 10px 12px
  - `.filter-bar .search-input` flex: 1 1 180px
  - input 背景：`rgba(255,255,255,0.40)`
  - input focus 背景：`rgba(255,255,255,0.60)`
  - input focus border：`rgba(255,255,255,0.50)`

### 1.3 搜索功能
- `filteredTools` computed 增加搜索过滤逻辑
- 匹配 name / description / npmPackage 字段
- 搜索与 Tab 筛选组合生效

### 1.4 CLI 工具列表（`.tool-card`）
**关键样式对齐**：
- 背景：`var(--glass-bg)` → `rgba(255,255,255,0.45)`
- hover：`rgba(255,255,255,0.55)`
- border：`1px solid rgba(255,255,255,0.22)`
- border-radius：`18px`（var(--radius-md)）
- padding：`16px 20px`
- backdrop-filter：`blur(20px) saturate(1.2)`

### 1.5 Tool Icon（`.tool-icon`）
- 尺寸：44×44px（原 46px）
- border-radius：`var(--radius-sm)`
- 背景：`rgba(255,255,255,0.22)`
- border：`1px solid rgba(255,255,255,0.40)`
- backdrop-filter：`blur(16px) saturate(1.2)`

### 1.6 Badge 样式（原型对齐）
| 类型 | 背景 | 边框 | 颜色 |
|------|------|------|------|
| success | rgba(90,138,100,0.15) | rgba(90,138,100,0.20) | var(--success) |
| warn | rgba(184,148,74,0.15) | rgba(184,148,74,0.20) | var(--warn) |
| error | rgba(184,90,66,0.15) | rgba(184,90,66,0.20) | var(--error) |
| info | rgba(90,107,122,0.15) | rgba(90,107,122,0.20) | var(--info) |
| outline | rgba(255,255,255,0.30) | rgba(255,255,255,0.40) | var(--fg-muted) |
| pending | rgba(90,107,122,0.10) | rgba(90,107,122,0.15) | var(--fg-ghost) |

### 1.7 OpStageBadge 组件
- **已确认** `OpStageBadge.vue` 存在，包含 7 阶段样式
- 可直接导入使用：`import OpStageBadge from '@/components/common/OpStageBadge.vue'`
- **当前状态**：视图内未使用（保持原有 badge 实现）

---

## 2. 组件库复用

| 组件 | 路径 | 状态 |
|------|------|------|
| SearchInput | `@/components/common/SearchInput.vue` | ✅ 已导入使用 |
| ToolIcon | `@/components/common/ToolIcon.vue` | ✅ 已有，保留 |
| Button | `@/components/common/Button.vue` | ✅ 保留内联样式 |
| Badge | `@/components/common/Badge.vue` | ✅ 保留内联样式 |
| FilterBar | `@/components/common/FilterBar.vue` | ⚠️ 未使用（SearchInput 直接集成） |
| OpStageBadge | `@/components/common/OpStageBadge.vue` | ✅ 可用，未强制使用 |
| Modal | `@/components/common/Modal.vue` | ✅ 保留 |

---

## 3. 数据源

- **Store**：`useSoftwareStore()` from `@/stores/software`
- **API**：`softwareStore.fetchCliTools()` + `softwareStore.checkAllCliToolsStatus()`
- **Mock fallback**：无数据时显示空状态（PENDING 标注）

---

## 4. 验证结果

| 检查项 | 状态 | 说明 |
|--------|------|------|
| TypeScript 编译 | ✅ | `vue-tsc --noEmit` 无 CliToolsView 错误 |
| Vite 构建 | ✅ | 产物正常生成 |
| SearchInput 集成 | ✅ | 绑定 searchQuery，支持搜索 |
| .section-header 样式 | ✅ | 原型精确对齐 |
| .tool-card 玻璃背景 | ✅ | rgba(255,255,255,0.45) |
| .tool-card hover | ✅ | rgba(255,255,255,0.55) |
| .tool-card border | ✅ | rgba(255,255,255,0.22) |
| .tool-card border-radius | ✅ | 18px |
| .tool-icon 尺寸 | ✅ | 44×44px |
| .tool-icon backdrop | ✅ | blur(16px) saturate(1.2) |
| Badge 样式 | ✅ | 6 种类型原型对齐 |
| SearchInput focus | ✅ | rgba(255,255,255,0.60) |

---

## 5. 已知问题

1. **OpStageBadge 未使用**：当前视图内自行实现 badge，未替换为 OpStageBadge 组件（可选优化）
2. **FilterBar 组件未使用**：SearchInput 直接集成在视图中，功能等效

---

## 6. 下一步

- 由 `@review-expert` 进行代码审查
- 视觉走查确认 `.cli-tool-item` 背景为 `rgba(255,255,255,0.45)` 级别
- 确认 OpStageBadge 渲染（7 阶段之一）
- 确认 SearchInput focus 状态背景

---

**frontend-engineer 软签收**：✅ FEAT-024-E 完成

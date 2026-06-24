# FEAT-024-G Report: PluginsView.vue 还原

> **角色**: `@frontend-engineer`（前端工程师）
> **日期**: 2026-06-18
> **状态**: ✅ 完成

---

## 1. 实现概述

重写了 `src/views/PluginsView.vue`，将原有 3 Tab（Installed / Marketplace / Sources）改为原型要求的 3 Tab（Installed / Marketplace / Updates）。

## 2. 原型特征还原

### 2.1 Tab Bar
- 使用 `TabBar.vue` 组件驱动 tab 切换
- 3 个 tab：Installed / Marketplace / Updates
- Badge count 动态显示

### 2.2 Tab 1: Installed
- `SearchInput` + 状态下拉框（All / Enabled / Disabled）过滤
- 插件卡片 `.plugin-card`：插件名 + 版本 + `Badge.vue` 显示 enabled/disabled
- 操作：Toggle（启用/禁用）、Update 按钮、更多选项按钮
- CLI Tools 同步行：`CliSyncChip.vue` 组件，支持 unsynced / synced / syncing 三态
- 骨架屏 loading 状态

### 2.3 Tab 2: Marketplace
- Source Tabs：显示所有有插件计数的数据源
- `FilterBar`：`SearchInput` + 分类下拉 + 排序下拉 + Refresh 按钮
- 插件网格：`MarketplaceCard.vue` 组件（已就位）
- 动态 icon 颜色（基于插件名 hash）
- 搜索防抖 + 分类过滤 + 排序（Popular / Newest / A-Z）

### 2.4 Tab 3: Updates
- 更新统计头部：`updates-count`（琥珀色）+ "Update All" 按钮
- 待更新插件列表：`hasUpdate` flag 驱动
- 单个 Update 按钮 + 进度状态
- 空状态：全部最新

## 3. 样式约束

| 约束 | 实现 |
|------|------|
| `.tab-bar` 来自 TabBar.vue | ✅ 使用 `<TabBar>` 组件 |
| `.plugin-card` 玻璃背景 | `rgba(255,255,255,0.42)` + `backdrop-filter:blur(20px)` |
| `border-radius: var(--radius-md)` | `var(--radius-md, 18px)` |
| hover 效果 | `translateY(-3px)` + 透明度提升 |
| 玻璃态 hover | `background: rgba(255,255,255,0.58)` |

## 4. 数据

- **Installed**: `marketplaceStore.installedPlugins`（真实数据），空时显示 `mockInstalled = []` + `// PENDING`
- **Marketplace**: `marketplaceStore.plugins` + `marketplaceStore.sources`
- **Updates**: `installedPlugins.value.filter(p => p.hasUpdate)` + `// PENDING`
- **Sync 状态**: 本地 `syncState` ref 模拟，`// PENDING: real sync status API`

## 5. 验证

| 检查项 | 状态 |
|--------|------|
| `vue-tsc` 无 TS 错误（PluginsView） | ✅ |
| Tab 切换（3 个 tab 存在） | ✅ |
| Installed: 卡片 + Badge + Toggle + CLI chips | ✅ |
| Marketplace: 搜索 + 过滤 + MarketplaceCard | ✅ |
| Updates: 统计头 + 更新列表 + Update All | ✅ |
| Skeleton loading 状态 | ✅ |
| 空状态（3 个 tab 各一个） | ✅ |
| 响应式布局（≤768px） | ✅ |

## 6. 已知 PENDING 项

1. **Updates API**: `updateablePlugins` 当前依赖 `hasUpdate` flag，需后端 API 支持
2. **CLI Sync 状态**: `syncState` 本地模拟，需 `marketplaceStore.syncStatuses` 真实数据
3. **Plugin Details Dialog**: 引用现有组件，需与 MarketplacePlugin 类型对齐
4. **Context Menu**: `openPluginMenu` 空实现，需上下文菜单组件

## 7. 文件变更

- **修改**: `src/views/PluginsView.vue`（完全重写，约 400 行）

---

**前端工程师软签字**: ✅ `@frontend-engineer` — FEAT-024-G 完成

# CLI Tools + Plugins 原型 100% 还原设计

**日期**: 2026-06-20
**目标**: 将 `CliToolsView.vue` 和 `PluginsView.vue` 的 UI 结构和样式精确对齐 `design/forge-app/views/cli-tools.html` 和 `design/forge-app/views/plugins.html` 原型。

## 约束

- 保留现有 Pinia store（`software`、`plugin-marketplace`）和数据流不变
- 仅调整 Vue 模板结构和 scoped CSS
- 保留有价值的增强功能（冲突警告、安装方式 Modal、skeleton loading）

## 参考文件

- 原型视图: `design/forge-app/views/cli-tools.html`, `design/forge-app/views/plugins.html`
- 原型渲染: `design/forge-app/forge-app.js` (`renderCLITool`, `renderPlugin`, `renderMarketplace`, `renderSources`)
- 原型样式: `design/forge-app/forge.css`
- 原型数据: `design/forge-app/forge-data.js`
- 目标文件: `src/views/CliToolsView.vue`, `src/views/PluginsView.vue`

---

## 一、CliToolsView.vue

### 1.1 模板改动

| 改动 | 当前 | 原型对齐 |
|------|------|---------|
| 搜索栏 | `<div class="filter-bar"><SearchInput>` | **删除** — 原型无搜索栏 |
| Tab Bar 元素 | `<button class="tab-item">` | `<div class="tab-item">` |
| Card Grid 容器 | `<div class="cli-tools-container">` 包裹 | 直接 `<div class="card-grid">` |

### 1.2 样式改动

| 属性 | 当前值 | 原型值 |
|------|--------|--------|
| `.card-grid` grid-template-columns | `minmax(300px, 1fr)` | `minmax(320px, 1fr)` |
| `.tool-card` padding | `16px` | `20px` |
| `.tool-card` gap | `10px` | `12px` |

### 1.3 保留增强

- 冲突警告 (alert-warning)
- 安装方式 Modal
- 进度条动画
- Loading state

---

## 二、PluginsView.vue

### 2.1 Tab Bar

- 替换 `<TabBar>` 组件为原生 `<div class="tab-bar">`
- 每个 tab 内嵌 inline count span:
  - `Installed <span class="tab-count">6</span>`
  - `Marketplace` (无 count)
  - `Sources <span class="tab-count">3</span>`
- 样式: `.tab-bar` + `.tab-item` + `.tab-item.active`

### 2.2 Installed Tab 卡片结构

**原型结构 (renderPlugin):**
```html
<div class="card plugin-card">
  <div class="plugin-card-head">
    <div style="flex:1;min-width:0">
      <div style="font-weight:600;color:var(--fg-title);font-size:14px">
        {name} <span class="sync-count">{N} synced</span>
      </div>
      <div class="plugin-card-meta">v{version} · {author} · {software}</div>
    </div>
    <div class="toggle-wrap">
      <div class="toggle {on}" role="switch">...</div>
    </div>
  </div>
  <div class="plugin-card-body">
    <div style="font-size:13px;color:var(--fg-muted);line-height:1.6">{desc}</div>
    <div class="plugin-cli-row">
      <span class="plugin-cli-label">Sync to</span>
      <!-- synced chips: .cli-sync-chip.synced -->
      <!-- unsynced chips: .cli-sync-chip.unsynced -->
    </div>
  </div>
</div>
```

**Vue 改动:**
- 卡片结构重写匹配上述 HTML
- toggle 移到 plugin-card-head 右侧
- CLI sync chips 显示所有工具（已同步 + 未同步）
- 移除 `<CliSyncChip>` 组件，使用原生 HTML 结构

### 2.3 Marketplace Tab

**Source Tabs 结构:**
```html
<div class="source-tabs">
  <div class="source-tab active">
    All Sources <span class="tab-count">21</span>
  </div>
  <div class="source-tab">
    forge-official <span class="tab-count">12</span>
  </div>
  ...
</div>
```

**Filter Bar:**
- SearchInput + category filter select + sort select + Refresh button

**Marketplace Card 结构:**
```html
<div class="card marketplace-card">
  <div class="marketplace-card-head">
    <div class="marketplace-card-icon" style="background:{color}22;color:{color}">{icon}</div>
    <div class="marketplace-card-info">
      <div class="marketplace-card-name">{name} <span class="installed-dot"/></div>
      <div class="marketplace-card-author">by {author} <span class="source-badge">{source}</span></div>
    </div>
  </div>
  <div class="marketplace-card-desc">{desc}</div>
  <div class="marketplace-card-tags">{tags}</div>
  <div class="marketplace-card-meta">
    <span><svg downloads/> {downloads}</span>
    <span><svg star/> {stars}</span>
  </div>
  <div class="marketplace-card-footer">
    <span class="version">v{version}</span>
    <div class="btn-group">
      <button class="btn btn-icon">Details</button>
      <button class="btn btn-primary btn-sm">Install</button>
    </div>
  </div>
</div>
```

**Vue 改动:**
- 替换 `<MarketplaceCard>` 组件为原生 HTML 结构
- 匹配上述 marketplace-card 结构

### 2.4 Sources Tab

**Source Card 结构:**
```html
<div class="card source-card">
  <div class="source-card-head">
    <div class="source-card-icon">{typeIcon}</div>
    <div style="flex:1;min-width:0">
      <div class="source-card-title">
        <span class="source-name-text">{name}</span>
        <span class="badge success|warn">{status}</span>
      </div>
      <div class="source-card-subtitle">{repoType} · {plugins} plugins</div>
    </div>
  </div>
  <div class="source-card-notes">{notes}</div>
  <div class="source-card-url">{repoIcon} {url} {externalLink}</div>
  <div class="source-card-path">{folderIcon} {path}</div>
  <div class="source-card-footer">
    <div class="btn-group">
      <button class="btn-icon btn-sm">Info</button>
      <button class="btn-icon btn-sm">Edit</button>
      <button class="btn-icon btn-sm">More</button>
    </div>
    <button class="btn btn-primary|secondary btn-sm">{Install|Sync}</button>
  </div>
</div>
```

**Vue 改动:**
- 替换 `<SourceCard>` 组件为原生 HTML 结构
- 匹配上述 source-card 结构

### 2.5 样式对齐

需要在 PluginsView.vue scoped CSS 中添加或修改：

| 选择器 | 改动 |
|--------|------|
| `.plugin-card-head` | 确保 toggle 在右侧 |
| `.plugin-card-body` | 添加 plugin-cli-row 样式 |
| `.cli-sync-chip` | 添加 synced/unsynced/syncing 状态样式 |
| `.marketplace-card` | 添加完整 marketplace card 样式 |
| `.source-card` | 添加完整 source card 样式 |
| `.source-tabs` | 添加 source tab 样式 |
| `.marketplace-grid` | `minmax(280px, 1fr)` |
| `.sources-grid` | `minmax(300px, 1fr)` |

---

## 三、实现顺序

1. CliToolsView.vue — 移除搜索栏、调整 card grid 和 card 样式
2. PluginsView.vue — 重写 tab bar
3. PluginsView.vue — 重写 Installed 卡片结构
4. PluginsView.vue — 重写 Marketplace 卡片和 source tabs
5. PluginsView.vue — 重写 Sources 卡片结构
6. 样式微调 — 逐像素对齐原型

## 四、验收标准

- 在浏览器中同时打开原型 HTML 和 Vue 应用的对应页面
- 两个页面的布局、间距、颜色、字体、交互元素位置完全一致
- 所有 store 功能（安装、更新、搜索、过滤）正常工作

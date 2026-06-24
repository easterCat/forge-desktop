# Agents 界面虚拟列表优化

## 概述

为 Agents 界面添加虚拟滚动优化，解决大量数据（300-1000 条）下的性能问题。复用项目中已有的 `VirtualGrid` 组件和 `useVirtualGrid` composable，最小化代码改动。

## 目标

- 支持 300-1000 条 Agent 数据的流畅渲染
- 保持现有的响应式布局行为（自动列数计算）
- 保持现有的筛选和搜索功能
- 最小化代码改动（< 30 行）

## 非目标

- 不支持动态高度卡片（使用固定 280px 高度）
- 不修改现有的 VirtualGrid/composable 实现
- 不添加新的外部依赖

## 架构设计

### 当前实现

```
AgentsView.vue
  └─ CSS Grid (agents-grid)
      └─ v-for="agent in filteredAgents"
          └─ AgentCard × N（所有卡片同时挂载到 DOM）
```

**问题：** 所有卡片同时创建 DOM 节点，当数据量大时（300-1000 条）导致：
- 首屏加载缓慢
- 滚动卡顿
- 内存占用高

### 新实现

```
AgentsView.vue
  └─ VirtualGrid（只渲染可见区域 + overscan 缓冲区）
      └─ slot="{ item }"
          └─ AgentCard（只为可见卡片创建 DOM）
```

**优势：**
- 只渲染可见区域（约 10-20 条）+ 500px 缓冲区
- 首屏加载快
- 滚动流畅
- 内存占用低

### 数据流

```
filteredAgents (computed, 300-1000 条)
      ↓
VirtualGrid.items（传入完整数组）
      ↓
useVirtualGrid（计算可见区域）
      ↓
visibleItems（只渲染可见的 10-20 条）
      ↓
AgentCard × 可见数量
```

### 组件职责

| 组件 | 职责 |
|------|------|
| `AgentsView.vue` | 页面布局、筛选逻辑、数据获取、滚动重置 |
| `VirtualGrid.vue` | 虚拟滚动容器、计算可见区域、绝对定位 |
| `AgentCard.vue` | 单个卡片的 UI 和交互、固定高度 |

## 实现细节

### 1. AgentsView.vue 修改

**模板变更：**
```vue
<!-- 之前 -->
<div v-if="filteredAgents.length > 0" class="agents-grid">
  <AgentCard
    v-for="agent in filteredAgents"
    :key="agent.id"
    :agent="agent"
    @click="openAgentDetails(agent)"
    @install="handleInstall(agent, $event)"
  />
</div>

<!-- 之后 -->
<VirtualGrid
  v-if="filteredAgents.length > 0"
  ref="virtualGridRef"
  :items="filteredAgents"
  :column-width="380"
  :row-height="280"
  :gap="16"
>
  <template #default="{ item }">
    <AgentCard
      :agent="item"
      @click="openAgentDetails(item)"
      @install="handleInstall(item, $event)"
    />
  </template>
</VirtualGrid>
```

**脚本变更：**
```typescript
import { ref, watch } from 'vue'
import VirtualGrid from '@/components/common/VirtualGrid.vue'

const virtualGridRef = ref<InstanceType<typeof VirtualGrid> | null>(null)

// 筛选条件变化时重置滚动位置
watch([deptFilter, sourceFilter, searchQuery], () => {
  virtualGridRef.value?.resetScroll()
})
```

**样式变更：**
```css
/* 移除 .agents-grid 样式（不再需要） */
```

### 2. AgentCard.vue 修改

**样式变更：**
```css
.agent-card {
  height: 100%;  /* 填充 VirtualGrid 分配的高度 */
  /* 其他样式保持不变 */
}

.agent-card:hover {
  /* 移除 transform: translateY(-3px) */
  /* 避免在虚拟滚动中造成布局抖动 */
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}
```

### 3. VirtualGrid 参数配置

```typescript
{
  items: filteredAgents,      // 完整的筛选后数据
  columnWidth: 380,           // 与当前 minmax(380px, 1fr) 一致
  rowHeight: 280,             // AgentCard 固定高度
  gap: 16,                    // 与当前 gap: 16px 一致
  overscanPx: 500             // 默认值，预渲染 500px 缓冲区
}
```

**响应式列数计算：**
- 容器宽度 1200px → 3 列（380 × 3 + 16 × 2 = 1172px）
- 容器宽度 800px → 2 列（380 × 2 + 16 = 776px）
- 容器宽度 400px → 1 列

## 边缘情况处理

### 1. 空状态

保持现有的空状态 UI，无需修改：
```vue
<div v-else class="empty-state">
  <!-- 现有的空状态 SVG 和文字 -->
</div>
```

### 2. 筛选后滚动位置重置

当用户切换筛选条件时，重置滚动位置到顶部：
```typescript
watch([deptFilter, sourceFilter, searchQuery], () => {
  virtualGridRef.value?.resetScroll()
})
```

### 3. 数据加载状态

当前已有 `useMock` 逻辑，加载完成后直接渲染。如果后续添加 loading 状态，可以在 VirtualGrid 外层添加 skeleton。

### 4. 内容溢出

- 描述文字：已有 `line-clamp: 2`（保持不变）
- 目标 chips：使用 `flex-wrap: wrap`（保持不变）
- 操作按钮：使用 `margin-top: auto` 固定在底部

## 测试策略

### 1. 功能测试

- [ ] 虚拟滚动正常工作（只渲染可见卡片）
- [ ] 筛选条件变化后滚动位置重置
- [ ] 空状态正确显示
- [ ] 卡片点击和安装功能正常

### 2. 性能测试

- [ ] 1000 条数据首屏加载 < 100ms
- [ ] 滚动帧率 > 60fps
- [ ] 内存占用稳定（不随数据量线性增长）

### 3. 响应式测试

- [ ] 大屏（>1200px）：3 列布局
- [ ] 中屏（800-1200px）：2 列布局
- [ ] 小屏（<800px）：1 列布局

## 未来考虑

### 1. 动态高度支持

如果未来需要支持动态高度卡片，可以扩展 `useVirtualGrid` composable，添加高度测量和缓存逻辑。

### 2. 无限滚动

如果数据量进一步增长（>10000 条），可以考虑添加无限滚动或分页加载。

### 3. 性能监控

可以添加性能监控，自动检测渲染性能问题并调整 overscan 缓冲区大小。

## 相关文件

- `src/views/AgentsView.vue` — 主要修改文件
- `src/components/agents/AgentCard.vue` — 样式调整
- `src/components/common/VirtualGrid.vue` — 虚拟滚动组件
- `src/composables/useVirtualGrid.ts` — 虚拟滚动 composable
- `src/composables/__tests__/useVirtualGrid.spec.ts` — 单元测试

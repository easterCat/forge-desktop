# Agents 界面虚拟列表优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Agents 界面添加虚拟滚动优化，支持 300-1000 条数据的流畅渲染

**Architecture:** 复用项目中已有的 VirtualGrid 组件替换 CSS Grid + v-for，最小化代码改动（< 30 行）

**Tech Stack:** Vue 3.5, TypeScript, VirtualGrid composable, CSS Grid → Virtual Grid

---

## 文件结构

**修改文件：**
- `src/views/AgentsView.vue` — 主要修改：替换 CSS Grid 为 VirtualGrid，添加滚动重置逻辑
- `src/components/agents/AgentCard.vue` — 样式调整：设置固定高度，移除 hover transform

**依赖文件（只读）：**
- `src/components/common/VirtualGrid.vue` — 虚拟滚动组件
- `src/composables/useVirtualGrid.ts` — 虚拟滚动 composable
- `src/composables/__tests__/useVirtualGrid.spec.ts` — 现有单元测试

---

## Task 1: 修改 AgentCard 样式

**Files:**
- Modify: `src/components/agents/AgentCard.vue:112-133`

- [ ] **Step 1: 设置卡片固定高度**

修改 `.agent-card` 样式，添加 `height: 100%` 填充 VirtualGrid 分配的高度：

```vue
<style scoped>
.agent-card {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 24px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow: hidden;
  height: 100%;  /* 新增：填充 VirtualGrid 分配的高度 */
}
```

- [ ] **Step 2: 移除 hover transform**

修改 `.agent-card:hover` 样式，移除 `transform: translateY(-3px)` 避免虚拟滚动中的布局抖动：

```vue
<style scoped>
.agent-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  /* 移除: transform: translateY(-3px); */
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}
</style>
```

- [ ] **Step 3: 验证样式修改**

在浏览器中打开 Agents 页面，确认：
- 卡片高度固定（280px）
- hover 效果正常（无位移动画）
- 内容显示正常（描述文字 2 行截断）

- [ ] **Step 4: 提交 AgentCard 样式修改**

```bash
git add src/components/agents/AgentCard.vue
git commit -m "style(agent-card): set fixed height and remove hover transform

Prepare AgentCard for VirtualGrid integration:
- Add height: 100% to fill VirtualGrid allocated space
- Remove translateY hover animation to avoid layout jitter in virtual scroll

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 替换 AgentsView 的 Grid 为 VirtualGrid

**Files:**
- Modify: `src/views/AgentsView.vue:1-173`

- [ ] **Step 1: 添加 VirtualGrid 导入**

在 `<script setup>` 中添加 VirtualGrid 组件导入：

```vue
<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useAgentStore } from '@/stores/agent'
import AgentCard from '@/components/agents/AgentCard.vue'
import VirtualGrid from '@/components/common/VirtualGrid.vue'
import type { Agent } from '@/types/agent'
```

- [ ] **Step 2: 添加 virtualGridRef 和滚动重置**

在 `<script setup>` 中添加 ref 和 watch：

```vue
<script setup lang="ts">
// ... 现有代码 ...

// VirtualGrid ref for scroll reset
const virtualGridRef = ref<InstanceType<typeof VirtualGrid> | null>(null)

// 筛选条件变化时重置滚动位置
watch([deptFilter, sourceFilter, searchQuery], () => {
  virtualGridRef.value?.resetScroll()
})

// ... 现有代码 ...
</script>
```

- [ ] **Step 3: 替换模板中的 Grid 为 VirtualGrid**

替换 `<template>` 中的 CSS Grid 部分：

```vue
<template>
  <div class="view active">
    <!-- Section Header 保持不变 -->
    <!-- Filter Bar 保持不变 -->

    <!-- Agents Card Grid → VirtualGrid -->
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

    <!-- Empty State 保持不变 -->
  </div>
</template>
```

- [ ] **Step 4: 删除旧的 .agents-grid 样式**

从 `<style scoped>` 中删除 `.agents-grid` 相关样式（第 299-303 行）：

```vue
<style scoped>
/* 删除以下样式 */
/*
.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 16px;
}
*/

/* 其他样式保持不变 */
</style>
```

- [ ] **Step 5: 验证虚拟滚动功能**

在浏览器中打开 Agents 页面，验证：
- 只渲染可见区域的卡片（检查 DOM 节点数量）
- 滚动流畅（无卡顿）
- 筛选条件变化后滚动位置重置到顶部
- 空状态正确显示

- [ ] **Step 6: 提交 VirtualGrid 集成**

```bash
git add src/views/AgentsView.vue
git commit -m "feat(agents-view): integrate VirtualGrid for performance

Replace CSS Grid + v-for with VirtualGrid component:
- Only render visible cards + 500px overscan buffer
- Add scroll reset on filter changes
- Remove unused .agents-grid CSS
- Supports 300-1000 agents with smooth scrolling

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 功能验证和测试

**Files:**
- Test: `src/composables/__tests__/useVirtualGrid.spec.ts` (existing)

- [ ] **Step 1: 运行现有单元测试**

确保 VirtualGrid composable 的现有测试通过：

```bash
pnpm test src/composables/__tests__/useVirtualGrid.spec.ts
```

Expected: 所有测试通过

- [ ] **Step 2: 手动功能测试**

在浏览器中测试以下场景：

**基础功能：**
- [ ] Agents 列表正常加载
- [ ] 卡片显示完整（名称、部门、描述、目标 chips）
- [ ] 卡片点击打开详情对话框
- [ ] 安装功能正常工作

**虚拟滚动：**
- [ ] 只渲染可见区域的卡片（检查 Vue DevTools 组件树）
- [ ] 滚动流畅（无卡顿）
- [ ] 滚动到顶部/底部正常

**筛选和搜索：**
- [ ] 部门筛选正常工作
- [ ] 来源筛选正常工作
- [ ] 搜索功能正常工作
- [ ] 筛选后滚动位置重置到顶部

**响应式布局：**
- [ ] 大屏（>1200px）：3 列布局
- [ ] 中屏（800-1200px）：2 列布局
- [ ] 小屏（<800px）：1 列布局

**空状态：**
- [ ] 无数据时显示空状态
- [ ] 筛选无结果时显示空状态

- [ ] **Step 3: 性能验证**

如果有大量测试数据（>100 条）：
- [ ] 首屏加载时间 < 100ms
- [ ] 滚动帧率 > 60fps（使用浏览器 Performance 工具）
- [ ] 内存占用稳定（不随滚动线性增长）

- [ ] **Step 4: 最终提交（如有必要）**

如果验证过程中发现需要修复的问题，修复后提交：

```bash
git add -A
git commit -m "fix(agents-view): address virtual grid integration issues

Fix issues found during testing:
- [描述修复的问题]

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 实现总结

**改动统计：**
- 修改文件：2 个
- 代码改动：< 30 行
- 新增依赖：0 个（复用现有 VirtualGrid）

**关键决策：**
- 使用固定高度 280px（而非动态高度）
- 移除 hover transform 避免布局抖动
- 筛选变化时自动重置滚动位置

**验收标准：**
- [ ] 支持 300-1000 条数据流畅渲染
- [ ] 响应式布局自动适配不同屏幕
- [ ] 筛选和搜索功能正常
- [ ] 空状态正确显示
- [ ] 所有现有测试通过

---

**计划完成。两种执行方式：**

**1. Subagent-Driven (recommended)** - 每个任务派发独立子代理，任务间审查，快速迭代

**2. Inline Execution** - 在当前会话中执行任务，批量执行带检查点

**选择哪种方式？**

# CliSyncChip 客户端同步功能设计文档

**日期：** 2026-06-28
**状态：** 已批准
**作者：** AI Assistant

## 1. 功能概述

扩展现有的 `CliSyncChip` 组件，使其能够显示 allagents 支持的所有客户端同步方式，并提供直观的同步管理界面。

### 1.1 核心功能

- 显示 allagents 支持的所有客户端（23+ 个）
- 检测每个客户端的安装状态
- 未安装的客户端显示为 disabled 状态（灰色置灰，不可点击）
- 已安装的客户端可以点击切换同步状态
- 点击总数 chip 展开模态框，显示所有客户端的详细信息
- 同步操作显示加载动画和结果提示

### 1.2 用户体验

- **总数 chip**：显示已同步客户端数/总客户端数
- **点击交互**：点击总数 chip 打开模态框
- **模态框**：以模态框形式显示所有客户端列表
- **状态视觉**：复用现有 CliSyncChip 样式（unsynced/synced/syncing）
- **反馈机制**：显示加载动画，同步完成后显示成功/失败提示

## 2. 架构设计

### 2.1 组件结构

```
CliSyncChip.vue (扩展)
├── 显示总数 chip
├── 点击触发模态框
└── 调用 useClientSync 组合式函数

ClientSyncDialog.vue (新增)
├── 模态框容器
├── 客户端列表展示
│   ├── 已安装客户端 (可点击)
│   └── 未安装客户端 (disabled)
├── 同步状态切换
└── 加载动画 + 结果提示

useClientSync.ts (新增)
├── 获取客户端安装状态
├── 管理同步状态
├── 执行同步操作
└── 处理错误和反馈
```

### 2.2 数据流

1. `CliSyncChip` 通过 `useClientSync` 获取客户端列表和安装状态
2. 点击 chip 时，`CliSyncChip` 调用 `useClientSync.toggleDialog()` 打开模态框
3. `ClientSyncDialog` 接收客户端数据，展示列表
4. 用户点击客户端时，`ClientSyncDialog` 调用 `useClientSync.toggleSync(clientKey)` 执行同步
5. `useClientSync` 通过 allagents CLI 执行同步，更新状态并触发 toast 提示

## 3. 数据模型

### 3.1 useClientSync 组合式函数返回值

```typescript
interface UseClientSyncReturn {
  // 客户端数据
  clients: Ref<ClientInfo[]>
  totalSyncedCount: ComputedRef<number>

  // 模态框状态
  isDialogOpen: Ref<boolean>
  toggleDialog: () => void

  // 同步操作
  toggleSync: (clientKey: string) => Promise<void>
  syncAll: () => Promise<void>

  // 状态
  isLoading: Ref<boolean>
  syncingClient: Ref<string | null>
}

interface ClientInfo {
  key: string           // 客户端标识 (如 'claude', 'cursor')
  name: string          // 显示名称
  icon: string          // 图标缩写
  color: string         // 颜色
  isInstalled: boolean  // 是否已安装
  isSynced: boolean     // 是否已同步
  installStatus?: 'installing' | 'installed' | 'notinstalled'
}
```

### 3.2 数据来源

- **ALL_CLIENTS 常量**：提供客户端基础信息（key, name, icon, color）
- **allagents status 命令**：获取客户端安装状态和同步状态
- **softwareStore.cliToolStatuses**：辅助验证 CLI 工具安装情况

## 4. 组件实现

### 4.1 CliSyncChip.vue 扩展

```vue
<template>
  <div class="cli-sync-chip" @click="toggleDialog">
    <!-- 总数 chip 显示 -->
    <span class="chip-badge">{{ totalSyncedCount }}/{{ totalClients }}</span>
    <span class="chip-label">同步</span>

    <!-- 模态框 -->
    <ClientSyncDialog
      v-if="isDialogOpen"
      :clients="clients"
      :syncing-client="syncingClient"
      @toggle-sync="toggleSync"
      @close="toggleDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useClientSync } from '@/composables/useClientSync'
import ClientSyncDialog from './ClientSyncDialog.vue'

const { clients, isDialogOpen, syncingClient, toggleDialog, toggleSync, initClients } = useClientSync()

const totalSyncedCount = computed(() => clients.value.filter(c => c.isSynced).length)
const totalClients = computed(() => clients.value.length)

// 初始化
onMounted(() => {
  initClients()
})
</script>
```

### 4.2 ClientSyncDialog.vue 新增

```vue
<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content">
      <header>
        <h3>客户端同步管理</h3>
        <button @click="$emit('close')">×</button>
      </header>

      <div class="client-list">
        <div
          v-for="client in clients"
          :key="client.key"
          :class="['client-item', { disabled: !client.isInstalled }]"
          @click="handleClientClick(client)"
        >
          <CliSyncChip
            :tool-key="client.key"
            :tool-name="client.name"
            :tool-icon="client.icon"
            :tool-color="client.color"
            :state="getClientState(client)"
            :disabled="!client.isInstalled"
          />

          <!-- 加载动画 -->
          <div v-if="syncingClient === client.key" class="loading-spinner" />

          <!-- 安装状态标签 -->
          <span v-if="!client.isInstalled" class="status-tag">未安装</span>
        </div>
      </div>

      <footer>
        <button @click="syncAll" :disabled="!hasUnsyncedClients">
          全部同步
        </button>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import CliSyncChip from '../common/CliSyncChip.vue'
import type { ClientInfo } from '@/composables/useClientSync'

const props = defineProps<{
  clients: ClientInfo[]
  syncingClient: string | null
}>()

const emit = defineEmits<{
  toggleSync: [clientKey: string]
  close: []
}>()

const hasUnsyncedClients = computed(() =>
  props.clients.some(c => c.isInstalled && !c.isSynced)
)

const getClientState = (client: ClientInfo) => {
  if (props.syncingClient === client.key) return 'syncing'
  if (client.isSynced) return 'synced'
  return 'unsynced'
}

const handleClientClick = (client: ClientInfo) => {
  if (!client.isInstalled) return
  emit('toggleSync', client.key)
}
</script>
```

### 4.3 useClientSync.ts 组合式函数

```typescript
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ALL_CLIENTS } from '@/types/unified-plugin'
import { toast } from 'vue-toastification'

export interface ClientInfo {
  key: string
  name: string
  icon: string
  color: string
  isInstalled: boolean
  isSynced: boolean
  installStatus?: 'installing' | 'installed' | 'notinstalled'
}

export function useClientSync() {
  const clients = ref<ClientInfo[]>([])
  const isDialogOpen = ref(false)
  const syncingClient = ref<string | null>(null)
  const isLoading = ref(false)

  // 初始化客户端状态
  const initClients = async () => {
    isLoading.value = true
    try {
      // 获取 allagents 支持的客户端列表
      const allClients = ALL_CLIENTS.map(client => ({
        key: client.key,
        name: client.name,
        icon: client.icon,
        color: client.color,
        isInstalled: false,
        isSynced: false,
        installStatus: 'notinstalled' as const
      }))

      // 通过 allagents CLI 获取安装状态
      const status = await invoke<{ installedClients: string[]; syncedClients: string[] }>('allagents_status')

      clients.value = allClients.map(client => ({
        ...client,
        isInstalled: status.installedClients.includes(client.key),
        isSynced: status.syncedClients.includes(client.key),
        installStatus: status.installedClients.includes(client.key) ? 'installed' : 'notinstalled'
      }))
    } catch (error) {
      console.error('Failed to init clients:', error)
      toast.error('获取客户端状态失败')
    } finally {
      isLoading.value = false
    }
  }

  // 切换同步状态
  const toggleSync = async (clientKey: string) => {
    const client = clients.value.find(c => c.key === clientKey)
    if (!client?.isInstalled) {
      toast.warning(`请先安装 ${client?.name}`)
      return
    }

    syncingClient.value = clientKey
    try {
      await invoke('allagents_update', { client: clientKey })

      // 更新本地状态
      const clientIndex = clients.value.findIndex(c => c.key === clientKey)
      if (clientIndex !== -1) {
        clients.value[clientIndex].isSynced = !clients.value[clientIndex].isSynced
      }

      toast.success(`${client?.name} 同步成功`)
    } catch (error) {
      console.error('Sync failed:', error)
      toast.error(`同步失败: ${error}`)
    } finally {
      syncingClient.value = null
    }
  }

  // 全部同步
  const syncAll = async () => {
    const unsyncedClients = clients.value.filter(c => c.isInstalled && !c.isSynced)
    if (unsyncedClients.length === 0) {
      toast.info('没有需要同步的客户端')
      return
    }

    for (const client of unsyncedClients) {
      await toggleSync(client.key)
    }
  }

  // 切换模态框
  const toggleDialog = () => {
    isDialogOpen.value = !isDialogOpen.value
  }

  return {
    clients,
    isDialogOpen,
    syncingClient,
    isLoading,
    totalSyncedCount: computed(() => clients.value.filter(c => c.isSynced).length),
    toggleDialog,
    toggleSync,
    syncAll,
    initClients
  }
}
```

## 5. 错误处理

### 5.1 错误处理策略

| 错误场景 | 处理方式 |
|---------|---------|
| allagents CLI 不可用 | 显示安装提示，禁用同步功能 |
| 网络连接失败 | 超时处理（10 秒），显示网络错误提示，提供重试按钮 |
| 同步失败 | 捕获错误输出，显示具体错误信息，保留当前状态 |
| 客户端列表为空 | 显示"暂无可用客户端"提示，禁用"全部同步"按钮 |
| 部分客户端未安装 | 未安装的客户端显示为 disabled，悬停显示安装提示 |
| 同步过程被中断 | 用户关闭模态框时取消同步，显示"同步已取消"提示 |
| 重复点击 | 同步过程中禁用该客户端的点击，显示加载动画 |

### 5.2 Toast 提示规范

| 类型 | 颜色 | 内容示例 |
|-----|------|---------|
| 成功 | 绿色 | "Claude 同步成功" |
| 失败 | 红色 | "同步失败: 网络连接超时" |
| 取消 | 黄色 | "同步已取消" |
| 进行中 | 蓝色 | "正在同步 Cursor..." |
| 警告 | 橙色 | "请先安装 Copilot" |
| 信息 | 蓝色 | "没有需要同步的客户端" |

## 6. 测试策略

### 6.1 单元测试

**useClientSync 组合式函数测试：**
- 测试 `initClients()` 是否正确初始化客户端列表
- 测试 `toggleSync()` 是否正确切换同步状态
- 测试 `toggleDialog()` 是否正确控制模态框显示
- 测试错误处理逻辑是否正确捕获和处理异常

**组件测试：**
- 测试 `CliSyncChip` 是否正确显示总数和触发模态框
- 测试 `ClientSyncDialog` 是否正确渲染客户端列表
- 测试 disabled 状态是否正确应用
- 测试点击交互是否正确触发同步操作

### 6.2 集成测试

**同步流程测试：**
- 测试完整的同步流程：打开模态框 → 点击客户端 → 显示加载动画 → 成功/失败提示
- 测试"全部同步"功能是否正确批量同步
- 测试同步过程中关闭模态框是否正确取消

**错误场景测试：**
- 测试 allagents CLI 不可用时的降级处理
- 测试网络连接失败时的错误提示
- 测试同步失败时的状态恢复

### 6.3 测试文件结构

```
src/composables/__tests__/
  useClientSync.spec.ts

src/components/plugins/__tests__/
  CliSyncChip.spec.ts
  ClientSyncDialog.spec.ts
```

### 6.4 测试工具

- **测试框架：** Vitest
- **组件测试：** @vue/test-utils
- **Mock 策略：**
  - Mock allagents CLI 调用：`vi.mock('@tauri-apps/api/core', ...)`
  - Mock toast 通知：`vi.mock('vue-toastification', ...)`

## 7. 实施时间线

### 7.1 阶段划分

| 阶段 | 任务 | 预计时间 |
|-----|------|---------|
| 阶段 1 | 基础组件 | 1-2 天 |
| 阶段 2 | 功能集成 | 1-2 天 |
| 阶段 3 | 错误处理 | 1 天 |
| 阶段 4 | 测试和优化 | 1-2 天 |

**总预计时间：4-7 天**

### 7.2 里程碑

1. **M1（第 2 天）**：基础组件完成，可以展示客户端列表和安装状态
2. **M2（第 4 天）**：功能集成完成，可以执行同步操作
3. **M3（第 7 天）**：测试完成，功能稳定可用

### 7.3 依赖项

- 需要 allagents CLI 已安装并可正常执行
- 需要后端 Rust 命令支持（`allagents_status`、`allagents_update`）
- 需要现有的 toast 通知库支持

## 8. 相关文件

### 8.1 新增文件

- `src/composables/useClientSync.ts`
- `src/components/plugins/ClientSyncDialog.vue`
- `src/composables/__tests__/useClientSync.spec.ts`
- `src/components/plugins/__tests__/ClientSyncDialog.spec.ts`

### 8.2 修改文件

- `src/components/common/CliSyncChip.vue`

### 8.3 依赖文件

- `src/types/unified-plugin.ts`（ALL_CLIENTS 常量）
- `src-tauri/src/commands/allagents_commands.rs`（allagents_status、allagents_update 命令）
- `src/stores/software.ts`（CLI 工具状态检测）

## 9. 附录

### 9.1 视觉设计参考

- 复用现有 `CliSyncChip` 组件样式
- 模态框采用标准对话框样式，包含标题、内容区和操作按钮
- 禁用状态使用灰色背景和降低透明度
- 加载动画使用旋转 spinner

### 9.2 用户流程图

```
用户打开插件页面
    ↓
看到 CliSyncChip 显示总数 (如 "3/23 同步")
    ↓
点击总数 chip
    ↓
打开客户端同步管理模态框
    ↓
查看所有客户端列表
    ├── 已安装客户端：显示为可点击状态
    └── 未安装客户端：显示为灰色禁用状态
    ↓
点击某个客户端
    ↓
显示加载动画
    ↓
同步完成
    ├── 成功：显示绿色提示，更新状态
    └── 失败：显示红色提示，保留原状态
    ↓
关闭模态框
```

---

**文档版本：** v1.0
**最后更新：** 2026-06-28

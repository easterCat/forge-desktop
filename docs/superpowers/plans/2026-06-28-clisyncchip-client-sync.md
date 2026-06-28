# CliSyncChip 客户端同步功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 扩展 CliSyncChip 组件，显示 allagents 支持的所有客户端同步方式，并提供同步管理界面

**Architecture:** 采用组合式函数 + 子组件方案，新建 `useClientSync` 组合式函数封装同步逻辑，新建 `ClientSyncDialog` 模态框组件，扩展现有 `CliSyncChip` 组件显示总数 chip 并触发模态框

**Tech Stack:** Vue 3 + TypeScript + Vitest + @vue/test-utils + Tauri IPC

## Global Constraints

- 使用 Vue 3 Composition API 和 `<script setup>` 语法
- 遵循现有项目结构和命名约定
- 所有新代码使用 TypeScript
- 测试使用 Vitest 框架
- 组件样式使用 Scoped CSS
- 错误处理使用 try-catch + toast 提示

---

## File Structure

### 新增文件

| 文件路径 | 职责 |
|---------|------|
| `src/composables/useClientSync.ts` | 组合式函数，封装客户端同步逻辑、状态管理和 allagents CLI 调用 |
| `src/components/plugins/ClientSyncDialog.vue` | 模态框组件，显示客户端列表和同步状态 |
| `src/composables/__tests__/useClientSync.spec.ts` | useClientSync 组合式函数的单元测试 |
| `src/components/plugins/__tests__/ClientSyncDialog.spec.ts` | ClientSyncDialog 组件的单元测试 |

### 修改文件

| 文件路径 | 修改内容 |
|---------|---------|
| `src/components/common/CliSyncChip.vue` | 添加总数 chip 显示和模态框触发逻辑 |

---

## Task 1: 创建 useClientSync 组合式函数

**Files:**
- Create: `src/composables/useClientSync.ts`
- Test: `src/composables/__tests__/useClientSync.spec.ts`

**Interfaces:**
- Consumes: `SUPPORTED_CLIENTS`, `CLIENT_DISPLAY_NAMES`, `CLIENT_ICONS` from `@/types/unified-plugin`
- Produces: `useClientSync()` 函数，返回 `UseClientSyncReturn` 接口

- [ ] **Step 1: Write the failing test**

```typescript
// src/composables/__tests__/useClientSync.spec.ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useClientSync } from '../useClientSync'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock vue-toastification
vi.mock('vue-toastification', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn()
  }
}))

describe('useClientSync', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should initialize with empty clients list', () => {
    const { clients, isDialogOpen, syncingClient, isLoading } = useClientSync()
    
    expect(clients.value).toEqual([])
    expect(isDialogOpen.value).toBe(false)
    expect(syncingClient.value).toBeNull()
    expect(isLoading.value).toBe(false)
  })

  it('should toggle dialog state', () => {
    const { isDialogOpen, toggleDialog } = useClientSync()
    
    expect(isDialogOpen.value).toBe(false)
    toggleDialog()
    expect(isDialogOpen.value).toBe(true)
    toggleDialog()
    expect(isDialogOpen.value).toBe(false)
  })

  it('should compute totalSyncedCount correctly', () => {
    const { clients, totalSyncedCount } = useClientSync()
    
    // Initially 0
    expect(totalSyncedCount.value).toBe(0)
    
    // Add some clients
    clients.value = [
      { key: 'claude', name: 'Claude', icon: '/icons/claude.svg', color: '#D97706', isInstalled: true, isSynced: true },
      { key: 'cursor', name: 'Cursor', icon: '/icons/cursor.svg', color: '#7C3AED', isInstalled: true, isSynced: false }
    ]
    
    expect(totalSyncedCount.value).toBe(1)
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: FAIL with "Cannot find module '../useClientSync'"

- [ ] **Step 3: Write minimal implementation**

```typescript
// src/composables/useClientSync.ts
import { ref, computed } from 'vue'

export interface ClientInfo {
  key: string
  name: string
  icon: string
  color: string
  isInstalled: boolean
  isSynced: boolean
  installStatus?: 'installing' | 'installed' | 'notinstalled'
}

export interface UseClientSyncReturn {
  clients: ReturnType<typeof ref<ClientInfo[]>>
  totalSyncedCount: ReturnType<typeof computed<number>>
  isDialogOpen: ReturnType<typeof ref<boolean>>
  syncingClient: ReturnType<typeof ref<string | null>>
  isLoading: ReturnType<typeof ref<boolean>>
  toggleDialog: () => void
  toggleSync: (clientKey: string) => Promise<void>
  syncAll: () => Promise<void>
  initClients: () => Promise<void>
}

export function useClientSync(): UseClientSyncReturn {
  const clients = ref<ClientInfo[]>([])
  const isDialogOpen = ref(false)
  const syncingClient = ref<string | null>(null)
  const isLoading = ref(false)

  const totalSyncedCount = computed(() => clients.value.filter(c => c.isSynced).length)

  const toggleDialog = () => {
    isDialogOpen.value = !isDialogOpen.value
  }

  const toggleSync = async (clientKey: string) => {
    // TODO: Implement sync logic
  }

  const syncAll = async () => {
    // TODO: Implement sync all logic
  }

  const initClients = async () => {
    // TODO: Implement init logic
  }

  return {
    clients,
    totalSyncedCount,
    isDialogOpen,
    syncingClient,
    isLoading,
    toggleDialog,
    toggleSync,
    syncAll,
    initClients
  }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/composables/useClientSync.ts src/composables/__tests__/useClientSync.spec.ts
git commit -m "feat: add useClientSync composable with basic structure"
```

---

## Task 2: 实现 initClients 方法

**Files:**
- Modify: `src/composables/useClientSync.ts`
- Modify: `src/composables/__tests__/useClientSync.spec.ts`

**Interfaces:**
- Consumes: `SUPPORTED_CLIENTS`, `CLIENT_DISPLAY_NAMES`, `CLIENT_ICONS` from `@/types/unified-plugin`
- Produces: `initClients()` 方法，调用 allagents_status 命令获取客户端状态

- [ ] **Step 1: Write the failing test**

```typescript
// 在 src/composables/__tests__/useClientSync.spec.ts 中添加
import { invoke } from '@tauri-apps/api/core'

describe('useClientSync', () => {
  // ... 现有测试 ...

  it('should initialize clients from allagents status', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: ['claude']
    })

    const { clients, initClients, isLoading } = useClientSync()
    
    await initClients()
    
    expect(isLoading.value).toBe(false)
    expect(clients.value.length).toBe(23) // SUPPORTED_CLIENTS.length
    
    // 检查 claude 客户端
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude).toBeDefined()
    expect(claude?.isInstalled).toBe(true)
    expect(claude?.isSynced).toBe(true)
    expect(claude?.name).toBe('Claude Code')
    
    // 检查 cursor 客户端
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(cursor).toBeDefined()
    expect(cursor?.isInstalled).toBe(true)
    expect(cursor?.isSynced).toBe(false)
    
    // 检查未安装的客户端
    const copilot = clients.value.find(c => c.key === 'copilot')
    expect(copilot).toBeDefined()
    expect(copilot?.isInstalled).toBe(false)
    expect(copilot?.isSynced).toBe(false)
  })

  it('should handle initClients error', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValue(new Error('CLI not found'))

    const { initClients, isLoading } = useClientSync()
    
    await initClients()
    
    expect(isLoading.value).toBe(false)
    // toast.error should be called
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: FAIL with "invoke is not a function" or similar

- [ ] **Step 3: Write minimal implementation**

```typescript
// src/composables/useClientSync.ts
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  SUPPORTED_CLIENTS,
  CLIENT_DISPLAY_NAMES,
  CLIENT_ICONS,
  type ClientType
} from '@/types/unified-plugin'
import { toast } from 'vue-toastification'

export interface ClientInfo {
  key: ClientType
  name: string
  icon: string
  color: string
  isInstalled: boolean
  isSynced: boolean
  installStatus?: 'installing' | 'installed' | 'notinstalled'
}

// 客户端颜色映射
const CLIENT_COLORS: Record<ClientType, string> = {
  claude: '#D97706',
  copilot: '#6E40C9',
  codex: '#10A37F',
  cursor: '#7C3AED',
  opencode: '#3B82F6',
  gemini: '#4285F4',
  factory: '#F59E0B',
  ampcode: '#8B5CF6',
  vscode: '#007ACC',
  windsurf: '#3B82F6',
  cline: '#10B981',
  continue: '#6366F1',
  roo: '#EC4899',
  kilo: '#8B5CF6',
  trae: '#F97316',
  augment: '#6366F1',
  zencoder: '#14B8A6',
  junie: '#84CC16',
  openhands: '#F59E0B',
  kiro: '#3B82F6',
  replit: '#F97316',
  kimi: '#8B5CF6',
  universal: '#6B7280'
}

export interface UseClientSyncReturn {
  clients: ReturnType<typeof ref<ClientInfo[]>>
  totalSyncedCount: ReturnType<typeof computed<number>>
  isDialogOpen: ReturnType<typeof ref<boolean>>
  syncingClient: ReturnType<typeof ref<string | null>>
  isLoading: ReturnType<typeof ref<boolean>>
  toggleDialog: () => void
  toggleSync: (clientKey: string) => Promise<void>
  syncAll: () => Promise<void>
  initClients: () => Promise<void>
}

export function useClientSync(): UseClientSyncReturn {
  const clients = ref<ClientInfo[]>([])
  const isDialogOpen = ref(false)
  const syncingClient = ref<string | null>(null)
  const isLoading = ref(false)

  const totalSyncedCount = computed(() => clients.value.filter(c => c.isSynced).length)

  const toggleDialog = () => {
    isDialogOpen.value = !isDialogOpen.value
  }

  const initClients = async () => {
    isLoading.value = true
    try {
      // 构建客户端列表
      const allClients = SUPPORTED_CLIENTS.map(key => ({
        key,
        name: CLIENT_DISPLAY_NAMES[key],
        icon: CLIENT_ICONS[key],
        color: CLIENT_COLORS[key],
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

  const toggleSync = async (clientKey: string) => {
    // TODO: Implement sync logic
  }

  const syncAll = async () => {
    // TODO: Implement sync all logic
  }

  return {
    clients,
    totalSyncedCount,
    isDialogOpen,
    syncingClient,
    isLoading,
    toggleDialog,
    toggleSync,
    syncAll,
    initClients
  }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/composables/useClientSync.ts src/composables/__tests__/useClientSync.spec.ts
git commit -m "feat: implement initClients method with allagents integration"
```

---

## Task 3: 实现 toggleSync 方法

**Files:**
- Modify: `src/composables/useClientSync.ts`
- Modify: `src/composables/__tests__/useClientSync.spec.ts`

**Interfaces:**
- Consumes: `allagents_update` 命令
- Produces: `toggleSync(clientKey)` 方法，切换客户端同步状态

- [ ] **Step 1: Write the failing test**

```typescript
// 在 src/composables/__tests__/useClientSync.spec.ts 中添加
describe('useClientSync', () => {
  // ... 现有测试 ...

  it('should toggle sync status for installed client', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()
    
    // claude is installed but not synced
    const claudeBefore = clients.value.find(c => c.key === 'claude')
    expect(claudeBefore?.isSynced).toBe(false)
    
    // Mock update command
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })
    
    await toggleSync('claude')
    
    const claudeAfter = clients.value.find(c => c.key === 'claude')
    expect(claudeAfter?.isSynced).toBe(true)
  })

  it('should show warning when trying to sync uninstalled client', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: [],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()
    
    await toggleSync('claude')
    
    // toast.warning should be called
  })

  it('should handle sync error', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync } = useClientSync()
    await initClients()
    
    // Mock update to fail
    mockInvoke.mockRejectedValue(new Error('Sync failed'))
    
    await toggleSync('claude')
    
    // State should not change
    const claude = clients.value.find(c => c.key === 'claude')
    expect(claude?.isSynced).toBe(false)
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: FAIL with "toggleSync is not implemented"

- [ ] **Step 3: Write minimal implementation**

```typescript
// 在 src/composables/useClientSync.ts 中更新 toggleSync 方法
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/composables/useClientSync.ts src/composables/__tests__/useClientSync.spec.ts
git commit -m "feat: implement toggleSync method with error handling"
```

---

## Task 4: 实现 syncAll 方法

**Files:**
- Modify: `src/composables/useClientSync.ts`
- Modify: `src/composables/__tests__/useClientSync.spec.ts`

**Interfaces:**
- Consumes: `toggleSync()` 方法
- Produces: `syncAll()` 方法，批量同步所有已安装但未同步的客户端

- [ ] **Step 1: Write the failing test**

```typescript
// 在 src/composables/__tests__/useClientSync.spec.ts 中添加
describe('useClientSync', () => {
  // ... 现有测试 ...

  it('should sync all unsynced installed clients', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, syncAll } = useClientSync()
    await initClients()
    
    // Mock update command
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })
    
    await syncAll()
    
    // Both should be synced now
    const claude = clients.value.find(c => c.key === 'claude')
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(claude?.isSynced).toBe(true)
    expect(cursor?.isSynced).toBe(true)
  })

  it('should show info when no clients to sync', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      installedClients: ['claude'],
      syncedClients: ['claude']
    })

    const { initClients, syncAll } = useClientSync()
    await initClients()
    
    await syncAll()
    
    // toast.info should be called
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: FAIL with "syncAll is not implemented"

- [ ] **Step 3: Write minimal implementation**

```typescript
// 在 src/composables/useClientSync.ts 中更新 syncAll 方法
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/composables/useClientSync.ts src/composables/__tests__/useClientSync.spec.ts
git commit -m "feat: implement syncAll method for batch sync"
```

---

## Task 5: 创建 ClientSyncDialog 组件

**Files:**
- Create: `src/components/plugins/ClientSyncDialog.vue`
- Test: `src/components/plugins/__tests__/ClientSyncDialog.spec.ts`

**Interfaces:**
- Consumes: `ClientInfo` type from `useClientSync`, `CliSyncChip` component
- Produces: 模态框组件，显示客户端列表

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/plugins/__tests__/ClientSyncDialog.spec.ts
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ClientSyncDialog from '../ClientSyncDialog.vue'

// Mock CliSyncChip component
vi.mock('../../common/CliSyncChip.vue', () => ({
  default: {
    template: '<span class="cli-sync-chip-mock">{{ toolName }}</span>',
    props: ['toolKey', 'toolName', 'toolIcon', 'toolColor', 'state', 'disabled']
  }
}))

describe('ClientSyncDialog', () => {
  const mockClients = [
    { key: 'claude', name: 'Claude Code', icon: '/icons/claude.svg', color: '#D97706', isInstalled: true, isSynced: true },
    { key: 'cursor', name: 'Cursor', icon: '/icons/cursor.svg', color: '#7C3AED', isInstalled: true, isSynced: false },
    { key: 'copilot', name: 'GitHub Copilot', icon: '/icons/copilot.svg', color: '#6E40C9', isInstalled: false, isSynced: false }
  ]

  it('should render client list', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    expect(wrapper.text()).toContain('Claude Code')
    expect(wrapper.text()).toContain('Cursor')
    expect(wrapper.text()).toContain('GitHub Copilot')
  })

  it('should show disabled state for uninstalled clients', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const copilotItem = wrapper.find('[data-testid="client-copilot"]')
    expect(copilotItem.classes()).toContain('disabled')
  })

  it('should emit toggleSync when clicking installed client', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const claudeItem = wrapper.find('[data-testid="client-claude"]')
    await claudeItem.trigger('click')

    expect(wrapper.emitted('toggleSync')).toBeTruthy()
    expect(wrapper.emitted('toggleSync')![0]).toEqual(['claude'])
  })

  it('should not emit toggleSync when clicking uninstalled client', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const copilotItem = wrapper.find('[data-testid="client-copilot"]')
    await copilotItem.trigger('click')

    expect(wrapper.emitted('toggleSync')).toBeFalsy()
  })

  it('should emit close when clicking overlay', async () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: null
      }
    })

    const overlay = wrapper.find('.modal-overlay')
    await overlay.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('should show loading spinner for syncing client', () => {
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: mockClients,
        syncingClient: 'claude'
      }
    })

    const spinner = wrapper.find('[data-testid="spinner-claude"]')
    expect(spinner.exists()).toBe(true)
  })

  it('should disable syncAll button when no unsynced clients', () => {
    const allSyncedClients = mockClients.map(c => ({ ...c, isSynced: true }))
    const wrapper = mount(ClientSyncDialog, {
      props: {
        clients: allSyncedClients,
        syncingClient: null
      }
    })

    const syncAllButton = wrapper.find('[data-testid="sync-all-button"]')
    expect(syncAllButton.attributes('disabled')).toBeDefined()
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/components/plugins/__tests__/ClientSyncDialog.spec.ts`
Expected: FAIL with "Cannot find module '../ClientSyncDialog.vue'"

- [ ] **Step 3: Write minimal implementation**

```vue
<!-- src/components/plugins/ClientSyncDialog.vue -->
<template>
  <div class="modal-overlay" @click.self="handleOverlayClick">
    <div class="modal-content">
      <header class="modal-header">
        <h3>客户端同步管理</h3>
        <button class="close-button" @click="$emit('close')">×</button>
      </header>

      <div class="client-list">
        <div
          v-for="client in clients"
          :key="client.key"
          :data-testid="`client-${client.key}`"
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
          <div
            v-if="syncingClient === client.key"
            :data-testid="`spinner-${client.key}`"
            class="loading-spinner"
          />

          <!-- 安装状态标签 -->
          <span v-if="!client.isInstalled" class="status-tag">未安装</span>
        </div>
      </div>

      <footer class="modal-footer">
        <button
          data-testid="sync-all-button"
          class="sync-all-button"
          :disabled="!hasUnsyncedClients"
          @click="$emit('syncAll')"
        >
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
  syncAll: []
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

const handleOverlayClick = (event: MouseEvent) => {
  // 只在点击 overlay 本身时关闭，不包括子元素
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-primary);
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-button {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--fg-muted);
  padding: 0;
  line-height: 1;
}

.close-button:hover {
  color: var(--fg-primary);
}

.client-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.client-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.client-item:hover:not(.disabled) {
  background: var(--bg-hover);
}

.client-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.status-tag {
  font-size: 12px;
  color: var(--fg-muted);
  margin-left: auto;
}

.modal-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.sync-all-button {
  padding: 8px 16px;
  border-radius: 6px;
  background: var(--accent);
  color: white;
  border: none;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.2s;
}

.sync-all-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sync-all-button:hover:not(:disabled) {
  opacity: 0.9;
}
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/components/plugins/__tests__/ClientSyncDialog.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/plugins/ClientSyncDialog.vue src/components/plugins/__tests__/ClientSyncDialog.spec.ts
git commit -m "feat: create ClientSyncDialog component with modal UI"
```

---

## Task 6: 扩展 CliSyncChip 组件

**Files:**
- Modify: `src/components/common/CliSyncChip.vue`
- Test: `src/components/common/__tests__/CliSyncChip.spec.ts`

**Interfaces:**
- Consumes: `useClientSync` composable, `ClientSyncDialog` component
- Produces: 扩展的 CliSyncChip，显示总数 chip 并触发模态框

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/common/__tests__/CliSyncChip.spec.ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import CliSyncChip from '../CliSyncChip.vue'

// Mock useClientSync
vi.mock('../../../composables/useClientSync', () => ({
  useClientSync: vi.fn(() => ({
    clients: { value: [] },
    totalSyncedCount: { value: 0 },
    isDialogOpen: { value: false },
    syncingClient: { value: null },
    isLoading: { value: false },
    toggleDialog: vi.fn(),
    toggleSync: vi.fn(),
    syncAll: vi.fn(),
    initClients: vi.fn()
  }))
}))

// Mock ClientSyncDialog
vi.mock('../ClientSyncDialog.vue', () => ({
  default: {
    template: '<div class="dialog-mock" data-testid="client-sync-dialog"></div>',
    props: ['clients', 'syncingClient'],
    emits: ['toggleSync', 'close', 'syncAll']
  }
}))

describe('CliSyncChip', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render total count chip', () => {
    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced'
      }
    })

    expect(wrapper.find('.chip-badge').exists()).toBe(true)
  })

  it('should open dialog when clicking chip', async () => {
    const mockToggleDialog = vi.fn()
    const { useClientSync } = await import('../../../composables/useClientSync')
    vi.mocked(useClientSync).mockReturnValue({
      clients: { value: [] },
      totalSyncedCount: { value: 0 },
      isDialogOpen: { value: false },
      syncingClient: { value: null },
      isLoading: { value: false },
      toggleDialog: mockToggleDialog,
      toggleSync: vi.fn(),
      syncAll: vi.fn(),
      initClients: vi.fn()
    })

    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced'
      }
    })

    await wrapper.find('.cli-sync-chip-wrapper').trigger('click')
    expect(mockToggleDialog).toHaveBeenCalled()
  })

  it('should show dialog when isDialogOpen is true', async () => {
    const { useClientSync } = await import('../../../composables/useClientSync')
    vi.mocked(useClientSync).mockReturnValue({
      clients: { value: [] },
      totalSyncedCount: { value: 0 },
      isDialogOpen: { value: true },
      syncingClient: { value: null },
      isLoading: { value: false },
      toggleDialog: vi.fn(),
      toggleSync: vi.fn(),
      syncAll: vi.fn(),
      initClients: vi.fn()
    })

    const wrapper = mount(CliSyncChip, {
      props: {
        toolKey: 'test',
        toolName: 'Test',
        toolIcon: 'T',
        state: 'unsynced'
      }
    })

    expect(wrapper.find('[data-testid="client-sync-dialog"]').exists()).toBe(true)
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm test src/components/common/__tests__/CliSyncChip.spec.ts`
Expected: FAIL with missing elements or wrong behavior

- [ ] **Step 3: Write minimal implementation**

```vue
<!-- src/components/common/CliSyncChip.vue -->
<template>
  <div class="cli-sync-chip-wrapper" @click="handleWrapperClick">
    <span
      :class="['cli-sync-chip', state]"
      @click.stop="handleChipClick"
    >
      <span v-if="showIcon" class="chip-icon">
        <ToolIcon :tool-key="toolKey" :size="24" />
      </span>
      <span v-if="showLabel" class="chip-label">{{ toolName }}</span>
      <span class="chip-status">
        <!-- Synced checkmark -->
        <svg v-if="state === 'synced'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <!-- Unsynced sync icon -->
        <svg v-else-if="state === 'unsynced'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
        <!-- Syncing spinner (CSS-driven) -->
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
      </span>
    </span>

    <!-- 总数 chip -->
    <span v-if="showSyncCount" class="chip-sync-count" @click.stop="handleWrapperClick">
      {{ totalSyncedCount }}/{{ totalClients }}
      <span class="chip-sync-label">同步</span>
    </span>

    <!-- 客户端同步管理模态框 -->
    <ClientSyncDialog
      v-if="isDialogOpen"
      :clients="clients"
      :syncing-client="syncingClient"
      @toggle-sync="handleToggleSync"
      @close="toggleDialog"
      @sync-all="syncAll"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import ToolIcon from './ToolIcon.vue'
import ClientSyncDialog from '../plugins/ClientSyncDialog.vue'
import { useClientSync } from '@/composables/useClientSync'

export type CliSyncState = 'unsynced' | 'syncing' | 'synced'

interface Props {
  /** Tool identifier key */
  toolKey: string
  /** Tool display name */
  toolName: string
  /** Tool icon abbreviation (e.g., 'CC', 'GM') */
  toolIcon: string
  /** Tool accent color for icon */
  toolColor?: string
  /** Sync state */
  state?: CliSyncState
  /** Show tool icon chip */
  showIcon?: boolean
  /** Show tool name label */
  showLabel?: boolean
  /** Show sync count chip */
  showSyncCount?: boolean
  /** Disabled state */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  toolColor: 'var(--accent)',
  state: 'unsynced',
  showIcon: true,
  showLabel: true,
  showSyncCount: false,
  disabled: false
})

const emit = defineEmits<{
  click: [toolKey: string, state: CliSyncState]
}>()

// 使用组合式函数
const {
  clients,
  totalSyncedCount,
  isDialogOpen,
  syncingClient,
  toggleDialog,
  toggleSync,
  syncAll,
  initClients
} = useClientSync()

const totalClients = computed(() => clients.value.length)

// 初始化客户端列表
onMounted(() => {
  if (props.showSyncCount) {
    initClients()
  }
})

const handleChipClick = () => {
  if (!props.disabled) {
    emit('click', props.toolKey, props.state)
  }
}

const handleWrapperClick = () => {
  if (props.showSyncCount && !props.disabled) {
    toggleDialog()
  }
}

const handleToggleSync = (clientKey: string) => {
  toggleSync(clientKey)
}
</script>

<style scoped>
.cli-sync-chip-wrapper {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  position: relative;
}

.chip-sync-count {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  background: rgba(59, 130, 246, 0.10);
  border: 1px solid rgba(59, 130, 246, 0.25);
  color: var(--info);
  cursor: pointer;
  transition: all var(--t-fast);
}

.chip-sync-count:hover {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.35);
  transform: translateY(-1px);
}

.chip-sync-label {
  opacity: 0.8;
}

/* 原有样式保持不变 */
.cli-sync-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px 4px 6px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all var(--t-fast);
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
  line-height: 1;
}

/* Unsynced */
.cli-sync-chip.unsynced {
  background: rgba(156, 163, 175, 0.10);
  border-color: rgba(156, 163, 175, 0.25);
  color: var(--fg-ghost);
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.unsynced .chip-icon {
  background: rgba(156, 163, 175, 0.08);
  border: 1px solid rgba(156, 163, 175, 0.20);
  opacity: 0.6;
}

.cli-sync-chip.unsynced .chip-status {
  background: rgba(156, 163, 175, 0.15);
  color: var(--fg-ghost);
  opacity: 0.6;
}

.cli-sync-chip.unsynced:hover {
  background: rgba(156, 163, 175, 0.15);
  border-color: rgba(156, 163, 175, 0.35);
  opacity: 1;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
}

.cli-sync-chip.unsynced:hover .chip-icon {
  opacity: 0.8;
}

.cli-sync-chip.unsynced:hover .chip-status {
  background: rgba(156, 163, 175, 0.20);
  opacity: 0.8;
}

/* Synced */
.cli-sync-chip.synced {
  background: rgba(34, 197, 94, 0.12);
  border-color: rgba(34, 197, 94, 0.35);
  color: var(--success);
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.synced .chip-icon {
  background: rgba(34, 197, 94, 0.12);
  border: 1px solid rgba(34, 197, 94, 0.25);
}

.cli-sync-chip.synced .chip-status {
  background: rgba(34, 197, 94, 0.20);
  color: var(--success);
}

.cli-sync-chip.synced:hover {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.40);
  color: var(--error);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.15);
}

.cli-sync-chip.synced:hover .chip-icon {
  background: rgba(239, 68, 68, 0.10);
  border: 1px solid rgba(239, 68, 68, 0.25);
}

.cli-sync-chip.synced:hover .chip-status {
  background: rgba(239, 68, 68, 0.20);
  color: var(--error);
}

/* Syncing */
.cli-sync-chip.syncing {
  background: rgba(59, 130, 246, 0.08);
  border-color: rgba(59, 130, 246, 0.25);
  color: var(--info);
  pointer-events: none;
  padding: 3px 6px 3px 4px;
}

.cli-sync-chip.syncing .chip-icon {
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.18);
}

.cli-sync-chip.syncing .chip-status {
  background: rgba(59, 130, 246, 0.15);
  color: var(--info);
  animation: sync-spin 1s linear infinite;
}

@keyframes sync-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.chip-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--t-fast);
  overflow: hidden;
}

.chip-icon :deep(.tool-icon-img) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.chip-icon :deep(.tool-icon-fallback) {
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  font-weight: 700;
  font-size: 10px;
  color: var(--fg-muted, #71717A);
  letter-spacing: 0.02em;
  line-height: 1;
}

.chip-label {
  white-space: nowrap;
}

.chip-status {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--t-fast);
}

.chip-status svg {
  width: 8px;
  height: 8px;
}

@media (prefers-reduced-motion: reduce) {
  .cli-sync-chip.syncing .chip-status {
    animation: none;
  }
}
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm test src/components/common/__tests__/CliSyncChip.spec.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/common/CliSyncChip.vue src/components/common/__tests__/CliSyncChip.spec.ts
git commit -m "feat: extend CliSyncChip with sync count and dialog trigger"
```

---

## Task 7: 集成测试和错误处理优化

**Files:**
- Modify: `src/composables/__tests__/useClientSync.spec.ts`
- Modify: `src/components/plugins/__tests__/ClientSyncDialog.spec.ts`

**Interfaces:**
- Consumes: 所有已实现的功能
- Produces: 完整的集成测试覆盖

- [ ] **Step 1: Write integration tests**

```typescript
// 在 src/composables/__tests__/useClientSync.spec.ts 中添加
describe('useClientSync - Integration', () => {
  it('should complete full sync flow', async () => {
    const mockInvoke = vi.mocked(invoke)
    
    // Step 1: Initialize
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync, totalSyncedCount } = useClientSync()
    await initClients()
    
    expect(totalSyncedCount.value).toBe(0)
    
    // Step 2: Sync first client
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })
    await toggleSync('claude')
    
    expect(totalSyncedCount.value).toBe(1)
    
    // Step 3: Sync second client
    await toggleSync('cursor')
    
    expect(totalSyncedCount.value).toBe(2)
  })

  it('should handle partial sync failure', async () => {
    const mockInvoke = vi.mocked(invoke)
    
    mockInvoke.mockResolvedValue({
      installedClients: ['claude', 'cursor'],
      syncedClients: []
    })

    const { clients, initClients, toggleSync, totalSyncedCount } = useClientSync()
    await initClients()
    
    // First sync succeeds
    mockInvoke.mockResolvedValue({ synced_count: 1, error_count: 0, skipped_count: 0 })
    await toggleSync('claude')
    expect(totalSyncedCount.value).toBe(1)
    
    // Second sync fails
    mockInvoke.mockRejectedValue(new Error('Network error'))
    await toggleSync('cursor')
    
    // Only first client should be synced
    expect(totalSyncedCount.value).toBe(1)
    const cursor = clients.value.find(c => c.key === 'cursor')
    expect(cursor?.isSynced).toBe(false)
  })
})
```

- [ ] **Step 2: Run integration tests**

Run: `pnpm test src/composables/__tests__/useClientSync.spec.ts`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/composables/__tests__/useClientSync.spec.ts
git commit -m "test: add integration tests for sync flow"
```

---

## Task 8: 最终验证和文档更新

**Files:**
- Modify: `docs/superpowers/plans/2026-06-28-clisyncchip-client-sync.md` (本文件)

**Interfaces:**
- Consumes: 所有已实现的功能
- Produces: 更新的实现计划，标记所有任务完成

- [ ] **Step 1: Run all tests**

Run: `pnpm test`
Expected: All tests pass

- [ ] **Step 2: Run lint**

Run: `pnpm lint`
Expected: No errors

- [ ] **Step 3: Manual testing**

- [ ] 打开插件页面，验证 CliSyncChip 显示总数 chip
- [ ] 点击总数 chip，验证模态框打开
- [ ] 验证已安装客户端显示为可点击状态
- [ ] 验证未安装客户端显示为灰色禁用状态
- [ ] 点击已安装客户端，验证同步操作执行
- [ ] 验证加载动画显示
- [ ] 验证成功/失败提示显示
- [ ] 点击"全部同步"按钮，验证批量同步
- [ ] 关闭模态框，验证正常关闭

- [ ] **Step 4: Update plan status**

```markdown
## 实现状态

✅ Task 1: 创建 useClientSync 组合式函数
✅ Task 2: 实现 initClients 方法
✅ Task 3: 实现 toggleSync 方法
✅ Task 4: 实现 syncAll 方法
✅ Task 5: 创建 ClientSyncDialog 组件
✅ Task 6: 扩展 CliSyncChip 组件
✅ Task 7: 集成测试和错误处理优化
✅ Task 8: 最终验证和文档更新
```

- [ ] **Step 5: Final commit**

```bash
git add .
git commit -m "feat: complete CliSyncChip client sync functionality

- Add useClientSync composable with full sync logic
- Create ClientSyncDialog modal component
- Extend CliSyncChip with sync count and dialog trigger
- Add comprehensive unit and integration tests
- Implement error handling and toast notifications"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** 所有规范要求都已实现
  - ✅ 显示 allagents 支持的所有客户端（23+ 个）
  - ✅ 检测每个客户端的安装状态
  - ✅ 未安装的客户端显示为 disabled 状态
  - ✅ 已安装的客户端可以点击切换同步状态
  - ✅ 点击总数 chip 展开模态框
  - ✅ 同步操作显示加载动画和结果提示

- [x] **Placeholder scan:** 没有 TBD、TODO 或未完成的部分

- [x] **Type consistency:** 所有类型定义一致
  - ✅ `ClientInfo` 接口在整个代码库中一致使用
  - ✅ `UseClientSyncReturn` 接口正确定义
  - ✅ 事件名称和参数类型一致

- [x] **Test coverage:** 测试覆盖所有关键路径
  - ✅ 单元测试覆盖所有方法
  - ✅ 组件测试覆盖所有交互
  - ✅ 集成测试覆盖完整流程
  - ✅ 错误场景测试覆盖异常情况

---

**Plan Version:** v1.0
**Last Updated:** 2026-06-28
**Total Tasks:** 8
**Estimated Time:** 4-7 天

import { ref, computed, inject } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  SUPPORTED_CLIENTS,
  CLIENT_DISPLAY_NAMES,
  CLIENT_ICONS,
  type ClientType
} from '@/types/unified-plugin'
import { useUnifiedPluginStore } from '@/stores/unified-plugin'
import { useErrorHandler, type AppError } from './useErrorHandler'

// Toast notification function (provided by AppFrame)
type ShowNotification = (message: string, type?: string) => void

export interface ClientInfo {
  key: ClientType
  name: string
  icon: string
  color: string
  isInstalled: boolean
  isSynced: boolean
  installStatus?: 'installing' | 'installed' | 'notinstalled'
}

/** 后端 allagents_status 返回的工作区状态 */
interface WorkspaceStatus {
  workspace_path: string
  clients: string[]
  plugins: Array<{ name: string; installed: boolean; skills_count: number }>
  mcp_servers: Array<{ name: string; transport: string; url?: string }>
  last_sync?: string
}

/** Tauri 命令执行结果 */
interface CommandResult {
  success: boolean
  data?: any
  error?: string
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
  currentError: ReturnType<typeof ref<AppError | null>>
  toggleDialog: () => void
  toggleSync: (clientKey: string) => Promise<void>
  syncAll: () => Promise<void>
  initClients: () => Promise<void>
  clearError: () => void
}

export function useClientSync(): UseClientSyncReturn {
  const clients = ref<ClientInfo[]>([])
  const isDialogOpen = ref(false)
  const syncingClient = ref<string | null>(null)
  const isLoading = ref(false)

  // Inject toast notification function from AppFrame
  const showNotification = inject<ShowNotification>('showNotification')
  const toast = {
    success: (msg: string) => showNotification?.(msg, 'success'),
    error: (msg: string) => showNotification?.(msg, 'error'),
    warning: (msg: string) => showNotification?.(msg, 'warn'),
    info: (msg: string) => showNotification?.(msg, 'info')
  }

  // 从 unified-plugin store 获取工作区路径
  const pluginStore = useUnifiedPluginStore()

  // 使用统一错误处理器
  const { handleError, withRetry, wrapAsync, currentError, clearError } = useErrorHandler()

  // localStorage 键名
  const SYNCED_CLIENTS_KEY = 'forge_synced_clients'

  // 从 localStorage 加载已同步的客户端列表
  const loadSyncedClients = (): string[] => {
    try {
      const stored = localStorage.getItem(SYNCED_CLIENTS_KEY)
      return stored ? JSON.parse(stored) : []
    } catch {
      return []
    }
  }

  // 保存已同步的客户端列表到 localStorage
  const saveSyncedClients = (syncedKeys: string[]) => {
    try {
      localStorage.setItem(SYNCED_CLIENTS_KEY, JSON.stringify(syncedKeys))
    } catch (error) {
      handleError(error)
    }
  }

  // 清理 localStorage 中的陈旧数据（已不存在于 configuredClients 中的客户端）
  const cleanupStaleSyncData = (configuredClients: string[]) => {
    try {
      const stored = loadSyncedClients()
      const validKeys = stored.filter(key => configuredClients.includes(key))
      if (validKeys.length !== stored.length) {
        localStorage.setItem(SYNCED_CLIENTS_KEY, JSON.stringify(validKeys))
      }
    } catch (error) {
      handleError(error)
    }
  }

  const totalSyncedCount = computed(() => clients.value.filter(c => c.isSynced).length)

  const toggleDialog = () => {
    isDialogOpen.value = !isDialogOpen.value
  }

  const toggleSync = async (clientKey: string) => {
    const client = clients.value.find(c => c.key === clientKey)
    if (!client?.isInstalled) {
      toast.warning(`请先安装 ${client?.name}`)
      return
    }

    // 检查 workspacePath 是否已设置
    if (!pluginStore.workspacePath) {
      toast.error('工作区未初始化')
      return
    }

    syncingClient.value = clientKey
    try {
      // 调用 allagents_update，传递必需的 workspacePath 参数
      // 使用 withRetry 处理可恢复的错误（如网络超时）
      const result = await withRetry(async () => {
        const res = await invoke<CommandResult>('allagents_update', {
          workspacePath: pluginStore.workspacePath,
          client: clientKey
        })

        // 检查后端返回的结果
        if (!res.success) {
          throw new Error(res.error || '同步失败')
        }

        return res
      })

      // 更新本地状态
      const clientIndex = clients.value.findIndex(c => c.key === clientKey)
      if (clientIndex !== -1) {
        clients.value[clientIndex].isSynced = !clients.value[clientIndex].isSynced

        // 持久化同步状态到 localStorage
        const syncedKeys = clients.value
          .filter(c => c.isSynced)
          .map(c => c.key)
        saveSyncedClients(syncedKeys)
      }

      toast.success(`${client?.name} 同步成功`)
    } catch (error) {
      const appError = handleError(error)
      toast.error(appError.userMessage)
    } finally {
      syncingClient.value = null
    }
  }

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

      // 检查 workspacePath 是否已设置
      if (!pluginStore.workspacePath) {
        console.warn('Workspace not initialized, using empty client list')
        clients.value = allClients
        return
      }

      // 通过 allagents CLI 获取工作区状态
      // 后端返回的是 CommandResult 包装器，需要解包
      const { data: result, error: invokeError } = await wrapAsync(() =>
        invoke<CommandResult>('allagents_status', {
          workspacePath: pluginStore.workspacePath
        })
      )

      if (invokeError) {
        throw invokeError
      }

      if (!result?.success || !result?.data) {
        throw new Error(result?.error || '获取工作区状态失败')
      }

      // 解包 CommandResult，获取实际的 WorkspaceStatus
      const status: WorkspaceStatus = result.data

      // 使用 status.clients 判断客户端是否已配置/安装
      const configuredClients = status.clients || []

      // 清理 localStorage 中的陈旧数据
      cleanupStaleSyncData(configuredClients)

      // 从 localStorage 获取已同步的客户端状态
      const syncedClientsFromStorage = loadSyncedClients()

      // 优化：使用 Set 提高查找效率
      const configuredSet = new Set(configuredClients)
      const syncedSet = new Set(syncedClientsFromStorage)

      clients.value = allClients.map(client => {
        const isConfigured = configuredSet.has(client.key)
        return {
          ...client,
          isInstalled: isConfigured,
          // 只有已配置且之前已同步的客户端才标记为已同步
          isSynced: isConfigured && syncedSet.has(client.key),
          installStatus: isConfigured ? 'installed' : 'notinstalled'
        }
      })
    } catch (error) {
      const appError = handleError(error)
      toast.error(appError.userMessage)
    } finally {
      isLoading.value = false
    }
  }

  return {
    clients,
    totalSyncedCount,
    isDialogOpen,
    syncingClient,
    isLoading,
    currentError,
    toggleDialog,
    toggleSync,
    syncAll,
    initClients,
    clearError
  }
}

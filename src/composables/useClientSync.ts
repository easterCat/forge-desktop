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

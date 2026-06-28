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

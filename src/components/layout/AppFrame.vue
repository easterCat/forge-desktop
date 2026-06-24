<template>
  <div class="window-frame">
    <!-- Titlebar: using system default decorations (custom titlebar hidden) -->
    <!-- <AppTitlebar /> -->

    <!-- Main shell container -->
    <div class="shell">
      <!-- Sidebar navigation -->
      <Sidebar :collapsed="uiStore.sidebarCollapsed" />

      <!-- Main content area -->
      <div class="main">
        <!-- Top navigation bar -->
        <Topbar :title="currentTitle" @refresh="handleRefresh" @settings="handleSettings" @toggle-sidebar="handleToggleSidebar" />

        <!-- Content area with router -->
        <main class="content">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </main>
      </div>
    </div>

    <!-- Mobile bottom tab bar -->
    <MobileTabbar />

    <!-- Toast notifications -->
    <Transition name="toast">
      <div v-if="toast.visible" class="toast" :class="toast.type">
        <span class="toast-icon" :style="{ color: toastIconColor }">{{ toastIcon }}</span>
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, onMounted } from 'vue';
import { useRouter } from 'vue-router';
// import AppTitlebar from '@/components/layout/Titlebar.vue'; // hidden: using system decorations
import Sidebar from '@/components/layout/Sidebar.vue';
import Topbar from '@/components/layout/Topbar.vue';
import MobileTabbar from '@/components/layout/MobileTabbar.vue';
import { useThemeStore } from '@/stores/theme';
import { useUiStore } from '@/stores/ui';
import '@/assets/theme.css';

const router = useRouter();
const themeStore = useThemeStore();
const uiStore = useUiStore();

onMounted(() => {
  themeStore.initTheme();
});

const currentRoute = computed(() => router.currentRoute.value.path);

const pageTitles: Record<string, string> = {
  '/': 'Dashboard',
  '/cli-tools': 'CLI Tools',
  '/software': 'Software',
  '/plugins': 'Plugins',
  '/skills': 'Skills',
  '/mcp': 'MCP Servers',
  '/rules': 'Rules',
  '/backup': 'Backup & Restore',
  '/import-export': 'Import / Export',
  '/settings': 'Settings',
  '/prompts': 'Prompt Manager',
  '/agents': 'Agents',
};

const currentTitle = computed(() => {
  return pageTitles[currentRoute.value] || 'Dashboard';
});

const toast = ref<{
  visible: boolean;
  message: string;
  type: string;
}>({
  visible: false,
  message: '',
  type: ''
});

let toastTimeout: ReturnType<typeof setTimeout> | null = null;

const toastIcon = computed(() => {
  switch (toast.value.type) {
    case 'success': return '✓';
    case 'error': return '✕';
    case 'info': return 'ℹ';
    default: return 'ℹ';
  }
});

const toastIconColor = computed(() => {
  switch (toast.value.type) {
    case 'success': return 'var(--success)';
    case 'error': return 'var(--error)';
    case 'info': return 'var(--info)';
    default: return 'var(--info)';
  }
});

const showNotification = (message: string, type: string = 'info') => {
  if (toastTimeout) {
    clearTimeout(toastTimeout);
  }
  toast.value = {
    visible: true,
    message,
    type
  };
  toastTimeout = setTimeout(() => {
    toast.value.visible = false;
  }, 3000);
};

provide('showNotification', showNotification);

const handleRefresh = () => {
  showNotification('Refresh complete · 0 changes detected', 'success');
};

const handleSettings = () => {
  router.push('/settings');
};

const handleToggleSidebar = () => {
  uiStore.toggleSidebar();
};
</script>

<style scoped>
.window-frame {
  position: fixed;
  inset: 0;
  background: var(--glass-frame-bg);
  backdrop-filter: blur(40px) saturate(1.3);
  -webkit-backdrop-filter: blur(40px) saturate(1.3);
  /* border-radius: 16px; */ /* removed: using system decorations */
  display: flex;
  flex-direction: column;
  overflow: hidden;
  /* Smooth transition between titlebar and content */
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.shell {
  display: flex;
  flex: 1;
  height: 100%;
  overflow: hidden;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 24px 32px 40px;
  position: relative;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(30px);
}

</style>

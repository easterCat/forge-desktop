import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('@/views/DashboardView.vue'),
  },
  {
    path: '/cli-tools',
    name: 'CliTools',
    component: () => import('@/views/CliToolsView.vue'),
  },
  {
    path: '/software',
    name: 'Software',
    component: () => import('@/views/SoftwareManagementView.vue'),
  },
  {
    path: '/plugins',
    name: 'Plugins',
    component: () => import('@/views/PluginsView.vue'),
  },
  {
    path: '/skills',
    name: 'Skills',
    component: () => import('@/views/SkillsView.vue'),
  },
  {
    path: '/mcp',
    name: 'MCP',
    component: () => import('@/views/MCPView.vue'),
  },
  {
    path: '/rules',
    name: 'Rules',
    component: () => import('@/views/RulesView.vue'),
  },
  {
    path: '/backup',
    name: 'Backup',
    component: () => import('@/views/BackupView.vue'),
  },
  {
    path: '/import-export',
    name: 'ImportExport',
    component: () => import('@/views/ImportExportView.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/SettingsView.vue'),
  },
  {
    path: '/prompts',
    name: 'PromptManager',
    component: () => import('@/views/PromptManagerView.vue'),
  },
  {
    path: '/agents',
    name: 'Agents',
    component: () => import('@/views/AgentsView.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;

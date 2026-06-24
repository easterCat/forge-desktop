<template>
  <nav class="mobile-tabbar">
    <router-link
      v-for="tab in tabs"
      :key="tab.route"
      :to="tab.route"
      class="mobile-tab-item"
      :class="{ active: isActive(tab.route) }"
    >
      <component :is="tab.iconComponent" />
      <span>{{ tab.label }}</span>
    </router-link>
  </nav>
</template>

<script setup lang="ts">
import { h } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();

const isActive = (path: string) => {
  return route.path === path;
};

// SVG icon components for mobile tabbar
const HomeIcon = () => h('svg', {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  'stroke-linecap': 'round'
}, [
  h('rect', { x: '3', y: '3', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '14', y: '3', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '14', y: '14', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '3', y: '14', width: '7', height: '7', rx: '1' })
]);

const CLIIcon = () => h('svg', {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  'stroke-linecap': 'round'
}, [
  h('polyline', { points: '4 17 10 11 4 5' }),
  h('line', { x1: '12', y1: '19', x2: '20', y2: '19' })
]);

const PluginsIcon = () => h('svg', {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  'stroke-linecap': 'round'
}, [
  h('path', { d: 'M12 2L2 7l10 5 10-5-10-5z' }),
  h('path', { d: 'M2 17l10 5 10-5' }),
  h('path', { d: 'M2 12l10 5 10-5' })
]);

const SkillsIcon = () => h('svg', {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  'stroke-linecap': 'round'
}, [
  h('polygon', { points: '12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2' })
]);

const SettingsIcon = () => h('svg', {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  'stroke-linecap': 'round'
}, [
  h('circle', { cx: '12', cy: '12', r: '3' }),
  h('path', { d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z' })
]);

const tabs = [
  { route: '/', label: 'Home', iconComponent: HomeIcon },
  { route: '/cli-tools', label: 'CLI', iconComponent: CLIIcon },
  { route: '/plugins', label: 'Plugins', iconComponent: PluginsIcon },
  { route: '/skills', label: 'Skills', iconComponent: SkillsIcon },
  { route: '/settings', label: 'Settings', iconComponent: SettingsIcon },
];
</script>

<style scoped>
.mobile-tabbar {
  display: none;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 56px;
  background: var(--glass-topbar);
  backdrop-filter: blur(24px) saturate(1.3);
  -webkit-backdrop-filter: blur(24px) saturate(1.3);
  border-top: 1px solid var(--border);
  z-index: 100;
  align-items: center;
  justify-content: space-around;
  padding: 0 4px;
}

.mobile-tab-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 6px 8px;
  font-size: 10px;
  font-weight: 500;
  color: var(--fg-ghost);
  cursor: pointer;
  transition: color var(--t-fast);
  border-radius: var(--radius-sm);
  min-width: 48px;
  text-decoration: none;
}

.mobile-tab-item:hover {
  color: var(--fg-muted);
}

.mobile-tab-item.active {
  color: var(--accent);
}

.mobile-tab-item svg {
  width: 20px;
  height: 20px;
}

/* Show on mobile (≤768px) */
@media (max-width: 768px) {
  .mobile-tabbar {
    display: flex;
  }
}
</style>

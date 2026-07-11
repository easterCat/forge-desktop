<template>
  <div class="view active">
    <!-- Environment Overview Section -->
    <div class="section-header">
      <h2>Environment Overview</h2>
      <span class="count">Last scan: 2 min ago</span>
    </div>

    <!-- Stat Cards Row -->
    <div class="stats-row">
      <StatCard
        label="CLI Tools"
        :value="cliToolCount"
        :sub="isLoading ? '' : cliToolSub"
        tint="warm"
        accent
      />
      <StatCard
        label="Software"
        :value="softwareCount"
        :sub="isLoading ? '' : softwareSub"
        tint="cool"
      />
      <StatCard
        label="Plugins"
        :value="pluginCount"
        :sub="isLoading ? '' : pluginSub"
        tint="soft"
      />
      <StatCard
        label="Updates"
        :value="updateCount"
        :sub="isLoading ? '' : 'available'"
        tint="amber"
        :value-style="{ color: 'var(--warn)' }"
      />
    </div>

    <!-- Quick Actions Section -->
    <div class="section-header" style="margin-top: 8px">
      <h2>Quick Actions</h2>
    </div>

    <div class="card-grid">
      <div
        v-for="tool in installedTools"
        :key="tool.key"
        class="card"
        @click="navigateTo('/cli-tools')"
      >
        <div class="card-head">
          <div
            class="card-icon"
            :style="{
              background: (tool.color || '#5C5C5C') + '12',
              color: tool.color || '#5C5C5C',
              borderColor: (tool.color || '#5C5C5C') + '25',
              fontSize: '13px',
              fontWeight: '700',
              fontFamily: 'var(--font-mono)'
            }"
          >{{ tool.icon }}</div>
          <div style="flex: 1; min-width: 0">
            <div class="card-title">
              {{ tool.name }}
              <span v-if="tool.needsUpdate" class="badge warn">Update available</span>
              <span v-else class="badge success">Installed</span>
            </div>
            <div v-if="tool.pkg" class="card-subtitle">{{ tool.pkg }}</div>
          </div>
        </div>
        <div v-if="tool.desc" class="card-desc">{{ normalizeDesc(tool.desc) }}</div>
        <div class="card-meta">
          <div class="card-meta-item">
            <span>Version</span>
            <span class="value">{{ tool.current }}{{ tool.needsUpdate ? ' → ' + tool.latest : '' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import StatCard from '@/components/common/StatCard.vue';
import { useSoftwareStore } from '@/stores/software';
import { usePluginStore } from '@/stores/plugin';
import { useSkillStore } from '@/stores/skill';
import { useMCPStore } from '@/stores/mcp';
import { normalizeDesc } from '@/utils/text';

const router = useRouter();
const softwareStore = useSoftwareStore();
const pluginStore = usePluginStore();
const skillStore = useSkillStore();
const mcpStore = useMCPStore();

const isLoading = ref(true);

// --- Stat counts from stores ---
const cliToolCount = computed(() => softwareStore.cliToolStatuses
  ? Object.keys(softwareStore.cliToolStatuses).length
  : softwareStore.cliTools.length
);
const cliToolSub = computed(() => `${cliToolCount.value} detected`);

const softwareCount = computed(() => softwareStore.softwareList.length);
const softwareSub = computed(() => `${softwareCount.value} packages`);

const pluginCount = computed(() => pluginStore.plugins.length);
const pluginSub = computed(() => `${pluginCount.value} installed`);

const updateCount = computed(() => {
  return softwareStore.cliTools.filter((t) => t.needsUpdate).length;
});

// Installed CLI tools for Quick Actions grid
const installedTools = computed(() => {
  return softwareStore.cliTools.filter((t) => t.installed);
});

function navigateTo(path: string) {
  router.push(path);
}

async function loadData() {
  isLoading.value = true;
  try {
    await Promise.allSettled([
      softwareStore.fetchCliTools().catch(() => {}),
      softwareStore.checkAllCliToolsStatus().catch(() => {}),
      softwareStore.fetchSoftware().catch(() => {}),
      pluginStore.fetchPlugins().catch(() => {}),
      skillStore.fetchSkills().catch(() => {}),
      mcpStore.fetchServices().catch(() => {}),
    ]);
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
/* Stat cards row */
.stats-row {
  display: grid;
  grid-template-columns: repeat(var(--stats-columns, 4), 1fr);
  gap: var(--space-4, 16px);
  margin-bottom: var(--space-6, 24px);
}

/* Section header */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3, 12px);
  padding-bottom: var(--space-3, 12px);
  border-bottom: 1px solid var(--border);
}

.section-header h2 {
  font-size: var(--text-xl, 18px);
  font-weight: 600;
  color: var(--fg-title);
  letter-spacing: -0.01em;
}

.section-header .count {
  font-family: var(--font-mono);
  font-size: var(--text-sm, 12px);
  color: var(--fg-muted);
  background: var(--glass-bg);
  border: 1px solid var(--border);
  padding: var(--space-1, 4px) var(--space-3, 12px);
  border-radius: var(--radius-sm);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

/* Card grid for Quick Actions */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--space-4, 16px);
  align-items: stretch;
}

.card-grid .card {
  cursor: pointer;
}

.card-head {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.card-icon {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border);
}

.card-icon svg { width: 18px; height: 18px; }

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-title);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.card-subtitle {
  font-size: 11px;
  color: var(--fg-ghost);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.card-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  min-height: 2.1em;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  row-gap: 4px;
  word-break: break-word;
}

.card-meta-item {
  font-size: 11px;
  color: var(--fg-ghost);
  display: flex;
  align-items: center;
  gap: 5px;
}

.card-meta-item .value {
  color: var(--fg-muted);
  font-family: var(--font-mono);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
  max-width: 160px;
}

.announcement-text {
  font-size: 13px;
  color: var(--fg-muted);
}

</style>

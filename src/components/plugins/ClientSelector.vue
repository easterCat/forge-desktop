<script setup lang="ts">
/**
 * ClientSelector 组件
 *
 * 支持 23 个 AI 客户端的选择，用于配置同步目标。
 * 可多选，支持全选/取消全选。
 */

import { ref, computed } from 'vue';
import { SUPPORTED_CLIENTS, CLIENT_DISPLAY_NAMES } from '@/types/unified-plugin';
import type { ClientType } from '@/types/unified-plugin';

const props = defineProps<{
  /** 已选中的客户端 */
  modelValue: ClientType[];
  /** 是否禁用 */
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: ClientType[]): void;
}>();

// 搜索关键词
const searchQuery = ref('');

// 按类别分组的客户端
const clientGroups = computed(() => {
  const universal: ClientType[] = [];
  const provider: ClientType[] = [];

  for (const client of SUPPORTED_CLIENTS) {
    if (isUniversalClient(client)) {
      universal.push(client);
    } else {
      provider.push(client);
    }
  }

  return { universal, provider };
});

// 过滤后的客户端
const filteredClients = computed(() => {
  const all = [...clientGroups.value.universal, ...clientGroups.value.provider];

  if (!searchQuery.value) {
    return all;
  }

  const query = searchQuery.value.toLowerCase();
  return all.filter(client => {
    const name = CLIENT_DISPLAY_NAMES[client] ?? client;
    return (
      client.toLowerCase().includes(query) ||
      name.toLowerCase().includes(query)
    );
  });
});

// 全选状态
const isAllSelected = computed(() =>
  filteredClients.value.every(c => props.modelValue.includes(c))
);

// 判断是否为通用客户端
function isUniversalClient(client: ClientType): boolean {
  const universalClients = [
    'copilot', 'codex', 'opencode', 'gemini', 'ampcode',
    'vscode', 'replit', 'kimi',
  ];
  return universalClients.includes(client);
}

// 切换客户端选择
function toggleClient(client: ClientType) {
  if (props.disabled) return;

  const current = [...props.modelValue];
  const index = current.indexOf(client);

  if (index >= 0) {
    current.splice(index, 1);
  } else {
    current.push(client);
  }

  emit('update:modelValue', current);
}

// 全选/取消全选
function toggleAll() {
  if (props.disabled) return;

  if (isAllSelected.value) {
    // 取消全选当前过滤结果
    const current = props.modelValue.filter(
      c => !filteredClients.value.includes(c)
    );
    emit('update:modelValue', current);
  } else {
    // 全选当前过滤结果
    const current = new Set(props.modelValue);
    for (const client of filteredClients.value) {
      current.add(client);
    }
    emit('update:modelValue', Array.from(current));
  }
}

// 选择推荐的常用客户端
function selectRecommended() {
  if (props.disabled) return;

  const recommended: ClientType[] = ['claude', 'copilot', 'cursor'];
  emit('update:modelValue', [...recommended]);
}

// 清空选择
function clearSelection() {
  if (props.disabled) return;
  emit('update:modelValue', []);
}

// 获取客户端图标
function getClientIcon(client: ClientType): string {
  const icons: Record<string, string> = {
    claude: '🤖',
    copilot: '🐙',
    codex: '📝',
    cursor: '🖱️',
    opencode: '🔓',
    gemini: '💎',
    factory: '🏭',
    ampcode: '⚡',
    vscode: '🔵',
    windsurf: '🌊',
    cline: '🔗',
    continue: '▶️',
    roo: '🦘',
    kilo: '📦',
    trae: '🏗️',
    augment: '➕',
    zencoder: '🔐',
    junie: '🐴',
    openhands: '🤲',
    kiro: '🎯',
    replit: '💻',
    kimi: '🇰',
    universal: '🌐',
  };
  return icons[client] ?? '📦';
}
</script>

<template>
  <div class="client-selector">
    <!-- 头部操作 -->
    <div class="selector-header">
      <div class="header-left">
        <h3 class="selector-title">同步目标客户端</h3>
        <span class="selected-count">
          已选 {{ modelValue.length }} / {{ SUPPORTED_CLIENTS.length }}
        </span>
      </div>

      <div class="header-actions">
        <button
          class="action-btn"
          :disabled="disabled"
          @click="selectRecommended"
        >
          推荐选择
        </button>
        <button
          class="action-btn"
          :disabled="disabled"
          @click="toggleAll"
        >
          {{ isAllSelected ? '取消全选' : '全选' }}
        </button>
        <button
          class="action-btn"
          :disabled="disabled || modelValue.length === 0"
          @click="clearSelection"
        >
          清空
        </button>
      </div>
    </div>

    <!-- 搜索框 -->
    <div class="search-box">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索客户端..."
        class="search-input"
        :disabled="disabled"
      />
    </div>

    <!-- 客户端列表 -->
    <div class="client-list">
      <!-- 通用客户端 -->
      <div class="client-group">
        <div class="group-header">
          <span class="group-label">通用客户端</span>
          <span class="group-desc">共享 .agents/skills/</span>
        </div>
        <div class="client-grid">
          <label
            v-for="client in clientGroups.universal"
            :key="client"
            class="client-item"
            :class="{
              selected: modelValue.includes(client),
              disabled: disabled
            }"
          >
            <input
              type="checkbox"
              :checked="modelValue.includes(client)"
              :disabled="disabled"
              class="client-checkbox"
              @change="toggleClient(client)"
            />
            <span class="client-icon">{{ getClientIcon(client) }}</span>
            <span class="client-info">
              <span class="client-name">{{ CLIENT_DISPLAY_NAMES[client] ?? client }}</span>
              <span class="client-id">{{ client }}</span>
            </span>
          </label>
        </div>
      </div>

      <!-- 提供商专属客户端 -->
      <div class="client-group">
        <div class="group-header">
          <span class="group-label">提供商专属客户端</span>
          <span class="group-desc">各自独立目录</span>
        </div>
        <div class="client-grid">
          <label
            v-for="client in clientGroups.provider"
            :key="client"
            class="client-item"
            :class="{
              selected: modelValue.includes(client),
              disabled: disabled
            }"
          >
            <input
              type="checkbox"
              :checked="modelValue.includes(client)"
              :disabled="disabled"
              class="client-checkbox"
              @change="toggleClient(client)"
            />
            <span class="client-icon">{{ getClientIcon(client) }}</span>
            <span class="client-info">
              <span class="client-name">{{ CLIENT_DISPLAY_NAMES[client] ?? client }}</span>
              <span class="client-id">{{ client }}</span>
            </span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.client-selector {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.selector-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selector-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.selected-count {
  font-size: 13px;
  color: #666;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover:not(:disabled) {
  background: #f5f5f5;
  border-color: #bbb;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-box {
  position: relative;
}

.search-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: #4a90d9;
}

.client-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.client-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  font-size: 14px;
  font-weight: 500;
}

.group-desc {
  font-size: 12px;
  color: #888;
}

.client-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.client-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.client-item:hover:not(.disabled) {
  background: #f8f9fa;
  border-color: #ccc;
}

.client-item.selected {
  background: #e8f4fd;
  border-color: #4a90d9;
}

.client-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.client-checkbox {
  width: 16px;
  height: 16px;
  accent-color: #4a90d9;
}

.client-icon {
  font-size: 20px;
}

.client-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.client-name {
  font-size: 14px;
  font-weight: 500;
}

.client-id {
  font-size: 11px;
  color: #888;
}
</style>

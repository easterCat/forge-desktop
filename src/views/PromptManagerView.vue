<template>
  <div class="view prompt-manager-view">
    <div class="external-card">
      <div class="external-icon" aria-hidden="true">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
          <polyline points="15 3 21 3 21 9"/>
          <line x1="10" y1="14" x2="21" y2="3"/>
        </svg>
      </div>
      <h2 class="external-title">提示词优化器</h2>
      <p class="external-desc">
        该站点是独立的 Electron 应用,弹窗数据依赖本地 Electron 桥(本地存储、模型配置等),无法在 Tauri 的 iframe 中加载。
        建议在系统浏览器中打开以获得完整体验。
      </p>
      <div class="external-actions">
        <button class="primary-btn" @click="openInBrowser">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
          在浏览器中打开
        </button>
        <button class="secondary-btn" :disabled="copyState === 'copied'" @click="copyUrl">
          <svg v-if="copyState === 'idle'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          {{ copyState === 'copied' ? '已复制' : '复制链接' }}
        </button>
      </div>
      <p class="external-hint">地址:{{ promptUrl }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-shell';

const promptUrl = 'https://prompt.always200.com/';
const copyState = ref<'idle' | 'copied'>('idle');

const openInBrowser = async () => {
  try {
    await open(promptUrl);
  } catch (e) {
    console.error('Failed to open browser:', e);
  }
};

const copyUrl = async () => {
  try {
    await navigator.clipboard.writeText(promptUrl);
    copyState.value = 'copied';
    setTimeout(() => { copyState.value = 'idle'; }, 1500);
  } catch (e) {
    console.error('Failed to copy URL:', e);
  }
};
</script>

<style scoped>
.prompt-manager-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  overflow: hidden;
}

.external-card {
  max-width: 480px;
  width: 100%;
  padding: 32px 28px;
  text-align: center;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: 12px;
  box-shadow: var(--shadow-md);
  transition: all 200ms ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.external-card:hover {
  background: var(--bg-card-hover);
  border-color: rgba(255, 255, 255, 0.50);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

.external-icon {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.20);
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.18);
  color: var(--accent);
}

.external-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--fg-primary, #111827);
}

.external-desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--fg-muted, #6b7280);
}

.external-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.primary-btn,
.secondary-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.primary-btn {
  background: var(--accent);
  color: #ffffff;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.primary-btn:hover {
  filter: brightness(1.05);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

.secondary-btn {
  background: rgba(255, 255, 255, 0.18);
  color: var(--fg-secondary, #4b5563);
  border: 1px solid rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.secondary-btn:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}

.secondary-btn:disabled {
  cursor: default;
  color: var(--accent);
}

.external-hint {
  margin: 0;
  font-size: 11px;
  color: var(--fg-muted, #9ca3af);
  word-break: break-all;
}
</style>

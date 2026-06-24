<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="add-source-backdrop"
      @click.self="$emit('update:modelValue', false)"
    >
      <div class="add-source-dialog" role="dialog" aria-modal="true" aria-labelledby="add-source-title">
        <div class="add-source-header">
          <h3 id="add-source-title" class="add-source-title">添加仓库源</h3>
          <button
            class="add-source-close"
            aria-label="关闭"
            title="关闭"
            @click="$emit('update:modelValue', false)"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div class="add-source-body">
          <label class="add-source-label">
            仓库类型
          </label>
          <div class="repo-type-selector" role="radiogroup" aria-label="仓库类型">
            <button
              class="repo-type-option"
              :class="{ active: repoType === 'market' }"
              role="radio"
              :aria-checked="repoType === 'market'"
              @click="repoType = 'market'"
            >
              <span class="repo-type-name">market</span>
              <span class="repo-type-desc">市场仓库，含多个插件</span>
            </button>
            <button
              class="repo-type-option"
              :class="{ active: repoType === 'res' }"
              role="radio"
              :aria-checked="repoType === 'res'"
              @click="repoType = 'res'"
            >
              <span class="repo-type-name">res</span>
              <span class="repo-type-desc">单插件仓库</span>
            </button>
          </div>
          <label for="repo-url-input" class="add-source-label" style="margin-top:8px">
            仓库地址 (GitHub URL)
          </label>
          <input
            id="repo-url-input"
            v-model="localUrl"
            class="add-source-input"
            :class="{ error: localError }"
            type="text"
            placeholder="https://github.com/owner/repo"
            autocomplete="off"
            spellcheck="false"
            @keydown.enter="handleConfirm"
            @keydown.esc="$emit('update:modelValue', false)"
          />
          <div v-if="localError" class="add-source-error">
            {{ localError }}
          </div>
        </div>

        <div class="add-source-footer">
          <button
            class="btn btn-secondary btn-sm"
            @click="$emit('update:modelValue', false)"
          >
            取消
          </button>
          <button
            class="btn btn-primary btn-sm"
            :disabled="!localUrl.trim()"
            @click="handleConfirm"
          >
            确认
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { RepoType } from '@/types/plugin-marketplace';

const props = defineProps<{
  modelValue: boolean;
  error?: string | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'confirm': [url: string, repoType: RepoType];
}>();

const localUrl = ref('');
const localError = ref<string | null>(null);
const repoType = ref<RepoType>('market');

watch(() => props.modelValue, (open) => {
  if (open) {
    localUrl.value = '';
    localError.value = null;
    repoType.value = 'market';
  }
});

watch(() => props.error, (err) => {
  localError.value = err ?? null;
});

function handleConfirm() {
  const url = localUrl.value.trim();
  if (!url) {
    localError.value = '请输入仓库地址';
    return;
  }
  emit('confirm', url, repoType.value);
}
</script>

<style scoped>
/* =====================================================
   Add Source Dialog — Spacing System (8px grid)
   =====================================================
   Backdrop:  blur(8px), semi-transparent
   Dialog:    480px max, glass morphism
   Spacing:   8 / 16 / 20 / 24 / 32 (8px multiples)
   ===================================================== */

.add-source-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: 16px;
  animation: fadeIn 0.2s ease;
}

.add-source-dialog {
  background: var(--glass-bg, rgba(255, 255, 255, 0.48));
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  box-shadow:
    0 20px 60px rgba(0, 0, 0, 0.12),
    0 4px 16px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.50);
  width: 480px;
  max-width: calc(100vw - 32px);
  display: flex;
  flex-direction: column;
  animation: dialogIn 0.25s ease;
}

/* ---- Header ---- */
.add-source-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.add-source-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
  letter-spacing: -0.01em;
}

.add-source-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm, 8px);
  color: var(--fg-muted);
  cursor: pointer;
  transition: all var(--transition-fast, 0.15s);
  flex-shrink: 0;
}

.add-source-close:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

/* ---- Body ---- */
.add-source-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.add-source-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-muted);
  letter-spacing: 0.02em;
  text-transform: uppercase;
  margin-bottom: 4px;
}

/* ---- Type Selector ---- */
.repo-type-selector {
  display: flex;
  gap: 10px;
}

.repo-type-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 14px 12px;
  background: var(--glass-input, var(--bg-tertiary));
  border: 2px solid var(--border);
  border-radius: var(--radius-sm, 8px);
  cursor: pointer;
  transition: all var(--transition-fast, 0.15s);
}

.repo-type-option:hover {
  border-color: var(--fg-muted);
  background: var(--glass-input-focus, var(--bg-tertiary));
}

.repo-type-option.active {
  border-color: var(--accent);
  background: rgba(245, 158, 11, 0.08);
  box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.12);
}

.repo-type-name {
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-mono);
  color: var(--fg);
}

.repo-type-option.active .repo-type-name {
  color: var(--accent);
}

.repo-type-desc {
  font-size: 11px;
  color: var(--fg-muted);
  line-height: 1.3;
}

/* ---- Input ---- */
.add-source-input {
  width: 100%;
  padding: 10px 14px;
  font-size: 13px;
  font-family: var(--font-mono);
  background: var(--glass-input, var(--bg-tertiary));
  border: 1px solid var(--border);
  border-radius: var(--radius-sm, 8px);
  color: var(--fg);
  outline: none;
  transition: all var(--transition-fast, 0.15s);
  box-sizing: border-box;
}

.add-source-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.12);
  background: var(--glass-input-focus, var(--bg-tertiary));
}

.add-source-input.error {
  border-color: var(--error, #ef4444);
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.12);
}

.add-source-input::placeholder {
  color: var(--fg-ghost);
}

.add-source-error {
  font-size: 12px;
  color: var(--error, #ef4444);
  padding: 4px 0 0;
  line-height: 1.4;
}

/* ---- Footer ---- */
.add-source-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px 16px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

/* ---- Animations ---- */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes dialogIn {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(8px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* ---- Responsive: Mobile (< 480px) ---- */
@media (max-width: 480px) {
  .add-source-backdrop {
    padding: 0;
    align-items: flex-end;
  }

  .add-source-dialog {
    width: 100%;
    max-width: 100%;
    max-height: 85vh;
    border-radius: var(--radius-xl) var(--radius-xl) 0 0;
    animation: slideUp 0.25s ease;
  }

  .add-source-body {
    padding: 16px;
  }

  .repo-type-selector {
    flex-direction: column;
    gap: 8px;
  }

  .repo-type-option {
    flex-direction: row;
    justify-content: flex-start;
    gap: 12px;
    padding: 12px 16px;
  }

  .repo-type-desc {
    text-align: left;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(100%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* ---- Reduced Motion ---- */
@media (prefers-reduced-motion: reduce) {
  .add-source-backdrop,
  .add-source-dialog,
  .repo-type-option,
  .add-source-input,
  .add-source-close {
    animation: none;
    transition: none;
  }
}
</style>

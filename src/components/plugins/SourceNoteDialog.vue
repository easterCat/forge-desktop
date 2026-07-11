<script setup lang="ts">
// `v-html` is intentional in this component: it renders the output of
// `renderMarkdown`, which uses a placeholder protocol that HTML-escapes
// user text at stash time and runs every link through `sanitizeUrl()`.
// The emitted HTML is restricted to the closed set of tags listed in
// `safeTags` (h1-h4, ul, ol, li, p, strong, em, code, pre, blockquote,
// a, hr). Reviewed against the S-1 XSS finding from the security audit.
/* eslint-disable vue/no-v-html */
import { computed, ref, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';

const props = defineProps<{
  visible: boolean;
  sourceId: string;
  sourceName: string;
  initialNote: string;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  save: [sourceId: string, note: string];
}>();

// Mirrors the renderer cap. Keeping this in sync with `MAX_INPUT_LEN` in
// `utils/markdown.ts` prevents the textarea from holding text that the
// renderer will silently truncate, so the user gets immediate feedback
// when their note is too long.
const MAX_NOTE_LEN = 256 * 1024;
const noteContent = ref('');
const activeTab = ref<'edit' | 'preview'>('edit');

// `renderMarkdown` returns HTML. With our placeholder-based renderer the
// output is already safe for `v-html`, but we still compute it through a
// `computed` so the heavy work is cached and skipped on every keystroke
// while editing.
const previewHtml = computed(() => renderMarkdown(noteContent.value));

const isOverLimit = computed(() => noteContent.value.length > MAX_NOTE_LEN);

// Sync content when dialog opens
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      noteContent.value = props.initialNote;
      activeTab.value = 'edit';
    }
  },
  { immediate: true },
);

function handleSave() {
  if (isOverLimit.value) return;
  emit('save', props.sourceId, noteContent.value);
  emit('update:visible', false);
}

function handleClose() {
  emit('update:visible', false);
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="note-dialog-overlay" @click.self="handleClose">
      <div class="note-dialog" role="dialog" aria-modal="true" aria-labelledby="note-dialog-title">
        <div class="note-dialog-header">
          <h3 id="note-dialog-title">备注 - {{ sourceName }}</h3>
          <button class="note-dialog-close" aria-label="关闭" @click="handleClose">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div class="note-dialog-tabs">
          <button
            class="note-tab"
            :class="{ active: activeTab === 'edit' }"
            @click="activeTab = 'edit'"
          >
            编辑
          </button>
          <button
            class="note-tab"
            :class="{ active: activeTab === 'preview' }"
            @click="activeTab = 'preview'"
          >
            预览
          </button>
        </div>

        <div class="note-dialog-body">
          <textarea
            v-if="activeTab === 'edit'"
            v-model="noteContent"
            class="note-textarea"
            placeholder="输入 Markdown 备注..."
            spellcheck="false"
            :maxlength="MAX_NOTE_LEN"
          />
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div
            v-else
            class="note-preview markdown-body"
            v-html="previewHtml"
          />
          <p v-if="isOverLimit" class="note-warning" role="alert">
            备注超过最大长度（{{ MAX_NOTE_LEN }} 字符），请精简后保存。
          </p>
        </div>

        <div class="note-dialog-footer">
          <button class="btn btn-outline btn-sm" :disabled="isOverLimit" @click="handleClose">取消</button>
          <button class="btn btn-primary btn-sm" :disabled="isOverLimit" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
/* =====================================================
   Note Dialog — Spacing System (8px grid)
   =====================================================
   Overlay:  transparent backdrop
   Dialog:   800px max, 80vh height
   Spacing:  8 / 16 / 24 / 32 / 40 (8px multiples)
   ===================================================== */

.note-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: 16px; /* safe area on mobile */
}

.note-dialog {
  background: var(--glass-bg, rgba(255, 255, 255, 0.48));
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  width: 800px;
  max-width: 90vw;
  height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow:
    0 20px 60px rgba(0, 0, 0, 0.12),
    0 4px 16px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.50);
}

/* ---- Header: title + close button ---- */
.note-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.note-dialog-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--fg);
  letter-spacing: -0.01em;
  line-height: 1.4;
}

.note-dialog-close {
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
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.note-dialog-close:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

/* ---- Tabs: edit / preview ---- */
.note-dialog-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.note-tab {
  flex: 1;
  padding: 10px 16px;
  background: none;
  border: none;
  color: var(--fg-ghost);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  margin-bottom: -1px; /* overlap border-bottom */
}

.note-tab:hover {
  color: var(--fg);
}

.note-tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* ---- Body: textarea / preview ---- */
.note-dialog-body {
  flex: 1;
  min-height: 0;
}

.note-textarea {
  width: 100%;
  height: 100%;
  padding: 16px;
  overflow: auto;
  background: var(--bg-input);
  color: var(--fg);
  border: none;
  border-radius: 0;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
}

.note-textarea::placeholder {
  color: var(--fg-ghost);
}

.note-preview {
  height: 100%;
  padding: 16px;
  overflow: auto;
  font-size: 14px;
  line-height: 1.6;
  color: var(--fg);
}

/* ---- Footer: cancel / save ---- */
.note-dialog-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.note-warning {
  margin: 8px 16px 0;
  padding: 8px 12px;
  background: var(--color-danger-soft, rgba(255, 80, 80, 0.12));
  color: var(--color-danger, #c0392b);
  border-radius: var(--radius-sm, 8px);
  font-size: 12px;
  line-height: 1.5;
}

/* ---- Markdown content styles ---- */
.markdown-body :deep(h1) {
  font-size: 1.5em;
  font-weight: 700;
  margin: 0 0 0.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border);
}

.markdown-body :deep(h2) {
  font-size: 1.3em;
  font-weight: 600;
  margin: 1em 0 0.5em;
}

.markdown-body :deep(h3) {
  font-size: 1.1em;
  font-weight: 600;
  margin: 0.8em 0 0.4em;
}

.markdown-body :deep(h4) {
  font-size: 1em;
  font-weight: 600;
  margin: 0.6em 0 0.3em;
}

.markdown-body :deep(p) {
  margin: 0 0 0.6em;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(strong) {
  font-weight: 600;
}

.markdown-body :deep(em) {
  font-style: italic;
}

.markdown-body :deep(code) {
  background: var(--bg-tertiary);
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 0.9em;
}

.markdown-body :deep(pre) {
  background: var(--bg-tertiary);
  padding: 12px 16px;
  border-radius: var(--radius-sm, 8px);
  overflow-x: auto;
  margin: 0.6em 0;
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 3px solid var(--accent);
  padding-left: 12px;
  margin: 0.6em 0;
  color: var(--fg-muted);
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.markdown-body :deep(li) {
  margin: 0.25em 0;
}

.markdown-body :deep(li:first-child) {
  margin-top: 0;
}

.markdown-body :deep(li:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--border);
  margin: 1em 0;
}

.markdown-body :deep(a) {
  color: var(--accent);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: var(--radius-sm, 8px);
}

/* ---- Responsive: Tablet (< 768px) ---- */
@media (max-width: 768px) {
  .note-dialog-overlay {
    padding: 12px;
    align-items: flex-end; /* bottom sheet on mobile */
  }

  .note-dialog {
    width: 100%;
    max-width: 100%;
    max-height: 85vh;
    border-radius: var(--radius-xl) var(--radius-xl) 0 0;
  }

  .note-dialog-header {
    padding: 16px;
  }

  .note-tab {
    padding: 10px 12px;
    font-size: 14px;
  }

  .note-textarea,
  .note-preview {
    padding: 12px 16px;
  }

  .note-dialog-footer {
    padding: 12px 16px;
  }
}

/* ---- Responsive: Mobile (< 480px) ---- */
@media (max-width: 480px) {
  .note-dialog-overlay {
    padding: 0;
  }

  .note-dialog {
    max-height: 100vh;
    border-radius: 0;
  }

  .note-dialog-header {
    padding: 12px 16px;
  }

  .note-dialog-header h3 {
    font-size: 14px;
  }

  .note-tab {
    padding: 8px 12px;
    font-size: 13px;
  }

  .note-textarea,
  .note-preview {
    padding: 12px;
    font-size: 13px;
  }

  .note-dialog-footer {
    padding: 10px 16px;
    gap: 6px;
  }
}

/* ---- Reduced Motion ---- */
@media (prefers-reduced-motion: reduce) {
  .note-dialog-close,
  .note-tab {
    transition: none;
  }
}
</style>

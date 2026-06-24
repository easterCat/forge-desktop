<template>
  <Teleport to="body">
  <div class="dialog-overlay" @click.self="onCancel">
    <div class="dialog" role="dialog" aria-modal="true">
      <div class="dialog-header">
        <h3>{{ isEdit ? '编辑推荐网站' : '新增推荐网站' }}</h3>
        <button class="close-btn" @click="onCancel" aria-label="关闭">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label for="site-name">网站名称</label>
          <input
            id="site-name"
            ref="nameInput"
            v-model.trim="form.name"
            type="text"
            placeholder="例如：SkillMP"
            maxlength="60"
            @keydown.enter="onSubmit"
          />
          <span v-if="errors.name" class="error-text">{{ errors.name }}</span>
        </div>

        <div class="form-group">
          <label for="site-url">跳转链接</label>
          <input
            id="site-url"
            v-model.trim="form.url"
            type="url"
            placeholder="https://example.com"
            @keydown.enter="onSubmit"
          />
          <span v-if="errors.url" class="error-text">{{ errors.url }}</span>
        </div>

        <div class="form-group">
          <label for="site-description">介绍</label>
          <textarea
            id="site-description"
            v-model.trim="form.description"
            rows="3"
            placeholder="用一段话介绍这个网站能提供哪些技能资源..."
            maxlength="240"
          ></textarea>
          <div class="hint-row">
            <span v-if="errors.description" class="error-text">{{ errors.description }}</span>
            <span v-else class="hint">建议 1-2 句话，不超过 240 字</span>
            <span class="char-count">{{ form.description.length }}/240</span>
          </div>
        </div>

        <div class="form-group">
          <label>所属区域</label>
          <div class="region-pills">
            <button
              v-for="opt in regionOptions"
              :key="opt.value"
              type="button"
              class="region-pill"
              :class="{ active: form.region === opt.value }"
              :style="form.region === opt.value ? { borderColor: opt.color, color: opt.color } : null"
              @click="form.region = opt.value"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="onCancel">取消</button>
        <button class="btn btn-primary" @click="onSubmit">
          {{ isEdit ? '保存修改' : '新增' }}
        </button>
      </div>
    </div>
  </div>
</Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue';
import type { RecommendedSite, SiteRegion } from '@/stores/recommended-sites';

const props = defineProps<{
  /** When provided, the dialog runs in edit mode and seeds the form. */
  site?: RecommendedSite | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (
    e: 'submit',
    payload: { name: string; description: string; url: string; region: SiteRegion }
  ): void;
}>();

const isEdit = computed(() => Boolean(props.site));

interface FormState {
  name: string;
  description: string;
  url: string;
  region: SiteRegion;
}

const form = reactive<FormState>({
  name: props.site?.name ?? '',
  description: props.site?.description ?? '',
  url: props.site?.url ?? '',
  region: props.site?.region ?? 'international',
});

const errors = reactive<{ name: string; url: string; description: string }>({
  name: '',
  url: '',
  description: '',
});

const regionOptions: { value: SiteRegion; label: string; color: string }[] = [
  { value: 'international', label: '国际', color: 'var(--info)' },
  { value: 'china', label: '国内', color: 'var(--error)' },
  { value: 'github', label: 'GitHub', color: 'var(--fg)' },
  { value: 'other', label: '其他', color: 'var(--fg-muted)' },
];

const nameInput = ref<HTMLInputElement | null>(null);

function normalizeUrl(value: string): string {
  const v = value.trim();
  if (!v) return v;
  // Auto-prepend https:// when the user typed a bare domain like "example.com".
  if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(v)) {
    return `https://${v}`;
  }
  return v;
}

function isValidUrl(value: string): boolean {
  try {
    const u = new URL(value);
    return u.protocol === 'http:' || u.protocol === 'https:';
  } catch {
    return false;
  }
}

function validate(): boolean {
  errors.name = '';
  errors.url = '';
  errors.description = '';

  if (!form.name) {
    errors.name = '请填写网站名称';
  } else if (form.name.length > 60) {
    errors.name = '名称不能超过 60 个字符';
  }

  const normalized = normalizeUrl(form.url);
  if (!form.url) {
    errors.url = '请填写跳转链接';
  } else if (!isValidUrl(normalized)) {
    errors.url = '请填写合法的 http(s) 链接';
  }

  if (!form.description) {
    errors.description = '请填写一段介绍';
  } else if (form.description.length > 240) {
    errors.description = '介绍不能超过 240 个字符';
  }

  return !errors.name && !errors.url && !errors.description;
}

function onSubmit(): void {
  if (!validate()) return;
  emit('submit', {
    name: form.name,
    description: form.description,
    url: normalizeUrl(form.url),
    region: form.region,
  });
}

function onCancel(): void {
  emit('close');
}

onMounted(async () => {
  // Focus the name field for fast keyboard entry.
  await nextTick();
  nameInput.value?.focus();
});
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.dialog {
  position: relative;
  width: 100%;
  max-width: 480px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  z-index: 1;
  display: flex;
  flex-direction: column;
  max-height: 90vh;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-secondary);
  color: var(--fg);
}

.dialog-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--fg-muted);
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 13px;
  color: var(--fg);
  font-family: inherit;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  resize: vertical;
}

.form-group input:focus,
.form-group textarea:focus {
  border-color: var(--accent);
  box-shadow: var(--focus-ring);
}

.error-text {
  font-size: 12px;
  color: var(--error);
}

.hint-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.hint {
  font-size: 11px;
  color: var(--fg-muted);
}

.char-count {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--fg-muted);
  margin-left: auto;
}

.region-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.region-pill {
  padding: 5px 12px;
  font-size: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.region-pill:hover {
  border-color: var(--border-hover);
  color: var(--fg);
}

.region-pill.active {
  background: var(--bg-primary);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}
</style>

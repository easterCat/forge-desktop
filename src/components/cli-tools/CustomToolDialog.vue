<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useSoftwareStore } from '@/stores/software'

interface Props {
  /** When provided, opens in edit mode pre-filled with the tool's data */
  tool?: { key: string; name: string; websiteUrl?: string } | null
}

const props = withDefaults(defineProps<Props>(), {
  tool: null,
})

const emit = defineEmits<{
  close: []
  saved: []
}>()

const softwareStore = useSoftwareStore()

const isEdit = computed(() => !!props.tool)
const title = computed(() => isEdit.value ? `Edit: ${props.tool?.name}` : 'Add Custom Tool')

const form = ref({
  name: '',
  key: '',
  installMethod: 'npm',
  installCommand: '',
  detectCommand: '',
  websiteUrl: '',
  pluginDir: '',
})

const errors = ref<Record<string, string>>({})
const isSubmitting = ref(false)

// Fill form when editing
watch(() => props.tool, (t) => {
  if (t) {
    form.value.name = t.name
    form.value.key = t.key
    form.value.websiteUrl = t.websiteUrl || ''
    form.value.pluginDir = ''
    form.value.installMethod = 'npm'
    form.value.installCommand = ''
    form.value.detectCommand = ''
  } else {
    form.value = {
      name: '', key: '', installMethod: 'npm',
      installCommand: '', detectCommand: '',
      websiteUrl: '', pluginDir: '',
    }
  }
  errors.value = {}
}, { immediate: true })

// Auto-generate key from name
watch(() => form.value.name, (name) => {
  if (!isEdit.value) {
    form.value.key = name
      .toLowerCase()
      .replace(/[^a-z0-9-]/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '')
  }
})

// Auto-fill detect command from key
watch(() => form.value.key, (key) => {
  if (!isEdit.value && key) {
    form.value.detectCommand = `${key} --version`
  }
})

function validate(): boolean {
  errors.value = {}
  if (!form.value.name.trim()) {
    errors.value.name = 'Tool name is required'
  }
  if (!form.value.key.trim()) {
    errors.value.key = 'Tool key is required'
  } else if (!/^[a-z0-9-]+$/.test(form.value.key)) {
    errors.value.key = 'Key must be lowercase letters, numbers, and hyphens only'
  }
  if (!form.value.detectCommand.trim()) {
    errors.value.detectCommand = 'Detect command is required'
  }
  return Object.keys(errors.value).length === 0
}

async function handleSubmit() {
  if (!validate()) return

  isSubmitting.value = true
  try {
    await softwareStore.addCustomCliTool({
      key: form.value.key.trim(),
      name: form.value.name.trim(),
      installMethod: form.value.installMethod,
      installCommand: form.value.installCommand.trim(),
      detectCommand: form.value.detectCommand.trim(),
      websiteUrl: form.value.websiteUrl.trim() || undefined,
      pluginDir: form.value.pluginDir.trim() || undefined,
    })
    emit('saved')
  } catch {
    // error is handled by the store
  } finally {
    isSubmitting.value = false
  }
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <div class="dialog-overlay" @click.self="handleClose">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ title }}</h3>
        <button class="close-btn" aria-label="Close" @click="handleClose">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <form class="dialog-body" @submit.prevent="handleSubmit">
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Tool Name <span class="required">*</span></label>
            <input
              v-model="form.name"
              type="text"
              class="form-input"
              :class="{ 'has-error': errors.name }"
              placeholder="My Custom CLI"
              autocomplete="off"
            />
            <span v-if="errors.name" class="form-error">{{ errors.name }}</span>
          </div>
          <div class="form-group">
            <label class="form-label">Tool Key <span class="required">*</span></label>
            <input
              v-model="form.key"
              type="text"
              class="form-input form-input-mono"
              :class="{ 'has-error': errors.key }"
              placeholder="my-custom-cli"
              autocomplete="off"
            />
            <span v-if="errors.key" class="form-error">{{ errors.key }}</span>
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Install Method <span class="required">*</span></label>
            <select v-model="form.installMethod" class="form-input form-select">
              <option value="npm">npm</option>
              <option value="curl-bash">curl | bash</option>
              <option value="brew">brew</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">Website URL</label>
            <input
              v-model="form.websiteUrl"
              type="url"
              class="form-input"
              placeholder="https://example.com"
              autocomplete="off"
            />
          </div>
        </div>

        <div class="form-group">
          <label class="form-label">
            Install Command <span class="required">*</span>
          </label>
          <textarea
            v-model="form.installCommand"
            class="form-input form-textarea"
            :placeholder="form.installMethod === 'npm' ? 'npm install -g my-cli@latest' : form.installMethod === 'brew' ? 'brew install my-cli' : 'curl -fsSL https://example.com/install.sh | bash'"
            rows="2"
          ></textarea>
          <span class="form-hint">The shell command used to install this tool</span>
        </div>

        <div class="form-group">
          <label class="form-label">Detect Command <span class="required">*</span></label>
          <input
            v-model="form.detectCommand"
            type="text"
            class="form-input form-input-mono"
            :class="{ 'has-error': errors.detectCommand }"
            placeholder="my-custom-cli --version"
            autocomplete="off"
          />
          <span class="form-hint">Command to check if the tool is installed (e.g. which, --version)</span>
          <span v-if="errors.detectCommand" class="form-error">{{ errors.detectCommand }}</span>
        </div>

        <div class="form-group">
          <label class="form-label">Plugin Directory</label>
          <input
            v-model="form.pluginDir"
            type="text"
            class="form-input form-input-mono"
            placeholder="~/.mycli/plugins"
            autocomplete="off"
          />
          <span class="form-hint">Directory for syncing plugins (optional)</span>
        </div>
      </form>

      <div class="dialog-footer">
        <button type="button" class="btn btn-secondary btn-sm" @click="handleClose">Cancel</button>
        <button
          type="submit"
          class="btn btn-primary btn-sm"
          :disabled="isSubmitting"
          @click="handleSubmit"
        >
          {{ isSubmitting ? 'Saving...' : (isEdit ? 'Save Changes' : 'Add Tool') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal, 50);
  animation: overlay-in 0.2s ease;
}

@keyframes overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  width: 100%;
  max-width: 520px;
  max-height: 90vh;
  background: rgba(255, 255, 255, 0.56);
  backdrop-filter: blur(40px) saturate(1.4);
  -webkit-backdrop-filter: blur(40px) saturate(1.4);
  border: 1px solid rgba(255, 255, 255, 0.40);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.50);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: dialog-in 0.2s ease;
}

@keyframes dialog-in {
  from { transform: scale(0.96) translateY(8px); opacity: 0; }
  to { transform: scale(1) translateY(0); opacity: 1; }
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.15);
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  cursor: pointer;
  color: var(--fg-ghost);
  transition: all var(--t-fast);
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.10);
  color: var(--fg);
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.15);
}

/* Form styles */
.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-muted);
  letter-spacing: 0.02em;
}

.required {
  color: var(--error);
}

.form-input {
  padding: 8px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(8px);
  font-size: 13px;
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast), box-shadow var(--t-fast);
  width: 100%;
  box-sizing: border-box;
}

.form-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12);
}

.form-input.has-error {
  border-color: var(--error);
}

.form-input-mono {
  font-family: var(--font-mono);
  font-size: 12px;
}

.form-select {
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717A' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 32px;
}

.form-textarea {
  resize: vertical;
  min-height: 52px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.5;
}

.form-error {
  font-size: 11px;
  color: var(--error);
}

.form-hint {
  font-size: 11px;
  color: var(--fg-ghost);
}
</style>

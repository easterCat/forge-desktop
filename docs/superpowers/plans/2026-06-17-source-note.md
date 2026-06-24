# 数据源备注功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Plugins > Sources 标签页的每个数据源卡片添加 Markdown 备注功能，支持添加、编辑、查看备注，方便快速了解仓库用途。

**Architecture:** 采用独立 JSON 文件存储备注数据，Rust 后端提供读写 Tauri command，前端 Pinia store 管理状态，新增 SourceNoteDialog 弹窗组件负责编辑交互，自定义轻量 Markdown 渲染器支持基本语法。

**Tech Stack:** Rust (serde_json), Vue 3 + TypeScript, Pinia, Tauri 2.0

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src-tauri/src/services/plugin_marketplace.rs` | 修改 | 新增 `SourceNotesRegistry`、`source_notes_path()`、`read_source_notes()`、`write_source_notes()` |
| `src-tauri/src/commands/plugin_marketplace.rs` | 修改 | 新增 `get_source_notes`、`save_source_note` Tauri command |
| `src-tauri/src/lib.rs` | 修改 | 注册 2 个新 command |
| `src/stores/plugin-marketplace.ts` | 修改 | 新增 `sourceNotes` state + `loadSourceNotes` / `saveSourceNote` action |
| `src/utils/markdown.ts` | 新增 | 轻量 Markdown 渲染函数 |
| `src/components/plugins/SourceNoteDialog.vue` | 新增 | 备注编辑弹窗组件 |
| `src/views/PluginsView.vue` | 修改 | 源卡片增加备注图标按钮 + 引入 SourceNoteDialog |

---

## Task 1: Rust 后端 - 备注数据服务

**Files:**
- Modify: `src-tauri/src/services/plugin_marketplace.rs`

- [ ] **Step 1: 在 `user_sources_path()` 函数之后添加 `SourceNotesRegistry` 结构体和相关函数**

在 `src-tauri/src/services/plugin_marketplace.rs` 中找到 `user_sources_path()` 函数（约第 184 行），在其后添加：

```rust
// ---------------------------------------------------------------------------
// Source notes (FEAT-020)
// User-authored Markdown notes for each source, persisted to
// `$FORGE_HOME/plugins/source_notes.json`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceNotesRegistry {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub notes: HashMap<String, String>,
}

pub fn source_notes_path() -> PathBuf {
    plugins_dir().join("source_notes.json")
}

/// Read all source notes from disk.
/// Returns an empty map if the file does not exist or cannot be parsed.
pub fn read_source_notes() -> HashMap<String, String> {
    let path = source_notes_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<SourceNotesRegistry>(&content) {
            Ok(reg) => reg.notes,
            Err(e) => {
                log::warn!("Failed to parse source_notes.json: {}", e);
                HashMap::new()
            }
        },
        Err(e) => {
            log::warn!("Failed to read source_notes.json: {}", e);
            HashMap::new()
        }
    }
}

/// Write all source notes to disk.
pub fn write_source_notes(notes: &HashMap<String, String>) -> Result<(), String> {
    let path = source_notes_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create plugins dir: {}", e))?;
    }
    let reg = SourceNotesRegistry {
        version: "1".to_string(),
        notes: notes.clone(),
    };
    let json = serde_json::to_string_pretty(&reg)
        .map_err(|e| format!("Failed to serialize source_notes.json: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write source_notes.json: {}", e))?;
    Ok(())
}
```

确保文件顶部有 `use std::collections::HashMap;` 导入（如果没有则添加）。

- [ ] **Step 2: 验证 Rust 代码编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/services/plugin_marketplace.rs
git commit -m "feat: add source notes service layer (read/write/registry)"
```

---

## Task 2: Rust 后端 - Tauri Commands

**Files:**
- Modify: `src-tauri/src/commands/plugin_marketplace.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 `commands/plugin_marketplace.rs` 末尾添加两个新 command**

在文件末尾（`update_source_repo_type` 函数之后）添加：

```rust
// ---------------------------------------------------------------------------
// Source notes commands (FEAT-020)
// ---------------------------------------------------------------------------

/// Return all source notes as a map of source_id → markdown string.
#[tauri::command]
pub async fn get_source_notes() -> HashMap<String, String> {
    log::info!("Getting source notes");
    plugin_marketplace::read_source_notes()
}

/// Save a note for a source. Passing an empty string deletes the note.
/// Returns `Ok(true)` on success.
#[tauri::command]
pub async fn save_source_note(
    source_id: String,
    note: String,
) -> Result<bool, String> {
    log::info!("Saving source note for: {}", source_id);
    let mut notes = plugin_marketplace::read_source_notes();
    if note.is_empty() {
        notes.remove(&source_id);
    } else {
        notes.insert(source_id, note);
    }
    plugin_marketplace::write_source_notes(&notes)?;
    Ok(true)
}
```

确保文件顶部有 `use std::collections::HashMap;` 导入（如果没有则添加）。

- [ ] **Step 2: 在 `lib.rs` 的 `invoke_handler` 中注册新 command**

打开 `src-tauri/src/lib.rs`，找到 `invoke_handler`（约第 156 行），在 `update_source_repo_type,`（约第 199 行）之后添加：

```rust
            commands::plugin_marketplace::get_source_notes,
            commands::plugin_marketplace::save_source_note,
```

- [ ] **Step 3: 验证 Rust 代码编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/plugin_marketplace.rs src-tauri/src/lib.rs
git commit -m "feat: add get_source_notes and save_source_note Tauri commands"
```

---

## Task 3: 前端 Store - 备注状态管理

**Files:**
- Modify: `src/stores/plugin-marketplace.ts`

- [ ] **Step 1: 在 store 的 state 区域添加 `sourceNotes`**

在 `src/stores/plugin-marketplace.ts` 中找到 `sourceInstallProgress` 的定义（约第 63 行），在其后添加：

```typescript
  // Source notes state (FEAT-020)
  const sourceNotes = ref<Record<string, string>>({});
```

- [ ] **Step 2: 在 `loadSourceStatus` 函数末尾添加 `loadSourceNotes` 调用**

找到 `loadSourceStatus` 函数（约第 502 行），在函数体的 `}` 之前（`catch` 块之后）添加：

```typescript
    // Load source notes (FEAT-020)
    await loadSourceNotes();
```

- [ ] **Step 3: 在 store 的 actions 区域添加 `loadSourceNotes` 和 `saveSourceNote`**

在 `loadSourceStatus` 函数之后添加：

```typescript
  // ---------------------------------------------------------------------------
  // Source notes (FEAT-020)
  // ---------------------------------------------------------------------------

  async function loadSourceNotes(): Promise<void> {
    try {
      sourceNotes.value = await invoke<Record<string, string>>('get_source_notes');
    } catch (e) {
      console.error('loadSourceNotes error:', e);
    }
  }

  async function saveSourceNote(sourceId: string, note: string): Promise<void> {
    await invoke('save_source_note', { sourceId, note });
    if (note) {
      sourceNotes.value[sourceId] = note;
    } else {
      delete sourceNotes.value[sourceId];
    }
  }
```

- [ ] **Step 4: 在 store 的 return 语句中导出新 state 和 actions**

找到 store 的 `return` 语句，在其中添加 `sourceNotes`、`loadSourceNotes`、`saveSourceNote`。

- [ ] **Step 5: 验证 TypeScript 编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit
```

Expected: 编译成功，无错误

- [ ] **Step 6: Commit**

```bash
git add src/stores/plugin-marketplace.ts
git commit -m "feat: add sourceNotes state and load/save actions to store"
```

---

## Task 4: 前端工具 - Markdown 渲染器

**Files:**
- Create: `src/utils/markdown.ts`

- [ ] **Step 1: 创建 `src/utils/markdown.ts`**

```typescript
/**
 * Lightweight Markdown renderer for source notes.
 * Supports: headings, bold, italic, inline code, code blocks,
 * unordered/ordered lists, links, blockquotes, horizontal rules.
 * No external dependencies.
 */

export function renderMarkdown(md: string): string {
  if (!md) return '';

  // Escape HTML to prevent XSS
  let html = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Code blocks (``` ... ```)
  html = html.replace(/```([\s\S]*?)```/g, (_match, code: string) => {
    return `<pre><code>${code.trim()}</code></pre>`;
  });

  // Headings (# H1 ~ #### H4)
  html = html.replace(/^#### (.+)$/gm, '<h4>$1</h4>');
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');

  // Horizontal rule (---)
  html = html.replace(/^---$/gm, '<hr>');

  // Blockquotes (> text)
  html = html.replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>');

  // Bold (**text**)
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');

  // Italic (*text*)
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');

  // Inline code (`code`)
  html = html.replace(/`(.+?)`/g, '<code>$1</code>');

  // Links ([text](url))
  html = html.replace(
    /\[(.+?)\]\((.+?)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>'
  );

  // Unordered lists (- item or * item)
  html = html.replace(/^[\-\*] (.+)$/gm, '<li>$1</li>');
  html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, '<ul>$1</ul>');

  // Ordered lists (1. item)
  html = html.replace(/^\d+\. (.+)$/gm, '<li>$1</li>');
  // Wrap consecutive <li> in <ol> (only if not already in <ul>)
  html = html.replace(/(?<!<\/ul>)((?:<li>.*<\/li>\n?)+)(?!<\/ul>)/g, '<ol>$1</ol>');

  // Paragraphs: wrap remaining lines in <p>
  // Split by double newlines for paragraph boundaries
  html = html
    .split(/\n\n+/)
    .map(block => {
      const trimmed = block.trim();
      if (!trimmed) return '';
      // Don't wrap blocks that already have HTML tags
      if (/^<(h[1-6]|ul|ol|li|pre|blockquote|hr)/.test(trimmed)) {
        return trimmed;
      }
      return `<p>${trimmed.replace(/\n/g, '<br>')}</p>`;
    })
    .join('\n');

  return html;
}
```

- [ ] **Step 2: 验证 TypeScript 编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit
```

Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src/utils/markdown.ts
git commit -m "feat: add lightweight markdown renderer for source notes"
```

---

## Task 5: 前端组件 - SourceNoteDialog

**Files:**
- Create: `src/components/plugins/SourceNoteDialog.vue`

- [ ] **Step 1: 创建 `src/components/plugins/SourceNoteDialog.vue`**

```vue
<script setup lang="ts">
import { ref, watch } from 'vue';
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

const noteContent = ref('');
const activeTab = ref<'edit' | 'preview'>('edit');

// Sync content when dialog opens
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      noteContent.value = props.initialNote;
      activeTab.value = 'edit';
    }
  }
);

function handleSave() {
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
      <div class="note-dialog">
        <div class="note-dialog-header">
          <h3>备注 - {{ sourceName }}</h3>
          <button class="note-dialog-close" @click="handleClose">
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
          />
          <div
            v-else
            class="note-preview markdown-body"
            v-html="renderMarkdown(noteContent)"
          />
        </div>

        <div class="note-dialog-footer">
          <button class="btn btn-outline btn-sm" @click="handleClose">取消</button>
          <button class="btn btn-primary btn-sm" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.note-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.note-dialog {
  background: var(--bg-primary, #1a1a1a);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  width: 560px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.note-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color, #333);
}

.note-dialog-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #e0e0e0);
}

.note-dialog-close {
  background: none;
  border: none;
  color: var(--text-secondary, #888);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.note-dialog-close:hover {
  color: var(--text-primary, #e0e0e0);
  background: var(--bg-hover, rgba(255, 255, 255, 0.1));
}

.note-dialog-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border-color, #333);
}

.note-tab {
  flex: 1;
  padding: 8px 16px;
  background: none;
  border: none;
  color: var(--text-secondary, #888);
  cursor: pointer;
  font-size: 13px;
  border-bottom: 2px solid transparent;
  transition: all 0.15s;
}

.note-tab:hover {
  color: var(--text-primary, #e0e0e0);
}

.note-tab.active {
  color: var(--accent-color, #4a9eff);
  border-bottom-color: var(--accent-color, #4a9eff);
}

.note-dialog-body {
  flex: 1;
  min-height: 300px;
  max-height: 50vh;
  overflow: auto;
}

.note-textarea {
  width: 100%;
  height: 100%;
  min-height: 300px;
  padding: 12px 16px;
  background: var(--bg-secondary, #111);
  color: var(--text-primary, #e0e0e0);
  border: none;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
}

.note-textarea::placeholder {
  color: var(--text-tertiary, #555);
}

.note-preview {
  padding: 12px 16px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary, #e0e0e0);
}

/* Markdown content styles */
.markdown-body :deep(h1) {
  font-size: 1.5em;
  font-weight: 700;
  margin: 0.5em 0;
  border-bottom: 1px solid var(--border-color, #333);
  padding-bottom: 0.3em;
}

.markdown-body :deep(h2) {
  font-size: 1.3em;
  font-weight: 600;
  margin: 0.5em 0;
}

.markdown-body :deep(h3) {
  font-size: 1.1em;
  font-weight: 600;
  margin: 0.5em 0;
}

.markdown-body :deep(h4) {
  font-size: 1em;
  font-weight: 600;
  margin: 0.5em 0;
}

.markdown-body :deep(p) {
  margin: 0.5em 0;
}

.markdown-body :deep(strong) {
  font-weight: 600;
}

.markdown-body :deep(em) {
  font-style: italic;
}

.markdown-body :deep(code) {
  background: var(--bg-code, rgba(255, 255, 255, 0.1));
  padding: 0.15em 0.4em;
  border-radius: 3px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.9em;
}

.markdown-body :deep(pre) {
  background: var(--bg-code, rgba(255, 255, 255, 0.1));
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 3px solid var(--accent-color, #4a9eff);
  padding-left: 12px;
  margin: 0.5em 0;
  color: var(--text-secondary, #aaa);
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.markdown-body :deep(li) {
  margin: 0.25em 0;
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-color, #333);
  margin: 1em 0;
}

.markdown-body :deep(a) {
  color: var(--accent-color, #4a9eff);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.note-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color, #333);
}
</style>
```

- [ ] **Step 2: 验证 TypeScript 编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit
```

Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src/components/plugins/SourceNoteDialog.vue
git commit -m "feat: add SourceNoteDialog component with edit/preview tabs"
```

---

## Task 6: 前端集成 - PluginsView 备注入口

**Files:**
- Modify: `src/views/PluginsView.vue`

- [ ] **Step 1: 添加 import 和 ref**

在 `PluginsView.vue` 的 `<script setup>` 中，找到其他组件的 import 语句，添加：

```typescript
import SourceNoteDialog from '@/components/plugins/SourceNoteDialog.vue';
```

在现有的 ref 声明区域（如 `switchingSourceId` 附近）添加：

```typescript
// FEAT-020: Source notes
const noteDialogVisible = ref(false);
const noteDialogSourceId = ref('');
const noteDialogSourceName = ref('');
```

- [ ] **Step 2: 添加辅助函数和方法**

在现有的 `handleSwitchType` 函数之后添加：

```typescript
// FEAT-020: Source notes
function getSourceNote(sourceId: string): string {
  return marketplaceStore.sourceNotes[sourceId] ?? '';
}

function handleOpenNote(source: PluginSource) {
  noteDialogSourceId.value = source.id;
  noteDialogSourceName.value = source.nameZh || source.name;
  noteDialogVisible.value = true;
}

async function handleSaveNote(sourceId: string, note: string) {
  try {
    await marketplaceStore.saveSourceNote(sourceId, note);
    showNotification?.({ type: 'success', message: '备注已保存' });
  } catch (e) {
    showNotification?.({ type: 'error', message: `备注保存失败: ${e}` });
  }
}
```

- [ ] **Step 3: 在源卡片 header 中添加备注图标按钮**

在模板中找到源卡片的 `source-card-header` 区域，在 `source-card-name` div 内部、repo type tag 之前，添加备注按钮：

```html
<div class="source-card-name">
  {{ source.name }}
  <button
    class="source-note-btn"
    :class="{ 'has-note': !!getSourceNote(source.id) }"
    :title="getSourceNote(source.id) ? '查看备注' : '添加备注'"
    @click.stop="handleOpenNote(source)"
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
      <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
    </svg>
  </button>
  <span v-if="source.repoType === 'res'" class="repo-type-tag res">res</span>
  <span v-else-if="source.repoType === 'market'" class="repo-type-tag market">market</span>
</div>
```

- [ ] **Step 4: 在模板末尾添加 SourceNoteDialog 组件**

在 `</div>` 结束标签（sources-container 结束）之前，添加：

```html
<!-- FEAT-020: Source note dialog -->
<SourceNoteDialog
  v-model:visible="noteDialogVisible"
  :source-id="noteDialogSourceId"
  :source-name="noteDialogSourceName"
  :initial-note="getSourceNote(noteDialogSourceId)"
  @save="handleSaveNote"
/>
```

- [ ] **Step 5: 添加备注按钮的样式**

在 `<style scoped>` 中找到 `.source-card-name` 相关样式，添加：

```css
.source-note-btn {
  background: none;
  border: none;
  color: var(--text-tertiary, #666);
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
  transition: all 0.15s;
}

.source-note-btn:hover {
  color: var(--text-primary, #e0e0e0);
  background: var(--bg-hover, rgba(255, 255, 255, 0.1));
}

.source-note-btn.has-note {
  color: var(--accent-color, #4a9eff);
}
```

- [ ] **Step 6: 验证 TypeScript 编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npx tsc --noEmit
```

Expected: 编译成功，无错误

- [ ] **Step 7: Commit**

```bash
git add src/views/PluginsView.vue
git commit -m "feat: integrate source note button and dialog into PluginsView"
```

---

## Task 7: 最终验证

- [ ] **Step 1: 完整构建验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager && npm run build
```

Expected: 构建成功，无错误

- [ ] **Step 2: 功能测试**

启动应用，导航到 Plugins > Sources 标签页：
1. 验证每个源卡片上都有备注图标按钮
2. 点击图标打开备注弹窗
3. 输入 Markdown 内容（标题、列表、粗体等）
4. 切换到预览 tab 验证渲染效果
5. 保存备注，验证图标变为高亮状态
6. 重新打开弹窗验证内容持久化
7. 保存空内容验证备注删除

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete source note feature (FEAT-020)"
```

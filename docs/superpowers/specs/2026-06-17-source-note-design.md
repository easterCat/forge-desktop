# 数据源备注功能设计

> Plugins > Sources 标签页，为每个数据源添加 Markdown 备注能力，方便快速了解仓库用途。

## 概述

在 Plugins 页面的 Sources 标签页中，每个数据源卡片（包括预置源和用户添加的源）支持添加、编辑、查看 Markdown 格式的备注。备注通过卡片标题栏的图标入口打开弹窗进行编辑，采用源码编辑 + 预览切换的交互方式。

## 适用范围

- 所有数据源：预置源（Anthropic Official、awesome-claude-code-plugins、cc-marketplace）和用户自定义源
- 主要用途：仓库简介/用途说明

---

## 数据模型与持久化

### 存储文件

`$FORGE_HOME/plugins/source_notes.json`

```json
{
  "version": "1",
  "notes": {
    "anthropics": "# Anthropic 官方\n35+ 官方维护插件，质量有保障"
  }
}
```

- key = `source.id`
- value = Markdown 源码字符串
- 无备注的 source 不出现在 notes map 中（保存空内容时删除该 key）

### Rust 后端

**services/plugin_marketplace.rs** 新增：

```rust
pub struct SourceNotesRegistry {
    pub version: String,
    pub notes: HashMap<String, String>,
}

pub fn source_notes_path() -> PathBuf {
    plugins_dir().join("source_notes.json")
}

pub fn read_source_notes() -> HashMap<String, String> {
    // 文件不存在 → 返回空 map
    // 反序列化失败 → 打印警告，返回空 map
}

pub fn write_source_notes(notes: &HashMap<String, String>) -> Result<(), String> {
    // 序列化为 pretty JSON，写入文件
}
```

**commands/plugin_marketplace.rs** 新增 2 个 Tauri command：

```rust
#[tauri::command]
pub fn get_source_notes() -> HashMap<String, String> {
    read_source_notes()
}

#[tauri::command]
pub fn save_source_note(source_id: String, note: String) -> Result<bool, String> {
    let mut notes = read_source_notes();
    if note.is_empty() {
        notes.remove(&source_id);
    } else {
        notes.insert(source_id, note);
    }
    write_source_notes(&notes)?;
    Ok(true)
}
```

需在 Tauri 的 `invoke_handler` 中注册这两个新 command。

### 前端 Store

**stores/plugin-marketplace.ts** 新增：

```ts
// state
sourceNotes: {} as Record<string, string>,

// actions
async loadSourceNotes() {
  this.sourceNotes = await invoke('get_source_notes');
},

async saveSourceNote(sourceId: string, note: string) {
  await invoke('save_source_note', { sourceId, note });
  if (note) {
    this.sourceNotes[sourceId] = note;
  } else {
    delete this.sourceNotes[sourceId];
  }
},
```

在 `loadSourceStatus()` 末尾追加 `await this.loadSourceNotes()`。

---

## UI 组件

### 源卡片备注入口

在 `PluginsView.vue` 的源卡片 `source-card-header` 内，源名称右侧、repo type tag 之前，插入备注图标按钮：

```html
<button
  class="source-note-btn"
  :class="{ 'has-note': !!getSourceNote(source.id) }"
  :title="getSourceNote(source.id) ? '查看备注' : '添加备注'"
  @click="handleOpenNote(source)"
>
  <!-- 便签/铅笔 SVG 图标 -->
</button>
```

样式：
- 有备注：图标使用主题色高亮
- 无备注：图标灰色（`text-gray-400`）
- hover：显示 tooltip（"查看备注" 或 "添加备注"）

### SourceNoteDialog.vue

路径：`src/components/plugins/SourceNoteDialog.vue`

**Props：**

| Prop | 类型 | 说明 |
|------|------|------|
| `visible` | `boolean` | 弹窗显示状态 |
| `sourceId` | `string` | 数据源 ID |
| `sourceName` | `string` | 数据源名称（弹窗标题） |
| `initialNote` | `string` | 当前备注内容 |

**Emits：**

| Event | 参数 | 说明 |
|-------|------|------|
| `update:visible` | `boolean` | 关闭弹窗 |
| `save` | `sourceId: string, note: string` | 保存备注 |

**弹窗结构：**

```
┌──────────────────────────────────────┐
│  备注 - Anthropic Official     [✕]  │
├──────────────────────────────────────┤
│  [编辑]  [预览]                       │
├──────────────────────────────────────┤
│                                      │
│  (编辑模式)                           │
│  ┌────────────────────────────────┐  │
│  │ # Anthropic 官方仓库            │  │
│  │                                │  │
│  │ 包含 35+ 官方维护的插件...       │  │
│  │                                │  │
│  └────────────────────────────────┘  │
│  textarea: monospace 字体, 全宽       │
│  placeholder: "输入 Markdown 备注..." │
│                                      │
│  (预览模式)                           │
│  渲染后的 Markdown HTML               │
│                                      │
├──────────────────────────────────────┤
│              [取消]  [保存]           │
└──────────────────────────────────────┘
```

**交互逻辑：**

1. 打开弹窗时，`noteContent` 初始化为 `initialNote`
2. 切换 tab 不丢弃编辑内容
3. 点击「保存」→ emit `save(sourceId, noteContent)` → 关闭弹窗
4. 点击「取消」或关闭 → 直接关闭弹窗，不保存
5. 保存空内容等同于删除备注

### PluginsView.vue 集成

新增 ref：

```ts
const noteDialogVisible = ref(false);
const noteDialogSourceId = ref('');
const noteDialogSourceName = ref('');
```

新增方法：

```ts
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

模板中引入组件：

```html
<SourceNoteDialog
  v-model:visible="noteDialogVisible"
  :source-id="noteDialogSourceId"
  :source-name="noteDialogSourceName"
  :initial-note="getSourceNote(noteDialogSourceId)"
  @save="handleSaveNote"
/>
```

---

## Markdown 渲染

不引入第三方库，用正则转换支持以下语法：

| 语法 | 转换目标 |
|------|----------|
| `# H1` ~ `#### H4` | `<h1>` ~ `<h4>` |
| `**粗体**` | `<strong>` |
| `*斜体*` | `<em>` |
| `` `行内代码` `` | `<code>` |
| ` ``` 代码块 ``` ` | `<pre><code>` |
| `- ` / `* ` 无序列表 | `<ul><li>` |
| `1. ` 有序列表 | `<ol><li>` |
| `[text](url)` | `<a>` |
| `> ` 引用 | `<blockquote>` |
| `---` | `<hr>` |
| 空行 | `<br>` |

实现为 `renderMarkdown(md: string): string` 工具函数，放在 `src/utils/markdown.ts`。

预览区域使用 `v-html` 渲染，配合 scoped 样式控制排版。

---

## 错误处理

| 场景 | 处理方式 |
|------|----------|
| `source_notes.json` 文件不存在 | `read_source_notes()` 返回空 map |
| 文件损坏/反序列化失败 | 打印 console.warn，返回空 map |
| 写入失败（权限/磁盘满） | Tauri command 返回错误，前端 toast 提示「备注保存失败」 |
| 并发写入 | 无锁，后写覆盖先写（沿用项目现有模式，备注场景并发概率极低） |
| 保存空内容 | 删除该 source 的备注条目，图标恢复灰色 |

---

## 不做的事（YAGNI）

- 不做备注的导入/导出
- 不做备注的版本历史
- 不做全局备注搜索
- 不做预置源的默认备注模板

---

## 改动文件清单

| 文件 | 改动类型 | 说明 |
|------|----------|------|
| `src-tauri/src/services/plugin_marketplace.rs` | 修改 | 新增 `SourceNotesRegistry`、`source_notes_path`、`read_source_notes`、`write_source_notes` |
| `src-tauri/src/commands/plugin_marketplace.rs` | 修改 | 新增 `get_source_notes`、`save_source_note` command |
| `src-tauri/src/lib.rs` | 修改 | 在 `invoke_handler` 中注册 2 个新 command（第 155 行附近） |
| `src/stores/plugin-marketplace.ts` | 修改 | 新增 `sourceNotes` state + `loadSourceNotes` / `saveSourceNote` action |
| `src/views/PluginsView.vue` | 修改 | 源卡片增加备注图标按钮 + 引入 SourceNoteDialog |
| `src/components/plugins/SourceNoteDialog.vue` | 新增 | 备注编辑弹窗组件 |
| `src/utils/markdown.ts` | 新增 | 轻量 Markdown 渲染函数 |

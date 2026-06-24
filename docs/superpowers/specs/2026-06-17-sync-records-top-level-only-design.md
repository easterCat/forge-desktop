# sync_records.json 只记录顶层条目

## 问题

`sync_records.json` 的 `synced_files` 字段记录了插件目录下每一个文件的完整相对路径。ECC 插件一个就记录了 3162 个文件，导致 JSON 文件膨胀到 322KB（6776 行）。随着更多插件安装，体积会持续增长。

## 现状分析

- `synced_files` 的唯一用途是在 unsync 时作为 fallback：当 `fs::remove_dir_all` 失败时，逐文件删除
- `unsync_plugin_from_cli_tool` 的主要删除策略已经是 `fs::remove_dir_all(&target_path)` — 直接删整个同步目录
- target_path 是 Forge 创建的独立目录（`~/.claude/plugins/cache/<source>/<plugin>/<version>/`），整目录删除是安全的

## 方案

修改 `copy_dir_inner` 函数，只记录顶层条目（第一层目录名和文件名），而非递归记录所有文件路径。

### 修改文件

`src-tauri/src/commands/plugin_sync.rs`

### 变更 1：`copy_dir_inner` 文件收集逻辑

当前逻辑（行 103-141）：递归遍历，每个文件都 push 完整相对路径到 `copied_files`。

新逻辑：
- 遇到目录时：仍然递归复制，但只 push 顶层目录名（带 `/` 后缀标识为目录）
- 遇到文件时：仍然复制，但只 push 文件名（不含子目录前缀）

结果向量中的条目格式：
- 目录：`skills/`、`commands/`、`docs/`
- 文件：`plugin.json`、`README.md`、`.gitignore`

### 变更 2：`remove_synced_files` 兼容目录条目

当前逻辑（行 146-157）：对每个 synced_file 调用 `fs::remove_file`。

新逻辑：对每个条目，用 `path.is_dir()` 判断：
- 目录：调用 `fs::remove_dir_all` 删除整个子目录
- 文件：保持 `fs::remove_file` 不变

这比依赖 `/` 后缀约定更可靠，因为 target_path 是 Forge 创建的独立目录，存储的路径在磁盘上实际存在，`is_dir()` 判断准确。

### 不需要改动的部分

- `PluginSyncRecord` 结构体 — `synced_files: Vec<String>` 类型不变
- `save_sync_records` / `load_sync_records` — 无变化
- `sync_plugin_to_cli_tool` — 调用方式不变
- 前端代码 — 完全不受影响
- 测试 — 现有测试不依赖 synced_files 的具体条目内容

## 预期效果

| 插件 | 当前文件数 | 优化后条目数 |
|------|-----------|-------------|
| ECC | 3162 | ~20-30 |
| agent-skills | 118 | ~15-20 |
| superpowers | 147 | ~20-25 |
| **JSON 体积** | **322KB** | **~5-10KB** |

## 风险

- **低风险**：主删除路径（`remove_dir_all`）不变，fallback 行为等价（删目录 = 递归删其下所有文件）
- **兼容性**：已有的 sync_records.json 中的旧格式（完整路径列表）仍然能被新的 `remove_synced_files` 正常处理 — 文件路径和目录路径都能被删除

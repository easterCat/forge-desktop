# Claude Plugins CLI

Claude Code 插件管理命令行工具。

## 安装

CLI 已集成到 `package.json` 中，可通过以下方式调用：

```bash
# 使用 npm scripts
npm run plugins -- <command>

# 或直接调用
node scripts/plugins/index.mjs <command>

# 或通过 bin（需要 npm link）
plugins <command>
```

## 命令

### install

从 GitHub 仓库安装插件。

```bash
npm run plugins -- install <repo_url> [options]

# 示例
npm run plugins -- install https://github.com/anthropics/claude-plugins-official
npm run plugins -- install https://github.com/anthropics/claude-plugins-official --only plugin1,plugin2
npm run plugins -- install https://github.com/anthropics/claude-plugins-official --branch develop
```

**选项：**
- `--source <name>` - 自定义源名称（默认从 URL 推断）
- `--branch <name>` - Git 分支（默认 main）
- `--only <names>` - 仅安装指定插件（逗号分隔）
- `--dry-run` - 仅显示将要安装的插件，不实际安装

### uninstall

卸载插件（软删除）。

```bash
npm run plugins -- uninstall <plugin_name> [options]

# 示例
npm run plugins -- uninstall my-plugin
npm run plugins -- uninstall my-plugin --source anthropics
npm run plugins -- uninstall my-plugin --force  # 同时删除本地文件
npm run plugins -- uninstall my-plugin --purge  # 完全从 marketplace 中移除
```

**选项：**
- `--source <name>` - 指定源（避免同名插件歧义）
- `--force` - 同时删除本地插件文件
- `--purge` - 完全从 marketplace 中移除（包括 removed 列表）

### list

列出已安装的插件。

```bash
npm run plugins -- list [options]

# 示例
npm run plugins -- list
npm run plugins -- list --removed    # 查看已卸载插件
npm run plugins -- list --sources    # 查看所有源
npm run plugins -- list --json       # JSON 格式输出
```

**选项：**
- `--removed` - 显示已卸载的插件
- `--sources` - 显示所有配置的源
- `--json` - JSON 格式输出

## marketplace.json 格式

```json
{
  "version": "1.0.0",
  "lastSyncAt": "2026-06-14T14:30:00Z",
  "sources": {
    "anthropics": {
      "repoUrl": "https://github.com/anthropics/claude-plugins-official",
      "type": "official",
      "external": false,
      "lastSyncAt": "2026-06-14T14:30:00Z",
      "pluginCount": 35
    }
  },
  "plugins": {
    "anthropics": [
      {
        "name": "example-plugin",
        "description": "Plugin description",
        "version": "1.0.0",
        "author": "anthropics",
        "repoUrl": "https://github.com/anthropics/claude-plugins-official/tree/main/plugins/example-plugin",
        "installedPath": "plugins/anthropics/example-plugin",
        "external": false
      }
    ]
  },
  "removed": [
    {
      "name": "old-plugin",
      "source": "anthropics",
      "removedAt": "2026-06-14T14:00:00Z",
      "reason": "manual uninstall"
    }
  ]
}
```

## 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 参数错误或命令执行失败 |
| 2 | 网络错误（克隆失败） |
| 3 | 插件未找到 |

## 故障排除

### Git 未安装

确保系统已安装 git：

```bash
git --version
```

### 克隆超时

对于大型仓库，可能需要更长的超时时间。可以通过环境变量调整：

```bash
GIT_CLONE_TIMEOUT=600000 npm run plugins -- install <repo_url>
```

### 权限错误

确保对项目目录有读写权限：

```bash
chmod -R u+rw .
```

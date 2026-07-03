# Forge Desktop

Tauri 2.0 + Vue 3 + TypeScript + Rust 构建的 AI 工具配置管理桌面应用，SQLite 存储。

## 命令

```bash
npm install          # 安装依赖
npm run dev          # 启动开发（前端 + Tauri）
npm run dev:web      # 仅前端
npm run tauri build  # 构建生产版本
npm test             # 运行测试
npm run lint         # 代码检查
```

## 结构

```
src/                # Vue 3 前端
├── views/          # 页面（12 个）
├── stores/         # Pinia（统一 Store 架构）
├── components/     # UI 组件
├── composables/    # 组合式函数
├── services/       # 服务层（workspace.yaml 生成等）
├── types/          # 类型定义
└── utils/          # 工具函数

src-tauri/          # Rust 后端
├── src/commands/   # Tauri 命令
├── src/services/   # 服务
└── src/db/         # SQLite
```

## AllAgents 集成架构

项目集成了 allagents CLI 工具，用于统一管理 skills/plugins/agents/rules/MCPs 的注册、生命周期和配置同步。

### 核心组件

```
src/types/unified-plugin.ts        # 统一类型系统
src/services/allagents-config.ts   # workspace.yaml 生成器
src/composables/useAllAgents.ts    # 前端调用封装
src/composables/useErrorHandler.ts # 统一错误处理

src/stores/unified-plugin.ts       # 统一 Plugin Store
src/stores/unified-skill.ts        # 统一 Skill Store
src/stores/unified-agent.ts        # Agent 适配器
src/stores/unified-rule.ts         # Rule 适配器
src/stores/unified-mcp.ts          # MCP 适配器
src/stores/unified-plugin-adapter.ts # Plugin 适配器

src-tauri/src/services/allagents_service.rs  # AllAgents CLI 执行器
src-tauri/src/commands/allagents_commands.rs  # 17 个 Tauri IPC 命令
```

### Tauri 命令 (17 个)

```typescript
// 工作区管理
allagents_init              // 初始化工作区
allagents_update            // 同步所有插件（含事件发射）
allagents_status            // 获取工作区状态

// 插件管理
allagents_plugin_install    // 安装插件
allagents_plugin_uninstall  // 卸载插件
allagents_plugin_list       // 列出插件

// 技能管理
allagents_skill_list        // 列出技能
allagents_skill_add         // 添加技能
allagents_skill_remove      // 移除技能

// MCP 管理
allagents_mcp_add           // 添加 MCP 服务器
allagents_mcp_remove        // 移除 MCP 服务器
allagents_mcp_list          // 列出 MCP 服务器
allagents_mcp_update        // 同步 MCP 配置

// Marketplace 管理
allagents_marketplace_add   // 添加 marketplace 源
allagents_marketplace_remove // 移除 marketplace 源
allagents_marketplace_list  // 列出 marketplace 源

// 配置生成
allagents_generate_config   // 生成 workspace.yaml
```

### 事件系统

```typescript
// 监听同步事件
import { listen } from '@tauri-apps/api/event';

listen('allagents:event', (event) => {
  const { type, ...data } = event.payload;
  // type: 'sync-progress' | 'sync-complete' | 'sync-error' | 'config-changed'
});
```

### 客户端支持

支持 23 个 AI 客户端的同步：
- **通用客户端**: Copilot, Codex, OpenCode, Gemini, Amp Code, VS Code, Replit, Kimi
- **专属客户端**: Codex, Cursor, Factory, Windsurf, Cline, Continue, Roo, Kilo, Trae, Augment, Zencoder, Junie, OpenHands, Kiro

## 规则

- 所有回复都要使用中文
- 新代码使用统一类型系统 (`UnifiedPlugin`, `UnifiedMCP`)
- Store 迁移采用适配器模式，保留现有 API 兼容
- 错误处理使用 `useErrorHandler` 组合式函数

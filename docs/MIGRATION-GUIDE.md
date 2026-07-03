# 迁移指南：旧 Store → 新统一架构

> 本指南帮助开发者从旧的分散 Store 迁移到新的统一 Store 架构。

---

## 1. 概述

### 旧架构（已废弃）

| Store | 文件 | 功能 |
|-------|------|------|
| `useSkillStore` | `stores/skill.ts` | 基础 CRUD |
| `useSkillMarketplaceStore` | `stores/skill-marketplace.ts` | 市场浏览/安装 |
| `useSkillImportStore` | `stores/skill-import.ts` | 本地导入/仓库管理 |
| `useAnthropicSkillsStore` | `stores/anthropic-skills.ts` | Anthropic 官方/远程源 |
| `useSkillsShStore` | `stores/skills-sh.ts` | skills.sh 排行榜 |
| `usePluginMarketplaceStore` | `stores/plugin-marketplace.ts` | 插件市场 |
| `useAgentStore` | `stores/agent.ts` | 代理管理 |
| `useRuleStore` | `stores/rule.ts` | 规则管理 |
| `useMcpStore` | `stores/mcp.ts` | MCP 管理 |
| `useMcpMarketplaceStore` | `stores/mcp-marketplace.ts` | MCP 市场 |

### 新架构（推荐）

| Store | 文件 | 功能 |
|-------|------|------|
| `useUnifiedPluginStore` | `stores/unified-plugin.ts` | 统一 Plugin Store |
| `useUnifiedSkillStore` | `stores/unified-skill.ts` | 统一 Skill Store |
| `useUnifiedAgentStore` | `stores/unified-agent.ts` | Agent 适配器 |
| `useUnifiedRuleStore` | `stores/unified-rule.ts` | Rule 适配器 |
| `useUnifiedMcpStore` | `stores/unified-mcp.ts` | MCP 适配器 |
| `useUnifiedPluginAdapterStore` | `stores/unified-plugin-adapter.ts` | Plugin 适配器 |

---

## 2. 迁移步骤

### 2.1 Skill Store 迁移

**旧代码：**
```typescript
import { useSkillStore } from '@/stores/skill';
import { useSkillMarketplaceStore } from '@/stores/skill-marketplace';
import { useAnthropicSkillsStore } from '@/stores/anthropic-skills';

const skillStore = useSkillStore();
const marketplaceStore = useSkillMarketplaceStore();
const anthropicStore = useAnthropicSkillsStore();

// 加载技能
await skillStore.fetchSkills();
await marketplaceStore.fetchSkills();
await anthropicStore.fetchList();

// 安装技能
await marketplaceStore.installSkill(skill, projectPath);
await anthropicStore.install(skillId, targetDir);
```

**新代码：**
```typescript
import { useUnifiedSkillStore } from '@/stores/unified-skill';

const skillStore = useUnifiedSkillStore();

// 加载技能（自动从 allagents 获取）
await skillStore.fetchSkills();

// 安装技能（统一接口）
await skillStore.installSkill(skill, targetDir);

// 检查是否已安装
const installed = skillStore.isSkillInstalled('skill-name');

// 获取已安装技能
const installedSkills = skillStore.installedSkills;
```

### 2.2 Agent Store 迁移

**旧代码：**
```typescript
import { useAgentStore } from '@/stores/agent';

const agentStore = useAgentStore();

// 加载代理
await agentStore.fetchAgents();

// 安装到目标
await agentStore.installAgent(agentId, 'claude-code');
```

**新代码：**
```typescript
import { useUnifiedAgentStore } from '@/stores/unified-agent';

const agentStore = useUnifiedAgentStore();

// 加载代理
await agentStore.fetchAgents();

// 安装到目标（支持 23 个客户端）
await agentStore.installAgent(agentId, 'claude');

// 通过 allagents 同步所有代理
await agentStore.syncAllAgents();
```

### 2.3 Rule Store 迁移

**旧代码：**
```typescript
import { useRuleStore } from '@/stores/rule';

const ruleStore = useRuleStore();

// 加载规则
await ruleStore.fetchRules();

// 创建规则
await ruleStore.createRule(softwareId, name, type, filePath, content);
```

**新代码：**
```typescript
import { useUnifiedRuleStore } from '@/stores/unified-rule';

const ruleStore = useUnifiedRuleStore();

// 加载规则
await ruleStore.fetchRules();

// 创建规则
await ruleStore.createRule(softwareId, name, type, filePath, content);

// 通过 workspace.files 同步规则
await ruleStore.syncRules();
```

### 2.4 MCP Store 迁移

**旧代码：**
```typescript
import { useMcpStore } from '@/stores/mcp';
import { useMcpMarketplaceStore } from '@/stores/mcp-marketplace';

const mcpStore = useMcpStore();
const mcpMarketplaceStore = useMcpMarketplaceStore();

// 加载 MCP 服务器
await mcpStore.fetchServices();

// 添加 MCP 服务器
await mcpStore.addService(serviceData);
```

**新代码：**
```typescript
import { useUnifiedMcpStore } from '@/stores/unified-mcp';

const mcpStore = useUnifiedMcpStore();

// 加载 MCP 服务器（从 allagents）
await mcpStore.fetchServers();

// 添加 MCP 服务器
await mcpStore.addServer(serverData);

// 启动健康检查
mcpStore.startHealthCheck(60000); // 每 60 秒检查一次

// 获取健康状态统计
const stats = mcpStore.healthStats;
// { healthy: 5, unhealthy: 1, unknown: 2, total: 8 }

// 记录审计日志
mcpStore.logAudit('server-name', 'config-change', '更新了 URL');

// 获取审计日志
const auditLog = mcpStore.getAuditLog('server-name');
```

### 2.5 Plugin Store 迁移

**旧代码：**
```typescript
import { usePluginMarketplaceStore } from '@/stores/plugin-marketplace';

const pluginStore = usePluginMarketplaceStore();

// 加载来源
await pluginStore.fetchSources();

// 安装插件
await pluginStore.installPlugin(plugin, projectPath);
```

**新代码：**
```typescript
import { useUnifiedPluginAdapterStore } from '@/stores/unified-plugin-adapter';

const pluginStore = useUnifiedPluginAdapterStore();

// 加载来源
await pluginStore.fetchSources();

// 安装插件
await pluginStore.installPlugin(plugin, { scope: 'project' });

// 同步所有插件
await pluginStore.syncAll();
```

---

## 3. 统一类型

### UnifiedPlugin

```typescript
interface UnifiedPlugin {
  id: string;
  name: string;
  description?: string;
  version?: string;
  source: PluginSource;
  scope: 'user' | 'project';
  type: 'skill' | 'agent' | 'rule' | 'mcp' | 'hook' | 'command';
  tags: string[];
  categories: string[];
  installed: boolean;
  enabled: boolean;
  installedAt?: string;
  syncTargets: SyncTarget[];
  syncStatus: 'synced' | 'pending' | 'error' | 'conflict';
  targetClients: string[];
  allagentsSpec?: string;
}
```

### UnifiedMCP

```typescript
interface UnifiedMCP {
  name: string;
  transport: 'http' | 'stdio';
  url?: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  headers?: Record<string, string>;
  clients?: string[];
  groupIds: string[];
  tags: string[];
  healthStatus: 'healthy' | 'unhealthy' | 'unknown';
  lastHealthCheck?: string;
  auditLog: AuditEntry[];
}
```

---

## 4. 错误处理

使用统一的错误处理组合式函数：

```typescript
import { useErrorHandler } from '@/composables/useErrorHandler';

const { currentError, clearError, withRetry, wrapAsync } = useErrorHandler();

// 重试操作
const data = await withRetry(() => someAsyncOperation());

// 包装异步操作
const { data, error } = await wrapAsync(() => someAsyncOperation());
if (error) {
  console.error(error.userMessage); // 用户友好的错误消息
}

// 清除错误
clearError();
```

---

## 5. 客户端选择

使用新的客户端选择组件：

```vue
<script setup>
import ClientSelector from '@/components/plugins/ClientSelector.vue';
import { ref } from 'vue';

const selectedClients = ref(['claude', 'cursor']);
</script>

<template>
  <ClientSelector
    v-model="selectedClients"
    :disabled="false"
  />
</template>
```

---

## 6. 事件监听

监听 allagents 同步事件：

```typescript
import { listen } from '@tauri-apps/api/event';

// 监听同步进度
const unlisten = await listen('allagents:event', (event) => {
  const { type, ...data } = event.payload;

  switch (type) {
    case 'sync-progress':
      console.log(`同步进度: ${data.message}`);
      break;
    case 'sync-complete':
      console.log(`同步完成: ${data.synced_count} 个文件`);
      break;
    case 'sync-error':
      console.error(`同步错误: ${data.error}`);
      break;
  }
});

// 停止监听
unlisten();
```

---

## 7. 注意事项

1. **渐进式迁移**：旧 Store 仍可使用，新 Store 通过适配器模式保持向后兼容
2. **类型统一**：所有新代码应使用 `UnifiedPlugin` 和 `UnifiedMCP` 类型
3. **错误处理**：使用 `useErrorHandler` 替代分散的 try/catch
4. **客户端支持**：新架构支持 23 个客户端，旧架构仅支持 3 个
5. **同步机制**：新架构通过 allagents CLI 统一同步，旧架构各自实现

# AllAgents 集成 — 开发计划

> **项目**: Forge Desktop | **版本**: v1.0 | **日期**: 2026-06-26
> **目标**: 集成 allagents CLI 工具，统一管理 skills/plugins/agents/rules/MCPs 的注册、生命周期、配置同步

---

## 一、项目总览

### 1.1 现状问题

当前项目管理 5 种内容类型，共涉及：

| 模块 | Stores | Rust Commands | Rust Services | 问题 |
|------|--------|--------------|---------------|------|
| **Plugins** | 2 个 (1220+ 行) | 4 个 | 3 个 | 硬编码同步目标，仅支持 3 个 CLI |
| **Skills** | 5 个 (碎片化) | 6 个 | 1 个 | 4 套不同 Skill 类型接口 |
| **Agents** | 1 个 | 1 个 | 0 个 | 无 marketplace，部门列表硬编码 |
| **Rules** | 1 个 | 1 个 | 0 个 | View 仍用 mock 数据 |
| **MCP** | 2 个 | 4 个 | 2 个 | 功能完整但同步逻辑分散 |

**核心痛点**: 类型碎片化（4 套 Skill 接口）、同步逻辑重复（5 种不同实现）、硬编码配置、无统一依赖管理。

### 1.2 目标架构

```
Forge Desktop (Tauri 2.0 + Vue 3)
    │
    ├── UI Layer (Pinia Stores)
    │   └── UnifiedPluginStore (替代现有 11 个 Store)
    │
    ├── Service Layer (Rust)
    │   ├── AllAgentsService (新增: CLI 执行器 + workspace.yaml 生成)
    │   ├── HealthService (保留: MCP 健康检查)
    │   ├── AuditService (保留: 操作审计)
    │   └── RulesService (保留: Rules 独立管理)
    │
    └── External
        └── allagents CLI (npm 包, 子进程调用)
            ├── 23 个客户端同步
            ├── Marketplace 生态
            └── 声明式 workspace.yaml
```

---

## 二、里程碑与任务分解

### M1: 调研准备 (第 1 周) ✅ 已完成

> **目标**: 验证 allagents 在本项目环境中的可行性，完成技术调研

| # | 任务 | 负责 | 人天 | 交付物 | 状态 |
|---|------|------|------|--------|------|
| 1.1 | allagents CLI 功能验证测试 | 后端 | 1.5 | 测试报告 | ✅ 完成 |
| 1.2 | Windows 平台兼容性测试 (Node/Bun fallback) | 后端 | 1 | 兼容性报告 | ✅ 完成 |
| 1.3 | 现有模块代码审计 (5 模块) | 全栈 | 1 | 审计报告 | ✅ 完成 |
| 1.4 | 迁移方案细化 + 团队评审 | 架构师 | 0.5 | 评审记录 | ✅ 完成 |

**验收标准**:
- [x] allagents 在 Windows 11 上正常运行 (`allagents --version`)
- [x] `allagents init` / `allagents update` 端到端通过
- [x] 现有 5 个模块的功能点清单完成
- [x] 迁移方案经团队评审通过

**测试用例**:
```bash
# 验证 allagents 安装
npm install -g allagents
allagents --version

# 验证 workspace 初始化
mkdir test-workspace && cd test-workspace
allagents init . --from EntityProcess/allagents
cat workspace.yaml

# 验证插件安装
allagents plugin install code-review@claude-plugins-official --json
allagents skill list --json

# 验证 MCP 管理
allagents mcp add test-server "https://example.com/mcp" --transport http --json
allagents mcp list --json

# 验证同步
allagents update --dry-run --json
```

---

### M2: 核心框架集成 (第 2-3 周) ✅ 已完成

> **目标**: 建立 AllAgentsService 核心框架，实现 workspace.yaml 自动生成和 CLI 调用

| # | 任务 | 负责 | 人天 | 交付物 | 状态 |
|---|------|------|------|--------|------|
| 2.1 | AllAgentsService Rust 模块开发 | 后端 | 3 | `allagents_service.rs` | ✅ 完成 |
| 2.2 | workspace.yaml 生成器 (TypeScript) | 前端 | 2 | `allagents-config.ts` | ✅ 完成 |
| 2.3 | Tauri Commands 封装 (17 个命令) | 后端 | 2 | `commands/allagents_commands.rs` | ✅ 完成 |
| 2.4 | 统一类型系统设计 | 前端 | 2 | `types/unified-plugin.ts` | ✅ 完成 |
| 2.5 | 事件流系统 (进度/错误) | 全栈 | 1 | 事件定义 + 监听 | ✅ 完成 |

**验收标准**:
- [x] `allagents init` → `allagents update` 流程端到端通过
- [x] workspace.yaml 自动生成正确，通过 allagents 验证
- [x] 所有 Tauri Commands 可调用并返回 JSON 结果
- [x] 同步进度事件可被前端监听

**核心文件**:
```
src-tauri/src/
├── services/allagents_service.rs    # 核心服务 (新增)
├── commands/allagents.rs            # Tauri 命令 (新增)
└── lib.rs                           # 注册新命令

src/
├── services/allagents-config.ts     # workspace.yaml 生成器 (新增)
├── types/unified-plugin.ts          # 统一类型 (新增)
└── composables/useAllAgents.ts      # 前端调用封装 (新增)
```

---

### M3: 插件逐个迁移适配 (第 4-6 周)

> **目标**: 将 5 个模块逐步迁移到 AllAgents 架构

#### M3.1: Skills 模块迁移 (第 4 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.1.1 | 合并 5 个 Skill Store → 1 个 UnifiedSkillStore | 前端 | 2 | ✅ 完成 |
| 3.1.2 | 统一 4 套 Skill 类型接口 | 前端 | 1 | ✅ 完成 |
| 3.1.3 | 迁移 Rust Skill Commands | 后端 | 1 | ✅ 完成 |

**验收标准**:
- [x] 5 个 Store 合并为 1 个 UnifiedSkillStore
- [x] 所有现有 Skill 功能正常
- [x] 无功能回退

#### M3.2: Plugins 模块迁移 (第 5 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.2.1 | 重构 PluginStore，委托安装/同步到 AllAgentsService | 全栈 | 2 | ✅ 完成 |
| 3.2.2 | 保留 marketplace 来源管理逻辑 | 全栈 | 1 | ✅ 完成 |
| 3.2.3 | 移除 plugin_sync.rs 硬编码同步 | 后端 | 1 | ✅ 完成 |

**验收标准**:
- [x] 插件安装/卸载通过 AllAgentsService 执行
- [x] Marketplace 来源管理保留
- [x] 同步目标扩展到 23 个客户端

#### M3.3: Agents 模块迁移 (第 5 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.3.1 | AgentStore 重构，同步委托 AllAgentsService | 全栈 | 1 | ✅ 完成 |
| 3.3.2 | 扩展客户端支持 (3 → 23) | 后端 | 1 | ✅ 完成 |

**验收标准**:
- [x] Agent 安装/卸载通过 AllAgentsService
- [x] 支持选择 23 个客户端中的任意子集

#### M3.4: MCP 模块迁移 (第 5-6 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.4.1 | MCPStore 重构，同步层委托 AllAgentsService | 全栈 | 2 | ✅ 完成 |
| 3.4.2 | 保留 health check / audit log / grouping | 后端 | 1 | ✅ 完成 |

**验收标准**:
- [x] MCP 添加/移除通过 AllAgentsService
- [x] 健康检查、审计日志、分组功能保留
- [x] HTTP-to-stdio 代理使用 allagents 内置能力

#### M3.5: Rules 模块增强 (第 6 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.5.1 | RulesView 补全 (替换 mock 数据) | 前端 | 1 | ✅ 完成 |
| 3.5.2 | Rules 通过 workspace.files 机制同步 | 全栈 | 1 | ✅ 完成 |

**验收标准**:
- [x] RulesView 使用真实数据
- [x] Rules 文件通过 workspace.yaml 同步到客户端

#### M3.6: UI 适配 (第 6 周) ✅ 已完成

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 3.6.1 | SkillsView 合并 (统一入口) | 前端 | 2 | ✅ 完成 |
| 3.6.2 | 客户端选择 UI (23 个客户端) | 前端 | 1 | ✅ 完成 |

**验收标准**:
- [x] 统一的插件管理界面
- [x] 客户端选择器支持 23 个客户端

---

### M4: 集成测试与优化 (第 7-8.5 周) ✅ 已完成

> **目标**: 端到端验证，性能优化，Bug 修复

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 4.1 | 端到端集成测试 | 测试 | 3 | ✅ 完成 |
| 4.2 | 性能测试 (100 插件同步 < 30s) | 后端 | 1 | ✅ 完成 |
| 4.3 | 错误处理和边界情况 | 全栈 | 2 | ✅ 完成 |
| 4.4 | 向后兼容性验证 | 全栈 | 1 | ✅ 完成 |
| 4.5 | Bug 修复 | 全栈 | 2 | ✅ 完成 |

**验收标准**:
- [x] 所有测试用例通过 (28 个测试)
- [x] 同步 100 个插件耗时 < 30 秒
- [x] 无 P0/P1 级别 Bug
- [x] 旧接口作为 fallback 可用

---

### M5: 文档编写与发布 (第 8.5-9 周) ✅ 已完成

> **目标**: 完善文档，准备发布

| # | 任务 | 负责 | 人天 | 状态 |
|---|------|------|------|------|
| 5.1 | 开发文档更新 | 全栈 | 1 | ✅ 完成 |
| 5.2 | 用户使用指南 | 前端 | 1 | ✅ 完成 |
| 5.3 | 迁移指南 (旧 → 新) | 架构师 | 0.5 | ✅ 完成 |
| 5.4 | 发布准备 | 全栈 | 0.5 | ✅ 完成 |

**验收标准**:
- [x] CLAUDE.md 已更新
- [x] 用户可按指南完成插件安装和同步
- [x] Release notes 已准备

---

## 三、风险清单与缓解措施

| # | 风险 | 概率 | 影响 | 缓解措施 |
|---|------|------|------|---------|
| R1 | allagents Windows 兼容性问题 | 中 | 高 | M1 阶段提前验证；Node.js fallback |
| R2 | CLI 调用性能瓶颈 | 中 | 中 | 批量操作；缓存机制；异步执行 |
| R3 | 迁移期间数据不一致 | 中 | 高 | 渐进式迁移；feature flag；保留旧接口 |
| R4 | allagents 上游变更 | 低 | 中 | 锁定版本；监控 release notes |
| R5 | 现有功能回退 | 低 | 高 | 完善回归测试；功能开关机制 |

---

## 四、资源需求

### 人力

| 角色 | 人数 | 投入 | 职责 |
|------|------|------|------|
| 架构师 | 1 | 20% | 方案设计、评审、技术决策 |
| 后端开发 (Rust) | 1-2 | 80% | AllAgentsService、Tauri Commands |
| 前端开发 (Vue) | 1-2 | 80% | Store 迁移、UI 适配、类型系统 |
| 测试 | 1 | 40% | 集成测试、兼容性测试 |

### 环境

- Windows 11 + Node.js >= 18 + npm
- allagents >= 1.12.0 (`npm install -g allagents`)
- 测试客户端: Claude Code、Cursor、Copilot

### 依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| allagents | >= 1.12.0 | CLI 工具 |
| js-yaml | ^4.1.0 | YAML 解析 |
| zod | ^3.22.0 | workspace.yaml 校验 |
| serde_yaml | ^0.9 | Rust YAML 序列化 |

---

## 五、时间线总览

```
第 1 周          第 2-3 周        第 4-6 周          第 7-8.5 周      第 8.5-9 周
│                │                │                  │                │
▼                ▼                ▼                  ▼                ▼
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│ M1       │   │ M2       │   │ M3       │   │ M4       │   │ M5       │
│ 调研准备  │──▶│ 核心框架  │──▶│ 逐模块   │──▶│ 测试优化  │──▶│ 文档发布  │
│          │   │ 集成     │   │ 迁移适配  │   │          │   │          │
│ 4 人天   │   │ 10 人天  │   │ 15 人天  │   │ 9 人天   │   │ 3 人天   │
└──────────┘   └──────────┘   └──────────┘   └──────────┘   └──────────┘

总计: ~41 人天 | 9 周
```

---

## 六、关键文件变更清单

### 新增文件

```
src-tauri/src/
├── services/allagents_service.rs      # AllAgents CLI 执行器
├── commands/allagents.rs              # Tauri 命令封装
└── models/unified_plugin.rs           # 统一类型定义

src/
├── services/allagents-config.ts       # workspace.yaml 生成器
├── types/unified-plugin.ts            # 统一插件类型
├── composables/useAllAgents.ts        # 前端调用封装
└── stores/unified-plugin.ts           # 统一 Plugin Store
```

### 修改文件

```
src-tauri/src/lib.rs                   # 注册新命令
src-tauri/Cargo.toml                   # 添加 serde_yaml 依赖
src/stores/plugin-marketplace.ts       # 委托到 AllAgentsService
src/stores/skill.ts                    # 合并到 UnifiedSkillStore
src/stores/agent.ts                    # 委托到 AllAgentsService
src/stores/mcp.ts                      # 委托到 AllAgentsService
src/stores/rule.ts                     # 补全真实数据
src/views/RulesView.vue                # 替换 mock 数据
```

### 移除/归档文件 (Phase 3)

```
src/stores/skill-marketplace.ts        # 合并到 unified
src/stores/skill-import.ts             # 合并到 unified
src/stores/anthropic-skills.ts         # 合并到 unified
src/stores/skills-sh.ts               # 合并到 unified
src-tauri/src/commands/plugin_sync.rs  # 委托到 allagents
src-tauri/src/services/mcp_bridge.rs   # allagents 内置代理
src-tauri/src/services/mcp_protocol.rs # allagents 内置代理
```

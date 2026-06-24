# Superpowers 使用教程

## 简介

Superpowers 是一个为编码代理提供完整软件开发工作流的工具集，通过自动触发的"技能"引导代理从设计到实现。

## 工作流程

```
brainstorming → writing-plans → subagent-driven-development → test-driven-development → systematic-debugging → verification-before-completion → finishing-a-development-branch
```

## Skill 命令与功能

### 核心工作流技能

| Skill 名称 | 命令 | 功能 | 触发条件 |
|-----------|------|------|---------|
| `brainstorming` | `/skill brainstorming` | 头脑风暴，探索想法并形成设计。一次问一个问题，提出 2-3 种方案，展示设计获取批准，保存设计文档 | 任何创造性工作 |
| `writing-plans` | `/skill writing-plans` | 编写实现计划。分解为 2-5 分钟的任务，包含文件路径、完整代码、验证步骤 | 有批准的设计规范 |
| `subagent-driven-development` | `/skill subagent-driven-development` | 子代理驱动开发。为每个任务分配独立子代理，两阶段审查（规范+代码质量） | 有实现计划 |
| `executing-plans` | `/skill executing-plans` | 批量执行计划。支持检查点和人工干预 | 有实现计划 |

### 测试与调试技能

| Skill 名称 | 命令 | 功能 | 触发条件 |
|-----------|------|------|---------|
| `test-driven-development` | `/skill test-driven-development` | 测试驱动开发。强制执行红-绿-重构循环：写失败测试 → 写最小实现 → 重构 | 实现功能或修复 bug |
| `systematic-debugging` | `/skill systematic-debugging` | 系统化调试。四阶段：根因调查 → 形成假设 → 验证假设 → 实施修复 | 遇到 bug 或意外行为 |
| `verification-before-completion` | `/skill verification-before-completion` | 验证完成。确认问题真正解决，避免症状修复 | 修复问题后 |

### 协作与审查技能

| Skill 名称 | 命令 | 功能 | 触发条件 |
|-----------|------|------|---------|
| `requesting-code-review` | `/skill requesting-code-review` | 请求代码审查。预审查清单，确保代码质量 | 代码编写完成后 |
| `receiving-code-review` | `/skill receiving-code-review` | 接收代码审查。响应审查反馈 | 收到审查反馈后 |

### 分支管理技能

| Skill 名称 | 命令 | 功能 | 触发条件 |
|-----------|------|------|---------|
| `using-git-worktrees` | `/skill using-git-worktrees` | 使用 Git Worktree。创建隔离工作区，支持并行开发 | 需要并行开发 |
| `finishing-a-development-branch` | `/skill finishing-a-development-branch` | 完成开发分支。验证测试，提供选项（合并/PR/保留/丢弃） | 任务完成后 |

### 并发与元技能

| Skill 名称 | 命令 | 功能 | 触发条件 |
|-----------|------|------|---------|
| `dispatching-parallel-agents` | `/skill dispatching-parallel-agents` | 并发子代理。并行执行多个子代理工作流 | 需要并行处理 |
| `writing-skills` | `/skill writing-skills` | 编写技能。创建新技能的最佳实践指南 | 创建自定义技能 |
| `using-superpowers` | `/skill using-superpowers` | 使用指南。技能系统介绍和使用说明 | 首次使用 |

## 核心原则

### TDD 铁律
```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```
- 先写失败测试
- 写最小实现代码
- 重构清理
- 没看到测试失败就不知道测试是否正确

### 调试铁律
```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```
- 先找根因再修复
- 症状修复 = 失败

### 设计原则
- **YAGNI** - 不要过度设计
- **DRY** - 不要重复代码
- **TDD** - 测试驱动开发
- **简单优先** - 复杂性降低是首要目标

## 使用示例

### 完整流程示例

```
用户：帮我实现用户登录功能

代理（自动触发 brainstorming）：
"我先了解项目上下文..."
"你希望支持哪些登录方式？（邮箱/密码、社交账号、手机号）"
"我推荐以下方案..."
"这是设计文档，请确认..."

用户：批准

代理（自动触发 writing-plans）：
"我正在使用 writing-plans 技能创建实现计划..."
保存计划到 docs/superpowers/plans/2026-06-21-user-login.md

用户：开始实现

代理（自动触发 subagent-driven-development）：
"我将为每个任务分配独立子代理..."
任务 1: 创建用户模型
  - 子代理执行
  - 两阶段审查
任务 2: 实现登录 API
  - 子代理执行
  - 两阶段审查
...

代理（自动触发 test-driven-development）：
"我将使用 TDD 实现..."
RED: 写失败测试
GREEN: 写最小实现
REFACTOR: 清理代码

代理（自动触发 verification-before-completion）：
"验证修复..."

代理（自动触发 finishing-a-development-branch）：
"所有测试通过，提供选项：合并/PR/保留/丢弃"
```

### 调试示例

```
用户：这个测试失败了

代理（自动触发 systematic-debugging）：
"我先调查根因..."
Phase 1: 根因调查
  - 读取错误信息
  - 复现问题
  - 检查最近更改
Phase 2: 形成假设
  "错误显示期望 'Email required'，但得到 undefined"
Phase 3: 验证假设
  "检查 submitForm 函数，发现缺少验证逻辑"
Phase 4: 实施修复
  "添加验证逻辑..."
```

## 技能触发机制

代理会在执行任务前**自动检查**相关技能。无需特殊操作，代理会自动获得 Superpowers。

关键点：
- 技能是**强制工作流**，不是建议
- 代理会自动选择合适的技能
- 用户也可以手动触发技能

## 更新与社区

```bash
# 更新
/plugin update superpowers

# 社区
- Discord: 加入社区获取支持
- Issues: 报告问题和建议
```

---

*Superpowers 由 [Jesse Vincent](https://blog.fsck.com) 和 [Prime Radiant](https://primeradiant.com) 团队构建。*

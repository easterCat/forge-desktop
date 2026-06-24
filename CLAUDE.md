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
├── stores/         # Pinia（19 个 store）
├── components/     # UI 组件
├── composables/    # 组合式函数
├── types/          # 类型定义
└── utils/          # 工具函数

src-tauri/          # Rust 后端
├── src/commands/   # Tauri 命令
├── src/services/   # 服务
└── src/db/         # SQLite
```

## 规则

- 所有回复都要使用中文

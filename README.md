# Forge

A cross-platform desktop application for managing AI development tool configurations.

## Features

### CLI Tools Management
- **Version Detection**: Automatically detect installed AI CLI tools (Claude Code, Codex, Gemini CLI, OpenCode, OpenClaw, Hermes, Cursor CLI)
- **Version Comparison**: Check current vs latest available versions via npm
- **One-Click Upgrade**: Automatically upgrade tools to latest versions
- **Multiple Install Methods**: Support for npm, curl|bash, and fallback methods
- **Conflict Diagnosis**: Detect npm conflicts, outdated Node versions, and PATH issues
- **Parallel Detection**: Check all tools simultaneously using rayon

### Software Detection
- Automatically detect 25+ installed AI tools and development software
- **AI Tools**: Cursor, Windsurf, Claude Desktop, Continue, Cody
- **Foundation**: Homebrew, Git, SSH, VS Code, iTerm2, Oh My Posh, Windows Terminal
- **Language Managers**: nvm, pyenv, goenv, jenv, asdf
- **Runtime & Containers**: Docker, Docker Compose, FFmpeg
- **Debug & Collaboration**: Apifox, Postman, Charles Proxy, Cyberduck, FileZilla
- **Productivity**: Snipaste, Obsidian, Excalidraw
- Track configuration paths and versions

### Plugin Management
- Install, update, and manage plugins for each supported software
- **Plugin Marketplace**: Browse and install plugins from preset sources (Anthropic, CCPlugins, community)
- **Custom Sources**: Add your own plugin repositories
- **Plugin Capabilities**: Probe plugin metadata, execute hooks, validate paths
- **MCP Detection**: Detect MCP capabilities in plugins
- Enable/disable plugins without uninstalling

### Skill Management
- Configure and manage AI agent skills from multiple sources:
- **Anthropic Skills**: Install and manage official Anthropic skills
- **Remote Skills**: Import skills from Git repositories
- **Skills.sh Marketplace**: Browse and install community skills from skills.sh
- **Skill Marketplace**: Aggregated skill marketplace with categories
- **Local Import**: Import skills from local directories
- **ZIP Installation**: Install skills from .zip packages
- Create custom skills for different workflows

### MCP Services
- Manage Model Context Protocol service connections
- **MCP Marketplace**: Browse and install MCP servers from the marketplace
- **Health Monitoring**: Automatic health checks for MCP endpoints
- **Sync Targets**: Sync MCP configurations to multiple targets
- Configure authentication, endpoints, and environment variables

### Rules Management
- Create and organize AI behavior rules (.mdc, .md, .txt)
- Toggle rules on/off without deletion
- Rule preview and syntax validation

### Prompt Manager
- Create, organize, and manage AI prompt templates
- Categorize prompts by use case

### Backup & Restore
- Protect your configurations with automated backups
- Selective restore of specific components
- View backup contents before restoring

### Settings
- GitHub Token management for API access
- Theme configuration (Dark / Warm)

## Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| **Framework** | Tauri | 2.0 |
| **Frontend** | Vue 3 + TypeScript + Vite | Vue 3.5.13 / TS 5.8.3 / Vite 6.3.5 |
| **State Management** | Pinia | 3.0.2 |
| **Styling** | Tailwind CSS | 3.4.17 |
| **Backend** | Rust | (Cargo.toml) |
| **Database** | SQLite (rusqlite) | 0.32 |
| **Async Runtime** | Tokio | 1.0 |

## Getting Started

### Prerequisites

- Node.js 20+
- Rust 1.75+
- npm or pnpm

### Installation

1. Install dependencies:

```bash
npm install
```

2. Run development server:

```bash
npm run dev
```

### Build

```bash
npm run tauri build
```

## Project Structure

```
forge-desktop/
├── src/                      # Vue 3 frontend
│   ├── App.vue               # Root component
│   ├── main.ts               # Entry point
│   ├── router/index.ts        # Route configuration (10 routes)
│   ├── views/                # 10 page views
│   │   ├── DashboardView.vue     # Dashboard
│   │   ├── CliToolsView.vue      # CLI tools management
│   │   ├── SoftwareManagementView.vue  # Software detection
│   │   ├── PluginsView.vue       # Plugin management
│   │   ├── SkillsView.vue        # Skill management
│   │   ├── MCPView.vue           # MCP servers
│   │   ├── RulesView.vue         # Rules management
│   │   ├── BackupView.vue        # Backup & restore
│   │   ├── SettingsView.vue      # Settings
│   │   └── PromptManagerView.vue # Prompt manager
│   ├── stores/                # 16 Pinia stores
│   ├── components/            # UI components
│   ├── composables/           # Composables
│   └── types/                 # TypeScript type definitions
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs            # Tauri command registry
│   │   ├── commands/          # 21 command modules
│   │   ├── services/          # 10 service modules
│   │   ├── models/            # Data models
│   │   └── db/                # SQLite database
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
└── docs/
    └── prd.md                 # Product Requirements Document
```

## Supported CLI Tools

| Tool | Install Methods | npm Package | Description |
|------|----------------|-------------|-------------|
| Claude Code | curl-bash, npm | @anthropic-ai/claude-code | Anthropic's AI coding assistant |
| Codex | npm | @openai/codex | OpenAI's code generation tool |
| Gemini CLI | npm | @google/gemini-cli | Google's Gemini AI CLI |
| OpenCode | curl-bash, npm | opencode-ai | OpenCode AI assistant |
| OpenClaw | npm | openclaw | AI-powered development CLI |
| Hermes | curl-bash | - | NousResearch agent framework |
| Cursor CLI | curl-bash | - | Cursor's CLI tool |

## Data Storage

| Path | Purpose |
|------|---------|
| `~/.local/share/forge/` | Application data directory |
| `~/.local/share/forge/forge.db` | SQLite database |
| `~/.local/share/forge/logs/` | Application logs |

## License

MIT

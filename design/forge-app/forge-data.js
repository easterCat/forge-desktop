/* Forge — AI Env Manager · Shared Data & Render Functions */

// === Timing constants ===
const PROGRESS_INTERVAL = 800;
const TOAST_DURATION = 2700;
const TOAST_FADE = 300;
const SYNC_RE_DURATION = 1500;
const SYNC_NEW_DURATION = 1800;
const STAGE_TIMEOUT = 3000;
const DEBOUNCE_MS = 300;
const CANCEL_CLEANUP = 2000;

const OPERATION_STAGES = ['idle','preparing','downloading','installing','verifying','completed','failed','cancelled'];
const STAGE_CONFIG = {idle:{label:'Idle',icon:'—'},preparing:{label:'Preparing',icon:'⏳'},downloading:{label:'Downloading',icon:'↓'},installing:{label:'Installing',icon:'⚙'},verifying:{label:'Verifying',icon:'✓'},completed:{label:'Completed',icon:'✔'},failed:{label:'Failed',icon:'✕'},cancelled:{label:'Cancelled',icon:'⊘'}};

const operations = new Map();
operations.set('gemini-cli',{stage:'downloading',progress:42,message:'↓'});
operations.set('deepseek-reasonix',{stage:'installing',progress:78,message:'⚙'});
function getOperation(key){return operations.get(key)||null}
function isAnyActive(){for(const[,op]of operations){if(['preparing','downloading','installing','verifying'].includes(op.stage))return true}return false}

const cliTools = [
  {key:'claude-code',name:'Claude Code',icon:'CC',color:'#B8944A',pkg:'@anthropic-ai/claude-code',desc:'Anthropic official AI coding tool',methods:['curl|bash','npm'],installed:true,current:'1.0.30',latest:'1.0.32',needsUpdate:true,path:'~/.claude/',conflicts:[],binaryNames:['claude']},
  {key:'codex',name:'Codex',icon:'Co',color:'#5A8A64',pkg:'@openai/codex',desc:'OpenAI code generation tool',methods:['npm'],installed:true,current:'0.1.25',latest:'0.1.25',needsUpdate:false,path:'~/.codex/',conflicts:[],binaryNames:['codex']},
  {key:'gemini-cli',name:'Gemini CLI',icon:'Gm',color:'#5A6B7A',pkg:'@google/gemini-cli',desc:'Google Gemini AI CLI',methods:['npm'],installed:true,current:'0.4.1',latest:'0.4.1',needsUpdate:false,path:'~/.gemini/',conflicts:[],binaryNames:['gemini']},
  {key:'opencode',name:'OpenCode',icon:'OC',color:'#5C5C5C',pkg:'opencode-ai',desc:'Open-source AI assistant',methods:['curl|bash','npm'],installed:false,current:'-',latest:'0.3.7',needsUpdate:false,path:'-',conflicts:[],binaryNames:[]},
  {key:'openclaw',name:'OpenClaw',icon:'Cl',color:'#B85A42',pkg:'openclaw',desc:'AI-driven development CLI',methods:['npm'],installed:false,current:'-',latest:'0.2.4',needsUpdate:false,path:'-',conflicts:[],binaryNames:[]},
  {key:'hermes',name:'Hermes',icon:'Hm',color:'#B8944A',pkg:null,desc:'NousResearch Agent framework',methods:['curl|bash'],installed:false,current:'-',latest:'1.0.1',needsUpdate:false,path:'-',conflicts:['npm-curl-fallback'],binaryNames:[]},
  {key:'cursor',name:'Cursor CLI',icon:'Cu',color:'#5A6B7A',pkg:null,desc:'Cursor IDE command-line agent',methods:['curl|bash'],installed:true,current:'0.47.9',latest:'0.47.9',needsUpdate:false,path:'~/.cursor/',conflicts:[],binaryNames:['cursor-agent']},
  {key:'deepseek-reasonix',name:'DeepSeek Reasonix',icon:'DS',color:'#6B7A5A',pkg:'deepseek-reasonix',desc:'DeepSeek reasoning-optimized CLI',methods:['npm'],installed:true,current:'0.2.3',latest:'0.2.3',needsUpdate:false,path:'~/.deepseek/',conflicts:[],binaryNames:[]},
  {key:'kiro',name:'Kiro',icon:'Ki',color:'#7A5A6B',pkg:'@kiro/cli',desc:'Kiro AI pair programming CLI',methods:['npm'],installed:false,current:'-',latest:'0.1.3',needsUpdate:false,path:'-',conflicts:[],binaryNames:['agent']},
  {key:'mimo-code',name:'MiMo Code',icon:'Mi',color:'#5A7A6B',pkg:'@mimo-ai/cli',desc:'MiMo AI coding assistant',methods:['curl|bash','npm'],installed:false,current:'-',latest:'0.1.0',needsUpdate:false,path:'-',conflicts:[],binaryNames:['mimo']}
];

const marketplacePlugins = [
  {name:'agent-browser',desc:'Browser automation for AI agents',author:'open-design',version:'1.4.2',source:'forge-official',categories:['automation','devtools'],downloads:1240,stars:89,installed:true,icon:'AB',color:'#5A8A64'},
  {name:'cursor-tools',desc:'Extended Cursor IDE tooling',author:'cursor-inc',version:'2.1.0',source:'forge-official',categories:['devtools','productivity'],downloads:980,stars:67,installed:true,icon:'CT',color:'#5A6B7A'},
  {name:'mcp-manager',desc:'MCP server management UI',author:'community',version:'0.8.1',source:'community-hub',categories:['devtools','data'],downloads:720,stars:45,installed:true,icon:'MM',color:'#B8944A'},
  {name:'git-sync',desc:'Git-based config synchronization',author:'devtools',version:'1.2.3',source:'forge-official',categories:['productivity','devtools'],downloads:1560,stars:112,installed:true,icon:'GS',color:'#5A8A64'},
  {name:'theme-engine',desc:'Custom theme management system',author:'uikit',version:'0.5.0',source:'community-hub',categories:['ui','productivity'],downloads:430,stars:28,installed:true,icon:'TE',color:'#7A5A6B'},
  {name:'skill-pack-ai',desc:'AI skill bundle for code generation',author:'ai-tools',version:'3.0.0',source:'ai-tools-pack',categories:['ai','devtools'],downloads:2100,stars:156,installed:true,icon:'SP',color:'#B85A42'},
  {name:'code-reviewer',desc:'Automated code review agent',author:'forge-team',version:'1.1.0',source:'forge-official',categories:['automation','devtools'],downloads:890,stars:72,installed:false,icon:'CR',color:'#5A6B7A'},
  {name:'api-tester',desc:'API endpoint testing toolkit',author:'devtools',version:'2.0.3',source:'forge-official',categories:['devtools','data'],downloads:670,stars:51,installed:false,icon:'AT',color:'#B8944A'},
  {name:'doc-generator',desc:'Auto-generate docs from code',author:'community',version:'0.9.1',source:'community-hub',categories:['productivity','devtools'],downloads:540,stars:38,installed:false,icon:'DG',color:'#5A8A64'},
  {name:'ai-translator',desc:'Multi-language AI translation',author:'ai-tools',version:'1.3.0',source:'ai-tools-pack',categories:['ai','productivity'],downloads:1800,stars:134,installed:false,icon:'TL',color:'#B85A42'},
  {name:'ui-components',desc:'Pre-built UI component library',author:'uikit',version:'2.2.1',source:'community-hub',categories:['ui','devtools'],downloads:920,stars:76,installed:false,icon:'UI',color:'#7A5A6B'},
  {name:'data-viz',desc:'Data visualization toolkit',author:'community',version:'1.0.0',source:'community-hub',categories:['data','ui'],downloads:380,stars:22,installed:false,icon:'DV',color:'#B8944A'},
  {name:'workflow-engine',desc:'Visual workflow builder',author:'forge-team',version:'0.7.0',source:'forge-official',categories:['automation','productivity'],downloads:450,stars:33,installed:false,icon:'WE',color:'#5A6B7A'},
  {name:'prompt-library',desc:'Curated prompt templates',author:'ai-tools',version:'1.5.2',source:'ai-tools-pack',categories:['ai','productivity'],downloads:2800,stars:198,installed:false,icon:'PL',color:'#B85A42'},
  {name:'docker-manager',desc:'Docker container management',author:'devtools',version:'1.1.0',source:'forge-official',categories:['devtools','data'],downloads:760,stars:58,installed:false,icon:'DM',color:'#5A8A64'},
  {name:'slack-integration',desc:'Slack workspace integration',author:'community',version:'0.6.0',source:'community-hub',categories:['productivity','automation'],downloads:310,stars:19,installed:false,icon:'SI',color:'#5A6B7A'},
  {name:'figma-sync',desc:'Figma design token sync',author:'uikit',version:'0.4.0',source:'community-hub',categories:['ui','productivity'],downloads:290,stars:15,installed:false,icon:'FS',color:'#7A5A6B'},
  {name:'test-runner',desc:'Smart test execution engine',author:'forge-team',version:'1.0.0',source:'forge-official',categories:['devtools','automation'],downloads:510,stars:41,installed:false,icon:'TR',color:'#5A8A64'},
  {name:'security-scan',desc:'Security vulnerability scanner',author:'devtools',version:'0.9.0',source:'forge-official',categories:['devtools','data'],downloads:440,stars:35,installed:false,icon:'SS',color:'#B85A42'},
  {name:'ai-assistant',desc:'General-purpose AI assistant',author:'ai-tools',version:'2.1.0',source:'ai-tools-pack',categories:['ai','productivity'],downloads:3200,stars:245,installed:false,icon:'AS',color:'#B85A42'},
  {name:'log-analyzer',desc:'Intelligent log analysis tool',author:'community',version:'0.8.0',source:'community-hub',categories:['data','devtools'],downloads:350,stars:24,installed:false,icon:'LA',color:'#B8944A'}
];

const software = [
  {name:'Cursor',key:'cursor',version:'0.47.9',configPath:'~/.cursor/',installed:true,lastChecked:'2 min ago',tier:'t1',desc:'AI-native code editor with multi-model support, inline completions, and agent mode',platform:'macOS · Win · Linux',icon:'Cu',color:'#7C3AED'},
  {name:'Claude Desktop',key:'claude-desktop',version:'0.9.2',configPath:'~/.claude/',installed:true,lastChecked:'2 min ago',tier:'t1',desc:'Anthropic desktop app for Claude conversations, artifacts, and MCP integrations',platform:'macOS · Win',icon:'CD',color:'#D97706'},
  {name:'Claude Code',key:'claude-code',version:'1.0.30',configPath:'~/.claude-code/',installed:true,lastChecked:'5 min ago',tier:'t1',desc:'CLI agent for coding tasks — reads codebases, runs commands, and edits files autonomously',platform:'macOS · Win · Linux',icon:'CC',color:'#D97706'},
  {name:'Windsurf',key:'windsurf',version:'1.6.4',configPath:'~/.windsurf/',installed:true,lastChecked:'2 min ago',tier:'t2',desc:'Codeium AI editor with Cascade agentic flows and multi-file editing',platform:'macOS · Win · Linux',icon:'Ws',color:'#0891B2'},
  {name:'Copilot',key:'copilot',version:'1.3.0',configPath:'~/.copilot/',installed:true,lastChecked:'2 min ago',tier:'t2',desc:'GitHub Copilot CLI and IDE integration with code suggestions and chat',platform:'macOS · Win · Linux',icon:'Co',color:'#059669'},
  {name:'Cursor CLI',key:'cursor-cli',version:'0.47.9',configPath:'~/.cursor-cli/',installed:true,lastChecked:'3 min ago',tier:'t2',desc:'Cursor command-line tools for headless agent workflows and CI integration',platform:'macOS · Win · Linux',icon:'Cz',color:'#7C3AED'},
  {name:'Continue',key:'continue',version:'0.9.21',configPath:'~/.continue/',installed:false,lastChecked:'2 min ago',tier:'t3',desc:'Open-source AI code assistant — autocomplete, chat, and edit with any LLM provider',platform:'macOS · Win · Linux',icon:'Ct',color:'#4F46E5'},
  {name:'Cody',key:'cody',version:'1.2.0',configPath:'~/.cody/',installed:false,lastChecked:'2 min ago',tier:'t3',desc:'Sourcegraph AI coding assistant with codebase-wide context and multi-repo search',platform:'macOS · Win · Linux',icon:'Cy',color:'#DC2626'},
  {name:'Gemini CLI',key:'gemini-cli',version:'0.4.1',configPath:'~/.gemini/',installed:true,lastChecked:'4 min ago',tier:'t2',desc:'Google Gemini command-line interface for AI-powered coding and file operations',platform:'macOS · Win · Linux',icon:'Gm',color:'#2563EB'},
  {name:'OpenCode',key:'opencode',version:'0.3.7',configPath:'~/.opencode/',installed:false,lastChecked:'5 min ago',tier:'t3',desc:'Open-source terminal-based AI coding assistant with multi-provider support',platform:'macOS · Linux',icon:'OC',color:'#0891B2'},
  {name:'DeepSeek Reasonix',key:'deepseek',version:'0.2.3',configPath:'~/.deepseek/',installed:false,lastChecked:'5 min ago',tier:'t3',desc:'DeepSeek reasoning engine CLI — chain-of-thought coding with math and logic focus',platform:'macOS · Linux',icon:'DS',color:'#4F46E5'},
  {name:'Kiro',key:'kiro',version:'0.1.0',configPath:'~/.kiro/',installed:false,lastChecked:'5 min ago',tier:'t3',desc:'Spec-driven AI editor — generates code from structured requirements and design docs',platform:'macOS · Win · Linux',icon:'Ki',color:'#DC2626'}
];

const plugins = [
  {name:'agent-browser',version:'1.4.2',author:'open-design',desc:'Browser automation for agents — navigate, click, screenshot, and extract data from web pages',enabled:true,installedAt:'2026-06-10',software:'Cursor',syncedWith:['claude-code','cursor']},
  {name:'cursor-tools',version:'2.1.0',author:'cursor-inc',desc:'Extended cursor tooling — multi-select, block operations, and advanced editing commands',enabled:true,installedAt:'2026-06-08',software:'Cursor',syncedWith:['claude-code']},
  {name:'mcp-manager',version:'0.8.1',author:'community',desc:'MCP server management — add, remove, configure, and monitor Model Context Protocol servers',enabled:true,installedAt:'2026-06-05',software:'Cursor',syncedWith:['cursor']},
  {name:'skill-pack-ai',version:'3.0.0',author:'ai-tools',desc:'AI skill bundle — curated collection of production-ready AI agent skills',enabled:false,installedAt:'2026-05-28',software:'Cursor',syncedWith:[]},
  {name:'git-sync',version:'1.2.3',author:'devtools',desc:'Git-based config sync — synchronize agent configs and rules across machines via Git',enabled:true,installedAt:'2026-06-01',software:'Claude Desktop',syncedWith:['claude-code','gemini-cli','codex']},
  {name:'theme-engine',version:'0.5.0',author:'uikit',desc:'Custom theme management — create, switch, and share visual themes for agent environments',enabled:false,installedAt:'2026-06-11',software:'Cursor',syncedWith:[]},
  {name:'code-reviewer',version:'1.1.0',author:'devtools',desc:'Automated code review — analyze PRs, suggest improvements, and check coding standards',enabled:true,installedAt:'2026-06-12',software:'Claude Code',syncedWith:['claude-code','cursor']},
  {name:'api-tester',version:'0.9.2',author:'community',desc:'API testing toolkit — send requests, validate responses, and generate test suites from OpenAPI specs',enabled:true,installedAt:'2026-06-09',software:'Cursor',syncedWith:['cursor','opencode']},
  {name:'docker-manager',version:'1.0.5',author:'devtools',desc:'Container management — build, run, and monitor Docker containers directly from your agent',enabled:true,installedAt:'2026-06-07',software:'Claude Desktop',syncedWith:['claude-code']},
  {name:'security-scan',version:'0.7.3',author:'community',desc:'Security vulnerability scanner — detect secrets, dependency risks, and code injection patterns',enabled:true,installedAt:'2026-06-14',software:'Cursor',syncedWith:['claude-code','cursor','gemini-cli']},
  {name:'doc-generator',version:'0.4.1',author:'ai-tools',desc:'Documentation generator — auto-generate README, API docs, and changelogs from source code',enabled:false,installedAt:'2026-06-03',software:'Cursor',syncedWith:[]},
  {name:'log-analyzer',version:'1.3.0',author:'devtools',desc:'Log analysis and debugging — parse, filter, and summarize application logs with AI insights',enabled:true,installedAt:'2026-06-15',software:'Claude Code',syncedWith:['claude-code']}
];

const skills = [
  {name:'imagegen',type:'agent',desc:'AI image generation skill — create illustrations, icons, and UI mockups from text prompts',software:'Cursor',source:'local',enabled:true},
  {name:'frontend-design',type:'command',desc:'Frontend UI design helper — generate responsive HTML/CSS from wireframes or descriptions',software:'Cursor',source:'local',enabled:true},
  {name:'brandkit',type:'agent',desc:'Brand guidelines generator — extract colors, fonts, and visual rules from brand assets',software:'Cursor',source:'local',enabled:true},
  {name:'skill-creator',type:'automation',desc:'Skill creation helper — scaffold new agent skills with proper structure and metadata',software:'Cursor',source:'local',enabled:true},
  {name:'canvas',type:'command',desc:'Live React canvas builder — render and iterate on React components in real-time',software:'Cursor',source:'local',enabled:false},
  {name:'shell',type:'automation',desc:'Shell command executor — run terminal commands with safety checks and output parsing',software:'Claude Desktop',source:'anthropic',enabled:true},
  {name:'documents',type:'agent',desc:'Document artifact creator — generate structured documents, reports, and presentations',software:'Cursor',source:'marketplace',enabled:true},
  {name:'spreadsheets',type:'command',desc:'Spreadsheet builder — create and manipulate CSV/Excel data with formula support',software:'Cursor',source:'skills-sh',enabled:false},
  {name:'test-writer',type:'automation',desc:'Test suite generator — auto-generate unit, integration, and E2E tests from source code',software:'Claude Code',source:'anthropic',enabled:true},
  {name:'git-workflow',type:'command',desc:'Git workflow automation — branch management, commit messages, PR templates, and merge strategies',software:'Cursor',source:'local',enabled:true},
  {name:'data-pipeline',type:'agent',desc:'Data pipeline builder — design and implement ETL workflows with validation and monitoring',software:'Claude Desktop',source:'marketplace',enabled:true},
  {name:'api-designer',type:'command',desc:'API designer — generate RESTful endpoints, OpenAPI specs, and client SDKs from requirements',software:'Cursor',source:'marketplace',enabled:true},
  {name:'debug-assistant',type:'automation',desc:'Debugging assistant — analyze stack traces, suggest fixes, and set conditional breakpoints',software:'Claude Code',source:'anthropic',enabled:true},
  {name:'refactor-tool',type:'command',desc:'Code refactoring tool — apply design patterns, extract components, and modernize syntax',software:'Cursor',source:'local',enabled:true},
  {name:'deploy-helper',type:'automation',desc:'Deployment helper — configure CI/CD pipelines, environment variables, and hosting platforms',software:'Cursor',source:'skills-sh',enabled:false}
];

const agentTargetTools = [
  {key:'claude-code',abbr:'CC',name:'Claude Code',color:'#D97706'},
  {key:'cursor',abbr:'Cu',name:'Cursor CLI',color:'#7C3AED'},
  {key:'copilot',abbr:'Co',name:'Copilot',color:'#059669'},
  {key:'gemini-cli',abbr:'Gm',name:'Gemini CLI',color:'#2563EB'},
  {key:'opencode',abbr:'OC',name:'OpenCode',color:'#0891B2'},
  {key:'deepseek',abbr:'DS',name:'DeepSeek',color:'#4F46E5'},
  {key:'kiro',abbr:'Ki',name:'Kiro',color:'#DC2626'},
  {key:'codex',abbr:'Cx',name:'Codex',color:'#9333EA'},
  {key:'openclaw',abbr:'Cl',name:'OpenClaw',color:'#B45309'},
  {key:'mimo-code',abbr:'Mi',name:'MiMo Code',color:'#0D9488'}
];

const agents = [
  {name:'Software Architect',emoji:'SA',department:'engineering',desc:'Designs system architecture and technical solutions',source:'agency-agents-zh',targets:['claude-code','cursor','copilot','gemini-cli']},
  {name:'Frontend Director',emoji:'FD',department:'engineering',desc:'Leads frontend architecture and Vue 3 implementation',source:'agency-agents-zh',targets:['claude-code','cursor','copilot']},
  {name:'Backend Director',emoji:'BD',department:'engineering',desc:'Oversees Rust backend and Tauri IPC layer',source:'agency-agents-zh',targets:['claude-code','cursor','opencode']},
  {name:'Design Director',emoji:'DD',department:'design',desc:'Guards design system consistency and visual quality',source:'agency-agents-zh',targets:['claude-code','cursor','gemini-cli']},
  {name:'Design UX',emoji:'UX',department:'design',desc:'User experience research and interaction design',source:'agency-agents-zh',targets:['claude-code','cursor']},
  {name:'QA Director',emoji:'QA',department:'quality',desc:'Test strategy and quality assurance oversight',source:'agency-agents-zh',targets:['claude-code','cursor','deepseek']},
  {name:'Performance Engineer',emoji:'PE',department:'engineering',desc:'Performance profiling and optimization',source:'agency-agents-zh',targets:['claude-code','opencode','kiro']},
  {name:'Product Manager',emoji:'PM',department:'product',desc:'Product strategy and requirement management',source:'agency-agents-zh',targets:['claude-code','cursor','copilot','gemini-cli']},
  {name:'Review Expert',emoji:'RE',department:'quality',desc:'Code review and quality gate enforcement',source:'agency-agents-zh',targets:['claude-code','cursor','codex','mimo-code']},
  {name:'Deployment Engineer',emoji:'DE',department:'engineering',desc:'CI/CD pipeline and release management',source:'agency-agents-zh',targets:['claude-code','openclaw','kiro']},
  {name:'Custom Agent',emoji:'CA',department:'custom',desc:'User-defined agent role',source:'custom',targets:[]}
];

const mcpServers = [
  {name:'git',endpoint:'stdio://git-mcp',auth:'none',healthy:true,tools:8,lastChecked:'1 min ago',group:'core'},
  {name:'node_repl',endpoint:'stdio://node-repl-mcp',auth:'none',healthy:true,tools:6,lastChecked:'1 min ago',group:'core'},
  {name:'gitlab',endpoint:'https://gitlab.com/api/v4/mcp',auth:'oauth',healthy:false,tools:12,lastChecked:'5 min ago',group:'integrations'},
  {name:'filesystem',endpoint:'stdio://fs-mcp',auth:'none',healthy:true,tools:14,lastChecked:'2 min ago',group:'core'},
  {name:'postgres',endpoint:'stdio://pg-mcp',auth:'token',healthy:true,tools:9,lastChecked:'3 min ago',group:'data'},
  {name:'puppeteer',endpoint:'stdio://puppeteer-mcp',auth:'none',healthy:false,tools:7,lastChecked:'12 min ago',group:'automation'},
  {name:'slack',endpoint:'https://slack.com/api/mcp',auth:'oauth',healthy:true,tools:11,lastChecked:'1 min ago',group:'integrations'},
  {name:'memory',endpoint:'stdio://memory-mcp',auth:'none',healthy:true,tools:5,lastChecked:'1 min ago',group:'core'},
  {name:'sentry',endpoint:'https://sentry.io/api/mcp',auth:'token',healthy:true,tools:8,lastChecked:'4 min ago',group:'monitoring'},
  {name:'docker',endpoint:'stdio://docker-mcp',auth:'none',healthy:true,tools:10,lastChecked:'2 min ago',group:'automation'}
];

const rulesFiles = [
  {name:'AGENTS.md',type:'md',software:'Cursor',size:'4.2 KB',active:true,modified:'2026-06-12 20:28',category:'agent'},
  {name:'coding-standards.mdc',type:'mdc',software:'Cursor',size:'2.1 KB',active:true,modified:'2026-06-11 14:00',category:'style'},
  {name:'commit-rules.md',type:'md',software:'Cursor',size:'1.8 KB',active:true,modified:'2026-06-10 09:15',category:'workflow'},
  {name:'project-context.md',type:'md',software:'Claude Desktop',size:'3.4 KB',active:false,modified:'2026-06-09 16:42',category:'context'},
  {name:'rust-conventions.mdc',type:'mdc',software:'Cursor',size:'3.1 KB',active:true,modified:'2026-06-08 11:20',category:'style'},
  {name:'pr-review-checklist.md',type:'md',software:'Claude Code',size:'1.5 KB',active:true,modified:'2026-06-07 15:30',category:'workflow'},
  {name:'api-design-rules.md',type:'md',software:'Claude Code',size:'2.8 KB',active:true,modified:'2026-06-06 09:00',category:'style'},
  {name:'test-coverage-policy.md',type:'md',software:'Cursor',size:'1.2 KB',active:false,modified:'2026-06-05 14:15',category:'workflow'},
  {name:'security-guidelines.md',type:'md',software:'Claude Desktop',size:'2.6 KB',active:true,modified:'2026-06-04 10:45',category:'security'},
  {name:'tauri-ipc-patterns.mdc',type:'mdc',software:'Cursor',size:'3.8 KB',active:true,modified:'2026-06-03 16:00',category:'context'}
];

const backups = [
  {name:'Scheduled Full',type:'scheduled',date:'2026-06-18 03:00',size:'52.4 MB',files:168,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools','MCP Servers']},
  {name:'Manual Snapshot',type:'manual',date:'2026-06-17 14:22',size:'51.1 MB',files:164,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools']},
  {name:'Incremental',type:'incremental',date:'2026-06-17 09:15',size:'3.8 MB',files:12,status:'completed',includes:['Cursor','Claude Desktop']},
  {name:'Scheduled Full',type:'scheduled',date:'2026-06-16 03:00',size:'49.7 MB',files:160,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools','MCP Servers']},
  {name:'Pre-update v2.4.0',type:'pre-update',date:'2026-06-15 22:10',size:'48.9 MB',files:158,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools']},
  {name:'Incremental',type:'incremental',date:'2026-06-15 11:30',size:'1.6 MB',files:6,status:'completed',includes:['Cursor']},
  {name:'Scheduled Full',type:'scheduled',date:'2026-06-14 03:00',size:'47.2 MB',files:154,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools']},
  {name:'Manual Snapshot',type:'manual',date:'2026-06-13 16:45',size:'46.8 MB',files:152,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools']},
  {name:'System Recovery',type:'system',date:'2026-06-12 09:00',size:'44.5 MB',files:148,status:'completed',includes:['Cursor','Claude Desktop','CLI Tools']},
  {name:'Pre-update v2.3.1',type:'pre-update',date:'2026-06-10 20:30',size:'43.2 MB',files:142,status:'completed',includes:['Cursor','Claude Desktop','Windsurf']},
  {name:'Incremental',type:'incremental',date:'2026-06-09 14:00',size:'2.4 MB',files:9,status:'completed',includes:['Claude Desktop','MCP Servers']},
  {name:'Scheduled Full',type:'scheduled',date:'2026-06-08 03:00',size:'41.8 MB',files:138,status:'completed',includes:['Cursor','Claude Desktop','Windsurf','CLI Tools']}
];

const sourcesData = [
  {name:'forge-official',url:'https://github.com/forge-plugins/official',type:'market',repoType:'GitHub',installed:true,plugins:12,path:'~/.forge/plugins/cache/forge-official/',notes:'Official curated plugins from the Forge team. Auto-syncs daily.'},
  {name:'community-hub',url:'https://github.com/forge-plugins/community',type:'market',repoType:'GitHub',installed:true,plugins:8,path:'~/.forge/plugins/cache/community-hub/',notes:'Community-maintained plugins. Contributions welcome.'},
  {name:'ai-tools-pack',url:'https://github.com/ai-tools/plugins',type:'git',repoType:'GitLab',installed:false,plugins:1,path:'—',notes:'Experimental AI tool plugins. Use with caution.'}
];

const THEMES = [
  {id:'warm',name:'Warm Glass',desc:'Default · Frosted warm',colors:['#F5F3F0','#F0EDE8','#D2CAB8','#2D2D2D','#5A8A64','#B8944A'],active:true},
  {id:'cool-mist',name:'Cool Mist',desc:'Blue-grey frost',colors:['#EEF1F5','#DDE2EA','#B8C4D4','#1E2A3A','#4A8A6A','#6B7FAA']},
  {id:'midnight',name:'Midnight',desc:'Dark glass mode',colors:['#1A1D24','#242830','#2E3340','#E0E4EC','#5A9A7A','#7A8AAA']},
  {id:'sakura',name:'Sakura',desc:'Soft pink tint',colors:['#F8F0F2','#F0DDE2','#E4B8C4','#3A2028','#8A5A6A','#C47A8A']},
  {id:'sage',name:'Sage',desc:'Muted green glass',colors:['#EFF3EE','#DAE4D8','#B8CCB4','#1E2A1E','#5A8A5A','#8AAA6A']},
  {id:'lavender',name:'Lavender',desc:'Purple frosted',colors:['#F2EFF8','#E0DAF0','#C4B8E0','#2A2038','#7A5AAA','#A070C0']},
  {id:'ocean',name:'Ocean',desc:'Deep teal glass',colors:['#ECF4F4','#D4E8E8','#A8D4D4','#0A2828','#2A8A8A','#4A9AAA']},
  {id:'ember',name:'Ember',desc:'Warm amber glow',colors:['#F8F2EC','#F0E0CC','#E0C8A0','#382010','#B87A3A','#D49A4A']},
  {id:'slate',name:'Slate',desc:'Neutral stone',colors:['#F0F0EE','#E0E0DC','#C8C8C0','#2A2A28','#6A7A6A','#8A8A7A']},
  {id:'aurora',name:'Aurora',desc:'Multi-hue shimmer',colors:['#F0F4F8','#DAE4F0','#A8C4E0','#14202C','#4A8ACC','#8A6ACC']},
  {id:'cream',name:'Cream',desc:'Light warm minimal',colors:['#FAF8F5','#F2EDE6','#E8DFD0','#2C2820','#7A8A5A','#AA9A60']},
  {id:'arctic',name:'Arctic',desc:'Cool blue-white',colors:['#F4F8FA','#E4EEF4','#C8DAE8','#142430','#3A7AAA','#5A9ACC']},
  {id:'rose-gold',name:'Rose Gold',desc:'Metallic warm pink',colors:['#F8F2F4','#F0DDE2','#E0C0C8','#301820','#B85A70','#D48A6A']},
  {id:'cyberpunk',name:'Cyberpunk',desc:'Neon dark',colors:['#0E0E18','#1A1A2E','#2A2A40','#E0E0F0','#FF2E63','#08D9D6']},
  {id:'forest',name:'Forest',desc:'Deep earthy green',colors:['#F0F4EE','#D8E4D0','#B0C8A0','#14200E','#3A6A2A','#5A8A4A']},
  {id:'desert',name:'Desert Sand',desc:'Warm sandy neutral',colors:['#F5EEE4','#E8D8C4','#D0BC98','#2C2014','#A07848','#C49A58']},
  {id:'cotton-candy',name:'Cotton Candy',desc:'Pastel rainbow',colors:['#F8F0F8','#F0DAF0','#E0C0E8','#282030','#C060A0','#60A0C0']},
  {id:'charcoal',name:'Charcoal',desc:'Dark neutral glass',colors:['#181818','#252525','#333333','#E8E8E8','#6A8A6A','#8A6A6A']},
  {id:'peach',name:'Peach Fuzz',desc:'2024 trending warm',colors:['#FBF0E8','#F5DCC8','#E8C0A0','#302018','#D48A58','#E8A070']},
  {id:'nordic',name:'Nordic',desc:'Scandinavian cool',colors:['#F4F6F8','#E2E8EE','#C8D4E0','#1C2430','#5A7A9A','#8A6A5A']}
];

// === Render functions ===

function renderCLITool(tool) {
  const op = getOperation(tool.key);
  const isPending = op && ['preparing','downloading','installing','verifying'].includes(op.stage);
  const statusBadge = isPending ? '<span class="badge progress">'+STAGE_CONFIG[op.stage].label+'</span>' : tool.installed ? (tool.needsUpdate ? '<span class="badge warn">Update available</span>' : '<span class="badge success">Installed</span>') : '<span class="badge outline">Not installed</span>';
  const progressSlot = isPending ? '<div class="progress-slot"><div class="progress-bar-wrap"><div class="progress-bar-fill" style="width:'+op.progress+'%"></div></div><div class="progress-msg">'+op.message+'</div></div>' : '';
  const buttons = [];
  if(isPending){buttons.push('<button class="btn btn-ghost btn-sm" onclick="cancelOperation(\''+tool.key+'\')">Cancel</button>')}
  else if(!tool.installed){buttons.push('<button class="btn btn-primary btn-sm" onclick="startInstall(\''+tool.key+'\')">Install</button>')}
  else if(tool.needsUpdate){buttons.push('<button class="btn btn-primary btn-sm" onclick="startUpgrade(\''+tool.key+'\')">Update</button>')}
  else{buttons.push('<button class="btn btn-secondary btn-sm" onclick="showNotification(\''+tool.name+' is up to date\',\'success\')">Check</button>')}
  buttons.push('<button class="btn-icon btn-sm" title="More actions" aria-label="More actions"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>');
  return '<div class="card" data-od-id="cli-tool-'+tool.key+'" data-key="'+tool.key+'"><div class="card-head"><div class="card-icon" style="background:'+tool.color+'12;color:'+tool.color+';border-color:'+tool.color+'25;font-size:13px;font-weight:700;font-family:var(--font-mono)">'+tool.icon+'</div><div style="flex:1;min-width:0"><div class="card-title">'+tool.name+' '+statusBadge+'</div>'+(tool.pkg?'<div class="card-subtitle">'+tool.pkg+'</div>':'')+'</div></div>'+(tool.desc?'<div class="card-desc">'+tool.desc+'</div>':'')+'<div class="card-meta">'+(tool.installed?'<div class="card-meta-item"><span>Version</span><span class="value">'+tool.current+(tool.needsUpdate?' → '+tool.latest:'')+'</span></div>':'')+'<div class="card-meta-item"><span>Latest</span><span class="value">'+tool.latest+'</span></div>'+(tool.installed?'<div class="card-meta-item"><span>Path</span><span class="value">'+tool.path+'</span></div>':'')+'</div><div class="card-divider"></div><div class="card-footer"><div class="card-footer-left">'+progressSlot+'</div><div class="card-footer-right btn-group">'+buttons.join('')+'</div></div></div>';
}

function renderSoftware(sw) {
  const badge = sw.installed ? '<span class="badge success">Detected</span>' : '<span class="badge outline">Not found</span>';
  const tierColors = {t0:'var(--accent)',t1:'var(--accent)',t2:'var(--info)',t3:'var(--fg-ghost)',t4:'var(--fg-ghost)',t5:'var(--fg-ghost)'};
  const tierLabels = {t0:'AI Tools',t1:'Foundation',t2:'Language Mgr',t3:'Runtime',t4:'Debug',t5:'Productivity'};
  const c = sw.color || '#5C5C5C';
  return '<div class="card sw-card" data-od-id="software-card-'+sw.key+'">'
    +'<div class="card-head">'
      +'<div class="card-icon" style="background:'+c+'12;color:'+c+';border-color:'+c+'25;font-size:14px;font-weight:700;font-family:var(--font-mono)">'+sw.icon+'</div>'
      +'<div style="flex:1;min-width:0">'
        +'<div class="card-title">'+sw.name+' '+badge+'</div>'
        +'<div class="card-subtitle">v'+sw.version+' · '+sw.platform+' · <span style="color:'+(tierColors[sw.tier]||'var(--fg-ghost)')+';text-transform:uppercase;font-weight:600;font-size:10px;letter-spacing:0.04em">'+(tierLabels[sw.tier]||sw.tier)+'</span></div>'
      +'</div>'
    +'</div>'
    +'<div class="card-desc">'+sw.desc+'</div>'
    +'<div class="card-meta">'
      +'<div class="card-meta-item"><span class="label">Config</span><span class="value" style="font-family:var(--font-mono);font-size:11px">'+sw.configPath+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Checked</span><span class="value">'+sw.lastChecked+'</span></div>'
    +'</div>'
    +'<div class="card-footer">'
      +'<div class="card-footer-left"></div>'
      +'<div class="card-footer-right btn-group">'
        +(sw.installed ? '<button class="btn btn-secondary btn-sm" onclick="showNotification(\'Opening '+sw.configPath+'\',\'info\')">Open Config</button>' : '<button class="btn btn-primary btn-sm">Install</button>')
        +'<button class="btn-icon btn-sm" title="More actions" aria-label="More actions"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
      +'</div>'
    +'</div>'
  +'</div>';
}

function renderSkill(s) {
  const tc = {agent:'#B8944A',command:'#5A6B7A',automation:'#5A8A64'};
  const tcBg = {agent:'rgba(184,148,74,0.10)',command:'rgba(90,107,122,0.10)',automation:'rgba(90,138,100,0.10)'};
  const toolIcons = cliTools.slice(0,8).map(t => {
    const synced = s.syncedWith && s.syncedWith.includes(t.key);
    return '<span class="skill-tool-icon '+(synced?'synced':'unsynced')+'" style="background:'+(synced?t.color+'22':'rgba(154,154,154,0.08)')+';color:'+(synced?t.color:'var(--fg-ghost)')+';--tool-color:'+t.color+'" title="'+t.name+(synced?' — synced':' — click to sync')+'" onclick="toggleSkillTool(\''+s.name+'\',\''+t.key+'\',this)">'+t.icon+'</span>';
  }).join('');
  return '<div class="card skill-card" data-od-id="skill-card-'+s.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'">'
    +'<div class="card-head">'
      +'<div class="card-icon" style="background:'+(tcBg[s.type]||'rgba(45,45,45,0.06)')+';color:'+(tc[s.type]||'var(--fg-ghost)')+'">'
        +'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>'
      +'</div>'
      +'<div style="flex:1;min-width:0">'
        +'<div class="card-title">'+s.name+' <span class="tag" style="color:'+tc[s.type]+';border-color:'+tc[s.type]+'30">'+s.type+'</span></div>'
        +'<div class="card-subtitle">'+s.software+' · '+s.source+'</div>'
      +'</div>'
    +'</div>'
    +'<div class="card-desc">'+s.desc+'</div>'
    +'<div class="card-meta">'
      +'<div class="card-meta-item"><span class="label">Type</span><span class="value">'+s.type+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Software</span><span class="value">'+s.software+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Status</span><span class="value">'+(s.enabled?'Enabled':'Disabled')+'</span></div>'
    +'</div>'
    +'<div class="skill-tool-icons">'+toolIcons+'</div>'
    +'<div class="card-divider"></div>'
    +'<div class="card-footer">'
      +'<div class="card-footer-left"><span class="badge '+(s.enabled?'success':'outline')+'">'+(s.enabled?'Active':'Inactive')+'</span></div>'
      +'<div class="card-footer-right">'
        +'<button class="btn btn-secondary btn-sm">Edit</button>'
        +'<button class="btn-icon btn-sm" aria-label="More actions"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
      +'</div>'
    +'</div>'
  +'</div>';
}

function renderMCPServer(s) {
  const healthColor = s.healthy ? 'var(--success)' : 'var(--error)';
  const healthBg = s.healthy ? 'rgba(90,138,100,0.10)' : 'rgba(184,90,66,0.10)';
  return '<div class="card mcp-card" data-od-id="mcp-card-'+s.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'">'
    +'<div class="card-head">'
      +'<div class="card-icon" style="background:'+healthBg+';color:'+healthColor+'">'
        +'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/></svg>'
      +'</div>'
      +'<div style="flex:1;min-width:0">'
        +'<div class="card-title">'+s.name+' <span class="badge '+(s.healthy?'success':'error')+'">'+(s.healthy?'Healthy':'Unreachable')+'</span></div>'
        +'<div class="card-subtitle">'+s.endpoint+'</div>'
      +'</div>'
    +'</div>'
    +'<div class="card-meta">'
      +'<div class="card-meta-item"><span class="label">Auth</span><span class="value">'+s.auth+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Tools</span><span class="value">'+s.tools+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Last Check</span><span class="value">'+s.lastChecked+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Group</span><span class="value">'+(s.group||'Default')+'</span></div>'
    +'</div>'
    +'<div class="card-divider"></div>'
    +'<div class="card-footer">'
      +'<div class="card-footer-left"><span class="badge '+(s.healthy?'success':'error')+'">'+(s.healthy?'Connected':'Disconnected')+'</span></div>'
      +'<div class="card-footer-right">'
        +'<button class="btn btn-secondary btn-sm" onclick="showNotification(\'Health check: '+s.name+' '+(s.healthy?'OK':'FAIL')+'\',\''+(s.healthy?'success':'error')+'\')">Check Health</button>'
        +'<button class="btn-icon btn-sm" aria-label="More actions"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
      +'</div>'
    +'</div>'
  +'</div>';
}

function renderRule(r) {
  const id = r.name.toLowerCase().replace(/[^a-z0-9]/g,'-');
  const isActive = r.active;
  const typeIcons = {
    md:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>',
    mdc:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M16 18l6-6-6-6"/><path d="M8 6l-6 6 6 6"/></svg>'
  };
  const typeBgs = {md:'rgba(90,107,122,0.10)',mdc:'rgba(90,138,100,0.10)'};
  const icon = typeIcons[r.type] || '<span style="font-size:12px;font-weight:600;font-family:var(--font-mono)">.'+r.type+'</span>';
  const bg = typeBgs[r.type] || 'rgba(45,45,45,0.06)';
  const badge = isActive ? '<span class="badge success">Active</span>' : '<span class="badge outline">Inactive</span>';
  return '<div class="card rule-card" data-od-id="rule-card-'+id+'">'
    +'<div class="card-head">'
      +'<div class="card-icon" style="background:'+bg+';color:var(--fg-muted)">'+icon+'</div>'
      +'<div style="flex:1;min-width:0">'
        +'<div class="card-title">'+r.name+' '+badge+'</div>'
        +'<div class="card-subtitle">'+r.software+' · '+r.size+'</div>'
      +'</div>'
    +'</div>'
    +'<div class="card-meta">'
      +'<div class="card-meta-item"><span class="label">Type</span><span class="value">.'+r.type+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Category</span><span class="value">'+(r.category||'general')+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Size</span><span class="value">'+r.size+'</span></div>'
      +'<div class="card-meta-item"><span class="label">Modified</span><span class="value">'+r.modified+'</span></div>'
    +'</div>'
    +'<div class="card-divider"></div>'
    +'<div class="card-footer">'
      +'<div class="card-footer-left">'+badge+'</div>'
      +'<div class="card-footer-right">'
        +'<button class="btn btn-secondary btn-sm">Edit</button>'
        +'<button class="btn-icon btn-sm" aria-label="More actions"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
      +'</div>'
    +'</div>'
  +'</div>';
}

function renderPlugin(p) {
  const enabledToggle = p.enabled
    ? '<div class="toggle-wrap"><div class="toggle on" role="switch" aria-checked="true" aria-label="'+p.name+' enabled" onclick="this.classList.toggle(\'on\');this.setAttribute(\'aria-checked\',this.classList.contains(\'on\'));showNotification(\''+p.name+' '+(p.enabled?'disabled':'enabled')+'\',\'info\')"></div></div>'
    : '<div class="toggle-wrap"><div class="toggle" role="switch" aria-checked="false" aria-label="'+p.name+' enabled" onclick="this.classList.toggle(\'on\');this.setAttribute(\'aria-checked\',this.classList.contains(\'on\'));showNotification(\''+p.name+' '+(p.enabled?'disabled':'enabled')+'\',\'info\')"></div></div>';
  const syncedChips = (p.syncedWith||[]).map(toolKey => {
    const tool = cliTools.find(t => t.key === toolKey);
    if(!tool) return '';
    return '<span class="cli-sync-chip synced" onclick="syncPluginToTool(\''+p.name+'\',\''+toolKey+'\',this)"><span class="chip-icon" style="background:'+tool.color+'22;color:'+tool.color+'">'+tool.icon+'</span><span class="chip-label">'+tool.name+'</span><span class="chip-status"><svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg></span></span>';
  }).join('');
  const unsyncedChips = cliTools.filter(t => !p.syncedWith || !p.syncedWith.includes(t.key)).slice(0,3).map(tool => {
    return '<span class="cli-sync-chip unsynced" onclick="syncPluginToTool(\''+p.name+'\',\''+tool.key+'\',this)"><span class="chip-icon" style="background:'+tool.color+'22;color:'+tool.color+'">'+tool.icon+'</span><span class="chip-label">'+tool.name+'</span><span class="chip-status"><svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg></span></span>';
  }).join('');
  return '<div class="card plugin-card" data-od-id="plugin-card-'+p.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'" data-name="'+p.name+'">'
    +'<div class="plugin-card-head">'
    +'<div style="flex:1;min-width:0">'
    +'<div style="font-weight:600;color:var(--fg-title);font-size:14px">'+p.name+'</div>'
    +'<div class="plugin-card-meta">v'+p.version+' · '+p.author+' · '+p.software+'</div>'
    +'</div>'
    +enabledToggle
    +'</div>'
    +'<div class="plugin-card-body">'
    +'<div style="font-size:13px;color:var(--fg-muted);line-height:1.6">'+p.desc+'</div>'
    +'<div class="plugin-cli-row">'
    +'<span class="plugin-cli-label">Sync to</span>'
    +syncedChips
    +unsyncedChips
    +'</div>'
    +'</div>'
    +'</div>';
}

function renderBackup(b, idx) {
  const typeIcons = {scheduled:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',manual:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>',incremental:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/><polyline points="17 6 23 6 23 12"/></svg>',system:'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>','pre-update':'<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>'};
  const typeColors = {scheduled:'var(--info)',manual:'var(--accent)',incremental:'var(--success)',system:'var(--error)','pre-update':'var(--warn)'};
  const typeLabels = {scheduled:'Scheduled',manual:'Manual',incremental:'Incremental',system:'System','pre-update':'Pre-update'};
  return '<div class="setting-group" data-od-id="backup-card-'+idx+'">'
    +'<div style="display:flex;align-items:center;gap:10px;margin-bottom:12px">'
      +'<div style="width:36px;height:36px;border-radius:var(--radius-sm);background:rgba(45,45,45,0.05);border:1px solid var(--border);display:flex;align-items:center;justify-content:center;color:'+typeColors[b.type]+'">'
        +(typeIcons[b.type]||typeIcons.manual)
      +'</div>'
      +'<span style="display:inline-flex;align-items:center;gap:4px;font-size:10px;font-weight:600;color:'+typeColors[b.type]+';background:rgba(45,45,45,0.04);padding:2px 8px;border-radius:99px;border:1px solid var(--border)">'+typeLabels[b.type]+'</span>'
    +'</div>'
    +'<h4 style="margin-bottom:6px">'+b.name+'</h4>'
    +'<div style="font-size:12px;color:var(--fg-ghost);font-family:var(--font-mono);margin-bottom:4px">'+b.date+'</div>'
    +'<div style="display:flex;align-items:center;gap:8px;font-size:12px;color:var(--fg-muted);margin-bottom:10px">'
      +'<span>'+b.size+'</span><span style="color:var(--border)">·</span><span>'+b.files+' files</span>'
      +'<span style="display:inline-flex;align-items:center;gap:4px;margin-left:auto;color:var(--success)"><span style="width:5px;height:5px;border-radius:50%;background:var(--success)"></span>OK</span>'
    +'</div>'
    +'<div style="display:flex;gap:4px;margin-bottom:14px;flex-wrap:wrap">'+b.includes.map(t=>'<span class="tag">'+t+'</span>').join('')+'</div>'
    +'<div style="display:flex;gap:8px">'
      +'<button class="btn btn-secondary btn-sm" style="flex:1" onclick="restoreBackup(\''+b.name+'\',\''+b.date+'\')">Restore</button>'
      +'<button class="btn btn-ghost btn-sm">Delete</button>'
    +'</div>'
  +'</div>';
}

function renderAgent(a) {
  const chips = agentTargetTools.map(t => {
    const sel = a.targets.includes(t.key);
    return '<span class="target-chip'+(sel?' selected':'')+'" data-key="'+t.key+'" data-od-id="target-chip-'+a.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'-'+t.key+'" title="'+t.name+'">'
      +'<span class="chip-dot" style="background:'+(sel?t.color:'rgba(154,154,154,0.3)')+'"></span>'
      +'<span class="chip-abbr">'+t.abbr+'</span></span>';
  }).join('');
  const count = a.targets.length;
  return '<div class="agent-card" data-od-id="agent-card-'+a.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'">'
    +'<div class="agent-card-header">'
    +'<div class="agent-icon">'+a.emoji+'</div>'
    +'<div><div class="agent-name">'+a.name+'</div>'
    +'<div class="agent-dept">'+a.department+' · '+a.source+'</div></div></div>'
    +'<div class="agent-desc">'+a.desc+'</div>'
    +'<div class="agent-targets-section">'
    +'<div class="agent-targets-label">Install to · '+count+' selected</div>'
    +'<div class="target-grid">'+chips+'</div></div>'
    +'<div class="agent-actions-row">'
    +'<button class="btn btn-primary btn-sm" onclick="showNotification(\''+a.name+' installed to '+count+' tools\',\'success\')">Install ('+count+')</button>'
    +'<button class="btn btn-secondary btn-sm" onclick="showNotification(\'Viewing '+a.name+' details\',\'info\')">View</button>'
    +'<button class="btn-icon btn-sm"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
    +'</div></div>';
}

function renderSources(data) {
  data = data || sourcesData;
  const el = document.getElementById('sourcesList');
  if(!el) return;
  el.innerHTML = data.map(s => {
    const statusBadge = s.installed
      ? '<span class="badge success">Installed</span>'
      : '<span class="badge warn">Pending</span>';
    const typeIcon = s.type === 'market'
      ? '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round"><path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>'
      : '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fg-muted)" stroke-width="2" stroke-linecap="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>';
    const repoIcon = s.repoType === 'GitHub'
      ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>'
      : '<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 01-.3-.94l1.22-3.78 2.44-7.51A.43.43 0 014.86 2a.42.42 0 01.52.34l1.58 7.16h6.08L14.62 2.3a.42.42 0 01.52-.34c.2.05.33.22.39.4l2.44 7.51 1.22 3.78a.84.84 0 01-.3.94l-.24.01z"/></svg>';
    const actionBtn = s.installed
      ? '<button class="btn btn-secondary btn-sm" onclick="showNotification(\''+s.name+' synced\',\'success\')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/></svg><span>Sync</span></button>'
      : '<button class="btn btn-primary btn-sm" onclick="showNotification(\''+s.name+' installed\',\'success\')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg><span>Install</span></button>';
    return '<div class="card source-card" data-od-id="source-card-'+s.name.toLowerCase().replace(/[^a-z0-9]/g,'-')+'">'
      +'<div class="source-card-head">'
      +'<div class="source-card-icon">'+typeIcon+'</div>'
      +'<div style="flex:1;min-width:0">'
      +'<div class="source-card-title"><span class="source-name-text">'+s.name+'</span>'+statusBadge+'</div>'
      +'<div class="source-card-subtitle">'+s.repoType+' <span class="sep">·</span> '+s.plugins+' plugins</div>'
      +'</div>'
      +'</div>'
      +'<div class="source-card-notes" title="'+s.notes+'">'+s.notes+'</div>'
      +'<div class="source-card-url" title="'+s.url+'">'
      +repoIcon+'<span class="url-text">'+s.url+'</span>'
      +'<a class="url-link" href="'+s.url+'" target="_blank" title="Open in browser">'
      +'<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>'
      +'</a></div>'
      +'<div class="source-card-path">'
      +'<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>'
      +'<span class="path-text">'+s.path+'</span></div>'
      +'<div class="source-card-footer">'
      +'<div class="btn-group">'
      +'<button class="btn-icon btn-sm" title="View details" onclick="showNotification(\'Opening '+s.name+' details\',\'info\')"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg></button>'
      +'<button class="btn-icon btn-sm" title="Edit notes"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg></button>'
      +'<button class="btn-icon btn-sm" title="More"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></svg></button>'
      +'</div>'
      +actionBtn
      +'</div>'
      +'</div>';
  }).join('');
}

function renderMarketplace(data) {
  data = data || marketplacePlugins;
  const grid = document.getElementById('marketplaceGrid');
  if(!grid) return;
  if(!data.length){
    grid.innerHTML='<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg><h3>No plugins found</h3><p>Try adjusting your search or filters.</p></div>';
    return;
  }
  grid.innerHTML = data.map(p => {
    const installedDot = p.installed ? '<span class="installed-dot" title="Installed"></span>' : '';
    const sourceBadge = '<span class="source-badge">'+p.source+'</span>';
    const tags = p.categories.map(c => '<span class="tag">'+c+'</span>').join('');
    const installBtn = p.installed
      ? '<button class="btn btn-secondary btn-sm" onclick="event.stopPropagation();showNotification(\''+p.name+' already installed\',\'info\')"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg><span>Installed</span></button>'
      : '<button class="btn btn-primary btn-sm" onclick="event.stopPropagation();installMarketplacePlugin(\''+p.name+'\')"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg><span>Install</span></button>';
    return '<div class="card marketplace-card" data-od-id="marketplace-card-'+p.name+'" onclick="openPluginDetail(marketplacePlugins.find(x=>x.name===\''+p.name+'\'))">'
      +'<div class="marketplace-card-head">'
        +'<div class="marketplace-card-icon" style="background:'+p.color+'22;color:'+p.color+'">'+p.icon+'</div>'
        +'<div class="marketplace-card-info">'
          +'<div class="marketplace-card-name">'+p.name+installedDot+'</div>'
          +'<div class="marketplace-card-author">by '+p.author+' '+sourceBadge+'</div>'
        +'</div>'
      +'</div>'
      +'<div class="marketplace-card-desc">'+p.desc+'</div>'
      +'<div class="marketplace-card-tags">'+tags+'</div>'
      +'<div class="marketplace-card-meta">'
        +'<span><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>'+p.downloads.toLocaleString()+'</span>'
        +'<span><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>'+p.stars+'</span>'
      +'</div>'
      +'<div class="marketplace-card-footer">'
        +'<span class="version">v'+p.version+'</span>'
        +'<div class="btn-group">'
          +'<button class="btn btn-icon" onclick="event.stopPropagation();openPluginDetail(marketplacePlugins.find(x=>x.name===\''+p.name+'\'))" title="Details"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg></button>'
          +installBtn
        +'</div>'
      +'</div>'
    +'</div>';
  }).join('');
}

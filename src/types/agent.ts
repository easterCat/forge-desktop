export interface Agent {
  id: string;
  name: string;
  description: string;
  emoji?: string;
  color?: string;
  department: string;
  content: string;
  source: string;
  tags?: string;
  installedTargets?: string;
  isCustom: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AgentImportResult {
  imported: number;
  skipped: number;
}

export interface AgentDepartment {
  id: string;
  name: string;
  emoji: string;
  count: number;
}

// Static department definitions matching agency-agents-zh structure
export const AGENT_DEPARTMENTS: AgentDepartment[] = [
  { id: 'academic', name: '学术部', emoji: '🎓' },
  { id: 'design', name: '设计部', emoji: '🎨' },
  { id: 'engineering', name: '工程部', emoji: '⚙️' },
  { id: 'finance', name: '金融部', emoji: '💰' },
  { id: 'game-development', name: '游戏开发', emoji: '🎮' },
  { id: 'hr', name: '人力资源', emoji: '👥' },
  { id: 'legal', name: '法务部', emoji: '⚖️' },
  { id: 'marketing', name: '营销部', emoji: '📢' },
  { id: 'paid-media', name: '付费媒体', emoji: '📺' },
  { id: 'product', name: '产品部', emoji: '📦' },
  { id: 'project-management', name: '项目管理', emoji: '📋' },
  { id: 'sales', name: '销售部', emoji: '💼' },
  { id: 'spatial-computing', name: '空间计算', emoji: '🥽' },
  { id: 'specialized', name: '专项部', emoji: '🔬' },
  { id: 'strategy', name: '战略部', emoji: '♟️' },
  { id: 'supply-chain', name: '供应链', emoji: '🚚' },
  { id: 'support', name: '支持部', emoji: '🛟' },
  { id: 'testing', name: '测试部', emoji: '🧪' },
].map(d => ({ ...d, count: 0 }));

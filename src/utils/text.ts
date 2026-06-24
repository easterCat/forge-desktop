/**
 * 将描述文本规范化到目标长度区间（30–40 字符，约 35 字）。
 * - 过短：在末尾补充上下文填充至 30 字符
 * - 过长：截断到 37 字符 + "..."
 * - 空值：返回占位文本
 */
export function normalizeDesc(desc: string, context?: string): string {
  if (!desc || !desc.trim()) {
    return context
      ? `${context}，可通过 Forge 一键安装与版本管理。`
      : '可通过 Forge 一键安装与版本管理，支持跨平台同步。'
  }

  const trimmed = desc.trim()

  // 已在目标区间内
  if (trimmed.length >= 30 && trimmed.length <= 40) {
    return trimmed
  }

  // 过长：截断到 37 字符 + "..."
  if (trimmed.length > 40) {
    return trimmed.slice(0, 37) + '...'
  }

  // 过短：根据上下文补充
  const fillers: Record<string, string> = {
    agent: '，支持智能推理与自主决策，自动完成任务。',
    command: '，支持快捷调用与参数化配置，提升效率。',
    automation: '，自动化执行重复操作，减少人工干预。',
    default: '，可通过 Forge 一键安装与版本管理。',
  }

  const filler = context && fillers[context] ? fillers[context] : fillers.default
  const result = trimmed + filler

  // 填充后如果超过 40，截断
  if (result.length > 40) {
    return result.slice(0, 37) + '...'
  }
  return result
}

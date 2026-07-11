<template>
  <div class="agent-card">
    <!-- Card Header -->
    <div class="agent-card-header">
      <div class="agent-icon">
        <span class="agent-abbr">{{ agent.emoji || agent.name?.slice(0, 2).toUpperCase() }}</span>
      </div>
      <div class="agent-info">
        <div class="agent-name">{{ agent.name }}</div>
        <div class="agent-dept">{{ agent.department }} · {{ agent.source }}</div>
      </div>
    </div>

    <!-- Description -->
    <div class="agent-desc">{{ agent.description }}</div>

    <!-- Install Targets Section -->
    <div class="agent-targets-section">
      <div class="agent-targets-label">
        Install to · {{ selectedTargets.size }} selected
      </div>
      <div class="target-grid">
        <button
          v-for="tool in TARGET_TOOLS"
          :key="tool.key"
          class="target-chip"
          :class="{ selected: selectedTargets.has(tool.key) }"
          @click.stop="toggleTarget(tool.key)"
        >
          <span
            class="chip-dot"
            :style="{ background: selectedTargets.has(tool.key) ? tool.color : 'rgba(154,154,154,0.3)' }"
          ></span>
          <span class="chip-abbr">{{ tool.abbr }}</span>
        </button>
      </div>
    </div>

    <!-- Actions Row -->
    <div class="agent-actions-row">
      <button
        class="btn btn-primary btn-sm"
        :disabled="selectedTargets.size === 0"
        @click.stop="$emit('install', Array.from(selectedTargets))"
      >
        Install ({{ selectedTargets.size }})
      </button>
      <button class="btn btn-secondary btn-sm" @click.stop="$emit('click')">
        View
      </button>
      <button class="btn-icon btn-sm" title="More actions" @click.stop>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="5" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="12" cy="19" r="1" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Agent } from '@/types/agent'

const props = defineProps<{
  agent: Agent
}>()

defineEmits<{
  (e: 'click'): void
  (e: 'install', targets: string[]): void
}>()

// Target tools from design prototype (forge-cross-platform-glass.html line 1146)
const TARGET_TOOLS = [
  { key: 'claude-code', abbr: 'CC', name: 'Claude Code', color: '#D97706' },
  { key: 'cursor', abbr: 'CU', name: 'Cursor', color: '#7C3AED' },
  { key: 'copilot', abbr: 'CO', name: 'Copilot', color: '#059669' },
  { key: 'gemini-cli', abbr: 'GM', name: 'Gemini CLI', color: '#2563EB' },
  { key: 'opencode', abbr: 'OC', name: 'OpenCode', color: '#0891B2' },
  { key: 'deepseek', abbr: 'DS', name: 'DeepSeek', color: '#4F46E5' },
  { key: 'codex', abbr: 'CX', name: 'Codex', color: '#9333EA' },
  { key: 'openclaw', abbr: 'CL', name: 'OpenClaw', color: '#B45309' },
  { key: 'mimo-code', abbr: 'MI', name: 'MiMo Code', color: '#0D9488' },
] as const

const selectedTargets = ref<Set<string>>(new Set())

function toggleTarget(key: string) {
  const next = new Set(selectedTargets.value)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  selectedTargets.value = next
}

onMounted(() => {
  if (props.agent.installedTargets) {
    try {
      const parsed = JSON.parse(props.agent.installedTargets) as string[]
      selectedTargets.value = new Set(parsed)
    } catch {
      selectedTargets.value = new Set()
    }
  }
})
</script>

<style scoped>
.agent-card {
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  padding: 18px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  transition: all var(--t-base);
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: hidden;
  height: 100%;
}

.agent-card:hover {
  background: rgba(255, 255, 255, 0.58);
  border-color: rgba(255, 255, 255, 0.50);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
}

/* Card Header */
.agent-card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.agent-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-sm);
  background: rgba(45, 45, 45, 0.06);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.agent-abbr {
  font-size: 16px;
  font-weight: 700;
  color: var(--fg-muted);
  font-family: var(--font-mono);
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg-title);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-dept {
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-top: 2px;
}

/* Description */
.agent-desc {
  font-size: 13px;
  color: var(--fg-muted);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin: 0;
}

/* Install Targets */
.agent-targets-section {
  margin-top: auto;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.40);
}

.agent-targets-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--fg-ghost);
  margin-bottom: 8px;
}

.target-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.target-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg-muted);
  border: 1px solid rgba(255, 255, 255, 0.32);
  cursor: pointer;
  transition: all var(--t-fast);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  user-select: none;
}

.target-chip:hover {
  border-color: rgba(255, 255, 255, 0.28);
  background: rgba(255, 255, 255, 0.22);
  color: var(--fg);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.target-chip.selected {
  background: rgba(45, 45, 45, 0.10);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}

.target-chip .chip-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1.5px solid rgba(255, 255, 255, 0.30);
}

.target-chip.selected .chip-dot {
  border-color: transparent;
}

.target-chip .chip-abbr {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

/* Actions Row */
.agent-actions-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 12px;
}

.agent-actions-row .btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Button icon */
.btn-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
  background: rgba(255, 255, 255, 0.30);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  cursor: pointer;
  transition: all var(--t-fast);
}

.btn-icon:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.40);
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
</style>

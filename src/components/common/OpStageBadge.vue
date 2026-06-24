<script setup lang="ts">
export type OpStage =
  | 'preparing'
  | 'downloading'
  | 'installing'
  | 'verifying'
  | 'completed'
  | 'failed'
  | 'cancelled'

interface Props {
  stage: OpStage
  label?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const STAGE_CONFIG: Record<OpStage, { label: string; icon: string }> = {
  preparing:  { label: 'Preparing',   icon: '⏳' },
  downloading: { label: 'Downloading', icon: '↓' },
  installing:  { label: 'Installing',  icon: '⚙' },
  verifying:   { label: 'Verifying',   icon: '✓' },
  completed:   { label: 'Completed',   icon: '✔' },
  failed:      { label: 'Failed',      icon: '✕' },
  cancelled:   { label: 'Cancelled',  icon: '⊘' },
}
</script>

<template>
  <span
    :class="['op-stage', stage]"
    :disabled="disabled"
    :title="STAGE_CONFIG[stage].label"
  >
    <span class="op-icon">{{ STAGE_CONFIG[stage].icon }}</span>
    <span class="op-label">{{ label ?? STAGE_CONFIG[stage].label }}</span>
  </span>
</template>

<style scoped>
.op-stage {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 600;
  font-family: var(--font-mono);
  letter-spacing: 0.02em;
  transition: all var(--t-fast);
  cursor: default;
}

.op-stage.preparing {
  background: rgba(90, 107, 122, 0.10);
  color: var(--info);
}

.op-stage.downloading {
  background: rgba(90, 107, 122, 0.10);
  color: var(--info);
}

.op-stage.installing {
  background: rgba(184, 148, 74, 0.10);
  color: var(--warn);
}

.op-stage.verifying {
  background: rgba(184, 148, 74, 0.10);
  color: var(--warn);
}

.op-stage.completed {
  background: rgba(90, 138, 100, 0.12);
  color: var(--success);
}

.op-stage.failed {
  background: rgba(184, 90, 66, 0.12);
  color: var(--error);
}

.op-stage.cancelled {
  background: rgba(255, 255, 255, 0.40);
  color: var(--fg-ghost);
}

/* Hover: background deepens 10% */
.op-stage.preparing:hover {
  background: rgba(90, 107, 122, 0.20);
}

.op-stage.downloading:hover {
  background: rgba(90, 107, 122, 0.20);
}

.op-stage.installing:hover {
  background: rgba(184, 148, 74, 0.20);
}

.op-stage.verifying:hover {
  background: rgba(184, 148, 74, 0.20);
}

.op-stage.completed:hover {
  background: rgba(90, 138, 100, 0.22);
}

.op-stage.failed:hover {
  background: rgba(184, 90, 66, 0.22);
}

.op-stage.cancelled:hover {
  background: rgba(255, 255, 255, 0.50);
}

/* Disabled */
.op-stage:disabled,
.op-stage[disabled] {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.op-icon {
  line-height: 1;
}

.op-label {
  white-space: nowrap;
}
</style>

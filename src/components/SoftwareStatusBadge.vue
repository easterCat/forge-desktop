<template>
  <span class="status-badge" :class="`status-${status}`" :title="STATUS_LABELS[status]">
    <component :is="iconComponent" :size="iconSize" class="status-icon" />
    <span class="status-label">{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { SoftwareStatus } from '@/types/software';
import { STATUS_LABELS } from '@/composables/useSoftwareStatus';
import SuccessIcon from '@/components/icons/status/SuccessIcon.vue';
import DownloadIcon from '@/components/icons/action/DownloadIcon.vue';
import ArrowUpIcon from '@/components/icons/action/ArrowUpIcon.vue';
import ErrorIcon from '@/components/icons/status/ErrorIcon.vue';

const props = withDefaults(defineProps<{
  status: SoftwareStatus;
  /** Compact mode shows icon-only (used in dense lists). */
  compact?: boolean;
}>(), {
  compact: false,
});

const label = computed(() => STATUS_LABELS[props.status]);
const iconSize = computed(() => (props.compact ? 12 : 14));

const iconComponent = computed(() => {
  switch (props.status) {
    case 'installed': return SuccessIcon;
    case 'notinstalled': return DownloadIcon;
    case 'outdated': return ArrowUpIcon;
    case 'unknown': return ErrorIcon;
    default: return ErrorIcon;
  }
});
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  line-height: 1.2;
  white-space: nowrap;
  border: 1px solid transparent;
}

.status-icon {
  flex-shrink: 0;
}

.status-installed {
  color: var(--success);
  background: var(--success-bg);
  border-color: rgba(16, 185, 129, 0.3);
}

.status-notinstalled {
  color: var(--fg-ghost);
  background: var(--bg-input);
  border-color: var(--border);
}

.status-outdated {
  color: var(--warn);
  background: var(--warn-bg);
  border-color: rgba(245, 158, 11, 0.3);
}

.status-unknown {
  color: var(--error);
  background: var(--error-bg);
  border-color: rgba(239, 68, 68, 0.3);
}

.status-badge.compact .status-label {
  display: none;
}
</style>

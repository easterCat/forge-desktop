<template>
  <span
    class="health-badge"
    :class="[status, size, { 'with-label': showLabel }]"
    role="img"
    :aria-label="`Server status: ${label}`"
  >
    <span class="health-dot" aria-hidden="true"></span>
    <span v-if="showLabel" class="health-label">{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

type HealthStatus = 'online' | 'offline' | 'error' | 'checking';

interface Props {
  status: HealthStatus;
  showLabel?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  showLabel: false,
  size: 'md',
});

const label = computed(() => {
  const labels: Record<HealthStatus, string> = {
    online: 'Online',
    offline: 'Offline',
    error: 'Error',
    checking: 'Checking...',
  };
  return labels[props.status];
});
</script>

<style scoped>
.health-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  --dot-size: 8px;
  --dot-color: #71717A;
}

.health-badge.sm {
  --dot-size: 6px;
}

.health-badge.md {
  --dot-size: 8px;
}

.health-badge.lg {
  --dot-size: 10px;
}

.health-dot {
  width: var(--dot-size);
  height: var(--dot-size);
  border-radius: 50%;
  background: var(--dot-color);
  transition: background-color 200ms ease, transform 200ms ease;
  flex-shrink: 0;
}

/* Online state - Green */
.health-badge.online .health-dot {
  background: var(--success);
  box-shadow: 0 2px 6px rgba(16, 185, 129, 0.35);
}

/* Offline state - Red */
.health-badge.offline .health-dot {
  background: var(--error);
}

/* Error state - Amber/Yellow */
.health-badge.error .health-dot {
  background: var(--warn);
}

/* Checking state - Cyan with pulse animation */
.health-badge.checking .health-dot {
  background: var(--info);
  animation: pulse 1.5s ease-in-out infinite;
}

.health-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted);
}

.health-badge.online .health-label {
  color: var(--success);
}

.health-badge.offline .health-label {
  color: var(--error);
}

.health-badge.error .health-label {
  color: var(--warn);
}

.health-badge.checking .health-label {
  color: var(--info);
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.7;
  }
}
</style>

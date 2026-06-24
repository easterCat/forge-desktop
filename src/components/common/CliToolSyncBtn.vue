<!--
  CliToolSyncBtn - 32px sync button used in the installed-plugins card.

  Shows the CLI tool's brand icon (via ToolIcon) with a green check badge
  when the plugin has been synced to that tool. Falls back to two-letter
  initials if no icon is mapped. Renders a spinner while a sync is in flight.
-->
<template>
  <button
    class="cli-tool-sync-btn"
    :class="{
      synced: synced,
      syncing: syncing,
      unsyncing: unsyncing,
    }"
    :title="title"
    :disabled="disabled"
    @click.stop="emit('click')"
  >
    <span v-if="syncing || unsyncing" class="sync-spinner" aria-hidden="true"></span>
    <template v-else>
      <ToolIcon :tool-key="toolKey" :alt="toolName" />
      <svg
        v-if="synced"
        class="cli-tool-synced-badge"
        width="10"
        height="10"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="3"
        aria-hidden="true"
      >
        <polyline points="20 6 9 17 4 12" />
      </svg>
    </template>
  </button>
</template>

<script setup lang="ts">
import ToolIcon from '@/components/common/ToolIcon.vue';

defineOptions({ name: 'CliToolSyncBtn' });

withDefaults(defineProps<{
  /** The CLI tool key used to look up the brand icon. */
  toolKey: string;
  /** The CLI tool display name (used for the alt text). */
  toolName: string;
  /** Whether this tool has the plugin synced. Controls the green badge. */
  synced?: boolean;
  /** True while a sync is in flight. Shows the spinner. */
  syncing?: boolean;
  /** True while an unsync is in flight. Shows the spinner. */
  unsyncing?: boolean;
  /** Disables the button (typically while a request is being made). */
  disabled?: boolean;
  /** Tooltip text shown on hover. */
  title?: string;
}>(), {
  synced: false,
  syncing: false,
  unsyncing: false,
  disabled: false,
  title: '',
});

const emit = defineEmits<{
  (e: 'click'): void;
}>();
</script>

<style scoped>
.cli-tool-sync-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--fg-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.2s ease;
  overflow: hidden;
}

.cli-tool-sync-btn :deep(img),
.cli-tool-sync-btn :deep(.tool-icon-img) {
  width: 70%;
  height: 70%;
  object-fit: contain;
  pointer-events: none;
  user-select: none;
}

.cli-tool-sync-btn :deep(.tool-icon-fallback) {
  font-size: 10px;
  font-weight: 700;
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  letter-spacing: 0.02em;
  line-height: 1;
  color: inherit;
  pointer-events: none;
  user-select: none;
}

.cli-tool-sync-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-bg, rgba(245, 158, 11, 0.1));
}

.cli-tool-sync-btn:disabled {
  cursor: wait;
  opacity: 0.7;
}

.cli-tool-sync-btn.synced {
  color: var(--success);
  border-color: var(--success);
  background: var(--success-bg, rgba(34, 197, 94, 0.15));
  box-shadow: 0 2px 8px rgba(34, 197, 94, 0.25);
  animation: synced-pulse 2s ease-in-out infinite;
}

.cli-tool-sync-btn.synced:hover:not(:disabled) {
  color: var(--error);
  border-color: var(--error);
  background: rgba(239, 68, 68, 0.1);
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.25);
  animation: none;
}

@keyframes synced-pulse {
  0%, 100% {
    box-shadow: 0 2px 8px rgba(34, 197, 94, 0.25);
  }
  50% {
    box-shadow: 0 2px 12px rgba(34, 197, 94, 0.35);
  }
}

.cli-tool-synced-badge {
  position: absolute;
  right: -2px;
  bottom: -2px;
  background: var(--success);
  color: white;
  border-radius: 4px;
  padding: 1px;
  box-shadow: 0 2px 6px rgba(34, 197, 94, 0.4);
  animation: badge-glow 2s ease-in-out infinite;
}

@keyframes badge-glow {
  0%, 100% {
    box-shadow: 0 2px 6px rgba(34, 197, 94, 0.4);
  }
  50% {
    box-shadow: 0 2px 10px rgba(34, 197, 94, 0.5);
  }
}

.sync-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: cli-tool-sync-spin 0.7s linear infinite;
}

@keyframes cli-tool-sync-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>

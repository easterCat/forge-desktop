<!--
  ToolIcon - Generic brand-icon renderer for CLI tools.

  Resolves a `toolKey` to the matching SVG in `src/assets/icons/tools/`
  and renders it as an <img> so each gradient/ID stays scoped to its
  own DOM tree (no cross-icon collisions on the same page).

  Source of truth: src/assets/icons/tools/*.svg
  New tools: drop a new SVG into that folder and add the key to TOOL_ICON_MAP.
-->
<template>
  <img
    v-if="src"
    :src="src"
    :alt="alt"
    :width="size"
    :height="size"
    :class="['tool-icon-img', $attrs.class]"
    :aria-hidden="alt ? undefined : 'true'"
    draggable="false"
  />
  <span v-else class="tool-icon-fallback" :aria-hidden="alt ? undefined : 'true'">
    {{ fallback }}
  </span>
</template>

<script setup lang="ts">
import { computed, useAttrs } from 'vue';

const props = withDefaults(defineProps<{
  /** The CLI tool key (e.g. "cursor", "claude-code") */
  toolKey: string;
  /** Pixel size for both width and height. The SVG itself is 48x48 — we scale
   *  with width/height attributes so it stays crisp at any size. */
  size?: number | string;
  /** Optional alt text. Omit for purely decorative usage. */
  alt?: string;
}>(), {
  size: 46,
  alt: '',
});

// Vite glob: import every tool SVG as a URL at build time. Eager = true so
// the URLs are resolved immediately and the result is a plain object.
// The path is relative to this component file: src/components/common/ →
// ../../assets/icons/tools/*.svg.
const iconModules = import.meta.glob<string>('../../assets/icons/tools/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
});

// Map of CLI tool key → SVG filename. Keep alphabetical so it's easy to scan.
const TOOL_ICON_MAP: Record<string, string> = {
  'claude-code': 'tool-claude-code.svg',
  'codex': 'tool-codex.svg',
  'copilot': 'tool-copilot.svg',
  'cursor': 'tool-cursor.svg',
  'deepseek-reasonix': 'tool-deepseek-reasonix.svg',
  'gemini-cli': 'tool-gemini-cli.svg',
  'hermes': 'tool-hermes.svg',
  'mimo-code': 'tool-mimo-code.svg',
  'openclaw': 'tool-openclaw.svg',
  'opencode': 'tool-opencode.svg',
  'qwen-code': 'tool-qwen-code.svg',
};

defineOptions({ name: 'ToolIcon', inheritAttrs: false });

// Suppress Vue's "Extraneous non-emits event listeners" warning for $attrs
// forwarding (we manually bind class above).
useAttrs();

const src = computed<string | null>(() => {
  const filename = TOOL_ICON_MAP[props.toolKey];
  if (!filename) return null;
  // glob keys look like "/Users/.../src/assets/icons/tools/tool-cursor.svg"
  // We look up the matching key by suffix to stay path-agnostic.
  const matchKey = Object.keys(iconModules).find((k) => k.endsWith(`/${filename}`));
  return matchKey ? iconModules[matchKey] : null;
});

// Fallback shown if the tool has no SVG yet: two-letter initials.
const fallback = computed<string>(() => {
  const key = props.toolKey || '';
  // Try to split on '-' for things like "gemini-cli" → "GC"
  const parts = key.split(/[-_\s]+/).filter(Boolean);
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase();
  return key.slice(0, 2).toUpperCase() || '??';
});
</script>

<style scoped>
.tool-icon-img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  user-select: none;
  pointer-events: none;
}

.tool-icon-fallback {
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  font-weight: 700;
  font-size: 12px;
  color: var(--fg-muted, #71717A);
  letter-spacing: 0.02em;
  line-height: 1;
  user-select: none;
}
</style>

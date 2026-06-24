<script setup lang="ts">
import { useTitlebar } from '@/composables/useTitlebar'
import { useGlassTheme } from '@/composables/useGlassTheme'

const { platform, minimize, toggleMaximize, close } = useTitlebar()
const { isWindowRounded } = useGlassTheme()
</script>

<template>
  <div class="titlebar" :class="{ rounded: isWindowRounded }">
    <!-- Mac traffic lights -->
    <div v-if="platform === 'macos'" class="titlebar-mac">
      <div class="traffic-light close" title="Close" @click="close" />
      <div class="traffic-light minimize" title="Minimize" @click="minimize" />
      <div class="traffic-light maximize" title="Maximize" @click="toggleMaximize" />
    </div>

    <!-- Center: app icon + name -->
    <div class="titlebar-center">
      <div class="app-icon">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
          <rect x="4" y="4" width="16" height="16" rx="3" fill="rgba(255,255,255,0.8)" />
          <rect x="8" y="8" width="8" height="8" rx="1.5" fill="#F5F3F0" />
        </svg>
      </div>
      <span>Forge</span>
    </div>

    <!-- Windows/Linux buttons -->
    <div v-if="platform !== 'macos'" class="titlebar-win">
      <div class="win-btn minimize" title="Minimize" @click="minimize">
        <svg viewBox="0 0 10 1" fill="currentColor"><rect width="10" height="1" /></svg>
      </div>
      <div class="win-btn maximize" title="Maximize" @click="toggleMaximize">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1"><rect x="0.5" y="0.5" width="9" height="9" rx="1" /></svg>
      </div>
      <div class="win-btn close" title="Close" @click="close">
        <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"><line x1="1" y1="1" x2="9" y2="9" /><line x1="9" y1="1" x2="1" y2="9" /></svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: var(--titlebar-h, 38px);
  min-height: var(--titlebar-h, 38px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  -webkit-app-region: drag;
  user-select: none;
  position: relative;
  z-index: 20;
}


.titlebar-mac {
  display: flex;
  align-items: center;
  gap: 8px;
}

.traffic-light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  position: relative;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.traffic-light::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 50%;
  opacity: 0;
  transition: opacity 150ms ease;
}

.traffic-light:hover::after {
  opacity: 1;
}

.traffic-light.close {
  background: #FF5F57;
}

.traffic-light.close::after {
  background: linear-gradient(180deg, #E0443E 0%, #C73E38 100%);
}

.traffic-light.minimize {
  background: #FEBC2E;
}

.traffic-light.minimize::after {
  background: linear-gradient(180deg, #DFA123 0%, #C49019 100%);
}

.traffic-light.maximize {
  background: #28C840;
}

.traffic-light.maximize::after {
  background: linear-gradient(180deg, #1AAB29 0%, #14982A 100%);
}

.titlebar-center {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-muted, #5C5C5C);
  display: flex;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.app-icon {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  background: var(--accent, #2D2D2D);
  display: flex;
  align-items: center;
  justify-content: center;
}

.titlebar-win {
  display: flex;
  align-items: center;
  gap: 2px;
}

.win-btn {
  width: 46px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  cursor: pointer;
  -webkit-app-region: no-drag;
  transition: background 150ms ease;
  color: var(--fg-muted, #5C5C5C);
}

.win-btn:hover {
}

.win-btn.close:hover {
}

.win-btn svg {
  width: 10px;
  height: 10px;
}
</style>

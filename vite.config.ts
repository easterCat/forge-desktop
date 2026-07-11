import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    // Audit 1.5 / P2-15: raised from chrome105 / safari13 to chrome130 /
    // safari17. The newer targets drop polyfills (e.g. native optional
    // chaining, top-level await, ??, class fields) that our codebase
    // already uses and that the old targets had to down-level.
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome130' : 'safari17',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        manualChunks: {
          'vendor-vue': ['vue', 'vue-router', 'pinia'],
          'vendor-tauri': ['@tauri-apps/api'],
          'vendor-utils': ['@vueuse/core'],
        },
      },
    },
  },
});

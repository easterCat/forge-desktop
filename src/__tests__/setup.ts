/**
 * Global test setup for Vitest
 *
 * Provides:
 * - localStorage polyfill (jsdom partial support)
 * - crypto.randomUUID polyfill
 * - window.matchMedia polyfill
 * - Tauri API core mock factory (can be overridden per-spec with vi.mock)
 * - IS_VITEST flag for runtime checks
 */

import { vi } from 'vitest';

// Mark test environment
process.env.IS_VITEST = '1';

// =============================================================================
// localStorage polyfill
// =============================================================================

const localStorageStore: Record<string, string> = {};

Object.defineProperty(globalThis, 'localStorage', {
  value: {
    getItem: vi.fn((key: string) => localStorageStore[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      localStorageStore[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete localStorageStore[key];
    }),
    clear: vi.fn(() => {
      for (const key of Object.keys(localStorageStore)) {
        delete localStorageStore[key];
      }
    }),
    get length() {
      return Object.keys(localStorageStore).length;
    },
    key: vi.fn((index: number) => Object.keys(localStorageStore)[index] ?? null),
  },
  writable: true,
});

// =============================================================================
// crypto.randomUUID polyfill
// =============================================================================

if (!globalThis.crypto?.randomUUID) {
  Object.defineProperty(globalThis.crypto ?? (globalThis as any).crypto, 'randomUUID', {
    value: vi.fn(() => {
      return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = (Math.random() * 16) | 0;
        const v = c === 'x' ? r : (r & 0x3) | 0x8;
        return v.toString(16);
      });
    }),
  });
}

// =============================================================================
// window.matchMedia polyfill
// =============================================================================

Object.defineProperty(globalThis, 'matchMedia', {
  writable: true,
  value: vi.fn((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(() => true),
  })),
});

// =============================================================================
// sessionStorage polyfill
// =============================================================================

const sessionStorageStore: Record<string, string> = {};

Object.defineProperty(globalThis, 'sessionStorage', {
  value: {
    getItem: vi.fn((key: string) => sessionStorageStore[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      sessionStorageStore[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete sessionStorageStore[key];
    }),
    clear: vi.fn(() => {
      for (const key of Object.keys(sessionStorageStore)) {
        delete sessionStorageStore[key];
      }
    }),
    get length() {
      return Object.keys(sessionStorageStore).length;
    },
    key: vi.fn((index: number) => Object.keys(sessionStorageStore)[index] ?? null),
  },
  writable: true,
});

// =============================================================================
// Tauri API core stub
// =============================================================================
// Individual specs should override with: vi.mock('@tauri-apps/api/core', ...)
// This stub prevents "module not found" in jsdom environment.

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({ success: true, data: null, error: null }),
}));

// =============================================================================
// Tauri event stub
// =============================================================================

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue({ unlisten: vi.fn() }),
  emit: vi.fn().mockResolvedValue(undefined),
}));

// =============================================================================
// Vue injection stub (for useErrorHandler / ClientSyncDialog)
// =============================================================================

vi.mock('vue', async () => {
  const actual = await vi.importActual('vue');
  return {
    ...actual,
    inject: vi.fn(() => undefined),
  };
});

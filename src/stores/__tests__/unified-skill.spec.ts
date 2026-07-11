import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useUnifiedSkillStore } from '../unified-skill';
import type { UnifiedPlugin } from '@/types/unified-plugin';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue({ unlisten: vi.fn() }),
  emit: vi.fn().mockResolvedValue(undefined),
}));

function makeSkill(overrides: Partial<UnifiedPlugin> = {}): UnifiedPlugin {
  return {
    id: 's1',
    name: 'test-skill',
    type: 'skill',
    source: { type: 'marketplace', marketplace: 'anthropic' },
    scope: 'user',
    installed: false,
    enabled: true,
    syncStatus: 'unknown',
    syncTargets: [],
    targetClients: [],
    tags: [],
    categories: [],
    description: '',
    ...overrides,
  };
}

describe('useUnifiedSkillStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('starts with empty skills', () => {
    const store = useUnifiedSkillStore();
    expect(store.skills).toEqual([]);
    expect(store.sources).toEqual([]);
    expect(store.repositories).toEqual([]);
  });

  it('skillsBySource returns grouped by 7 source types', () => {
    const store = useUnifiedSkillStore();
    const grouped = store.skillsBySource;
    expect(Object.keys(grouped)).toHaveLength(7);
    expect(grouped.marketplace).toEqual([]);
  });

  it('installedSkills filters installed', () => {
    const store = useUnifiedSkillStore();
    store.skills = [
      makeSkill({ id: '1', installed: true }),
      makeSkill({ id: '2', installed: false }),
    ];
    expect(store.installedSkills).toHaveLength(1);
  });

  it('filteredSkills applies search keyword', () => {
    const store = useUnifiedSkillStore();
    store.skills = [
      makeSkill({ id: '1', name: 'Code Review', description: 'Review code' }),
      makeSkill({ id: '2', name: 'Deploy', description: 'Deploy app' }),
    ];
    store.searchKeyword = 'review';
    expect(store.filteredSkills).toHaveLength(1);
    store.searchKeyword = '';
    expect(store.filteredSkills).toHaveLength(2);
  });

  it('filteredSkills applies selectedCategory filter', () => {
    const store = useUnifiedSkillStore();
    store.skills = [
      makeSkill({ id: '1', categories: ['dev'] }),
      makeSkill({ id: '2', categories: ['ops'] }),
    ];
    store.selectedCategory = 'dev';
    expect(store.filteredSkills).toHaveLength(1);
    store.selectedCategory = null;
    expect(store.filteredSkills).toHaveLength(2);
  });

  it('paginatedSkills slices correctly', () => {
    const store = useUnifiedSkillStore();
    store.skills = Array.from({ length: 25 }, (_, i) => makeSkill({ id: `s${i}` }));
    store.currentPage = 2;
    store.pageSize = 10;
    expect(store.paginatedSkills).toHaveLength(10);
    expect(store.paginatedSkills[0].id).toBe('s10');
  });

  it('totalPages computed from filteredSkills', () => {
    const store = useUnifiedSkillStore();
    store.skills = Array.from({ length: 25 }, (_, i) => makeSkill({ id: `s${i}` }));
    store.pageSize = 10;
    expect(store.totalPages).toBe(3);
  });

  it('reset clears all state', () => {
    const store = useUnifiedSkillStore();
    store.skills = [makeSkill()];
    store.searchKeyword = 'test';
    store.reset();
    expect(store.skills).toEqual([]);
    expect(store.searchKeyword).toBe('');
  });
});

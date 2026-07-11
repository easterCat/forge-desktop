import { describe, it, expect } from 'vitest';
import { normalizeDesc } from '../text';

describe('normalizeDesc', () => {
  it('returns placeholder for empty input', () => {
    expect(normalizeDesc('')).toBe('可通过 Forge 一键安装与版本管理，支持跨平台同步。');
  });

  it('returns placeholder for whitespace-only input', () => {
    expect(normalizeDesc('   ')).toBe('可通过 Forge 一键安装与版本管理，支持跨平台同步。');
  });

  it('returns placeholder with context for empty input', () => {
    expect(normalizeDesc('', 'agent')).toBe('agent，可通过 Forge 一键安装与版本管理。');
  });

  it('truncates to 37 + "..." when over 40 chars', () => {
    const long = '中文描述文字用于测试截断逻辑当超过四十个字符限制时会从第三十七个字符处截断加上省略号';
    expect(long.length).toBeGreaterThan(40);
    const result = normalizeDesc(long);
    expect(result).toContain('...');
    expect(result.length).toBe(40);
    expect(result.endsWith('...')).toBe(true);
  });

  it('returns input unchanged when in target range (30-40 chars)', () => {
    const mid = '中文描述恰好在三十到四十字符范围之间内容是很好的测试文本用于长描述校验';
    expect(mid.length).toBeGreaterThanOrEqual(30);
    expect(mid.length).toBeLessThanOrEqual(40);
    expect(normalizeDesc(mid)).toBe(mid);
  });

  it('appends agent filler to short input', () => {
    const result = normalizeDesc('短', 'agent');
    expect(result).toContain('智能推理');
  });

  it('appends command filler to short input', () => {
    const result = normalizeDesc('短', 'command');
    expect(result).toContain('快捷调用');
  });

  it('appends automation filler to short input', () => {
    const result = normalizeDesc('短', 'automation');
    expect(result).toContain('自动化');
  });

  it('appends default filler for unknown context', () => {
    const result = normalizeDesc('短', 'unknown-context');
    expect(result).toContain('Forge');
  });

  it('handles null input as empty', () => {
    expect(normalizeDesc(null as any)).toBe('可通过 Forge 一键安装与版本管理，支持跨平台同步。');
  });
});

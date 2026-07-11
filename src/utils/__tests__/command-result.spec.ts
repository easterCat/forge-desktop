/**
 * 统一测试：CommandResult 类型与辅助函数
 */
import { describe, it, expect } from 'vitest';
import { getCommandError, type CommandResult } from '@/utils/command-result';

describe('command-result', () => {
  describe('getCommandError', () => {
    it('returns error message when present', () => {
      const r: CommandResult = { success: false, error: 'disk full' };
      expect(getCommandError(r)).toBe('disk full');
    });

    it('falls back to default when error missing', () => {
      const r: CommandResult = { success: true };
      expect(getCommandError(r, 'oops')).toBe('oops');
    });

    it('uses built-in fallback when none provided', () => {
      const r: CommandResult = { success: false };
      expect(getCommandError(r)).toBe('Operation failed');
    });
  });
});
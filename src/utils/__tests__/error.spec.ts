import { describe, it, expect } from 'vitest';
import { extractError } from '../error';

describe('extractError', () => {
  it('returns message from Error instance', () => {
    const err = new Error('Something went wrong');
    expect(extractError(err)).toBe('Something went wrong');
  });

  it('returns string as-is', () => {
    expect(extractError('just a string error')).toBe('just a string error');
  });

  it('returns empty string for empty string error', () => {
    expect(extractError('')).toBe('');
  });

  it('JSON-stringifies plain objects', () => {
    expect(extractError({ code: 404, reason: 'Not Found' })).toBe('{"code":404,"reason":"Not Found"}');
  });

  it('JSON-stringifies arrays', () => {
    expect(extractError(['err1', 'err2'])).toBe('["err1","err2"]');
  });

  it('returns string coercion for numbers', () => {
    expect(extractError(0)).toBe('0');
    expect(extractError(500)).toBe('500');
  });

  it('returns string coercion for booleans', () => {
    expect(extractError(false)).toBe('false');
    expect(extractError(true)).toBe('true');
  });

  it('returns string coercion for null', () => {
    expect(extractError(null)).toBe('null');
  });

  it('returns undefined for undefined input (JSON.stringify returns undefined)', () => {
    const result = extractError(undefined);
    // JSON.stringify(undefined) returns undefined (not the string 'undefined'),
    // so extractError returns undefined directly
    expect(result).toBeUndefined();
  });

  it('handles Error with no message', () => {
    const err = new Error();
    expect(extractError(err)).toBe('');
  });

  it('handles circular reference', () => {
    const obj: any = { a: 1 };
    obj.self = obj;
    const result = extractError(obj);
    expect(typeof result).toBe('string');
  });
});

/**
 * Unit tests for mcp-schema-to-form utility functions
 *
 * Covers:
 * - schemaToFormFields: converts JSON Schema to FormField array
 * - mapJsonSchemaTypeToInputType: maps JSON Schema types to input types
 * - validateField: validates values against JSON Schema properties
 * - parseJsonSafe: safely parses JSON strings
 * - stringifyJsonSafe: safely stringifies values to JSON
 *
 * Run with: npx vitest run src/utils/__tests__/mcp-schema-to-form.spec.ts
 */

import { describe, it, expect } from 'vitest';
import {
  schemaToFormFields,
  mapJsonSchemaTypeToInputType,
  validateField,
  parseJsonSafe,
  stringifyJsonSafe,
  type FormField,
} from '../mcp-schema-to-form';
import type { JSONSchema, JSONSchemaProperty } from '@/types/mcp';

describe('mapJsonSchemaTypeToInputType', () => {
  it('maps string to text input', () => {
    expect(mapJsonSchemaTypeToInputType('string')).toBe('text');
  });

  it('maps number to number input', () => {
    expect(mapJsonSchemaTypeToInputType('number')).toBe('number');
  });

  it('maps integer to number input', () => {
    expect(mapJsonSchemaTypeToInputType('integer')).toBe('number');
  });

  it('maps boolean to checkbox input', () => {
    expect(mapJsonSchemaTypeToInputType('boolean')).toBe('checkbox');
  });

  it('maps array to textarea input', () => {
    expect(mapJsonSchemaTypeToInputType('array')).toBe('textarea');
  });

  it('maps object to textarea input', () => {
    expect(mapJsonSchemaTypeToInputType('object')).toBe('textarea');
  });

  it('defaults to text for unknown types', () => {
    expect(mapJsonSchemaTypeToInputType('unknown')).toBe('text');
  });

  it('defaults to text for undefined type', () => {
    expect(mapJsonSchemaTypeToInputType(undefined)).toBe('text');
  });
});

describe('schemaToFormFields', () => {
  it('converts string property to text field', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        name: {
          type: 'string',
          title: 'Name',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields).toHaveLength(1);
    expect(fields[0]).toMatchObject({
      name: 'name',
      type: 'text',
      label: 'Name',
    });
  });

  it('converts number property to number field', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        age: {
          type: 'number',
          title: 'Age',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].type).toBe('number');
  });

  it('converts boolean property to checkbox field', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        enabled: {
          type: 'boolean',
          title: 'Enabled',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].type).toBe('checkbox');
  });

  it('converts array property to textarea field', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        tags: {
          type: 'array',
          title: 'Tags',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].type).toBe('textarea');
  });

  it('converts object property to textarea field', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        metadata: {
          type: 'object',
          title: 'Metadata',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].type).toBe('textarea');
  });

  it('extracts title correctly', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        userName: {
          type: 'string',
          title: 'User Name',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].label).toBe('User Name');
  });

  it('formats label from property name when title is missing', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        firstName: {
          type: 'string',
        },
        last_name: {
          type: 'string',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].label).toBe('First Name');
    // formatLabel capitalizes first letter, doesn't capitalize each word
    expect(fields[1].label).toBe('Last name');
  });

  it('extracts description correctly', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        email: {
          type: 'string',
          title: 'Email',
          description: 'Enter your email address',
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].description).toBe('Enter your email address');
  });

  it('extracts required fields correctly', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        name: { type: 'string' },
        optional: { type: 'string' },
      },
      required: ['name'],
    };

    const fields = schemaToFormFields(schema);
    const nameField = fields.find((f) => f.name === 'name');
    const optionalField = fields.find((f) => f.name === 'optional');

    expect(nameField?.required).toBe(true);
    expect(optionalField?.required).toBe(false);
  });

  it('extracts enum values correctly', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        status: {
          type: 'string',
          title: 'Status',
          enum: ['active', 'inactive', 'pending'],
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].enum).toEqual(['active', 'inactive', 'pending']);
  });

  it('extracts default values correctly', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        count: {
          type: 'number',
          default: 10,
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].default).toBe(10);
  });

  it('handles nested properties for arrays', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        items: {
          type: 'array',
          items: {
            type: 'string',
            title: 'Item',
          },
        },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields[0].items).toBeDefined();
    expect(fields[0].items?.type).toBe('string');
  });

  it('returns empty array when no properties', () => {
    const schema: JSONSchema = {
      type: 'object',
    };

    const fields = schemaToFormFields(schema);
    expect(fields).toHaveLength(0);
  });

  it('handles multiple properties', () => {
    const schema: JSONSchema = {
      type: 'object',
      properties: {
        name: { type: 'string', title: 'Name' },
        age: { type: 'number', title: 'Age' },
        active: { type: 'boolean', title: 'Active' },
        config: { type: 'object', title: 'Config' },
      },
    };

    const fields = schemaToFormFields(schema);
    expect(fields).toHaveLength(4);
    expect(fields.map((f) => f.type)).toEqual(['text', 'number', 'checkbox', 'textarea']);
  });
});

describe('validateField', () => {
  it('returns valid for string type with string value', () => {
    const prop: JSONSchemaProperty = { type: 'string' };
    expect(validateField('hello', prop)).toEqual({ valid: true });
  });

  it('returns invalid for string type with number value', () => {
    const prop: JSONSchemaProperty = { type: 'string' };
    expect(validateField(123, prop)).toEqual({ valid: false, error: 'Expected a string' });
  });

  it('returns valid for number type with number value', () => {
    const prop: JSONSchemaProperty = { type: 'number' };
    expect(validateField(42, prop)).toEqual({ valid: true });
  });

  it('returns invalid for number type with string value', () => {
    const prop: JSONSchemaProperty = { type: 'number' };
    expect(validateField('42', prop)).toEqual({ valid: false, error: 'Expected a number' });
  });

  it('validates integer type correctly', () => {
    const prop: JSONSchemaProperty = { type: 'integer' };
    expect(validateField(42, prop)).toEqual({ valid: true });
    expect(validateField('42', prop)).toEqual({ valid: false, error: 'Expected a number' });
  });

  it('returns valid for boolean type with boolean value', () => {
    const prop: JSONSchemaProperty = { type: 'boolean' };
    expect(validateField(true, prop)).toEqual({ valid: true });
    expect(validateField(false, prop)).toEqual({ valid: true });
  });

  it('returns invalid for boolean type with non-boolean value', () => {
    const prop: JSONSchemaProperty = { type: 'boolean' };
    expect(validateField('true', prop)).toEqual({ valid: false, error: 'Expected a boolean' });
  });

  it('returns valid for array type with array value', () => {
    const prop: JSONSchemaProperty = { type: 'array' };
    expect(validateField([1, 2, 3], prop)).toEqual({ valid: true });
  });

  it('returns invalid for array type with non-array value', () => {
    const prop: JSONSchemaProperty = { type: 'array' };
    expect(validateField({ length: 3 }, prop)).toEqual({ valid: false, error: 'Expected an array' });
  });

  it('returns valid for object type with object value', () => {
    const prop: JSONSchemaProperty = { type: 'object' };
    expect(validateField({ key: 'value' }, prop)).toEqual({ valid: true });
  });

  it('returns invalid for object type with array value', () => {
    const prop: JSONSchemaProperty = { type: 'object' };
    expect(validateField([1, 2], prop)).toEqual({ valid: false, error: 'Expected an object' });
  });

  it('returns valid for enum values', () => {
    const prop: JSONSchemaProperty = { type: 'string', enum: ['a', 'b', 'c'] };
    expect(validateField('a', prop)).toEqual({ valid: true });
    expect(validateField('b', prop)).toEqual({ valid: true });
  });

  it('returns invalid for non-enum values', () => {
    const prop: JSONSchemaProperty = { type: 'string', enum: ['a', 'b', 'c'] };
    const result = validateField('d', prop);
    expect(result.valid).toBe(false);
    expect(result.error).toContain('Must be one of');
  });

  it('returns valid for undefined/null values (required check is separate)', () => {
    const prop: JSONSchemaProperty = { type: 'string' };
    expect(validateField(undefined, prop)).toEqual({ valid: true });
    expect(validateField(null, prop)).toEqual({ valid: true });
    expect(validateField('', prop)).toEqual({ valid: true });
  });
});

describe('parseJsonSafe', () => {
  it('parses valid JSON objects', () => {
    const result = parseJsonSafe('{"key": "value"}');
    expect(result).toEqual({ key: 'value' });
  });

  it('parses valid JSON arrays', () => {
    const result = parseJsonSafe('[1, 2, 3]');
    expect(result).toEqual([1, 2, 3]);
  });

  it('parses valid JSON primitives', () => {
    expect(parseJsonSafe('"hello"')).toBe('hello');
    expect(parseJsonSafe('42')).toBe(42);
    expect(parseJsonSafe('true')).toBe(true);
    expect(parseJsonSafe('null')).toBe(null);
  });

  it('returns original string for invalid JSON', () => {
    const invalid = 'not valid json {';
    const result = parseJsonSafe(invalid);
    expect(result).toBe(invalid);
  });

  it('handles empty string', () => {
    const result = parseJsonSafe('');
    expect(result).toBe('');
  });
});

describe('stringifyJsonSafe', () => {
  it('stringifies objects to JSON', () => {
    const result = stringifyJsonSafe({ key: 'value' });
    expect(result).toBe('{\n  "key": "value"\n}');
  });

  it('stringifies arrays to JSON', () => {
    const result = stringifyJsonSafe([1, 2, 3]);
    expect(result).toBe('[\n  1,\n  2,\n  3\n]');
  });

  it('stringifies primitives to JSON', () => {
    expect(stringifyJsonSafe('hello')).toBe('"hello"');
    expect(stringifyJsonSafe(42)).toBe('42');
    expect(stringifyJsonSafe(true)).toBe('true');
  });

  it('handles circular references gracefully', () => {
    const obj: Record<string, unknown> = { a: 1 };
    obj.self = obj; // Circular reference
    const result = stringifyJsonSafe(obj);
    expect(typeof result).toBe('string');
  });

  it('handles undefined and functions', () => {
    // JSON.stringify returns undefined for undefined, but our function returns the original
    const result = stringifyJsonSafe(undefined);
    // stringifyJsonSafe uses JSON.stringify which returns undefined for undefined input
    // The function returns the result directly, not a string
    expect(result).toBeUndefined();

    // Functions also stringify to undefined
    const fnResult = stringifyJsonSafe(() => {});
    expect(fnResult).toBeUndefined();
  });
});

describe('MCPImportDialog validation logic', () => {
  // Simulate the import validation logic from the dialog

  function validateServiceName(name: unknown): { valid: boolean; error?: string } {
    if (!name || typeof name !== 'string' || name.trim().length === 0) {
      return { valid: false, error: 'Service name is required' };
    }
    return { valid: true };
  }

  function validateServiceEndpoint(endpoint: unknown): { valid: boolean; error?: string } {
    if (!endpoint || typeof endpoint !== 'string' || endpoint.trim().length === 0) {
      return { valid: false, error: 'Service endpoint is required' };
    }
    return { valid: true };
  }

  function findDuplicates(
    importedServices: Array<{ name: string }>,
    existingServices: Array<{ name: string }>
  ): Array<{ name: string; isDuplicate: boolean }> {
    const existingNames = new Set(existingServices.map((s) => s.name));
    return importedServices.map((s) => ({
      ...s,
      isDuplicate: existingNames.has(s.name),
    }));
  }

  function normalizeImportedData(data: unknown): Array<Record<string, unknown>> {
    if (Array.isArray(data)) {
      return data.map((item) => normalizeService(item));
    }
    if (typeof data === 'object' && data !== null && 'services' in data) {
      const obj = data as Record<string, unknown>;
      if (Array.isArray(obj.services)) {
        return obj.services.map((item) => normalizeService(item));
      }
    }
    return [];
  }

  function normalizeService(service: unknown): Record<string, unknown> {
    if (typeof service !== 'object' || service === null) {
      return {};
    }
    const s = service as Record<string, unknown>;
    return {
      name: typeof s.name === 'string' ? s.name.trim() : '',
      endpoint: typeof s.endpoint === 'string' ? s.endpoint.trim() : '',
      protocol: s.protocol || 'http',
      authType: s.authType || s.auth_type || 'none',
      config: s.config || {},
    };
  }

  describe('validateServiceName', () => {
    it('validates service has name', () => {
      expect(validateServiceName('MyService')).toEqual({ valid: true });
    });

    it('rejects empty name', () => {
      expect(validateServiceName('')).toEqual({
        valid: false,
        error: 'Service name is required',
      });
    });

    it('rejects whitespace-only name', () => {
      expect(validateServiceName('   ')).toEqual({
        valid: false,
        error: 'Service name is required',
      });
    });

    it('rejects null/undefined name', () => {
      expect(validateServiceName(null)).toEqual({
        valid: false,
        error: 'Service name is required',
      });
      expect(validateServiceName(undefined)).toEqual({
        valid: false,
        error: 'Service name is required',
      });
    });
  });

  describe('validateServiceEndpoint', () => {
    it('validates service has endpoint', () => {
      expect(validateServiceEndpoint('http://localhost:8080')).toEqual({ valid: true });
      expect(validateServiceEndpoint('npx test-server')).toEqual({ valid: true });
    });

    it('rejects empty endpoint', () => {
      expect(validateServiceEndpoint('')).toEqual({
        valid: false,
        error: 'Service endpoint is required',
      });
    });

    it('rejects null/undefined endpoint', () => {
      expect(validateServiceEndpoint(null)).toEqual({
        valid: false,
        error: 'Service endpoint is required',
      });
    });
  });

  describe('findDuplicates', () => {
    it('identifies duplicates correctly', () => {
      const imported = [
        { name: 'Service1' },
        { name: 'Service2' },
        { name: 'Service3' },
      ];
      const existing = [{ name: 'Service1' }, { name: 'Service3' }];

      const result = findDuplicates(imported, existing);

      expect(result[0].isDuplicate).toBe(true);
      expect(result[1].isDuplicate).toBe(false);
      expect(result[2].isDuplicate).toBe(true);
    });

    it('returns all false when no duplicates', () => {
      const imported = [{ name: 'NewService1' }];
      const existing = [{ name: 'ExistingService' }];

      const result = findDuplicates(imported, existing);

      expect(result[0].isDuplicate).toBe(false);
    });

    it('handles empty arrays', () => {
      const result = findDuplicates([], []);
      expect(result).toEqual([]);
    });
  });

  describe('normalizeImportedData', () => {
    it('normalizes array of services', () => {
      const data = [
        { name: '  Service1  ', endpoint: ' http://a.local ', authType: 'bearer' },
        { name: 'Service2', endpoint: 'http://b.local' },
      ];

      const result = normalizeImportedData(data);

      expect(result).toHaveLength(2);
      expect(result[0].name).toBe('Service1');
      expect(result[0].endpoint).toBe('http://a.local');
      expect(result[0].authType).toBe('bearer');
    });

    it('normalizes object with services array', () => {
      const data = {
        version: '1.0',
        services: [
          { name: 'Service1', endpoint: 'http://a.local', authType: 'api-key' },
        ],
      };

      const result = normalizeImportedData(data);

      expect(result).toHaveLength(1);
      expect(result[0].name).toBe('Service1');
      expect(result[0].authType).toBe('api-key');
    });

    it('handles snake_case auth_type field', () => {
      const data = [{ name: 'Service1', auth_type: 'bearer' }];

      const result = normalizeImportedData(data);

      expect(result[0].authType).toBe('bearer');
    });

    it('defaults missing fields', () => {
      const data = [{ name: 'Service1' }];

      const result = normalizeImportedData(data);

      expect(result[0].protocol).toBe('http');
      expect(result[0].authType).toBe('none');
      expect(result[0].config).toEqual({});
    });

    it('returns empty array for invalid data', () => {
      expect(normalizeImportedData(null)).toEqual([]);
      expect(normalizeImportedData(undefined)).toEqual([]);
      expect(normalizeImportedData('not an object')).toEqual([]);
    });
  });
});

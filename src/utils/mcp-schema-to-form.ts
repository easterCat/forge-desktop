import type { JSONSchema, JSONSchemaProperty } from '@/types';

/**
 * Converts a JSON Schema to an array of form field definitions
 */
export function schemaToFormFields(schema: JSONSchema): FormField[] {
  if (!schema.properties) {
    return [];
  }

  return Object.entries(schema.properties).map(([name, prop]) => {
    const formField: FormField = {
      name,
      type: mapJsonSchemaTypeToInputType(prop.type),
      label: prop.title || formatLabel(name),
      required: schema.required?.includes(name) ?? false,
      default: prop.default,
      description: prop.description,
      enum: Array.isArray(prop.enum)
        ? (prop.enum.filter((v): v is string | number => typeof v === 'string' || typeof v === 'number'))
        : undefined,
    };

    // Handle nested objects and arrays
    if (prop.type === 'array' && prop.items) {
      formField.items = prop.items;
    }

    return formField;
  });
}

/**
 * Maps JSON Schema type to HTML input type
 */
export function mapJsonSchemaTypeToInputType(type?: string): string {
  const typeMap: Record<string, string> = {
    string: 'text',
    number: 'number',
    integer: 'number',
    boolean: 'checkbox',
    array: 'textarea',
    object: 'textarea',
  };

  return typeMap[type ?? 'string'] ?? 'text';
}

/**
 * Formats a camelCase or snake_case property name into a human-readable label
 */
function formatLabel(name: string): string {
  return name
    // Handle camelCase
    .replace(/([A-Z])/g, ' $1')
    // Handle snake_case
    .replace(/_/g, ' ')
    // Capitalize first letter
    .replace(/^./, str => str.toUpperCase())
    // Trim whitespace
    .trim();
}

/**
 * Validates a value against a JSON Schema property
 */
export function validateField(
  value: unknown,
  prop: JSONSchemaProperty
): { valid: boolean; error?: string } {
  // Check required
  if (value === undefined || value === null || value === '') {
    return { valid: true }; // Let the required check handle empty
  }

  // Type validation
  if (prop.type === 'string' && typeof value !== 'string') {
    return { valid: false, error: 'Expected a string' };
  }

  if ((prop.type === 'number' || prop.type === 'integer') && typeof value !== 'number') {
    return { valid: false, error: 'Expected a number' };
  }

  if (prop.type === 'boolean' && typeof value !== 'boolean') {
    return { valid: false, error: 'Expected a boolean' };
  }

  if (prop.type === 'array' && !Array.isArray(value)) {
    return { valid: false, error: 'Expected an array' };
  }

  if (prop.type === 'object' && (typeof value !== 'object' || Array.isArray(value))) {
    return { valid: false, error: 'Expected an object' };
  }

  // Enum validation
  if (prop.enum && !prop.enum.includes(value)) {
    return { valid: false, error: `Must be one of: ${prop.enum.join(', ')}` };
  }

  return { valid: true };
}

/**
 * Parses a JSON string into a value, returning the original string on failure
 */
export function parseJsonSafe(json: string): unknown {
  try {
    return JSON.parse(json);
  } catch {
    return json;
  }
}

/**
 * Stringifies a value to JSON, handling circular references
 */
export function stringifyJsonSafe(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

// Re-export FormField type for use in components
export interface FormField {
  name: string;
  type: string;
  label: string;
  required: boolean;
  default?: unknown;
  description?: string;
  enum?: Array<string | number>;
  items?: JSONSchemaProperty;
}

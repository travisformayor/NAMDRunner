/**
 * Template utility functions
 * Shared logic for template variable extraction, naming, and sample value generation
 */

import type { VariableDefinition } from '$lib/types/template';
import { getVariableTypeName } from '$lib/types/template';

/**
 * Extract variables from template text in order of first occurrence
 * Returns array of variable names preserving template text order
 */
export function extractVariablesFromTemplate(templateText: string): string[] {
  const regex = /\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}/g;
  const firstOccurrence = new Map<string, number>(); // variable → position

  let match;
  while ((match = regex.exec(templateText)) !== null) {
    const varName = match[1];
    if (!firstOccurrence.has(varName)) {
      // Track first occurrence only (handles duplicates)
      firstOccurrence.set(varName, match.index);
    }
  }

  // Sort by position in template (first occurrence order)
  return Array.from(firstOccurrence.entries())
    .sort((a, b) => a[1] - b[1])
    .map(([key, _]) => key);
}

/**
 * Generate a human-readable label from a variable name
 * Converts snake_case to Title Case
 */
export function generateLabel(varName: string): string {
  return varName
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

/**
 * Generate a sample value for template preview
 * Uses variable's default value or generates appropriate sample
 */
export function getSampleValue(varDef: VariableDefinition): string {
  const typeName = getVariableTypeName(varDef.var_type);

  if (typeName === 'Number' && 'Number' in varDef.var_type) {
    return String(varDef.var_type.Number.default);
  } else if (typeName === 'Text' && 'Text' in varDef.var_type) {
    return varDef.var_type.Text.default;
  } else if (typeName === 'Boolean' && 'Boolean' in varDef.var_type) {
    return varDef.var_type.Boolean.default ? 'yes' : 'no';
  } else if (typeName === 'FileUpload' && 'FileUpload' in varDef.var_type) {
    const extensions = varDef.var_type.FileUpload.extensions;
    const firstExt = extensions[0] || '.dat';
    return `${varDef.key}${firstExt}`;
  }
  return '';
}

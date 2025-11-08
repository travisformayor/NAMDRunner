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

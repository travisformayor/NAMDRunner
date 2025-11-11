import { logger } from '$lib/utils/logger';
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  Template,
  TemplateSummary
} from '$lib/types/template';
import type { ValidationResult } from '$lib/types/api';
import type { ApiResult } from '$lib/types/api';

// Store for all template summaries
export const templates = writable<TemplateSummary[]>([]);

// Store for currently selected/editing template
export const currentTemplate = writable<Template | null>(null);

// Loading state
export const templatesLoading = writable(false);

// Error state
export const templatesError = writable<string | null>(null);

// Derived store: Built-in templates (read-only)
export const builtInTemplates = derived(templates, $templates =>
  $templates.filter(t => t.is_builtin) // Convention: built-in templates end with _v1, _v2, etc.
);

// Derived store: User-created templates
export const userTemplates = derived(templates, $templates =>
  $templates.filter(t => !t.is_builtin) // User templates don't follow _v1 convention
);

/**
 * Load all templates from backend
 */
export async function loadTemplates(): Promise<void> {
  templatesLoading.set(true);
  templatesError.set(null);

  try {
    const result = await invoke<ApiResult<TemplateSummary[]>>('list_templates');

    if (result.success && result.data) {
      templates.set(result.data);
    } else {
      templatesError.set(result.error || 'Failed to load templates');
    }
  } catch (error) {
    templatesError.set(`Error loading templates: ${error}`);
    logger.error('[TemplateStore]', 'Failed to load templates', error);
  } finally {
    templatesLoading.set(false);
  }
}

/**
 * Load a specific template by ID
 */
export async function loadTemplate(templateId: string): Promise<Template | null> {
  templatesLoading.set(true);
  templatesError.set(null);

  try {
    const result = await invoke<ApiResult<Template>>('get_template', { template_id: templateId });

    if (result.success && result.data) {
      currentTemplate.set(result.data);
      return result.data;
    } else {
      templatesError.set(result.error || 'Template not found');
      return null;
    }
  } catch (error) {
    templatesError.set(`Error loading template: ${error}`);
    logger.error('[TemplateStore]', 'Failed to load template', error);
    return null;
  } finally {
    templatesLoading.set(false);
  }
}

/**
 * Create a new template
 */
export async function createTemplate(template: Template): Promise<boolean> {
  templatesLoading.set(true);
  templatesError.set(null);

  try {
    const result = await invoke<ApiResult<string>>('create_template', { template });

    if (result.success) {
      // Reload templates to get updated list
      await loadTemplates();
      return true;
    } else {
      templatesError.set(result.error || 'Failed to create template');
      return false;
    }
  } catch (error) {
    templatesError.set(`Error creating template: ${error}`);
    logger.error('[TemplateStore]', 'Failed to create template', error);
    return false;
  } finally {
    templatesLoading.set(false);
  }
}

/**
 * Update an existing template
 */
export async function updateTemplate(templateId: string, template: Template): Promise<boolean> {
  templatesLoading.set(true);
  templatesError.set(null);

  try {
    const result = await invoke<ApiResult<void>>('update_template', { template_id: templateId, template });

    if (result.success) {
      // Reload templates to get updated list
      await loadTemplates();
      // Update current template if it's the one being edited
      if (get(currentTemplate)?.id === templateId) {
        currentTemplate.set(template);
      }
      return true;
    } else {
      templatesError.set(result.error || 'Failed to update template');
      return false;
    }
  } catch (error) {
    templatesError.set(`Error updating template: ${error}`);
    logger.error('[TemplateStore]', 'Failed to update template', error);
    return false;
  } finally {
    templatesLoading.set(false);
  }
}

/**
 * Delete a template
 */
export async function deleteTemplate(templateId: string): Promise<boolean> {
  templatesLoading.set(true);
  templatesError.set(null);

  try {
    const result = await invoke<ApiResult<void>>('delete_template', { template_id: templateId });

    if (result.success) {
      // Reload templates to get updated list
      await loadTemplates();
      // Clear current template if it was deleted
      if (get(currentTemplate)?.id === templateId) {
        currentTemplate.set(null);
      }
      return true;
    } else {
      templatesError.set(result.error || 'Failed to delete template');
      return false;
    }
  } catch (error) {
    templatesError.set(`Error deleting template: ${error}`);
    logger.error('[TemplateStore]', 'Failed to delete template', error);
    return false;
  } finally {
    templatesLoading.set(false);
  }
}

/**
 * Validate template values
 */
export async function validateTemplateValues(
  templateId: string,
  values: Record<string, any>
): Promise<ValidationResult> {
  try {
    const result = await invoke<ValidationResult>('validate_template_values', {
      template_id: templateId,
      values
    });

    return result;
  } catch (error) {
    logger.error('[TemplateStore]', 'Failed to validate values', error);
    return {
      is_valid: false,
      issues: [`Validation error: ${error}`],
      warnings: [],
      suggestions: []
    };
  }
}

/**
 * Initialize template store (call on app startup)
 */
export async function initializeTemplateStore(): Promise<void> {
  await loadTemplates();
}

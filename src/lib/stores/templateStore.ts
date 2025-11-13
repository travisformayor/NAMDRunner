import { logger } from '$lib/utils/logger';
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  Template,
  TemplateSummary
} from '$lib/types/template';
import type { ValidationResult, ApiResult } from '$lib/types/api';

// Template store state
interface TemplateState {
  templates: TemplateSummary[];
  currentTemplate: Template | null;
  loading: boolean;
  error: string | null;
}

const initialState: TemplateState = {
  templates: [],
  currentTemplate: null,
  loading: false,
  error: null,
};

// Create template store
function createTemplateStore() {
  const { subscribe, update } = writable<TemplateState>(initialState);

  return {
    subscribe,

    // Load all templates from database
    async loadTemplates(): Promise<void> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<TemplateSummary[]>>('list_templates');

        if (result.success && result.data) {
          update(state => ({
            ...state,
            templates: result.data!,
            loading: false,
          }));
        } else {
          update(state => ({
            ...state,
            error: result.error || 'Failed to load templates',
            loading: false,
          }));
        }
      } catch (error) {
        logger.error('[Templates]', 'Failed to load templates', error);
        update(state => ({
          ...state,
          error: `Error loading templates: ${error}`,
          loading: false,
        }));
      }
    },

    // Load specific template by ID
    async loadTemplate(templateId: string): Promise<Template | null> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<Template>>('get_template', { template_id: templateId });

        if (result.success && result.data) {
          update(state => ({
            ...state,
            currentTemplate: result.data!,
            loading: false,
          }));
          return result.data;
        } else {
          update(state => ({
            ...state,
            error: result.error || 'Template not found',
            loading: false,
          }));
          return null;
        }
      } catch (error) {
        logger.error('[Templates]', 'Failed to load template', error);
        update(state => ({
          ...state,
          error: `Error loading template: ${error}`,
          loading: false,
        }));
        return null;
      }
    },

    // Create new template
    async createTemplate(template: Template): Promise<boolean> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<string>>('create_template', { template });

        if (result.success) {
          // Reload templates after creation
          await this.loadTemplates();
          return true;
        } else {
          update(state => ({
            ...state,
            error: result.error || 'Failed to create template',
            loading: false,
          }));
          return false;
        }
      } catch (error) {
        logger.error('[Templates]', 'Failed to create template', error);
        update(state => ({
          ...state,
          error: `Error creating template: ${error}`,
          loading: false,
        }));
        return false;
      }
    },

    // Update existing template
    async updateTemplate(templateId: string, template: Template): Promise<boolean> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<void>>('update_template', { template_id: templateId, template });

        if (result.success) {
          // Reload templates after update
          await this.loadTemplates();
          return true;
        } else {
          update(state => ({
            ...state,
            error: result.error || 'Failed to update template',
            loading: false,
          }));
          return false;
        }
      } catch (error) {
        logger.error('[Templates]', 'Failed to update template', error);
        update(state => ({
          ...state,
          error: `Error updating template: ${error}`,
          loading: false,
        }));
        return false;
      }
    },

    // Delete template
    async deleteTemplate(templateId: string): Promise<boolean> {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const result = await invoke<ApiResult<void>>('delete_template', { template_id: templateId });

        if (result.success) {
          // Clear current template if it was deleted
          update(state => ({
            ...state,
            currentTemplate: state.currentTemplate?.id === templateId ? null : state.currentTemplate,
            loading: false,
          }));

          // Reload templates after deletion
          await this.loadTemplates();
          return true;
        } else {
          update(state => ({
            ...state,
            error: result.error || 'Failed to delete template',
            loading: false,
          }));
          return false;
        }
      } catch (error) {
        logger.error('[Templates]', 'Failed to delete template', error);
        update(state => ({
          ...state,
          error: `Error deleting template: ${error}`,
          loading: false,
        }));
        return false;
      }
    },

    // Clear current template selection
    clearCurrentTemplate(): void {
      update(state => ({
        ...state,
        currentTemplate: null,
      }));
    },

    // Clear error state
    clearError(): void {
      update(state => ({
        ...state,
        error: null,
      }));
    },
  };
}

// Export store instance
export const templateStore = createTemplateStore();

// Derived stores for convenience
export const templates = derived(templateStore, $store => $store.templates);
export const currentTemplate = derived(templateStore, $store => $store.currentTemplate);
export const templatesLoading = derived(templateStore, $store => $store.loading);
export const templatesError = derived(templateStore, $store => $store.error);

// Derived: Built-in templates
export const builtInTemplates = derived(templates, $templates =>
  $templates.filter(t => t.is_builtin)
);

// Derived: User-created templates
export const userTemplates = derived(templates, $templates =>
  $templates.filter(t => !t.is_builtin)
);

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
    logger.error('[Templates]', 'Failed to validate values', error);
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
  await templateStore.loadTemplates();
}

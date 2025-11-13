<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { logger } from '$lib/utils/logger';
  import { templateStore, templatesError } from '$lib/stores/templateStore';
  import type { Template } from '$lib/types/template';
  import type { ApiResult } from '$lib/types/api';
  import { getVariableTypeName } from '$lib/types/template';
  import { extractVariablesFromTemplate, generateLabel } from '$lib/utils/template-utils';
  import VariableEditor from './VariableEditor.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import PreviewModal from '../ui/PreviewModal.svelte';
  import Dialog from '../ui/Dialog.svelte';

  // Props
  export let template: Template | null = null;
  export let mode: 'create' | 'edit' = 'create';
  export let onSaved: (template: Template) => void = () => {};
  export let onCancel: () => void = () => {};

  let id = template?.id ?? '';
  let name = template?.name ?? '';
  let description = template?.description ?? '';
  let namdConfigTemplate = template?.namd_config_template ?? '';
  let variables = template?.variables ?? {};
  let isSaving = false;
  let error: string | null = null;

  // Sync variables object with template text (debounced)
  let debounceTimer: ReturnType<typeof setTimeout>;
  function syncVariablesWithTemplate() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      const detectedVars = extractVariablesFromTemplate(namdConfigTemplate);
      const newVariables: Record<string, any> = {};

      // Keep existing variable metadata for detected vars
      for (const varKey of detectedVars) {
        if (variables[varKey]) {
          newVariables[varKey] = variables[varKey];
        } else {
          // New variable - create default Text type with variable name as default
          newVariables[varKey] = {
            key: varKey,
            label: generateLabel(varKey),
            var_type: { Text: { default: varKey } },
            help_text: null
          };
        }
      }

      variables = newVariables;
    }, 500);
  }

  // Watch template text changes
  $: if (namdConfigTemplate !== undefined) syncVariablesWithTemplate();

  // Variable editor state
  let showVariableEditor = false;
  let editingVariable: any = null;
  let editingVariableKey: string | null = null;

  // Test template state
  let showTestPreview = false;
  let testPreviewContent = '';
  let isGeneratingPreview = false;

  // Delete confirmation state
  let showDeleteConfirm = false;

  async function handleSave() {
    if (!id || !name || !namdConfigTemplate) {
      error = 'Please fill in all required fields';
      return;
    }

    isSaving = true;
    error = null;

    const now = new Date().toISOString();
    const templateData: Template = {
      id,
      name,
      description,
      namd_config_template: namdConfigTemplate,
      variables,
      created_at: template?.created_at ?? now,
      updated_at: now,
      is_builtin: false  // User-created or edited templates are always custom
    };

    let success = false;
    if (mode === 'create') {
      success = await templateStore.createTemplate(templateData);
    } else {
      success = await templateStore.updateTemplate(id, templateData);
    }

    isSaving = false;

    if (success) {
      onSaved(templateData);
    } else {
      // Use specific error from store (backend provides details)
      error = $templatesError || 'Failed to save template';
    }
  }

  function handleCancel() {
    onCancel();
  }

  function handleDelete() {
    if (!id || mode !== 'edit') return;
    showDeleteConfirm = true;
  }

  async function handleDeleteConfirm() {
    if (!id) return;

    const success = await templateStore.deleteTemplate(id);

    showDeleteConfirm = false;

    if (success) {
      onCancel(); // Navigate back after deletion
    }
  }

  function handleDeleteCancel() {
    showDeleteConfirm = false;
  }

  function handleEditVariable(key: string) {
    editingVariable = variables[key];
    editingVariableKey = key;
    showVariableEditor = true;
  }

  function handleVariableSaved(event: CustomEvent) {
    const varDef = event.detail;

    // If editing existing variable and key changed, delete old key
    if (editingVariableKey && editingVariableKey !== varDef.key) {
      const { [editingVariableKey]: deleted, ...rest } = variables;
      variables = rest;
    }

    // Add or update variable
    variables = { ...variables, [varDef.key]: varDef };

    showVariableEditor = false;
    editingVariable = null;
    editingVariableKey = null;
  }

  function handleVariableCancel() {
    showVariableEditor = false;
    editingVariable = null;
    editingVariableKey = null;
  }

  async function handleTestTemplate() {
    if (!id) {
      error = 'Please save the template before testing';
      return;
    }

    isGeneratingPreview = true;

    try {
      const result = await invoke<ApiResult<string>>('preview_template_with_defaults', { template_id: id });

      if (result.success && result.data) {
        testPreviewContent = result.data;
        showTestPreview = true;
      } else {
        error = result.error || 'Failed to generate preview';
        logger.error('[TemplateEditor]', `Preview failed: ${result.error || 'Unknown error'}`);
      }
    } catch (err) {
      error = 'Failed to generate preview';
      logger.error('[TemplateEditor]', 'Preview error', err);
    } finally {
      isGeneratingPreview = false;
    }
  }
</script>

<div class="template-editor">
  <h2>{mode === 'create' ? 'Create New Template' : 'Edit Template'}</h2>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <form on:submit|preventDefault={handleSave}>
    <div class="form-group">
      <label for="template-id">
        Template ID <span class="required">*</span>
      </label>
      <input
        id="template-id"
        type="text"
        bind:value={id}
        disabled={mode === 'edit'}
        placeholder="e.g., my_custom_template_v1"
        required
        class="form-control"
      />
      <p class="help-text">Unique identifier (lowercase, underscores only)</p>
    </div>

    <div class="form-group">
      <label for="template-name">
        Template Name <span class="required">*</span>
      </label>
      <input
        id="template-name"
        type="text"
        bind:value={name}
        placeholder="e.g., My Custom Simulation"
        required
        class="form-control"
      />
    </div>

    <div class="form-group">
      <label for="template-description">Description</label>
      <textarea
        id="template-description"
        bind:value={description}
        placeholder="Describe what this template is for..."
        rows="3"
        class="form-control"
      ></textarea>
    </div>

    <div class="form-group">
      <label for="namd-config">
        NAMD Configuration Template <span class="required">*</span>
      </label>
      <textarea
        id="namd-config"
        bind:value={namdConfigTemplate}
        placeholder="# NAMD Configuration Template"
        rows="20"
        required
        class="form-control code-editor"
      ></textarea>
      <p class="help-text">
        Variables are auto-detected from your template. Use <code>&#123;&#123;variable_name&#125;&#125;</code> syntax.
      </p>
    </div>

    <div class="form-group">
      <div class="variables-header">
        <span class="section-label">Template Variables</span>
        <span class="help-text-inline">Auto-detected from template text</span>
      </div>

      {#if Object.keys(variables).length > 0}
        <div class="variables-list">
          {#each Object.entries(variables) as [key, varDef]}
            <div class="variable-item">
              <div class="variable-info">
                <strong>{varDef.label}</strong>
                <span class="variable-key">{key}</span>
                <span class="variable-type">{getVariableTypeName(varDef.var_type)}</span>
              </div>
              <div class="variable-actions">
                <button type="button" class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEditVariable(key)}>Edit</button>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="help-text">No variables detected. Add variables using <code>&#123;&#123;variable_name&#125;&#125;</code> syntax in your template above.</p>
      {/if}
    </div>

    <div class="form-actions">
      <div class="form-actions-left">
        {#if mode === 'edit'}
          <button type="button" class="namd-button namd-button--destructive" on:click={handleDelete}>
            Delete Template
          </button>
        {/if}
      </div>
      <div class="form-actions-right">
        <button type="button" class="namd-button namd-button--secondary" on:click={handleCancel}>
          Cancel
        </button>
        <button type="button" class="namd-button namd-button--secondary" on:click={handleTestTemplate} disabled={isGeneratingPreview}>
          {isGeneratingPreview ? 'Generating Preview...' : 'Test Template'}
        </button>
        <button type="submit" class="namd-button namd-button--primary" disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Save Template'}
        </button>
      </div>
    </div>
  </form>
</div>

<!-- Variable Editor Modal -->
<Dialog open={showVariableEditor} size="md" onClose={handleVariableCancel}>
  <svelte:fragment slot="body">
    <VariableEditor
      variable={editingVariable}
      on:save={handleVariableSaved}
      on:cancel={handleVariableCancel}
    />
  </svelte:fragment>
</Dialog>

<!-- Test Template Preview -->
<PreviewModal
  isOpen={showTestPreview}
  title="Template Preview (with sample values)"
  content={testPreviewContent}
  onClose={() => showTestPreview = false}
/>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  isOpen={showDeleteConfirm}
  title="Delete Template?"
  message="Are you sure you want to delete '{name}'? This action cannot be undone."
  confirmText="Delete"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleDeleteConfirm}
  onCancel={handleDeleteCancel}
/>

<style>
  .template-editor {
    max-width: 900px;
    margin: 0 auto;
  }

  h2 {
    margin-bottom: var(--namd-spacing-lg);
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-2xl);
  }

  .form-group {
    margin-bottom: var(--namd-spacing-lg);
  }

  .form-group label {
    display: block;
    margin-bottom: var(--namd-spacing-sm);
    font-weight: var(--namd-font-weight-medium);
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
  }

  .required {
    color: var(--namd-error);
  }

  .form-control {
    width: 100%;
    padding: var(--namd-spacing-sm);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-sm);
    font-family: inherit;
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
  }

  .form-control:focus {
    outline: none;
    border-color: var(--namd-primary);
  }

  .code-editor {
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-xs);
  }

  .help-text {
    margin: var(--namd-spacing-xs) 0 0 0;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
  }

  .help-text code {
    background: var(--namd-bg-muted);
    padding: 0.125rem 0.25rem;
    border-radius: var(--namd-border-radius-sm);
    font-size: 0.875em;
    color: var(--namd-text-primary);
  }

  .help-text-inline {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
    font-style: italic;
  }

  .error-message {
    background: var(--namd-error-bg);
    border: 1px solid var(--namd-error-border);
    color: var(--namd-error-fg);
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius);
    margin-bottom: var(--namd-spacing-lg);
  }

  .form-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--namd-spacing-xl);
    padding-top: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
  }

  .form-actions-left {
    display: flex;
    gap: 1rem;
  }

  .form-actions-right {
    display: flex;
    gap: 1rem;
  }

  /* Variable Management */
  .variables-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--namd-spacing-md);
  }

  .variables-header .section-label {
    font-weight: var(--namd-font-weight-medium);
    font-size: var(--namd-font-size-sm);
    margin-bottom: 0;
    color: var(--namd-text-primary);
  }

  .variables-list {
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    overflow: hidden;
  }

  .variable-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--namd-spacing-sm) var(--namd-spacing-md);
    border-bottom: 1px solid var(--namd-border);
    background: var(--namd-bg-primary);
  }

  .variable-item:last-child {
    border-bottom: none;
  }

  .variable-info {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .variable-info strong {
    color: var(--namd-text-primary);
  }

  .variable-key {
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-xs);
    background: var(--namd-bg-muted);
    padding: 0.125rem var(--namd-spacing-xs);
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-text-secondary);
  }

  .variable-type {
    font-size: var(--namd-font-size-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    background: var(--namd-info-bg);
    color: var(--namd-info-fg);
    text-transform: uppercase;
    font-weight: var(--namd-font-weight-semibold);
  }

  .variable-actions {
    display: flex;
    gap: var(--namd-spacing-sm);
  }
</style>

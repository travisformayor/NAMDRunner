<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { logger } from '$lib/utils/logger';
  import { createTemplate, updateTemplate, deleteTemplate, templatesError } from '$lib/stores/templateStore';
  import type { Template } from '$lib/types/template';
  import type { PreviewResult } from '$lib/types/api';
  import { getVariableTypeName } from '$lib/types/template';
  import { extractVariablesFromTemplate, generateLabel } from '$lib/utils/template-utils';
  import VariableEditor from './VariableEditor.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import PreviewModal from '../ui/PreviewModal.svelte';

  const dispatch = createEventDispatcher();

  // Props
  export let template: Template | null = null;
  export let mode: 'create' | 'edit' = 'create';

  let id = template?.id ?? '';
  let name = template?.name ?? '';
  let description = template?.description ?? '';
  let namdConfigTemplate = template?.namd_config_template ?? '';
  let variables = template?.variables ?? {};
  let isSaving = false;
  let error: string | null = null;

  // Sync variables object with template text (debounced)
  let debounceTimer: number;
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
      success = await createTemplate(templateData);
    } else {
      success = await updateTemplate(id, templateData);
    }

    isSaving = false;

    if (success) {
      dispatch('saved', templateData);
    } else {
      // Use specific error from store (backend provides details)
      error = $templatesError || 'Failed to save template';
    }
  }

  function handleCancel() {
    dispatch('cancel');
  }

  function handleDelete() {
    if (!id || mode !== 'edit') return;
    showDeleteConfirm = true;
  }

  async function handleDeleteConfirm() {
    if (!id) return;

    const success = await deleteTemplate(id);

    showDeleteConfirm = false;

    if (success) {
      dispatch('cancel'); // Navigate back after deletion
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
      const result = await invoke<PreviewResult>('preview_template_with_defaults', { template_id: id });

      if (result.success && result.content) {
        testPreviewContent = result.content;
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
                <button type="button" class="btn btn-xs" on:click={() => handleEditVariable(key)}>Edit</button>
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
          <button type="button" class="btn btn-danger" on:click={handleDelete}>
            Delete Template
          </button>
        {/if}
      </div>
      <div class="form-actions-right">
        <button type="button" class="btn btn-secondary" on:click={handleCancel}>
          Cancel
        </button>
        <button type="button" class="btn btn-secondary" on:click={handleTestTemplate} disabled={isGeneratingPreview}>
          {isGeneratingPreview ? 'Generating Preview...' : 'Test Template'}
        </button>
        <button type="submit" class="btn btn-primary" disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Save Template'}
        </button>
      </div>
    </div>
  </form>
</div>

<!-- Variable Editor Modal -->
{#if showVariableEditor}
  <div
    class="modal-overlay"
    role="presentation"
    on:click={handleVariableCancel}
    on:keydown={(e) => e.key === 'Escape' && handleVariableCancel()}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-label="Variable Editor"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <VariableEditor
        variable={editingVariable}
        on:save={handleVariableSaved}
        on:cancel={handleVariableCancel}
      />
    </div>
  </div>
{/if}

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
    margin-bottom: 1.5rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    font-size: 0.875rem;
  }

  .required {
    color: var(--error, #d32f2f);
  }

  .form-control {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color, #ddd);
    border-radius: 4px;
    font-size: 0.875rem;
    font-family: inherit;
  }

  .form-control:focus {
    outline: none;
    border-color: var(--primary, #1976d2);
    box-shadow: 0 0 0 2px rgba(25, 118, 210, 0.1);
  }

  .code-editor {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.8125rem;
  }

  .help-text {
    margin: 0.25rem 0 0 0;
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
  }

  .help-text code {
    background: var(--code-bg, #f5f5f5);
    padding: 0.125rem 0.25rem;
    border-radius: 2px;
    font-size: 0.875em;
  }

  .help-text-inline {
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
    font-style: italic;
  }

  .error-message {
    background: var(--error-light, #ffebee);
    border: 1px solid var(--error, #d32f2f);
    color: var(--error-dark, #c62828);
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1.5rem;
  }

  .form-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 2rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-color, #e0e0e0);
  }

  .form-actions-left {
    display: flex;
    gap: 1rem;
  }

  .form-actions-right {
    display: flex;
    gap: 1rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary, #1976d2);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--primary-dark, #1565c0);
  }

  .btn-secondary {
    background: var(--secondary, #f5f5f5);
    color: var(--text-primary, #333);
  }

  .btn-secondary:hover {
    background: var(--secondary-dark, #e0e0e0);
  }

  /* Variable Management */
  .variables-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .variables-header .section-label {
    font-weight: 500;
    font-size: 0.875rem;
    margin-bottom: 0;
  }

  .btn-xs {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
  }

  .variables-list {
    border: 1px solid var(--border-color, #ddd);
    border-radius: 4px;
    overflow: hidden;
  }

  .variable-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .variable-item:last-child {
    border-bottom: none;
  }

  .variable-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .variable-key {
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 0.8125rem;
    background: var(--code-bg, #f5f5f5);
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    color: var(--text-secondary, #666);
  }

  .variable-type {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    background: var(--primary-light, #e3f2fd);
    color: var(--primary, #1976d2);
    text-transform: uppercase;
    font-weight: 600;
  }

  .variable-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-danger {
    background: var(--error, #d32f2f);
    color: white;
  }

  .btn-danger:hover {
    background: var(--error-dark, #c62828);
  }

  /* Modals */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--card-bg, white);
    border-radius: 8px;
    padding: 2rem;
    max-width: 600px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
  }
</style>

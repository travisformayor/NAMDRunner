<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { logger } from '$lib/utils/logger';
  import { extractVariablesFromTemplate } from '$lib/utils/template-utils';
  import { onMount } from 'svelte';
  import { templates, templateStore, validateTemplateValues } from '$lib/stores/templateStore';
  import type { Template, VariableDefinition } from '$lib/types/template';
  import { getVariableTypeName } from '$lib/types/template';

  // Props
  export let templateId: string = '';
  export let templateValues: Record<string, any> = {};

  let selectedTemplate: Template | null = null;
  let validationErrors: string[] = [];
  let fieldErrors: Record<string, string> = {};
  let isValidating = false;
  let lastLoadedTemplateId = '';

  // Separate variables by type, preserving template text order
  $: fileVariables = selectedTemplate
    ? (() => {
        const template = selectedTemplate;
        return extractVariablesFromTemplate(template.namd_config_template)
          .map(key => [key, template.variables[key]] as [string, any])
          .filter(([_, v]) => v && getVariableTypeName(v.var_type) === 'FileUpload');
      })()
    : [];

  $: parameterVariables = selectedTemplate
    ? (() => {
        const template = selectedTemplate;
        return extractVariablesFromTemplate(template.namd_config_template)
          .map(key => [key, template.variables[key]] as [string, any])
          .filter(([_, v]) => {
            if (!v) return false;
            const type = getVariableTypeName(v.var_type);
            return type === 'Number' || type === 'Text' || type === 'Boolean';
          });
      })()
    : [];

  onMount(async () => {
    await templateStore.loadTemplates();
  });

  // Reactively load template when templateId changes
  $: if (templateId && templateId !== lastLoadedTemplateId) {
    loadTemplateAndInitialize(templateId);
  } else if (!templateId && selectedTemplate) {
    selectedTemplate = null;
    lastLoadedTemplateId = '';
  }

  async function loadTemplateAndInitialize(newTemplateId: string) {
    const template = await templateStore.loadTemplate(newTemplateId);
    if (template) {
      selectedTemplate = template;
      lastLoadedTemplateId = newTemplateId;

      // Only initialize values if they don't exist (preserve user input)
      const newValues: Record<string, any> = { ...templateValues };

      for (const [key, varDef] of Object.entries(template.variables)) {
        // Skip if value already exists (preserves user input on tab switch)
        if (newValues[key] !== undefined) continue;

        const typeName = getVariableTypeName(varDef.var_type);

        if (typeName === 'Number' && 'Number' in varDef.var_type) {
          newValues[key] = varDef.var_type.Number.default;
        } else if (typeName === 'Text' && 'Text' in varDef.var_type) {
          newValues[key] = varDef.var_type.Text.default;
        } else if (typeName === 'Boolean' && 'Boolean' in varDef.var_type) {
          newValues[key] = varDef.var_type.Boolean.default;
        } else if (typeName === 'FileUpload') {
          newValues[key] = '';
        }
      }

      templateValues = newValues;
    }
  }

  async function handleFileSelect(key: string) {
    try {
      const varDef = selectedTemplate?.variables[key];
      if (!varDef || getVariableTypeName(varDef.var_type) !== 'FileUpload') return;

      // Use existing backend command for file selection
      const selected = await invoke('select_input_file') as { name: string; path: string; size: number; file_type: string } | null;

      if (selected) {
        // Store the full path for now - during job creation, we'll extract filename
        templateValues[key] = selected.path;
      }
    } catch (error) {
      logger.error('[DynamicJobForm]', 'File selection error', error);
    }
  }

  async function handleValidate() {
    if (!selectedTemplate) return;

    isValidating = true;
    const result = await validateTemplateValues(selectedTemplate.id, templateValues);
    validationErrors = result.issues;

    // Parse issues to extract field-specific errors
    fieldErrors = {};
    for (const error of result.issues) {
      // Try to extract field name from error message (format: "FieldLabel: error message")
      const colonIndex = error.indexOf(':');
      if (colonIndex > 0) {
        const fieldLabel = error.substring(0, colonIndex).trim();
        const errorMsg = error.substring(colonIndex + 1).trim();

        // Find the variable key by label
        for (const [key, varDef] of Object.entries(selectedTemplate.variables)) {
          if (varDef.label === fieldLabel) {
            fieldErrors[key] = errorMsg;
            break;
          }
        }
      }
    }

    isValidating = false;
  }

  function getVariableConfig(varDef: VariableDefinition) {
    const typeName = getVariableTypeName(varDef.var_type);

    if (typeName === 'Number' && 'Number' in varDef.var_type) {
      return varDef.var_type.Number;
    } else if (typeName === 'Text' && 'Text' in varDef.var_type) {
      return varDef.var_type.Text;
    } else if (typeName === 'Boolean' && 'Boolean' in varDef.var_type) {
      return varDef.var_type.Boolean;
    } else if (typeName === 'FileUpload' && 'FileUpload' in varDef.var_type) {
      return varDef.var_type.FileUpload;
    }
    return null;
  }
</script>

<div class="dynamic-job-form">
  <!-- Template Selection -->
  <div class="form-section">
    <h3>Simulation Template</h3>
    <div class="form-group">
      <label for="template-select">Choose Template</label>
      <select
        id="template-select"
        bind:value={templateId}
        class="namd-input"
      >
        <option value="">-- Select a template --</option>
        {#each $templates as template}
          <option value={template.id}>{template.name}</option>
        {/each}
      </select>
      {#if selectedTemplate}
        <p class="help-text">{selectedTemplate.description}</p>
      {/if}
    </div>
  </div>

  {#if selectedTemplate}
    <!-- File Upload Section -->
    {#if fileVariables.length > 0}
      <div class="form-section">
        <h3>Input Files</h3>
        <p class="section-description">Upload the required simulation input files.</p>

        {#each fileVariables as [key, varDef]}
          {@const config = getVariableConfig(varDef)}
          {@const hasError = fieldErrors[key]}
          <div class="form-group" class:required={varDef.required} class:has-error={hasError}>
            <label for={key}>
              {varDef.label}
              {#if varDef.required}<span class="required-mark">*</span>{/if}
            </label>

            <div class="file-input-group">
              <input
                type="text"
                id={key}
                bind:value={templateValues[key]}
                placeholder="No file selected"
                readonly
                class="form-control file-display"
                class:error={hasError}
              />
              <button
                type="button"
                class="namd-button namd-button--secondary"
                on:click={() => handleFileSelect(key)}
              >
                Browse...
              </button>
            </div>

            {#if hasError}
              <p class="error-text">{hasError}</p>
            {/if}
            {#if config && 'extensions' in config}
              <p class="help-text">
                Allowed extensions: {config.extensions.join(', ')}
              </p>
            {/if}
            {#if varDef.help_text}
              <p class="help-text">{varDef.help_text}</p>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Parameters Section -->
    {#if parameterVariables.length > 0}
      <div class="form-section">
        <h3>Simulation Parameters</h3>
        <p class="section-description">Configure the simulation settings.</p>

        <div class="parameters-grid">
          {#each parameterVariables as [key, varDef]}
            {@const typeName = getVariableTypeName(varDef.var_type)}
            {@const config = getVariableConfig(varDef)}
            {@const hasError = fieldErrors[key]}

            <div class="form-group" class:required={varDef.required} class:has-error={hasError}>
              <label for={key}>
                {varDef.label}
                {#if varDef.required}<span class="required-mark">*</span>{/if}
              </label>

              {#if typeName === 'Number' && config && 'min' in config}
                <input
                  type="number"
                  id={key}
                  bind:value={templateValues[key]}
                  min={config.min ?? undefined}
                  max={config.max ?? undefined}
                  step="any"
                  class="namd-input"
                  class:error={hasError}
                  required={varDef.required}
                />
              {:else if typeName === 'Text'}
                <input
                  type="text"
                  id={key}
                  bind:value={templateValues[key]}
                  class="namd-input"
                  class:error={hasError}
                  required={varDef.required}
                />
              {:else if typeName === 'Boolean' && config && 'default' in config}
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    id={key}
                    bind:checked={templateValues[key]}
                  />
                  <span>Enable</span>
                </label>
              {/if}

              {#if hasError}
                <p class="error-text">{hasError}</p>
              {/if}
              {#if varDef.help_text}
                <p class="help-text">{varDef.help_text}</p>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Validation Errors -->
    {#if validationErrors.length > 0}
      <div class="validation-errors">
        <h4>Validation Errors:</h4>
        <ul>
          {#each validationErrors as error}
            <li>{error}</li>
          {/each}
        </ul>
      </div>
    {/if}

    <!-- Validate Button -->
    <div class="form-actions">
      <button
        type="button"
        class="btn btn-secondary"
        on:click={handleValidate}
        disabled={isValidating}
      >
        {isValidating ? 'Validating...' : 'Validate Configuration'}
      </button>
    </div>
  {:else}
    <div class="empty-state">
      <p>Select a template to begin configuring your simulation.</p>
    </div>
  {/if}
</div>

<style>
  .dynamic-job-form {
    max-width: 900px;
  }

  .form-section {
    margin-bottom: 2rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .form-section:last-child {
    border-bottom: none;
  }

  .form-section h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    color: var(--text-primary, #333);
  }

  .section-description {
    color: var(--text-secondary, #666);
    margin-bottom: 1.5rem;
    font-size: 0.875rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group.required label {
    font-weight: 500;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary, #333);
  }

  .required-mark {
    color: var(--error, #d32f2f);
    margin-left: 0.25rem;
  }


  .file-input-group {
    display: flex;
    gap: var(--namd-spacing-sm);
  }

  .file-display {
    flex: 1;
    background: var(--namd-input-disabled-bg);
  }

  .parameters-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: auto;
  }

  .help-text {
    margin: 0.25rem 0 0 0;
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-secondary, #666);
  }

  .validation-errors {
    background: var(--error-light, #ffebee);
    border: 1px solid var(--error, #d32f2f);
    border-radius: 4px;
    padding: 1rem;
    margin-bottom: 1.5rem;
  }

  .validation-errors h4 {
    margin: 0 0 0.5rem 0;
    color: var(--error, #d32f2f);
  }

  .validation-errors ul {
    margin: 0;
    padding-left: 1.5rem;
  }

  .validation-errors li {
    color: var(--error-dark, #c62828);
  }

  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 2rem;
  }

  /* Error States */
  .has-error label {
    color: var(--namd-error);
  }

  .form-control.error {
    border-color: var(--namd-error);
    background: var(--namd-error-bg);
  }

  .form-control.error:focus {
    border-color: var(--namd-error);
    box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
  }

  .error-text {
    margin: var(--namd-spacing-xs) 0 0 0;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-error);
    font-weight: var(--namd-font-weight-medium);
  }
</style>

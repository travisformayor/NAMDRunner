<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { extractVariablesFromTemplate } from '$lib/utils/template-utils';
  import { onMount } from 'svelte';
  import { templates, templateStore, validateTemplateValues } from '$lib/stores/templateStore';
  import type { Template, VariableDefinition } from '$lib/types/template';
  import { getVariableTypeName } from '$lib/types/template';
  import type { ValidationResult } from '$lib/types/api';
  import ValidationDisplay from '../ui/ValidationDisplay.svelte';

  // Props
  export let templateId: string = '';
  export let templateValues: Record<string, any> = {};

  let selectedTemplate: Template | null = null;
  let validation: ValidationResult = { is_valid: true, issues: [], warnings: [], suggestions: [] };
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
    const varDef = selectedTemplate?.variables[key];
    if (!varDef || getVariableTypeName(varDef.var_type) !== 'FileUpload') return;

    // Use existing backend command for file selection
    const selected = (await invoke('select_input_file')) as {
      name: string;
      path: string;
      size: number;
    } | null;

    if (selected) {
      // Store the full path for now - during job creation, we'll extract filename
      templateValues[key] = selected.path;
    }
  }

  async function handleValidate() {
    if (!selectedTemplate) return;

    isValidating = true;
    validation = await validateTemplateValues(selectedTemplate.id, templateValues);

    // Parse issues to extract field-specific errors
    fieldErrors = {};
    for (const error of validation.issues) {
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
    <div class="namd-field-group">
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
          <div class="namd-field-group" class:required={varDef.required} class:has-error={hasError}>
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
                class="namd-input file-display"
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

            <div class="namd-field-group" class:required={varDef.required} class:has-error={hasError}>
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

    <!-- Validation Results -->
    <ValidationDisplay {validation} />

    <!-- Validate Button -->
    <div class="form-actions">
      <button
        type="button"
        class="namd-button namd-button--secondary"
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
    max-width: var(--namd-max-width-form);
  }

  .form-section {
    margin-bottom: var(--namd-spacing-xl);
    padding-bottom: var(--namd-spacing-xl);
    border-bottom: 1px solid var(--namd-border);
  }

  .form-section:last-child {
    border-bottom: none;
  }

  .form-section h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    color: var(--namd-text-primary);
  }

  .section-description {
    color: var(--namd-text-secondary);
    margin-bottom: var(--namd-spacing-lg);
    font-size: 0.875rem;
  }

  .namd-field-group {
    margin-bottom: var(--namd-spacing-lg);
  }

  .namd-field-group.required label {
    font-weight: 500;
  }

  .namd-field-group label {
    display: block;
    margin-bottom: var(--namd-spacing-sm);
    font-size: 0.875rem;
    color: var(--namd-text-primary);
  }

  .required-mark {
    color: var(--namd-error);
    margin-left: var(--namd-spacing-xs);
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
    gap: var(--namd-spacing-lg);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: auto;
  }

  .help-text {
    margin: var(--namd-spacing-xs) 0 0 0;
    font-size: 0.75rem;
    color: var(--namd-text-secondary);
  }

  .empty-state {
    text-align: center;
    padding: var(--namd-spacing-2xl);
    color: var(--namd-text-secondary);
  }


  /* Error States */
  .has-error label {
    color: var(--namd-error);
  }

  .namd-input.error {
    border-color: var(--namd-error);
    background: var(--namd-error-bg);
  }

  .namd-input.error:focus {
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

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { VariableDefinition, VariableType } from '$lib/types/template';

  const dispatch = createEventDispatcher();

  // Props
  export let variable: VariableDefinition | null = null;

  let key = variable?.key ?? '';
  let label = variable?.label ?? '';
  let helpText = variable?.help_text ?? '';
  let varType: 'Number' | 'Text' | 'Boolean' | 'FileUpload' = 'Text';

  // Type-specific fields - all required (no nulls)
  let numMin: number = 0;
  let numMax: number = 100;
  let numDefault: number = 0;
  let textDefault = '';
  let boolDefault = false;
  let fileExtensions = '';

  // Initialize from existing variable
  if (variable) {
    if ('Number' in variable.var_type) {
      varType = 'Number';
      numMin = variable.var_type.Number.min;
      numMax = variable.var_type.Number.max;
      numDefault = variable.var_type.Number.default;
    } else if ('Text' in variable.var_type) {
      varType = 'Text';
      textDefault = variable.var_type.Text.default;
    } else if ('Boolean' in variable.var_type) {
      varType = 'Boolean';
      boolDefault = variable.var_type.Boolean.default;
    } else if ('FileUpload' in variable.var_type) {
      varType = 'FileUpload';
      fileExtensions = variable.var_type.FileUpload.extensions.join(', ');
    }
  }

  function buildVariableType(): VariableType {
    switch (varType) {
      case 'Number':
        return {
          Number: {
            min: numMin,
            max: numMax,
            default: numDefault
          }
        };
      case 'Text':
        return {
          Text: {
            default: textDefault
          }
        };
      case 'Boolean':
        return {
          Boolean: {
            default: boolDefault
          }
        };
      case 'FileUpload':
        return {
          FileUpload: {
            extensions: fileExtensions.split(',').map(e => e.trim()).filter(Boolean)
          }
        };
    }
  }

  function handleSave() {
    const varDef: VariableDefinition = {
      key,
      label,
      var_type: buildVariableType(),
      help_text: helpText || null
    };

    dispatch('save', varDef);
  }

  function handleCancel() {
    dispatch('cancel');
  }
</script>

<div class="variable-editor">
  <h3>Variable Editor</h3>

  <form on:submit|preventDefault={handleSave}>
    <div class="form-group">
      <label for="var-key">Variable Key <span class="required">*</span></label>
      <input
        id="var-key"
        type="text"
        bind:value={key}
        placeholder="e.g., temperature"
        required
        class="form-control"
      />
      <p class="help-text">Used in template as &#123;&#123;{key || 'variable_key'}&#125;&#125;</p>
    </div>

    <div class="form-group">
      <label for="var-label">Display Label <span class="required">*</span></label>
      <input
        id="var-label"
        type="text"
        bind:value={label}
        placeholder="e.g., Temperature (K)"
        required
        class="form-control"
      />
    </div>

    <div class="form-group">
      <label for="var-type">Variable Type <span class="required">*</span></label>
      <select id="var-type" bind:value={varType} class="form-control">
        <option value="Number">Number</option>
        <option value="Text">Text</option>
        <option value="Boolean">Boolean</option>
        <option value="FileUpload">File Upload</option>
      </select>
    </div>

    <!-- Type-specific fields -->
    {#if varType === 'Number'}
      <div class="form-row">
        <div class="form-group">
          <label for="num-min">Minimum <span class="required">*</span></label>
          <input id="num-min" type="number" bind:value={numMin} step="any" required class="form-control" />
        </div>
        <div class="form-group">
          <label for="num-max">Maximum <span class="required">*</span></label>
          <input id="num-max" type="number" bind:value={numMax} step="any" required class="form-control" />
        </div>
        <div class="form-group">
          <label for="num-default">Default Value <span class="required">*</span></label>
          <input id="num-default" type="number" bind:value={numDefault} step="any" required class="form-control" />
        </div>
      </div>
    {:else if varType === 'Text'}
      <div class="form-group">
        <label for="text-default">Default Value <span class="required">*</span></label>
        <input id="text-default" type="text" bind:value={textDefault} required class="form-control" />
      </div>
    {:else if varType === 'Boolean'}
      <div class="form-group">
        <label for="bool-default">Default Value <span class="required">*</span></label>
        <select id="bool-default" bind:value={boolDefault} required class="form-control">
          <option value={true}>True</option>
          <option value={false}>False</option>
        </select>
      </div>
    {:else if varType === 'FileUpload'}
      <div class="form-group">
        <label for="file-ext">Allowed Extensions</label>
        <input
          id="file-ext"
          type="text"
          bind:value={fileExtensions}
          placeholder="e.g., .pdb, .psf, .prm"
          class="form-control"
        />
        <p class="help-text">Comma-separated list</p>
      </div>
    {/if}

    <div class="form-group">
      <label for="help-text">Help Text</label>
      <textarea
        id="help-text"
        bind:value={helpText}
        placeholder="Description shown to users..."
        rows="2"
        class="form-control"
      ></textarea>
    </div>

    <div class="form-actions">
      <button type="button" class="btn btn-secondary" on:click={handleCancel}>Cancel</button>
      <button type="submit" class="btn btn-primary">Save Variable</button>
    </div>
  </form>
</div>

<style>
  .variable-editor {
    max-width: 600px;
  }

  h3 {
    margin-bottom: var(--namd-spacing-lg);
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-xl);
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

  .form-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--namd-spacing-md);
  }

  .help-text {
    margin: var(--namd-spacing-xs) 0 0 0;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
  }

  .form-actions {
    display: flex;
    gap: var(--namd-spacing-md);
    justify-content: flex-end;
    margin-top: var(--namd-spacing-xl);
    padding-top: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
  }

  .btn {
    padding: var(--namd-spacing-sm) var(--namd-spacing-md);
    border: none;
    border-radius: var(--namd-border-radius-sm);
    cursor: pointer;
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    transition: all 0.15s ease;
  }

  .btn-primary {
    background: var(--namd-primary);
    color: var(--namd-primary-fg);
  }

  .btn-primary:hover {
    background: var(--namd-primary-hover);
  }

  .btn-secondary {
    background: var(--namd-secondary);
    color: var(--namd-secondary-fg);
  }

  .btn-secondary:hover {
    background: var(--namd-secondary-hover);
  }
</style>

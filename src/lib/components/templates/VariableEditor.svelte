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

  .form-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .help-text {
    margin: 0.25rem 0 0 0;
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
  }

  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 2rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-color, #e0e0e0);
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

  .btn-primary {
    background: var(--primary, #1976d2);
    color: white;
  }

  .btn-primary:hover {
    background: var(--primary-dark, #1565c0);
  }

  .btn-secondary {
    background: var(--secondary, #f5f5f5);
    color: var(--text-primary, #333);
  }

  .btn-secondary:hover {
    background: var(--secondary-dark, #e0e0e0);
  }
</style>

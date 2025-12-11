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

  export function getVariableDefinition(): VariableDefinition {
    return {
      key,
      label,
      var_type: buildVariableType(),
      help_text: helpText || null
    };
  }
</script>

<div class="namd-field-group">
      <label class="namd-label" for="var-key">Variable Key <span class="required">*</span></label>
      <input
        id="var-key"
        type="text"
        bind:value={key}
        placeholder="e.g., temperature"
        required
        class="namd-input"
      />
      <p class="help-text">Used in template as &#123;&#123;{key || 'variable_key'}&#125;&#125;</p>
    </div>

    <div class="namd-field-group">
      <label class="namd-label" for="var-label">Display Label <span class="required">*</span></label>
      <input
        id="var-label"
        type="text"
        bind:value={label}
        placeholder="e.g., Temperature (K)"
        required
        class="namd-input"
      />
    </div>

    <div class="namd-field-group">
      <label class="namd-label" for="var-type">Variable Type <span class="required">*</span></label>
      <select id="var-type" bind:value={varType} class="namd-input">
        <option value="Number">Number</option>
        <option value="Text">Text</option>
        <option value="Boolean">Boolean</option>
        <option value="FileUpload">File Upload</option>
      </select>
    </div>

    <!-- Type-specific fields -->
    {#if varType === 'Number'}
      <div class="form-row">
        <div class="namd-field-group">
          <label class="namd-label" for="num-min">Minimum <span class="required">*</span></label>
          <input id="num-min" type="number" bind:value={numMin} step="any" required class="namd-input" />
        </div>
        <div class="namd-field-group">
          <label class="namd-label" for="num-max">Maximum <span class="required">*</span></label>
          <input id="num-max" type="number" bind:value={numMax} step="any" required class="namd-input" />
        </div>
        <div class="namd-field-group">
          <label class="namd-label" for="num-default">Default Value <span class="required">*</span></label>
          <input id="num-default" type="number" bind:value={numDefault} step="any" required class="namd-input" />
        </div>
      </div>
    {:else if varType === 'Text'}
      <div class="namd-field-group">
        <label class="namd-label" for="text-default">Default Value <span class="required">*</span></label>
        <input id="text-default" type="text" bind:value={textDefault} required class="namd-input" />
      </div>
    {:else if varType === 'Boolean'}
      <div class="namd-field-group">
        <label class="namd-label" for="bool-default">Default Value <span class="required">*</span></label>
        <select id="bool-default" bind:value={boolDefault} required class="namd-input">
          <option value={true}>True</option>
          <option value={false}>False</option>
        </select>
      </div>
    {:else if varType === 'FileUpload'}
      <div class="namd-field-group">
        <label class="namd-label" for="file-ext">Allowed Extensions</label>
        <input
          id="file-ext"
          type="text"
          bind:value={fileExtensions}
          placeholder="e.g., .pdb, .psf, .prm"
          class="namd-input"
        />
        <p class="help-text">Comma-separated list</p>
      </div>
    {/if}

    <div class="namd-field-group">
      <label class="namd-label" for="help-text">Help Text</label>
      <textarea
        id="help-text"
        bind:value={helpText}
        placeholder="Description shown to users..."
        rows="2"
        class="namd-input"
      ></textarea>
    </div>

<style>
  .required {
    color: var(--namd-error);
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
</style>

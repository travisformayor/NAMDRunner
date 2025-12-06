<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ApiResult } from '$lib/types/api';
  import type { Template } from '$lib/types/template';
  import DynamicJobForm from './DynamicJobForm.svelte';
  import PreviewModal from '../ui/PreviewModal.svelte';

  export let jobName: string;
  export let templateId: string;
  export let templateValues: Record<string, any>;
  export let template: Template | null = null;
  export let errors: Record<string, string>;

  let showPreview = false;
  let previewContent = '';
  let isGeneratingPreview = false;

  async function handlePreview() {
    if (!templateId) {
      return;
    }

    isGeneratingPreview = true;

    const result = await invoke<ApiResult<string>>('preview_namd_config', {
      template_id: templateId,
      values: templateValues,
    });

    if (result.success && result.data) {
      previewContent = result.data;
      showPreview = true;
    }

    isGeneratingPreview = false;
  }
</script>

<div class="namd-tab-panel">
  <!-- Job Name -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Job Information</h3>
    </div>
    <div class="namd-field-group">
      <label class="namd-label" for="job-name">Job Name <span class="required">*</span></label>
      <input
        class="namd-input"
        id="job-name"
        type="text"
        bind:value={jobName}
        placeholder="e.g., dna_equilibration_run1"
        required
        class:error={errors.job_name}
      />
      {#if errors.job_name}
        <span class="error-text">{errors.job_name}</span>
      {/if}
      <p class="help-text">Unique identifier for this job</p>
    </div>
  </div>

  <!-- Template-Based Configuration -->
  <DynamicJobForm bind:templateId bind:templateValues bind:selectedTemplate={template} />

  <!-- Preview Button -->
  {#if templateId}
    <div class="namd-section">
      <button
        type="button"
        class="namd-button namd-button--secondary"
        on:click={handlePreview}
        disabled={isGeneratingPreview}
      >
        {isGeneratingPreview ? 'Generating Preview...' : 'Preview NAMD Configuration'}
      </button>
    </div>
  {/if}
</div>

<!-- Preview Modal -->
<PreviewModal
  isOpen={showPreview}
  title="NAMD Configuration Preview"
  content={previewContent}
  onClose={() => showPreview = false}
/>

<style>
  .help-text {
    margin: 0;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-xs);
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }

  .required {
    color: var(--namd-error);
  }
</style>

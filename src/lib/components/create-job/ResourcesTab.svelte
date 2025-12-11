<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ApiResult, JobPreset, ValidationResult } from '$lib/types/api';
  import ValidationDisplay from '../ui/ValidationDisplay.svelte';
  import { jobPresets, partitions, allQosOptions, validateResourceRequest, calculateJobCost, estimateQueueTime } from '$lib/stores/clusterConfig';
  import PreviewModal from '../ui/PreviewModal.svelte';

  export let resourceConfig: {
    cores: number;
    memory: string;
    walltime: string;
    partition: string;
    qos: string;
  };
  export let errors: Record<string, string>;

  let selectedPresetId = '';
  let validation: ValidationResult = { is_valid: true, issues: [], warnings: [], suggestions: [] };
  let costEstimate = { totalCost: 0, queueEstimate: 'Unknown' };
  let showScriptPreview = false;
  let scriptPreviewContent = '';
  let isGeneratingScript = false;

  // Real-time validation and cost calculation
  $: if (resourceConfig.cores || resourceConfig.memory || resourceConfig.walltime || resourceConfig.partition || resourceConfig.qos) {
    updateValidation();
    updateCostEstimate();
  }

  async function updateValidation() {
    validation = await validateResourceRequest(
      resourceConfig.cores,
      resourceConfig.memory,
      resourceConfig.walltime,
      resourceConfig.partition,
      resourceConfig.qos
    );
  }

  async function updateCostEstimate() {
    const partitionSpec = $partitions.find(p => p.name === resourceConfig.partition);
    const hasGpu = partitionSpec?.gpu_type ? true : false;
    const gpuCount = partitionSpec?.gpu_count || 1;

    const totalCost = await calculateJobCost(resourceConfig.cores, resourceConfig.walltime, hasGpu, gpuCount);
    const queueEstimate = await estimateQueueTime(resourceConfig.cores, resourceConfig.partition);

    costEstimate = { totalCost, queueEstimate };
  }

  function handlePresetSelect(preset: JobPreset) {
    selectedPresetId = preset.name;
    resourceConfig.cores = preset.cores;
    resourceConfig.memory = preset.memory;
    resourceConfig.walltime = preset.walltime;
    resourceConfig.partition = preset.partition;
    resourceConfig.qos = preset.qos;
  }

  async function handleScriptPreview() {
    isGeneratingScript = true;

    const result = await invoke<ApiResult<string>>('preview_slurm_script', {
      job_name: 'preview_job',
      cores: resourceConfig.cores,
      memory: resourceConfig.memory,
      walltime: resourceConfig.walltime,
      partition: resourceConfig.partition,
      qos: resourceConfig.qos,
    });

    if (result.success && result.data) {
      scriptPreviewContent = result.data;
      showScriptPreview = true;
    }

    isGeneratingScript = false;
  }
</script>

<div class="namd-tab-panel">
  <!-- Quick Presets -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Resource Presets</h3>
      <p class="section-description">Quick configurations for common scenarios</p>
    </div>

    <div class="preset-pills">
      {#each $jobPresets as preset}
        <button
          type="button"
          class="preset-pill"
          class:selected={selectedPresetId === preset.name}
          on:click={() => handlePresetSelect(preset)}
        >
          <span class="preset-name">{preset.name}</span>
          <span class="preset-specs">({preset.cores}c, {preset.memory}, {preset.walltime})</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Manual Configuration -->
  <div class="namd-section">
    <details open>
      <summary class="manual-config-summary">Advanced: Manual Configuration</summary>
      <div class="resource-grid">
        <div class="namd-field-group">
          <label class="namd-label" for="cores">Cores *</label>
          <input
            class="namd-input"
            id="cores"
            type="number"
            bind:value={resourceConfig.cores}
            min="1"
            max={$partitions.find(p => p.name === resourceConfig.partition)?.max_cores ?? 64}
            class:error={errors.cores}
          />
          {#if errors.cores}
            <span class="error-text">{errors.cores}</span>
          {/if}
        </div>

        <div class="namd-field-group">
          <label class="namd-label" for="memory">Memory *</label>
          <input
            class="namd-input"
            id="memory"
            type="text"
            bind:value={resourceConfig.memory}
            placeholder="32GB"
            class:error={errors.memory}
          />
          {#if errors.memory}
            <span class="error-text">{errors.memory}</span>
          {/if}
        </div>

        <div class="namd-field-group">
          <label class="namd-label" for="walltime">Wall Time *</label>
          <input
            class="namd-input"
            id="walltime"
            type="text"
            bind:value={resourceConfig.walltime}
            placeholder="24:00:00"
            class:error={errors.walltime}
          />
          {#if errors.walltime}
            <span class="error-text">{errors.walltime}</span>
          {/if}
        </div>

        <div class="namd-field-group">
          <label class="namd-label" for="partition">Partition *</label>
          <select
            class="namd-input"
            id="partition"
            bind:value={resourceConfig.partition}
            class:error={errors.partition}
          >
            {#each $partitions as partition}
              <option value={partition.name}>{partition.title}</option>
            {/each}
          </select>
          {#if errors.partition}
            <span class="error-text">{errors.partition}</span>
          {/if}
        </div>

        <div class="namd-field-group">
          <label class="namd-label" for="qos">QoS *</label>
          <select
            class="namd-input"
            id="qos"
            bind:value={resourceConfig.qos}
            class:error={errors.qos}
          >
            {#each $allQosOptions as qos}
              <option value={qos.name}>{qos.title}</option>
            {/each}
          </select>
          {#if errors.qos}
            <span class="error-text">{errors.qos}</span>
          {/if}
        </div>
      </div>
    </details>
  </div>

  <!-- Validation Bar -->
  <div class="namd-section">
    <div class="validation-bar" class:valid={validation.is_valid} class:invalid={!validation.is_valid}>
      <div class="validation-status">
        {#if validation.is_valid}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20,6 9,17 4,12"/>
          </svg>
          <span>Valid Configuration</span>
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
          <span>{validation.issues.length} Issue{validation.issues.length === 1 ? '' : 's'}</span>
        {/if}
      </div>
      <div class="validation-stats">
        <span class="stat-item">Cost: {costEstimate.totalCost} SU</span>
        <span class="stat-item">Queue: {costEstimate.queueEstimate}</span>
      </div>
    </div>

    <!-- Expandable Issues/Warnings/Suggestions -->
    <ValidationDisplay {validation} collapsible={true} />

    <!-- Preview Script Button -->
    <div class="preview-section">
      <button
        type="button"
        class="namd-button namd-button--secondary"
        on:click={handleScriptPreview}
        disabled={isGeneratingScript}
      >
        {isGeneratingScript ? 'Generating Preview...' : 'Preview SLURM Script'}
      </button>
    </div>
  </div>
</div>

<!-- Script Preview Modal -->
<PreviewModal
  isOpen={showScriptPreview}
  title="SLURM Script Preview"
  content={scriptPreviewContent}
  onClose={() => showScriptPreview = false}
/>

<style>
  .section-description {
    color: var(--namd-text-secondary);
    margin: 0;
    font-size: var(--namd-font-size-base);
  }

  .preset-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-sm);
  }

  .preset-pill {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-md);
    background: var(--namd-bg-primary);
    border: 2px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    cursor: pointer;
    transition: all 0.2s ease;
    flex: 1;
    min-width: 160px;
  }

  .preset-pill:hover {
    border-color: var(--namd-primary);
    box-shadow: var(--namd-shadow-md);
  }

  .preset-pill.selected {
    border-color: var(--namd-primary);
    border-width: 3px;
    background-color: var(--namd-primary-bg);
    box-shadow: var(--namd-shadow-lg);
  }

  .preset-pill.selected .preset-name {
    color: var(--namd-primary);
  }

  .preset-name {
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .preset-specs {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
    font-family: var(--namd-font-mono);
  }

  .manual-config-summary {
    cursor: pointer;
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-secondary);
    margin-bottom: var(--namd-spacing-md);
  }

  .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--namd-spacing-lg);
    margin-top: var(--namd-spacing-md);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-xs);
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }

  .validation-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius);
    border: 2px solid var(--namd-border);
  }

  .validation-bar.valid {
    border-color: var(--namd-success);
    background: var(--namd-success-bg);
  }

  .validation-bar.invalid {
    border-color: var(--namd-error);
    background: var(--namd-error-bg);
  }

  .validation-status {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    font-weight: var(--namd-font-weight-medium);
  }

  .validation-bar.valid .validation-status {
    color: var(--namd-success);
  }

  .validation-bar.invalid .validation-status {
    color: var(--namd-error);
  }

  .validation-stats {
    display: flex;
    gap: var(--namd-spacing-lg);
  }

  .stat-item {
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-secondary);
    font-family: var(--namd-font-mono);
  }

  .preview-section {
    margin-top: var(--namd-spacing-md);
  }
</style>

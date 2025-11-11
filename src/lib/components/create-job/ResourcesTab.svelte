<script lang="ts">
  import { logger } from '$lib/utils/logger';
  import { invoke } from '@tauri-apps/api/core';
  import type { PreviewResult } from '$lib/types/api';
  import { jobPresets, partitions, allQosOptions, validateResourceRequest, calculateJobCost, estimateQueueTime, walltimeToHours } from '$lib/stores/clusterConfig';
  import type { JobPreset } from '$lib/types/api';
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
  let validation: any = { is_valid: true, issues: [], warnings: [], suggestions: [] };
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
    const walltimeHours = walltimeToHours(resourceConfig.walltime);
    const partitionSpec = $partitions.find(p => p.id === resourceConfig.partition);
    const hasGpu = partitionSpec?.gpu_type ? true : false;
    const gpuCount = partitionSpec?.gpu_count || 1;

    const totalCost = await calculateJobCost(resourceConfig.cores, walltimeHours, hasGpu, gpuCount);
    const queueEstimate = await estimateQueueTime(resourceConfig.cores, resourceConfig.partition);

    costEstimate = { totalCost, queueEstimate };
  }

  function handlePresetSelect(preset: JobPreset) {
    selectedPresetId = preset.id;
    resourceConfig.cores = preset.config.cores;
    resourceConfig.memory = preset.config.memory;
    resourceConfig.walltime = preset.config.wall_time;
    resourceConfig.partition = preset.config.partition;
    resourceConfig.qos = preset.config.qos;
  }

  async function handleScriptPreview() {
    isGeneratingScript = true;

    try {
      const result = await invoke<PreviewResult>('preview_slurm_script', {
        job_name: 'preview_job',
        cores: resourceConfig.cores,
        memory: resourceConfig.memory,
        walltime: resourceConfig.walltime,
        partition: resourceConfig.partition || null,
        qos: resourceConfig.qos || null
      });

      if (result.success && result.content) {
        scriptPreviewContent = result.content;
        showScriptPreview = true;
      } else {
        logger.error('[ResourcesTab]', `Script preview failed: ${result.error || 'Unknown error'}`);
      }
    } catch (error) {
      logger.error('[ResourcesTab]', 'Script preview error', error);
    } finally {
      isGeneratingScript = false;
    }
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
          class:selected={selectedPresetId === preset.id}
          on:click={() => handlePresetSelect(preset)}
        >
          <span class="preset-name">{preset.name}</span>
          <span class="preset-specs">({preset.config.cores}c, {preset.config.memory}, {preset.config.wall_time})</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Manual Configuration -->
  <div class="namd-section">
    <details>
      <summary class="manual-config-summary">Advanced: Manual Configuration</summary>
      <div class="resource-grid">
        <div class="field-group">
          <label class="namd-label" for="cores">Cores *</label>
          <input
            class="namd-input"
            id="cores"
            type="number"
            bind:value={resourceConfig.cores}
            min="1"
            max={parseInt($partitions.find(p => p.id === resourceConfig.partition)?.cores_per_node ?? '64')}
            class:error={errors.cores}
          />
          {#if errors.cores}
            <span class="error-text">{errors.cores}</span>
          {/if}
        </div>

        <div class="field-group">
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

        <div class="field-group">
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

        <div class="field-group">
          <label class="namd-label" for="partition">Partition *</label>
          <select
            class="namd-input"
            id="partition"
            bind:value={resourceConfig.partition}
            class:error={errors.partition}
          >
            {#each $partitions as partition}
              <option value={partition.id}>{partition.name} - {partition.title}</option>
            {/each}
          </select>
          {#if errors.partition}
            <span class="error-text">{errors.partition}</span>
          {/if}
        </div>

        <div class="field-group">
          <label class="namd-label" for="qos">QoS *</label>
          <select
            class="namd-input"
            id="qos"
            bind:value={resourceConfig.qos}
            class:error={errors.qos}
          >
            {#each $allQosOptions as qos}
              <option value={qos.id}>{qos.name} - {qos.description}</option>
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

    <!-- Expandable Issues/Warnings -->
    {#if validation.issues.length > 0 || validation.warnings.length > 0}
      <details class="validation-details">
        <summary>Show Details</summary>
        {#if validation.issues.length > 0}
          <div class="issues-list">
            <strong>Issues:</strong>
            <ul>
              {#each validation.issues as issue}
                <li class="issue-error">{issue}</li>
              {/each}
            </ul>
          </div>
        {/if}
        {#if validation.warnings.length > 0}
          <div class="warnings-list">
            <strong>Warnings:</strong>
            <ul>
              {#each validation.warnings as warning}
                <li class="issue-warning">{warning}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </details>
    {/if}

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
    font-size: var(--namd-font-size-sm);
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
    gap: 0.25rem;
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
    background-color: rgba(59, 130, 246, 0.05);
    box-shadow: var(--namd-shadow-md);
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

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
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
    background: rgba(46, 125, 50, 0.05);
  }

  .validation-bar.invalid {
    border-color: var(--namd-error);
    background: rgba(220, 38, 38, 0.05);
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
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-secondary);
    font-family: var(--namd-font-mono);
  }

  .validation-details {
    margin-top: var(--namd-spacing-md);
  }

  .validation-details summary {
    cursor: pointer;
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .issues-list, .warnings-list {
    margin-top: var(--namd-spacing-sm);
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius-sm);
  }

  .issues-list {
    background: var(--namd-error-bg);
    border: 1px solid var(--namd-error);
  }

  .warnings-list {
    background: var(--namd-warning-bg);
    border: 1px solid var(--namd-warning-border);
  }

  .issues-list ul, .warnings-list ul {
    margin: var(--namd-spacing-sm) 0 0 0;
    padding-left: 1.5rem;
  }

  .issue-error {
    color: var(--namd-error);
  }

  .issue-warning {
    color: var(--namd-warning-fg);
  }

  .preview-section {
    margin-top: var(--namd-spacing-md);
  }
</style>

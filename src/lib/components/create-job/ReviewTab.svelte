<script lang="ts">
  import type { NAMDConfig } from '../../types/api';

  export let job_name: string;

  export let resourceConfig: {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
  };

  export let namdConfig: NAMDConfig;

  export let uploadedFiles: { name: string; size: number; type: string; path: string }[];
  export let errors: Record<string, string>;
  export let formatFileSize: (bytes: number) => string;
  export let onSubmit: () => void;
  export let onCancel: () => void;
  export let isSubmitting: boolean = false;
  export let uploadProgress: Map<string, { percentage: number }> = new Map();
</script>

<div class="namd-tab-panel">
  <div class="namd-section">
    <div class="namd-section-header" style="margin-bottom: var(--namd-spacing-xl);">
      <h3 class="namd-section-title" style="font-size: var(--namd-font-size-xl); margin-bottom: var(--namd-spacing-sm);">Review Configuration</h3>
      <p class="section-description">Review your job configuration and submit when ready</p>
      {#if Object.keys(errors).length > 0}
        <div class="validation-summary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span>
            {Object.keys(errors).length} validation error{Object.keys(errors).length === 1 ? '' : 's'} found.
            Please fix these issues before creating the job.
          </span>
        </div>
      {/if}
    </div>

    <!-- Resource Summary -->
    <div class="review-section">
      <h4 class="review-section-title">Resources</h4>
      <div class="review-grid">
        <div class="review-item" class:error={errors.partition}>
          <span class="review-label">Partition:</span>
          <span class="review-value">{resourceConfig.partition}</span>
          {#if errors.partition}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.qos}>
          <span class="review-label">QOS:</span>
          <span class="review-value">{resourceConfig.qos}</span>
          {#if errors.qos}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.cores}>
          <span class="review-label">Cores:</span>
          <span class="review-value">{resourceConfig.cores}</span>
          {#if errors.cores}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.memory}>
          <span class="review-label">Memory:</span>
          <span class="review-value">{resourceConfig.memory} GB</span>
          {#if errors.memory}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.wallTime}>
          <span class="review-label">Wall Time:</span>
          <span class="review-value">{resourceConfig.wallTime}</span>
          {#if errors.wallTime}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Configuration Summary -->
    <div class="review-section">
      <h4 class="review-section-title">NAMD Configuration</h4>
      <div class="review-grid">
        <div class="review-item" class:error={errors.job_name}>
          <span class="review-label">Job Name:</span>
          <span class="review-value">{job_name || 'Not set'}</span>
          {#if errors.job_name}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.outputname}>
          <span class="review-label">Output Name:</span>
          <span class="review-value">{namdConfig.outputname || 'Not set'}</span>
          {#if errors.outputname}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.steps}>
          <span class="review-label">Simulation Steps:</span>
          <span class="review-value">{namdConfig.steps?.toLocaleString() || 'Not set'}</span>
          {#if errors.steps}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.temperature}>
          <span class="review-label">Temperature:</span>
          <span class="review-value">{namdConfig.temperature || 'Not set'} K</span>
          {#if errors.temperature}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.timestep}>
          <span class="review-label">Timestep:</span>
          <span class="review-value">{namdConfig.timestep || 'Not set'} fs</span>
          {#if errors.timestep}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item">
          <span class="review-label">DCD Frequency:</span>
          <span class="review-value">{namdConfig.dcd_freq || 'Not set'}</span>
        </div>
      </div>
    </div>

    <!-- Files Summary -->
    <div class="review-section">
      <h4 class="review-section-title">Input Files</h4>
      {#if uploadedFiles.length > 0}
        <div class="files-summary" class:error={errors.files}>
          {#each uploadedFiles as file}
            <div class="file-summary-item">
              {#if uploadProgress.has(file.name)}
                <div class="file-progress-bg" style="width: {uploadProgress.get(file.name)?.percentage || 0}%"></div>
              {/if}
              <div class="file-content">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                  <polyline points="14,2 14,8 20,8"/>
                </svg>
                <span class="file-name">{file.name}</span>
                <span class="file-size">({formatFileSize(file.size)})</span>
                {#if uploadProgress.has(file.name)}
                  <span class="file-progress-text">{uploadProgress.get(file.name)?.percentage.toFixed(0)}%</span>
                {/if}
              </div>
            </div>
          {/each}
          {#if errors.files}
            <div class="error-text">{errors.files}</div>
          {/if}
        </div>
      {:else}
        <div class="no-files" class:error={errors.files}>
          No files uploaded
          {#if errors.files}
            <span class="error-indicator">⚠</span>
            <div class="error-text">{errors.files}</div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="review-actions">
      <button
        type="button"
        class="namd-button namd-button--secondary"
        on:click={onCancel}
        disabled={isSubmitting}
      >
        Back to Jobs
      </button>
      <button
        type="button"
        class="namd-button namd-button--primary"
        on:click={onSubmit}
        disabled={isSubmitting || Object.keys(errors).length > 0}
      >
        {isSubmitting ? "Creating Job..." : "Create Job"}
      </button>
    </div>
  </div>
</div>

<style>

  .section-description {
    color: var(--namd-text-secondary);
    margin: 0 0 var(--namd-spacing-md) 0;
  }

  .validation-summary {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-md);
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
    border-radius: var(--namd-border-radius-sm);
    border: 1px solid var(--namd-warning-border);
  }

  .review-section {
    margin-bottom: var(--namd-spacing-xl);
    padding: var(--namd-spacing-lg);
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius);
    border: 1px solid var(--namd-border);
  }

  .review-section-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-md) 0;
  }

  .review-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .review-item {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-primary);
    border-radius: var(--namd-border-radius-sm);
  }

  .review-item.error {
    border: 1px solid var(--namd-error);
    background-color: var(--namd-error-bg);
  }

  .review-label {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-secondary);
    min-width: 80px;
  }

  .review-value {
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
    flex: 1;
  }

  .error-indicator {
    color: var(--namd-error);
    font-size: var(--namd-font-size-lg);
  }

  .files-summary {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .files-summary.error {
    border: 1px solid var(--namd-error);
    background-color: var(--namd-error-bg);
    padding: var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
  }

  .file-summary-item {
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-primary);
    border-radius: var(--namd-border-radius-sm);
  }

  .file-progress-bg {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: linear-gradient(90deg, rgba(59, 130, 246, 0.2) 0%, rgba(59, 130, 246, 0.4) 100%);
    transition: width 0.3s ease;
    z-index: 0;
    pointer-events: none;
  }

  .file-content {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    width: 100%;
  }

  .file-name {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .file-size {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .file-progress-text {
    margin-left: auto;
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-primary);
    font-size: var(--namd-font-size-sm);
  }

  .no-files {
    color: var(--namd-text-secondary);
    font-style: italic;
    padding: var(--namd-spacing-md);
    text-align: center;
  }

  .no-files.error {
    border: 1px solid var(--namd-error);
    background-color: var(--namd-error-bg);
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-error-fg);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-sm);
    margin-top: var(--namd-spacing-xs);
  }

  .review-actions {
    display: flex;
    justify-content: space-between;
    gap: var(--namd-spacing-md);
    margin-top: var(--namd-spacing-xl);
    padding-top: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
  }

</style>
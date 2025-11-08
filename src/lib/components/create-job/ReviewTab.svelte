<script lang="ts">
  export let jobName: string;
  export let templateId: string;
  export let templateValues: Record<string, any>;
  export let resourceConfig: {
    cores: number;
    memory: string;
    walltime: string;
    partition: string;
    qos: string;
  };
  export let errors: Record<string, string>;
  export let uploadProgress: Map<string, { percentage: number }>;
  export let uploadFileList: string[] = [];
  export let onSubmit: () => void;
  export let onCancel: () => void;
  export let isSubmitting: boolean = false;

  // Build file list with progress (backend tells us which files exist)
  $: filesWithProgress = uploadFileList.map(fileName => ({
    name: fileName,
    progress: uploadProgress.get(fileName)?.percentage || 0
  }));
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
          <span class="review-value">{resourceConfig.memory}</span>
          {#if errors.memory}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.walltime}>
          <span class="review-label">Wall Time:</span>
          <span class="review-value">{resourceConfig.walltime}</span>
          {#if errors.walltime}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Template Configuration Summary -->
    <div class="review-section">
      <h4 class="review-section-title">Configuration</h4>
      <div class="review-grid">
        <div class="review-item" class:error={errors.job_name}>
          <span class="review-label">Job Name:</span>
          <span class="review-value">{jobName || 'Not set'}</span>
          {#if errors.job_name}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        <div class="review-item" class:error={errors.template}>
          <span class="review-label">Template:</span>
          <span class="review-value">{templateId || 'Not selected'}</span>
          {#if errors.template}
            <span class="error-indicator">⚠</span>
          {/if}
        </div>
        {#each Object.entries(templateValues) as [key, value]}
          {#if typeof value !== 'string' || !value.includes('/')}
            <div class="review-item">
              <span class="review-label">{key}:</span>
              <span class="review-value">{value}</span>
            </div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- Input Files with Upload Progress -->
    <div class="review-section">
      <h4 class="review-section-title">Input Files</h4>
      {#if filesWithProgress.length > 0}
        <div class="files-summary">
          {#each filesWithProgress as file}
            <div class="file-summary-item">
              <!-- Animated progress background -->
              <div class="file-progress-bg" style="width: {file.progress}%"></div>

              <div class="file-content">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                  <polyline points="14,2 14,8 20,8"/>
                </svg>
                <span class="file-name">{file.name}</span>
                {#if file.progress > 0}
                  <span class="file-progress-text">{file.progress.toFixed(0)}%</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="no-files">No files to upload</div>
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
        {isSubmitting ? 'Creating Job...' : 'Create Job'}
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

  .file-summary-item {
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
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

  .review-actions {
    display: flex;
    justify-content: space-between;
    gap: var(--namd-spacing-md);
    margin-top: var(--namd-spacing-xl);
    padding-top: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
  }
</style>

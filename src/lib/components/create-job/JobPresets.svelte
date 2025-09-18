<script lang="ts">
  import { JOB_PRESETS, type JobPreset } from '../../data/cluster-config';

  export let onPresetSelect: (preset: JobPreset) => void;
  export let selectedPresetId: string = '';

  // Use centralized presets data
  const presets = JOB_PRESETS;

  function handlePresetClick(preset: JobPreset) {
    selectedPresetId = preset.id;
    onPresetSelect(preset);
  }
</script>

<div class="job-presets">
  <div class="presets-header">
    <h3 class="presets-title">Resource Templates</h3>
    <p class="presets-description">
      Apply a template to quickly configure resources for common NAMD scenarios
    </p>
  </div>

  <div class="presets-grid">
    {#each presets as preset}
      <div
        class="preset-card"
        class:selected={selectedPresetId === preset.id}
        class:standard={preset.category === 'production'}
        on:click={() => handlePresetClick(preset)}
        role="button"
        tabindex="0"
        on:keydown={(e) => e.key === 'Enter' && handlePresetClick(preset)}
      >
        <div class="card-header">
          <div class="preset-icon">{preset.icon}</div>
          <div class="preset-info">
            <div class="preset-name">
              {preset.name}
              {#if preset.category === 'production'}
                <span class="namd-badge namd-badge--success">Standard</span>
              {/if}
            </div>
            <div class="preset-description">{preset.description}</div>
          </div>

          <!-- Resource summary (inline) -->
          <div class="resource-inline">
            <span class="resource-spec">{preset.config.cores} cores</span>
            <span class="resource-spec">{preset.config.memory}GB</span>
            <span class="resource-spec">{preset.config.partition}</span>
            {#if preset.requiresGpu}
              <span class="resource-spec gpu">GPU</span>
            {/if}
          </div>
        </div>

        <!-- Quick stats -->
        <div class="card-stats">
          <div class="stat-item">
            <span class="stat-value">{preset.estimatedCost}</span>
            <span class="stat-label">Cost</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{preset.estimatedQueue}</span>
            <span class="stat-label">Queue</span>
          </div>
        </div>

        <!-- Selection indicator -->
        {#if selectedPresetId === preset.id}
          <div class="selected-indicator">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20,6 9,17 4,12"/>
            </svg>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Custom option -->
  <div class="custom-option">
    <div class="custom-card">
      <div class="custom-icon">⚙️</div>
      <div class="custom-content">
        <h4 class="custom-title">Custom Configuration</h4>
        <p class="custom-description">
          Already know what you need? Configure resources manually using the form below.
        </p>
      </div>
    </div>
  </div>
</div>

<style>
  .job-presets {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .presets-header {
    text-align: center;
  }

  .presets-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-sm) 0;
  }

  .presets-description {
    color: var(--namd-text-secondary);
    margin: 0;
    font-size: var(--namd-font-size-sm);
  }

  .presets-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--namd-spacing-sm);
  }

  @media (min-width: 768px) {
    .presets-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1024px) {
    .presets-grid {
      grid-template-columns: repeat(4, 1fr);
    }
  }

  .preset-card {
    position: relative;
    background-color: var(--namd-bg-primary);
    border: 2px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-md);
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
    min-height: 120px;
  }

  .preset-card:hover {
    border-color: var(--namd-primary);
    box-shadow: var(--namd-shadow-md);
  }

  .preset-card.selected {
    border-color: var(--namd-primary);
    background-color: rgba(59, 130, 246, 0.05);
    box-shadow: var(--namd-shadow-md);
  }

  .preset-card.standard {
    border-color: var(--namd-success);
  }

  .preset-card.standard.selected {
    border-color: var(--namd-primary);
  }

  .card-header {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
    flex: 1;
  }

  .preset-icon {
    font-size: 1.5rem;
    align-self: flex-start;
  }

  .preset-info {
    flex: 1;
  }

  .preset-name {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    font-size: var(--namd-font-size-md);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin-bottom: var(--namd-spacing-xs);
    flex-wrap: wrap;
  }


  .preset-description {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
    line-height: 1.3;
    margin-bottom: var(--namd-spacing-sm);
  }

  .resource-inline {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-xs);
  }

  .resource-spec {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-family: var(--namd-font-mono);
    font-weight: var(--namd-font-weight-medium);
  }

  .resource-spec.gpu {
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
  }

  .card-stats {
    display: flex;
    justify-content: space-between;
    gap: var(--namd-spacing-sm);
    margin-top: auto;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .stat-value {
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    line-height: 1.2;
  }

  .stat-label {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
    font-weight: var(--namd-font-weight-medium);
  }

  .selected-indicator {
    position: absolute;
    top: var(--namd-spacing-sm);
    right: var(--namd-spacing-sm);
    color: var(--namd-primary);
    background-color: var(--namd-bg-primary);
    border-radius: 50%;
    padding: var(--namd-spacing-xs);
    box-shadow: var(--namd-shadow);
  }

  .resource-summary {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius-sm);
  }

  .resource-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--namd-font-size-xs);
  }

  .resource-label {
    color: var(--namd-text-secondary);
    font-weight: var(--namd-font-weight-medium);
  }

  .resource-value {
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
    font-weight: var(--namd-font-weight-medium);
  }

  .resource-value.partition {
    color: var(--namd-primary);
  }

  .estimates {
    display: flex;
    justify-content: space-between;
    gap: var(--namd-spacing-md);
  }

  .estimate-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .estimate-label {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
    font-weight: var(--namd-font-weight-medium);
    margin-bottom: var(--namd-spacing-xs);
  }

  .estimate-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    font-weight: var(--namd-font-weight-semibold);
  }

  .gpu-notice {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
  }

  .use-cases {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .use-cases-label {
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .use-cases-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-xs);
  }

  .use-case {
    background-color: var(--namd-accent);
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
  }

  .action-hint {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--namd-spacing-sm);
    padding-top: var(--namd-spacing-sm);
    border-top: 1px solid var(--namd-border);
    color: var(--namd-text-muted);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
  }

  .custom-option {
    margin-top: var(--namd-spacing-lg);
  }

  .custom-card {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    padding: var(--namd-spacing-lg);
    background-color: var(--namd-bg-muted);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .custom-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  .custom-content {
    flex: 1;
  }

  .custom-title {
    font-size: var(--namd-font-size-md);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-xs) 0;
  }

  .custom-description {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
    margin: 0;
    line-height: 1.4;
  }
</style>
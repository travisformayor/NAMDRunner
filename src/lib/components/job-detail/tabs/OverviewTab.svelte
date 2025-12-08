<script lang="ts">
  import type { JobInfo } from '../../../types/api';
  import { getStatusBadgeClass } from '../../../utils/file-helpers';

  export let job: JobInfo;

  // Reactive computed values for SLURM config
  $: slurmConfig = {
    cores: job.slurm_config.cores,
    memory: job.slurm_config.memory,
    wallTime: job.slurm_config.walltime,
    partition: job.slurm_config.partition || 'N/A',
  };

  // Extract key template values for display
  $: steps = job.template_values?.steps || 0;
  $: temperature = job.template_values?.temperature || 0;
  $: timestep = job.template_values?.timestep || 0;
</script>

<div class="namd-tab-panel">
  <div class="overview-content">
    <!-- Resource Allocation -->
    <div class="overview-section">
      <h3>Resource Allocation</h3>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Cores</span>
          <span class="info-value">{slurmConfig.cores}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Memory</span>
          <span class="info-value">{slurmConfig.memory}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Wall Time</span>
          <span class="info-value">{slurmConfig.wallTime}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Partition</span>
          <span class="info-value">{slurmConfig.partition}</span>
        </div>
      </div>
    </div>

    <!-- Job Information -->
    <div class="overview-section">
      <h3>Job Information</h3>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Status</span>
          <span class="info-value namd-status-badge {getStatusBadgeClass(job.status)}">{job.status}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Template</span>
          <span class="info-value">{job.template_id || 'N/A'}</span>
        </div>
        {#if typeof steps === 'number' && steps > 0}
          <div class="info-item">
            <span class="info-label">Simulation Steps</span>
            <span class="info-value">{steps.toLocaleString()}</span>
          </div>
        {/if}
        {#if typeof temperature === 'number' && temperature > 0}
          <div class="info-item">
            <span class="info-label">Temperature</span>
            <span class="info-value">{temperature} K</span>
          </div>
        {/if}
        {#if typeof timestep === 'number' && timestep > 0}
          <div class="info-item">
            <span class="info-label">Timestep</span>
            <span class="info-value">{timestep} fs</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Template Values -->
    <div class="overview-section">
      <h3>Template Configuration</h3>
      <div class="template-values">
        {#if job.template_values && Object.keys(job.template_values).length > 0}
          <div class="info-grid">
            {#each Object.entries(job.template_values) as [key, value]}
              <div class="info-item">
                <span class="info-label">{key}</span>
                <span class="info-value">{typeof value === 'object' ? JSON.stringify(value) : value}</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="namd-text-sm">No template values available</p>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .namd-tab-panel {
    padding: var(--namd-spacing-lg);
  }

  .overview-content {
    max-width: var(--namd-max-width-form);
  }

  .overview-section {
    margin-bottom: var(--namd-spacing-xl);
  }

  .overview-section h3 {
    margin-bottom: var(--namd-spacing-md);
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-medium);
  }

  .progress-card {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-md);
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--namd-spacing-sm);
  }

  .progress-bar {
    height: 8px;
    background: var(--namd-bg-primary);
    border-radius: var(--namd-border-radius-sm);
    overflow: hidden;
    margin-bottom: var(--namd-spacing-sm);
  }

  .progress-fill {
    height: 100%;
    background: var(--namd-primary);
    transition: width 0.3s ease;
  }

  .progress-details {
    display: flex;
    justify-content: space-between;
    color: var(--namd-text-secondary);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .info-label {
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-secondary);
    font-weight: var(--namd-font-weight-medium);
  }

  .info-value {
    font-size: var(--namd-font-size-md);
    color: var(--namd-text-primary);
  }

  .template-values {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-md);
  }
</style>

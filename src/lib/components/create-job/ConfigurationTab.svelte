<script lang="ts">
  import type { NAMDConfig } from '../../types/api';

  export let job_name: string;
  export let namdConfig: NAMDConfig;
  export let errors: Record<string, string>;
</script>

<div class="namd-tab-panel">
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">NAMD Configuration</h3>
    </div>
    <div class="config-grid">
      <div class="field-group">
        <label class="namd-label" for="jobName">Job Name *</label>
        <input
          class="namd-input"
          id="jobName"
          type="text"
          bind:value={job_name}
          placeholder="my-simulation"
          class:error={errors.job_name}
        />
        {#if errors.job_name}
          <span class="error-text">{errors.job_name}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="outputName">Output Basename *</label>
        <input
          class="namd-input"
          id="outputName"
          type="text"
          bind:value={namdConfig.outputname}
          placeholder="output"
          class:error={errors.outputname}
        />
        {#if errors.outputname}
          <span class="error-text">{errors.outputname}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="simulationSteps">Simulation Steps *</label>
        <input
          class="namd-input"
          id="simulationSteps"
          type="number"
          bind:value={namdConfig.steps}
          min="1"
          class:error={errors.steps}
        />
        {#if errors.steps}
          <span class="error-text">{errors.steps}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="temperature">Temperature (K) *</label>
        <input
          class="namd-input"
          id="temperature"
          type="number"
          bind:value={namdConfig.temperature}
          min="1"
          class:error={errors.temperature}
        />
        {#if errors.temperature}
          <span class="error-text">{errors.temperature}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="timestep">Timestep (fs) *</label>
        <input
          class="namd-input"
          id="timestep"
          type="number"
          bind:value={namdConfig.timestep}
          min="0.1"
          step="0.1"
          class:error={errors.timestep}
        />
        {#if errors.timestep}
          <span class="error-text">{errors.timestep}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="dcdFreq">DCD Frequency</label>
        <input
          class="namd-input"
          id="dcdFreq"
          type="number"
          bind:value={namdConfig.dcd_freq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="restartFreq">Restart Frequency</label>
        <input
          class="namd-input"
          id="restartFreq"
          type="number"
          bind:value={namdConfig.restart_freq}
          min="1"
        />
      </div>
    </div>
  </div>
</div>

<style>

  .config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--namd-spacing-lg);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-xs);
    margin-top: var(--namd-spacing-xs);
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }

</style>
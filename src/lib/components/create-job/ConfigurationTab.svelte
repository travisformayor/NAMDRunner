<script lang="ts">
  export let namdConfig: {
    jobName: string;
    simulationSteps: number;
    temperature: number;
    timestep: number;
    outputName: string;
    dcdFreq: number;
    restartFreq: number;
  };

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
          bind:value={namdConfig.jobName}
          placeholder="my-simulation"
          class:error={errors.jobName}
        />
        {#if errors.jobName}
          <span class="error-text">{errors.jobName}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="outputName">Output Basename *</label>
        <input
          class="namd-input"
          id="outputName"
          type="text"
          bind:value={namdConfig.outputName}
          placeholder="output"
          class:error={errors.outputName}
        />
        {#if errors.outputName}
          <span class="error-text">{errors.outputName}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="simulationSteps">Simulation Steps *</label>
        <input
          class="namd-input"
          id="simulationSteps"
          type="number"
          bind:value={namdConfig.simulationSteps}
          min="1"
          class:error={errors.simulationSteps}
        />
        {#if errors.simulationSteps}
          <span class="error-text">{errors.simulationSteps}</span>
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
          bind:value={namdConfig.dcdFreq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="restartFreq">Restart Frequency</label>
        <input
          class="namd-input"
          id="restartFreq"
          type="number"
          bind:value={namdConfig.restartFreq}
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
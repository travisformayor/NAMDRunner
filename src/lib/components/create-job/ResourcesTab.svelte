<script lang="ts">
  import JobPresets from './JobPresets.svelte';
  import PartitionSelector from './PartitionSelector.svelte';
  import CompactQosSelector from './CompactQosSelector.svelte';
  import ResourceValidator from './ResourceValidator.svelte';

  export let resourceConfig: {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
  };

  export let errors: Record<string, string>;
  export let selectedPresetId: string;
  export let onPresetSelect: (preset: any) => void;
  export let onPartitionChange: (partition: string) => void;
  export let onQosChange: (qos: string) => void;
</script>

<div class="namd-tab-panel">
  <!-- Job Presets -->
  <div class="namd-section">
    <JobPresets
      onPresetSelect={onPresetSelect}
      selectedPresetId={selectedPresetId}
    />
  </div>

  <!-- Partition Selection -->
  <div class="namd-section">
    <PartitionSelector
      selectedPartition={resourceConfig.partition}
      onChange={onPartitionChange}
    />
  </div>

  <!-- SLURM Resource Allocation -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">SLURM Resource Allocation</h3>
    </div>
    <div class="resource-grid">
      <div class="field-group">
        <label class="namd-label" for="cores">Cores *</label>
        <input
          class="namd-input"
          id="cores"
          type="number"
          bind:value={resourceConfig.cores}
          min="1"
          max="1024"
          placeholder="128"
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
        <label class="namd-label" for="wallTime">Wall Time *</label>
        <input
          class="namd-input"
          id="wallTime"
          type="text"
          bind:value={resourceConfig.wallTime}
          placeholder="24:00:00"
          class:error={errors.wallTime}
        />
        {#if errors.wallTime}
          <span class="error-text">{errors.wallTime}</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- QOS Selection -->
  <div class="namd-section">
    <CompactQosSelector
      selectedPartition={resourceConfig.partition}
      selectedQos={resourceConfig.qos}
      wallTime={resourceConfig.wallTime}
      onChange={onQosChange}
    />
  </div>

  <!-- Resource Validation -->
  <div class="namd-section">
    <ResourceValidator
      cores={resourceConfig.cores}
      memory={resourceConfig.memory}
      wallTime={resourceConfig.wallTime}
      partition={resourceConfig.partition}
      qos={resourceConfig.qos}
    />
  </div>
</div>

<style>

  .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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
<script lang="ts">
  import { getQosForPartition, suggestQos, walltimeToHours } from '../../data/cluster-config';

  export let selectedPartition: string;
  export let selectedQos: string;
  export let wallTime: string;
  export let onChange: (qos: string) => void;

  // Filter QOS options based on selected partition
  $: availableQos = getQosForPartition(selectedPartition);

  // Auto-suggest QOS based on walltime
  $: suggestedQos = suggestQos(walltimeToHours(wallTime), selectedPartition);

  function handleQosSelect(qosId: string) {
    selectedQos = qosId;
    onChange(qosId);
  }

  // Auto-update QOS when suggestions change
  $: if (suggestedQos !== selectedQos && availableQos.some(q => q.id === suggestedQos)) {
    handleQosSelect(suggestedQos);
  }
</script>

<div class="compact-qos-selector">
  <label class="namd-label">Quality of Service (QOS)</label>

  <div class="qos-options">
    {#each availableQos as qos}
      <label class="qos-option">
        <input
          type="radio"
          bind:group={selectedQos}
          value={qos.id}
          on:change={() => handleQosSelect(qos.id)}
        />
        <span class="qos-label">
          {qos.name}
          {#if qos.id === 'normal'}
            <span class="namd-badge namd-badge--success">Default</span>
          {/if}
          {#if suggestedQos === qos.id && suggestedQos !== selectedQos}
            <span class="namd-badge namd-badge--warning">Suggested</span>
          {/if}
        </span>
        <span class="qos-description">
          {#if qos.id === 'normal'}
            {qos.maxWalltimeHours}h max, {qos.nodeLimit} nodes max
          {:else if qos.id === 'long'}
            {qos.maxWalltimeHours}h max ({Math.floor(qos.maxWalltimeHours / 24)} days), {qos.nodeLimit} nodes max
          {:else if qos.id === 'mem'}
            {qos.maxWalltimeHours}h max ({Math.floor(qos.maxWalltimeHours / 24)} days), {qos.nodeLimit} nodes max
          {:else if qos.id === 'testing'}
            {qos.maxWalltimeHours}h max, {qos.nodeLimit} nodes max
          {:else}
            {qos.title}
          {/if}
        </span>
      </label>
    {/each}
  </div>

  <div class="qos-help">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <line x1="12" y1="8" x2="12" y2="12"/>
      <line x1="12" y1="16" x2="12.01" y2="16"/>
    </svg>
    <span class="help-text">
      QOS determines priority and resource limits for your job.
    </span>
  </div>
</div>

<style>
  .compact-qos-selector {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .qos-options {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius);
  }

  .qos-option {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    cursor: pointer;
    padding: var(--namd-spacing-xs);
    border-radius: var(--namd-border-radius-sm);
    transition: background-color 0.15s ease;
  }

  .qos-option:hover {
    background-color: var(--namd-bg-primary);
  }

  .qos-option input[type="radio"] {
    margin: 0;
    flex-shrink: 0;
  }

  .qos-label {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-xs);
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
    min-width: 80px;
  }


  .qos-description {
    flex: 1;
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-secondary);
    line-height: 1.3;
  }

  .qos-help {
    display: flex;
    align-items: flex-start;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-info-bg);
    color: var(--namd-info-fg);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    line-height: 1.4;
  }

  .qos-help svg {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .help-text {
    flex: 1;
  }
</style>
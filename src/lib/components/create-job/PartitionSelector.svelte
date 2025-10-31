<script lang="ts">
  import { partitions } from '../../stores/clusterConfig';
  import type { PartitionSpec } from '../../types/cluster';

  export let selectedPartition: string;  // No hardcoded default! Parent provides backend default
  export let onChange: (partition: string) => void;

  function handlePartitionSelect(partitionId: string) {
    if (typeof window !== 'undefined' && window.sshConsole) {
      window.sshConsole.addDebug(`[USER] Selected partition: ${partitionId}`);
    }
    selectedPartition = partitionId;
    onChange(partitionId);
  }
</script>

<div class="partition-selector">
  <div class="selector-header">
    <h3 class="selector-title">Choose Hardware Partition</h3>
    <p class="selector-description">
      Select the compute resources that best match your simulation requirements
    </p>
  </div>

  <div class="partition-grid">
    {#each $partitions as partition}
      <div
        class="partition-card"
        class:selected={selectedPartition === partition.id}
        class:standard={partition.is_standard}
        on:click={() => handlePartitionSelect(partition.id)}
        role="button"
        tabindex="0"
        on:keydown={(e) => e.key === 'Enter' && handlePartitionSelect(partition.id)}
      >
        <div class="card-header">
          <div class="partition-icon">
            {#if partition.gpu_type}
              🔥
            {:else}
              ⚡
            {/if}
          </div>
          <div class="partition-info">
            <div class="partition-name">
              {partition.name}
              {#if partition.is_standard}
                <span class="standard-badge">Standard</span>
              {/if}
            </div>
            <div class="partition-title">{partition.title}</div>
          </div>

          <!-- Key specs (inline) -->
          <div class="specs-inline">
            <span class="spec-chip">{partition.cores_per_node} cores</span>
            <span class="spec-chip">{partition.ram_per_core}</span>
            {#if partition.gpu_type}
              <span class="spec-chip gpu">{partition.gpu_type}</span>
            {/if}
          </div>
        </div>

        <!-- Quick stats -->
        <div class="card-stats">
          <div class="stat-item">
            <span class="stat-value">{partition.nodes}</span>
            <span class="stat-label">Nodes</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{partition.max_walltime}</span>
            <span class="stat-label">Max Time</span>
          </div>
        </div>

        <!-- Selection indicator -->
        {#if selectedPartition === partition.id}
          <div class="selected-indicator">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20,6 9,17 4,12"/>
            </svg>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Help section - Generated from backend data, no hardcoded cluster info! -->
  <div class="help-section">
    <details>
      <summary>Need help choosing a partition?</summary>
      <div class="help-content">
        <ul>
          {#each $partitions.filter(p => p.is_standard) as partition}
            <li>
              <strong>{partition.name}</strong> - {partition.description}
              {#if partition.is_default}
                <span class="default-badge">(Recommended)</span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    </details>
  </div>
</div>

<style>
  .partition-selector {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .selector-header {
    text-align: center;
  }

  .selector-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-sm) 0;
  }

  .selector-description {
    color: var(--namd-text-secondary);
    margin: 0;
    font-size: var(--namd-font-size-sm);
  }

  .partition-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--namd-spacing-sm);
  }

  @media (min-width: 768px) {
    .partition-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1200px) {
    .partition-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .partition-card {
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

  .partition-card:hover {
    border-color: var(--namd-primary);
    box-shadow: var(--namd-shadow-md);
  }

  .partition-card.selected {
    border-color: var(--namd-primary);
    background-color: rgba(59, 130, 246, 0.05);
    box-shadow: var(--namd-shadow-md);
  }

  .partition-card.standard {
    border-color: var(--namd-success);
  }

  .partition-card.standard:hover {
    border-color: var(--namd-success);
  }

  .card-header {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
    flex: 1;
  }

  .partition-icon {
    font-size: 1.5rem;
    align-self: flex-start;
  }

  .partition-info {
    flex: 1;
  }

  .partition-name {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-md);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin-bottom: var(--namd-spacing-xs);
    flex-wrap: wrap;
  }

  .standard-badge {
    background-color: var(--namd-success-bg);
    color: var(--namd-success-fg);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: 9999px;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .partition-title {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
    line-height: 1.3;
    margin-bottom: var(--namd-spacing-sm);
  }

  .specs-inline {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-xs);
  }

  .spec-chip {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-family: var(--namd-font-mono);
    font-weight: var(--namd-font-weight-medium);
  }

  .spec-chip.gpu {
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

  .help-section {
    margin-top: var(--namd-spacing-lg);
  }

  .help-section details {
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-md);
  }

  .help-section summary {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
    cursor: pointer;
    margin-bottom: var(--namd-spacing-sm);
  }

  .help-content {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
    line-height: 1.6;
  }

  .help-content ul {
    margin: 0;
    padding-left: var(--namd-spacing-lg);
  }

  .help-content li {
    margin-bottom: var(--namd-spacing-xs);
  }
</style>
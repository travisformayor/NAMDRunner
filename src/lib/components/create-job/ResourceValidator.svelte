<script lang="ts">
  import { onMount } from 'svelte';
  import { validateResourceRequest, calculateJobCost, estimateQueueTime, walltimeToHours, getPartition, billingRates } from '../../stores/clusterConfig';
  import type { ValidationResult } from '../../types/cluster';

  export let cores: number;
  export let memory: string;
  export let wallTime: string;
  export let partition: string;
  export let qos: string;

  // State for async validation result
  let validation: ValidationResult = {
    is_valid: true,
    issues: [],
    warnings: [],
    suggestions: []
  };

  // State for async cost estimate
  let costEstimate = {
    coreCost: 0,
    gpuCost: 0,
    totalCost: 0,
    queueEstimate: 'Unknown',
    hasGpu: false
  };

  // Trigger validation and cost calculation when inputs change
  $: if (cores || memory || wallTime || partition || qos) {
    updateValidation();
    updateCostEstimate();
  }

  async function updateCostEstimate() {
    costEstimate = await calculateCostEstimate(cores, memory, wallTime, partition);
  }

  async function updateValidation() {
    // Pass values directly to backend - no frontend parsing
    validation = await validateResourceRequest(cores, memory, wallTime, partition, qos);
    // Log only if validation fails
    if (!validation.is_valid && typeof window !== 'undefined' && window.sshConsole) {
      window.sshConsole.addDebug(`[ResourceValidator] Validation failed: ${validation.issues.join(', ')}`);
    }
  }

  async function calculateCostEstimate(cores: number, memory: string, wallTime: string, partition: string) {
    const walltimeHours = walltimeToHours(wallTime);

    // Determine if partition has GPU
    const partitionSpec = getPartition(partition);
    const hasGpu = partitionSpec?.gpu_type ? true : false;
    const gpuCount = partitionSpec?.gpu_count || 1;

    // Calculate costs via backend (single source of truth!)
    const totalCost = await calculateJobCost(cores, walltimeHours, hasGpu, gpuCount);

    // Calculate breakdown using billing rates from backend
    const rates = $billingRates;
    const coreCost = rates ? cores * walltimeHours * rates.cpu_cost_per_core_hour : cores * walltimeHours;
    const gpuCost = hasGpu && rates ? gpuCount * walltimeHours * rates.gpu_cost_per_gpu_hour : 0;

    // Get queue time estimate from backend
    const queueEstimate = await estimateQueueTime(cores, partition);

    return {
      coreCost: Math.round(coreCost),
      gpuCost: Math.round(gpuCost),
      totalCost,
      queueEstimate,
      hasGpu
    };
  }
</script>

<div class="resource-validator">
  <div class="validator-header">
    <h3 class="validator-title">Resource Validation & Cost Estimate</h3>
    <p class="validator-description">
      Real-time validation and cost estimation for your resource request
    </p>
  </div>

  <div class="validation-grid">
    <!-- Validation Status -->
    <div class="validation-status">
      <div class="status-header">
        <div class="status-icon" class:valid={validation.is_valid} class:invalid={!validation.is_valid}>
          {#if validation.is_valid}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20,6 9,17 4,12"/>
            </svg>
          {:else}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="15" y1="9" x2="9" y2="15"/>
              <line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
          {/if}
        </div>
        <div class="status-text">
          <div class="status-title">
            {validation.is_valid ? 'Configuration Valid' : 'Configuration Issues'}
          </div>
          <div class="status-subtitle">
            {validation.is_valid ? 'Ready to submit' : 'Please fix issues below'}
          </div>
        </div>
      </div>

      <!-- Issues -->
      {#if validation.issues.length > 0}
        <div class="issues">
          <div class="issues-title">Issues:</div>
          <ul class="issues-list">
            {#each validation.issues as issue}
              <li class="issue error">{issue}</li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Warnings -->
      {#if validation.warnings.length > 0}
        <div class="warnings">
          <div class="warnings-title">Warnings:</div>
          <ul class="warnings-list">
            {#each validation.warnings as warning}
              <li class="warning">{warning}</li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Suggestions -->
      {#if validation.suggestions.length > 0}
        <div class="suggestions">
          <div class="suggestions-title">Suggestions:</div>
          <ul class="suggestions-list">
            {#each validation.suggestions as suggestion}
              <li class="suggestion">{suggestion}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>

    <!-- Cost Estimate -->
    <div class="cost-estimate">
      <div class="cost-header">
        <div class="cost-icon">💰</div>
        <div class="cost-title">Estimated Cost</div>
      </div>

      <div class="cost-breakdown">
        <div class="cost-item">
          <span class="cost-label">CPU Cost:</span>
          <span class="cost-value">{costEstimate.coreCost} SU</span>
        </div>
        {#if costEstimate.hasGpu}
          <div class="cost-item">
            <span class="cost-label">GPU Cost:</span>
            <span class="cost-value gpu">{costEstimate.gpuCost} SU</span>
          </div>
        {/if}
        <div class="cost-item total">
          <span class="cost-label">Total Cost:</span>
          <span class="cost-value">{costEstimate.totalCost} SU</span>
        </div>
      </div>

      <div class="queue-estimate">
        <div class="queue-label">Expected Queue Time:</div>
        <div class="queue-value">{costEstimate.queueEstimate}</div>
      </div>

      <div class="cost-note">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        SU = Service Units. Costs are estimates based on cluster billing rates.
      </div>
    </div>
  </div>
</div>

<style>
  .resource-validator {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .validator-header {
    text-align: center;
  }

  .validator-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-sm) 0;
  }

  .validator-description {
    color: var(--namd-text-secondary);
    margin: 0;
    font-size: var(--namd-font-size-sm);
  }

  .validation-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--namd-spacing-lg);
  }

  @media (min-width: 768px) {
    .validation-grid {
      grid-template-columns: 1fr 1fr;
    }
  }

  .validation-status {
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
  }

  .status-header {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    margin-bottom: var(--namd-spacing-lg);
  }

  .status-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-icon.valid {
    background-color: var(--namd-success-bg);
    color: var(--namd-success);
  }

  .status-icon.invalid {
    background-color: var(--namd-error-bg);
    color: var(--namd-error);
  }

  .status-text {
    flex: 1;
  }

  .status-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin-bottom: var(--namd-spacing-xs);
  }

  .status-subtitle {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-secondary);
  }

  .issues, .warnings, .suggestions {
    margin-top: var(--namd-spacing-md);
  }

  .issues-title, .warnings-title, .suggestions-title {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-semibold);
    margin-bottom: var(--namd-spacing-sm);
  }

  .issues-title {
    color: var(--namd-error);
  }

  .warnings-title {
    color: var(--namd-warning);
  }

  .suggestions-title {
    color: var(--namd-info);
  }

  .issues-list, .warnings-list, .suggestions-list {
    margin: 0;
    padding-left: var(--namd-spacing-lg);
  }

  .issue, .warning, .suggestion {
    font-size: var(--namd-font-size-sm);
    line-height: 1.5;
    margin-bottom: var(--namd-spacing-xs);
  }

  .issue.error {
    color: var(--namd-error-fg);
  }

  .warning {
    color: var(--namd-warning-fg);
  }

  .suggestion {
    color: var(--namd-info-fg);
  }

  .cost-estimate {
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
  }

  .cost-header {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    margin-bottom: var(--namd-spacing-lg);
  }

  .cost-icon {
    font-size: 1.5rem;
  }

  .cost-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .cost-breakdown {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
    margin-bottom: var(--namd-spacing-lg);
  }

  .cost-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--namd-font-size-sm);
  }

  .cost-item.total {
    padding-top: var(--namd-spacing-sm);
    border-top: 1px solid var(--namd-border);
    font-weight: var(--namd-font-weight-semibold);
  }

  .cost-label {
    color: var(--namd-text-secondary);
  }

  .cost-value {
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
    font-weight: var(--namd-font-weight-medium);
  }

  .cost-value.gpu {
    color: var(--namd-warning);
  }

  .queue-estimate {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    margin-bottom: var(--namd-spacing-lg);
    padding: var(--namd-spacing-md);
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius-sm);
  }

  .queue-label {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
    font-weight: var(--namd-font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .queue-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    font-weight: var(--namd-font-weight-semibold);
  }

  .cost-note {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    line-height: 1.4;
  }
</style>
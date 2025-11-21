<script lang="ts">
  import type { ValidationResult } from '$lib/types/api';

  export let validation: ValidationResult;
  export let collapsible: boolean = false;
</script>

{#if validation.issues.length > 0 || validation.warnings.length > 0 || validation.suggestions.length > 0}
  {#if collapsible}
    <details class="validation-details">
      <summary>Show Details</summary>
      <div class="validation-content">
        {#if validation.issues.length > 0}
          <div class="validation-section">
            <h4>Issues:</h4>
            <ul>
              {#each validation.issues as issue}
                <li class="issue-error">{issue}</li>
              {/each}
            </ul>
          </div>
        {/if}
        {#if validation.warnings.length > 0}
          <div class="validation-section">
            <h4>Warnings:</h4>
            <ul>
              {#each validation.warnings as warning}
                <li class="issue-warning">{warning}</li>
              {/each}
            </ul>
          </div>
        {/if}
        {#if validation.suggestions.length > 0}
          <div class="validation-section">
            <h4>Suggestions:</h4>
            <ul>
              {#each validation.suggestions as suggestion}
                <li class="issue-suggestion">{suggestion}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    </details>
  {:else}
    <div class="validation-results">
      {#if validation.issues.length > 0}
        <div class="validation-section">
          <h4>Validation Errors:</h4>
          <ul>
            {#each validation.issues as issue}
              <li class="issue-error">{issue}</li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if validation.warnings.length > 0}
        <div class="validation-section">
          <h4>Warnings:</h4>
          <ul>
            {#each validation.warnings as warning}
              <li class="issue-warning">{warning}</li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if validation.suggestions.length > 0}
        <div class="validation-section">
          <h4>Suggestions:</h4>
          <ul>
            {#each validation.suggestions as suggestion}
              <li class="issue-suggestion">{suggestion}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
{/if}

<style>
  .validation-details summary {
    cursor: pointer;
    font-weight: 500;
    padding: 0.5rem;
    user-select: none;
  }

  .validation-details summary:hover {
    background: var(--namd-surface-hover);
  }

  .validation-content {
    padding: 0.5rem;
  }

  .validation-results,
  .validation-content {
    border-radius: 4px;
  }

  .validation-section {
    margin-bottom: 1rem;
  }

  .validation-section:last-child {
    margin-bottom: 0;
  }

  .validation-section h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.9rem;
  }

  .validation-section ul {
    margin: 0;
    padding-left: 1.5rem;
  }

  .issue-error {
    color: var(--namd-error);
  }

  .issue-warning {
    color: var(--namd-warning-fg);
  }

  .issue-suggestion {
    color: var(--namd-info-fg);
  }
</style>

<script lang="ts">
  export let label: string;
  export let id: string;
  export let type: 'text' | 'number' | 'email' = 'text';
  export let value: string | number;
  export let placeholder: string = '';
  export let required: boolean = false;
  export let error: string = '';
  export let min: number | undefined = undefined;
  export let max: number | undefined = undefined;
  export let step: number | undefined = undefined;
  export let disabled: boolean = false;

  // Handle value updates
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    if (type === 'number') {
      value = target.valueAsNumber;
    } else {
      value = target.value;
    }
  }
</script>

<div class="namd-field-group">
  <label class="namd-label" for={id}>
    {label}
    {#if required}*{/if}
  </label>
  <input
    class="namd-input"
    class:error={error}
    {id}
    {type}
    {placeholder}
    {min}
    {max}
    {step}
    {disabled}
    value={value}
    on:input={handleInput}
  />
  {#if error}
    <span class="namd-error-text">{error}</span>
  {/if}
</div>

<style>
  .namd-field-group {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .namd-error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-xs);
    margin-top: var(--namd-spacing-xs);
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }
</style>
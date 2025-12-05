<script lang="ts">
  /**
   * Dialog - The primitive modal component
   * All other modals/dialogs should use this as their base
   */
  export let open: boolean = false;
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let onClose: () => void;
  export let showCloseButton: boolean = true;

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="dialog-overlay" on:click={handleBackdropClick} on:keydown={handleKeydown}>
    <div class="dialog dialog--{size}" on:click|stopPropagation role="dialog" aria-modal="true" tabindex="-1">
      {#if $$slots.header || showCloseButton}
        <div class="dialog-header">
          <slot name="header" />
          {#if showCloseButton}
            <button class="dialog-close" on:click={onClose} type="button" aria-label="Close">×</button>
          {/if}
        </div>
      {/if}

      {#if $$slots.body}
        <div class="dialog-body">
          <slot name="body" />
        </div>
      {:else}
        <div class="dialog-body">
          <slot />
        </div>
      {/if}

      {#if $$slots.footer}
        <div class="dialog-footer">
          <slot name="footer" />
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--namd-z-modal);
  }

  .dialog {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    width: 90%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dialog--sm {
    max-width: 400px;
  }

  .dialog--md {
    max-width: 600px;
  }

  .dialog--lg {
    max-width: var(--namd-max-width-form);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--namd-spacing-lg);
    border-bottom: 1px solid var(--namd-border);
    flex-shrink: 0;
  }

  .dialog-close {
    background: none;
    border: none;
    font-size: 24px;
    color: var(--namd-text-secondary);
    cursor: pointer;
    padding: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--namd-border-radius-sm);
    flex-shrink: 0;
    margin-left: var(--namd-spacing-md);
  }

  .dialog-close:hover {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-primary);
  }

  .dialog-body {
    padding: var(--namd-spacing-lg);
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .dialog-footer {
    display: flex;
    gap: var(--namd-spacing-sm);
    justify-content: flex-end;
    padding: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
    flex-shrink: 0;
  }
</style>

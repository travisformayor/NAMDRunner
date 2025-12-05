<script lang="ts">
  /**
   * AlertDialog - Replacement for native alert()
   * Wrapper around Dialog for simple notifications
   */
  import Dialog from './Dialog.svelte';

  export let open: boolean = false;
  export let title: string;
  export let message: string;
  export let variant: 'success' | 'error' | 'warning' | 'info' = 'info';
  export let onClose: () => void;

  const icons = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ'
  };
</script>

<Dialog {open} size="sm" {onClose}>
  <svelte:fragment slot="header">
    <h2 class="alert-title">
      <span class="alert-icon alert-icon--{variant}">{icons[variant]}</span>
      {title}
    </h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="alert-message">{@html message}</div>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button class="namd-button namd-button--primary" on:click={onClose} type="button">
      OK
    </button>
  </svelte:fragment>
</Dialog>

<style>
  .alert-title {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    margin: 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .alert-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    font-weight: var(--namd-font-weight-bold);
    font-size: var(--namd-font-size-base);
    flex-shrink: 0;
  }

  .alert-icon--success {
    background-color: var(--namd-success-bg);
    color: var(--namd-success-fg);
  }

  .alert-icon--error {
    background-color: var(--namd-error-bg);
    color: var(--namd-error-fg);
  }

  .alert-icon--warning {
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
  }

  .alert-icon--info {
    background-color: var(--namd-info-bg);
    color: var(--namd-info-fg);
  }

  .alert-message {
    line-height: var(--namd-line-height-relaxed);
    color: var(--namd-text-primary);
  }
</style>

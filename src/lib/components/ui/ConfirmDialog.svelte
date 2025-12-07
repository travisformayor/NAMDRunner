<script lang="ts">
  /**
   * ConfirmDialog - Confirmation dialog with Cancel/Confirm buttons
   * Can also be used as AlertDialog when showCancel is false
   * Wrapper around Dialog for confirmation actions
   */
  import Dialog from './Dialog.svelte';

  export let isOpen: boolean = false;
  export let title: string;
  export let message: string;
  export let confirmText: string = 'Confirm';
  export let cancelText: string = 'Cancel';
  export let confirmStyle: 'primary' | 'destructive' = 'primary';
  export let showCancel: boolean = true;
  export let variant: 'success' | 'error' | 'warning' | 'info' | null = null;
  export let onConfirm: () => void = () => {};
  export let onCancel: () => void = () => {};

  const icons = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ'
  };

  function handleConfirm() {
    onConfirm();
  }

  function handleCancel() {
    onCancel();
  }
</script>

<Dialog open={isOpen} size="sm" onClose={handleCancel}>
  <svelte:fragment slot="header">
    <h2 class="confirm-title">
      {#if variant}
        <span class="confirm-icon confirm-icon--{variant}">{icons[variant]}</span>
      {/if}
      {title}
    </h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="confirm-message">{@html message}</div>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    {#if showCancel}
      <button
        type="button"
        class="namd-button namd-button--secondary"
        on:click={handleCancel}
      >
        {cancelText}
      </button>
    {/if}
    <button
      type="button"
      class="namd-button namd-button--{confirmStyle === 'destructive' ? 'destructive' : 'primary'}"
      on:click={handleConfirm}
    >
      {confirmText}
    </button>
  </svelte:fragment>
</Dialog>

<style>
  .confirm-title {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    margin: 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .confirm-icon {
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

  .confirm-icon--success {
    background-color: var(--namd-success-bg);
    color: var(--namd-success-fg);
  }

  .confirm-icon--error {
    background-color: var(--namd-error-bg);
    color: var(--namd-error-fg);
  }

  .confirm-icon--warning {
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
  }

  .confirm-icon--info {
    background-color: var(--namd-info-bg);
    color: var(--namd-info-fg);
  }

  .confirm-message {
    line-height: var(--namd-line-height-relaxed);
    color: var(--namd-text-primary);
    white-space: pre-line;
  }
</style>

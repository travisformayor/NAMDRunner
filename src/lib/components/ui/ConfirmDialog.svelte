<script lang="ts">
  /**
   * ConfirmDialog - Confirmation dialog with Cancel/Confirm buttons
   * Wrapper around Dialog for confirmation actions
   */
  import Dialog from './Dialog.svelte';

  export let isOpen: boolean = false;
  export let title: string;
  export let message: string;
  export let confirmText: string = 'Confirm';
  export let cancelText: string = 'Cancel';
  export let confirmStyle: 'primary' | 'destructive' = 'primary';
  export let onConfirm: () => void = () => {};
  export let onCancel: () => void = () => {};

  function handleConfirm() {
    onConfirm();
  }

  function handleCancel() {
    onCancel();
  }
</script>

<Dialog open={isOpen} size="sm" onClose={handleCancel}>
  <svelte:fragment slot="header">
    <h2 class="confirm-title">{title}</h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="confirm-message">{@html message}</div>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button
      type="button"
      class="namd-button namd-button--secondary"
      on:click={handleCancel}
    >
      {cancelText}
    </button>
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
    margin: 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .confirm-message {
    line-height: 1.6;
    color: var(--namd-text-primary);
    white-space: pre-line;
  }
</style>

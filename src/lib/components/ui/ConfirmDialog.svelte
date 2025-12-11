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
    <h2 class="dialog-title">
      {#if variant}
        <span class="dialog-icon dialog-icon--{variant}">{icons[variant]}</span>
      {/if}
      {title}
    </h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="dialog-message">{@html message}</div>
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

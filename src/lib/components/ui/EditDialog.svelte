<script lang="ts">
  /**
   * EditDialog - Standard wrapper for edit/add dialogs
   * Provides consistent structure with header, form body, and Cancel/Save footer
   * Wrapper around Dialog for editing entities (partitions, QoS, presets, etc.)
   */
  import Dialog from './Dialog.svelte';

  export let isOpen: boolean = false;
  export let title: string;
  export let onSave: () => void;
  export let onClose: () => void;
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let saveButtonText: string = 'Save';
  export let cancelButtonText: string = 'Cancel';
</script>

<Dialog open={isOpen} {size} {onClose}>
  <svelte:fragment slot="header">
    <h2 class="dialog-title">{title}</h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="dialog-form">
      <slot name="form" />
    </div>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button class="namd-button namd-button--secondary" on:click={onClose} type="button">
      {cancelButtonText}
    </button>
    <button class="namd-button namd-button--primary" on:click={onSave} type="button">
      {saveButtonText}
    </button>
  </svelte:fragment>
</Dialog>

<style>
  .dialog-form {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }
</style>

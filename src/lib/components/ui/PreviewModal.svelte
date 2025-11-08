<script lang="ts">
  /**
   * PreviewModal - Code/text preview modal
   * Wrapper around Dialog for displaying code/configuration previews
   */
  import Dialog from './Dialog.svelte';

  export let isOpen: boolean = false;
  export let title: string;
  export let content: string;
  export let onClose: () => void;
</script>

<Dialog open={isOpen} size="lg" {onClose}>
  <svelte:fragment slot="header">
    <h3 class="preview-title">{title}</h3>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <pre class="preview-content">{content}</pre>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button class="namd-button namd-button--secondary" on:click={onClose} type="button">
      Close
    </button>
  </svelte:fragment>
</Dialog>

<style>
  .preview-title {
    margin: 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .preview-content {
    background: var(--namd-code-bg);
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius-sm);
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-primary);
    overflow-x: auto;
    white-space: pre;
    margin: 0;
    line-height: 1.5;
  }
</style>

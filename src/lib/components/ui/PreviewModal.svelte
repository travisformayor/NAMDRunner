<script lang="ts">
  export let isOpen: boolean = false;
  export let title: string;
  export let content: string;
  export let onClose: () => void;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="modal-overlay"
    role="presentation"
    on:click={onClose}
    on:keydown={handleKeydown}
  >
    <div
      class="modal modal-large"
      role="dialog"
      aria-modal="true"
      aria-labelledby="preview-modal-title"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <h3 id="preview-modal-title">{title}</h3>
      <pre class="preview-content">{content}</pre>
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={onClose}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--card-bg, white);
    border-radius: 8px;
    padding: 2rem;
    max-width: 900px;
    width: 95%;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal h3 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
  }

  .preview-content {
    background: var(--code-bg, #f5f5f5);
    padding: 1rem;
    border-radius: 4px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 0.8125rem;
    overflow-x: auto;
    white-space: pre;
    max-height: 60vh;
    overflow-y: auto;
    flex: 1;
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-secondary {
    background: var(--secondary, #f5f5f5);
    color: var(--text-primary, #333);
  }

  .btn-secondary:hover {
    background: var(--secondary-dark, #e0e0e0);
  }
</style>

<script lang="ts">
  /**
   * Reusable confirmation dialog component
   * Used for important actions that need user confirmation (Create Job, Delete Job, etc.)
   */
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

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleCancel();
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if isOpen}
  <div class="dialog-overlay" onclick={handleCancel} onkeydown={handleKeydown}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <div class="dialog-header">
        <h2>{title}</h2>
        <button class="close-btn" onclick={handleCancel} title="Close">×</button>
      </div>

      <div class="dialog-content">
        <div class="message">{@html message}</div>

        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" onclick={handleCancel}>
            {cancelText}
          </button>
          <button
            type="button"
            class="btn {confirmStyle === 'destructive' ? 'btn-destructive' : 'btn-primary'}"
            onclick={handleConfirm}
          >
            {confirmText}
          </button>
        </div>
      </div>
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
    z-index: 1000;
  }

  .dialog {
    background: var(--namd-bg-primary, white);
    border-radius: 8px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow: auto;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 20px 0 20px;
    border-bottom: 1px solid var(--namd-border-color, #e5e7eb);
    margin-bottom: 20px;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--namd-text-primary, #111827);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    color: var(--namd-text-secondary, #6b7280);
    cursor: pointer;
    padding: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .close-btn:hover {
    background-color: var(--namd-bg-hover, #f3f4f6);
    color: var(--namd-text-primary, #374151);
  }

  .dialog-content {
    padding: 0 20px 20px 20px;
  }

  .message {
    margin-bottom: 20px;
    line-height: 1.6;
    color: var(--namd-text-primary, #374151);
    white-space: pre-line;
  }

  .dialog-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: 14px;
  }

  .btn-primary {
    background-color: var(--namd-primary-color, #3b82f6);
    color: white;
  }

  .btn-primary:hover {
    background-color: var(--namd-primary-hover, #2563eb);
  }

  .btn-destructive {
    background-color: var(--namd-danger-color, #dc2626);
    color: white;
  }

  .btn-destructive:hover {
    background-color: var(--namd-danger-hover, #b91c1c);
  }

  .btn-secondary {
    background-color: var(--namd-bg-secondary, #f3f4f6);
    color: var(--namd-text-primary, #374151);
    border: 1px solid var(--namd-border-color, #d1d5db);
  }

  .btn-secondary:hover {
    background-color: var(--namd-bg-hover, #e5e7eb);
  }
</style>

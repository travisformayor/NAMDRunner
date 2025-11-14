<script lang="ts">
  import { toasts } from '../../stores/toasts';
  import { fade, slide } from 'svelte/transition';

  function getIcon(variant: string): string {
    switch (variant) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
        return 'ℹ';
      default:
        return 'ℹ';
    }
  }
</script>

<div class="namd-toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="namd-toast namd-toast--{toast.variant}"
      transition:slide={{ duration: 200 }}
    >
      <div class="namd-toast-icon">
        {getIcon(toast.variant)}
      </div>
      <div class="namd-toast-message">
        {toast.message}
      </div>
      <button
        class="namd-toast-close"
        on:click={() => toasts.dismiss(toast.id)}
        aria-label="Dismiss notification"
      >
        ✕
      </button>
    </div>
  {/each}
</div>

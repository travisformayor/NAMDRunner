<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { isConnected } from '../../stores/session';
  import { lastSyncTime, hasEverSynced, isSyncing } from '../../stores/jobs';

  let currentTime = new Date();
  let updateTimer: number;

  const dispatch = createEventDispatcher<{ sync: void }>();

  // Update current time every 10 seconds to refresh relative time display
  onMount(() => {
    updateTimer = window.setInterval(() => {
      currentTime = new Date();
    }, 10000);
  });

  onDestroy(() => {
    if (updateTimer) {
      clearInterval(updateTimer);
    }
  });

  function handleSync() {
    if (!$isConnected || $isSyncing) return;
    dispatch('sync');
  }

  function formatSyncTime(date: Date, now: Date): string {
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);

    if (diffSecs < 60) {
      return 'Just now';
    } else if (diffMins < 60) {
      return `${diffMins} minutes ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hours ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  // Reactive status text updates when stores OR currentTime changes
  $: statusText = (() => {
    if ($isConnected) {
      if ($hasEverSynced) {
        return `Last synced: ${formatSyncTime($lastSyncTime, currentTime)}`;
      } else {
        return `Not synced yet`;
      }
    } else {
      // Offline mode - show cached data message without timestamp
      return `Offline - showing cached data`;
    }
  })();
</script>

<!-- Match React mockup: flex items-center justify-between -->
<div class="sync-status">
  <div class="sync-left">
    <span class="status-text" class:offline={!$isConnected}>
      {statusText}
    </span>

    <button
      class="namd-button namd-button--ghost sync-button"
      on:click={handleSync}
      disabled={!$isConnected || $isSyncing}
    >
      <svg class="sync-icon" class:spinning={$isSyncing} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
        <path d="M21 3v5h-5"/>
        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
        <path d="M3 21v-5h5"/>
      </svg>
      Sync Now
    </button>
  </div>
</div>

<style>
  .sync-status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-secondary);
  }

  .sync-left {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
  }

  .status-text {
    color: var(--namd-text-secondary);
  }

  .status-text.offline {
    color: var(--namd-text-secondary);
  }

  .sync-button {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-xs);
    padding: 0 var(--namd-spacing-sm);
    font-size: var(--namd-font-size-base);
    height: 2rem;
  }

  .sync-button:hover:not(:disabled) {
    background-color: var(--namd-accent);
    color: var(--namd-text-primary);
  }

  .sync-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .sync-icon {
    flex-shrink: 0;
    transition: transform var(--namd-transition-fast);
  }

  .sync-icon.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
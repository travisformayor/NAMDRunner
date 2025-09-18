<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { isConnected } from '../../stores/session';
  import { lastSyncTime, isSyncing } from '../../stores/jobs';

  let autoSync = false;
  let syncInterval = 5;

  const dispatch = createEventDispatcher<{ sync: void }>();

  function handleSync() {
    if (!$isConnected || $isSyncing) return;
    dispatch('sync');
  }

  function formatSyncTime(date: Date): string {
    const now = new Date();
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

  function getStatusText(): string {
    if ($isConnected) {
      return `Last synced: ${formatSyncTime($lastSyncTime)}`;
    } else {
      return `Offline - showing cached data from ${$lastSyncTime.toLocaleString()}`;
    }
  }

  $: statusText = getStatusText();
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

  <div class="sync-right">
    <div class="auto-sync-control">
      <label class="checkbox-wrapper">
        <input
          type="checkbox"
          bind:checked={autoSync}
          disabled={!$isConnected}
        />
        <span class="checkbox-label">Auto-sync: every</span>
      </label>
    </div>

    <input
      type="number"
      bind:value={syncInterval}
      disabled={!autoSync || !$isConnected}
      min="1"
      max="60"
      class="interval-input"
    />

    <span class="interval-label">min</span>
  </div>
</div>

<style>
  .sync-status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-muted);
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
    color: var(--namd-text-muted);
  }

  .sync-button {
    display: flex;
    align-items: center;
    gap: 0.25rem; /* equivalent to mr-1 in React */
    padding: 0 0.5rem; /* px-2 */
    font-size: var(--namd-font-size-sm);
    height: 2rem; /* h-8 */
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
    transition: transform 0.15s ease;
  }

  .sync-icon.spinning {
    animation: spin 1s linear infinite;
  }

  .sync-right {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
  }

  .auto-sync-control {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .checkbox-wrapper {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    cursor: pointer;
  }

  .checkbox-wrapper input[type="checkbox"] {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .checkbox-wrapper input[type="checkbox"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-label {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-secondary);
    user-select: none;
  }

  .interval-input {
    width: 64px;
    height: 32px;
    padding: var(--namd-spacing-xs);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-sm);
    text-align: center;
  }

  .interval-input:focus {
    outline: none;
    border-color: var(--namd-primary);
  }

  .interval-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .interval-label {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-secondary);
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 768px) {
    .sync-right {
      display: none;
    }
  }
</style>
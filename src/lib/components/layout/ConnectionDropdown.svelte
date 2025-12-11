<script lang="ts">
  import { isConnected, connectionState, sessionActions, lastError } from '../../stores/session';
  import { clusterConfig } from '../../stores/clusterConfig';

  let isOpen = false;
  let username = '';
  let password = '';

  // Reactive: update host when cluster config changes
  $: host = $clusterConfig?.default_host || '';
  let isConnecting = false;
  let connectionError = '';

  $: statusInfo = getStatusInfo($connectionState);

  function getStatusInfo(state: string) {
    switch (state) {
      case 'Connected':
        return {
          label: 'Connected',
          color: 'namd-connection-connected',
          dotColor: 'namd-connection-dot-connected'
        };
      case 'Connecting':
        return {
          label: 'Connecting...',
          color: 'namd-connection-connecting',
          dotColor: 'namd-connection-dot-connecting'
        };
      case 'Disconnected':
        return {
          label: 'Disconnected',
          color: 'namd-connection-disconnected',
          dotColor: 'namd-connection-dot-disconnected'
        };
      case 'Expired':
        return {
          label: 'Connection Expired',
          color: 'namd-connection-expired',
          dotColor: 'namd-connection-dot-expired'
        };
      default:
        return {
          label: 'Disconnected',
          color: 'namd-connection-disconnected',
          dotColor: 'namd-connection-dot-disconnected'
        };
    }
  }

  function toggleDropdown() {
    isOpen = !isOpen;
  }

  function closeDropdown() {
    isOpen = false;
  }

  async function handleConnect() {
    if (host && username && password) {
      isConnecting = true;
      connectionError = '';

      const success = await sessionActions.connect(host, username, password);
      if (success) {
        closeDropdown();
        password = '';
      } else {
        // Connection failed, checking for error message
        // Get detailed error from session store instead of generic message
        const errorMsg = $lastError || 'Connection failed - no error details available';
        connectionError = errorMsg;
      }

      isConnecting = false;
    }
  }

  async function handleDisconnect() {
    await sessionActions.disconnect();
    password = '';
    closeDropdown();
  }

  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.connection-dropdown')) {
      closeDropdown();
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="connection-dropdown">
  <button
    class="connection-trigger"
    on:click|stopPropagation={toggleDropdown}
    data-testid="connection-status-button"
  >
    <!-- Circle dot icon -->
    <svg class="status-dot {statusInfo.dotColor}" width="8" height="8" viewBox="0 0 8 8">
      <circle cx="4" cy="4" r="4" />
    </svg>
    <span class="status-label {statusInfo.color}">{statusInfo.label}</span>
    <!-- Chevron down icon -->
    <svg class="chevron" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="6,9 12,15 18,9"></polyline>
    </svg>
  </button>

  {#if isOpen}
    <div class="connection-dropdown-content" on:click|stopPropagation on:keydown={(e) => e.key === 'Escape' && (isOpen = false)} tabindex="0" role="menu">

      {#if $isConnected}
        <!-- Connected state -->
        <div class="connected-info">
          <div class="status-line">
            <svg class="status-dot namd-connection-dot-connected" width="8" height="8" viewBox="0 0 8 8">
              <circle cx="4" cy="4" r="4" />
            </svg>
            <span class="namd-connection-connected">Connected</span>
          </div>

          <div class="connection-details">
            <div>Host: {host}</div>
            <div>User: {username}</div>
          </div>

          <button class="namd-button namd-button--secondary" on:click={handleDisconnect}>
            Disconnect
          </button>
        </div>
      {:else}
        <!-- Disconnected state -->
        <div class="connection-form">
          <div class="status-line">
            <svg class="status-dot {statusInfo.dotColor}" width="8" height="8" viewBox="0 0 8 8">
              <circle cx="4" cy="4" r="4" />
            </svg>
            <span class="{statusInfo.color}">{statusInfo.label}</span>
          </div>

          <div class="form-fields">
            <div class="field-group">
              <label for="host">Host</label>
              <input
                class="namd-input"
                id="host"
                type="text"
                bind:value={host}
                placeholder="cluster.edu"
              />
            </div>

            <div class="field-group">
              <label for="username">Username</label>
              <input
                class="namd-input"
                id="username"
                type="text"
                bind:value={username}
                placeholder="username"
              />
            </div>

            <div class="field-group">
              <label for="password">Password</label>
              <input
                class="namd-input"
                id="password"
                type="password"
                bind:value={password}
                placeholder="password"
              />
            </div>

            {#if connectionError}
              <div class="connection-error">
                {connectionError}
              </div>
            {/if}

            <button
              class="namd-button namd-button--primary"
              on:click={handleConnect}
              disabled={!host || !username || !password || isConnecting}
            >
              {isConnecting ? 'Connecting...' : 'Connect'}
            </button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .connection-dropdown {
    position: relative;
  }

  .connection-trigger {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    transition: background-color var(--namd-transition-fast);
    font-size: var(--namd-font-size-base);
  }

  .connection-trigger:hover {
    background-color: var(--namd-accent);
  }

  .status-dot {
    flex-shrink: 0;
  }

  .status-label {
    font-weight: var(--namd-font-weight-medium);
  }

  .chevron {
    flex-shrink: 0;
    color: var(--namd-text-secondary);
  }

  .connection-dropdown-content {
    position: absolute;
    top: 100%;
    right: 0;
    width: 256px;
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    box-shadow: var(--namd-shadow-md);
    padding: var(--namd-spacing-md);
    margin-top: var(--namd-spacing-sm);
    z-index: var(--namd-z-dropdown);
  }

  .connected-info {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .connection-form {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .status-line {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .connection-details {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-primary);
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .field-group label {
    font-size: var(--namd-font-size-base);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .namd-button {
    width: 100%;
  }

  .connection-error {
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-error-bg);
    border: 1px solid var(--namd-error-border);
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-error);
    font-size: var(--namd-font-size-base);
    text-align: center;
  }
</style>
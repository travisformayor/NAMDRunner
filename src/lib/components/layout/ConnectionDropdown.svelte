<script lang="ts">
  import { isConnected, connectionState, sessionActions, lastError } from '../../stores/session';

  let isOpen = false;
  let host = 'login.rc.colorado.edu'; // Pre-populated default value
  let username = '';
  let password = '';
  let isConnecting = false;
  let connectionError = '';

  $: statusInfo = getStatusInfo($connectionState);

  function getStatusInfo(state: string) {
    switch (state) {
      case 'Connected':
        return {
          label: 'Connected',
          color: 'namd-connection-connected',
          dotColor: 'namd-connection-dot-connected',
          since: '10:30 AM'
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

      try {
        const success = await sessionActions.connect(host, username, password);
        if (success) {
          closeDropdown();
          password = ''; // Clear password on successful connection
        } else {
          // Connection failed, checking for error message
          // Get detailed error from session store instead of generic message
          const errorMsg = $lastError || 'Connection failed - no error details available';
          connectionError = errorMsg;
        }
      } catch (error) {
        // Connection threw exception
        const errorMsg = error instanceof Error ? error.message : 'Unknown error occurred';
        connectionError = errorMsg;
      } finally {
        isConnecting = false;
      }
    }
  }

  async function handleDisconnect() {
    try {
      await sessionActions.disconnect();
    } catch (error) {
      // Disconnect failed - error handled by UI state
    }
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
            <svg class="status-dot fill-green-600" width="8" height="8" viewBox="0 0 8 8">
              <circle cx="4" cy="4" r="4" />
            </svg>
            <span class="text-green-600">Connected</span>
          </div>

          <div class="connection-details">
            <div>Host: {host}</div>
            <div>User: {username}</div>
            <div>Since: {statusInfo.since}</div>
          </div>

          <button class="disconnect-button" on:click={handleDisconnect}>
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
                id="host"
                type="text"
                bind:value={host}
                placeholder="cluster.edu"
              />
            </div>

            <div class="field-group">
              <label for="username">Username</label>
              <input
                id="username"
                type="text"
                bind:value={username}
                placeholder="username"
              />
            </div>

            <div class="field-group">
              <label for="password">Password</label>
              <input
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
              class="connect-button"
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
    transition: background-color 0.15s ease;
    font-size: var(--namd-font-size-sm);
  }

  .connection-trigger:hover {
    background-color: var(--namd-accent);
  }

  .status-dot {
    flex-shrink: 0;
  }

  .status-dot.fill-green-600 {
    fill: #059669;
  }

  .status-dot.fill-red-600 {
    fill: #dc2626;
  }

  .status-dot.fill-yellow-600 {
    fill: #d97706;
  }

  .status-dot.fill-gray-600 {
    fill: #4b5563;
  }

  .status-label {
    font-weight: var(--namd-font-weight-medium);
  }

  .status-label.text-green-600 {
    color: #059669;
  }

  .status-label.text-red-600 {
    color: #dc2626;
  }

  .status-label.text-yellow-600 {
    color: #d97706;
  }

  .status-label.text-gray-600 {
    color: #4b5563;
  }

  .chevron {
    flex-shrink: 0;
    color: var(--namd-text-muted);
  }

  .connection-dropdown-content {
    position: absolute;
    top: 100%;
    right: 0;
    width: 256px;
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
    padding: var(--namd-spacing-md);
    margin-top: var(--namd-spacing-sm);
    z-index: 50;
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

  .text-green-600 {
    color: #059669;
  }

  .connection-details {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    font-size: var(--namd-font-size-sm);
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
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .field-group input {
    padding: var(--namd-spacing-sm);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-sm);
  }

  .field-group input:focus {
    outline: none;
    border-color: var(--namd-primary);
  }

  .disconnect-button,
  .connect-button {
    padding: var(--namd-spacing-sm) var(--namd-spacing-md);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    cursor: pointer;
    transition: all 0.15s ease;
    width: 100%;
  }

  .disconnect-button {
    background-color: transparent;
    border: 1px solid var(--namd-border);
    color: var(--namd-text-primary);
  }

  .disconnect-button:hover {
    background-color: var(--namd-bg-muted);
  }

  .connect-button {
    background-color: var(--namd-primary);
    border: 1px solid var(--namd-primary);
    color: var(--namd-primary-fg);
  }

  .connect-button:hover:not(:disabled) {
    background-color: var(--namd-primary-hover);
  }

  .connect-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .connection-error {
    padding: var(--namd-spacing-sm);
    background-color: rgba(220, 38, 38, 0.1);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: var(--namd-border-radius-sm);
    color: #dc2626;
    font-size: var(--namd-font-size-sm);
    text-align: center;
  }
</style>
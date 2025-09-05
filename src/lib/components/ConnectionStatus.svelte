<script lang="ts">
  import { connectionState, isConnected, sessionInfo, isConnecting, lastError } from '../stores/session';

  export let onConnect: () => void = () => {};
  export let onDisconnect: () => void = () => {};

  $: statusClass = getStatusClass($connectionState);
  $: statusText = getStatusText($connectionState, $isConnecting);

  function getStatusClass(state: string): string {
    switch (state) {
      case 'Connected':
        return 'status-connected';
      case 'Connecting':
        return 'status-connecting';
      case 'Expired':
        return 'status-expired';
      default:
        return 'status-disconnected';
    }
  }

  function getStatusText(state: string, connecting: boolean): string {
    if (connecting) return 'Connecting...';
    switch (state) {
      case 'Connected':
        return 'Connected';
      case 'Expired':
        return 'Connection Expired';
      default:
        return 'Disconnected';
    }
  }
</script>

<div class="connection-status">
  <div class="status-indicator">
    <div class="status-dot {statusClass}"></div>
    <span class="status-text">{statusText}</span>
  </div>

  {#if $sessionInfo}
    <div class="session-info">
      <div class="info-item">
        <span class="label">Host:</span>
        <span class="value">{$sessionInfo.host}</span>
      </div>
      <div class="info-item">
        <span class="label">User:</span>
        <span class="value">{$sessionInfo.username}</span>
      </div>
      <div class="info-item">
        <span class="label">Connected:</span>
        <span class="value">{new Date($sessionInfo.connectedAt).toLocaleTimeString()}</span>
      </div>
    </div>
  {/if}

  <div class="actions">
    {#if $isConnected}
      <button class="btn btn-secondary" onclick={onDisconnect} disabled={$isConnecting}>
        Disconnect
      </button>
    {:else}
      <button class="btn btn-primary" onclick={onConnect} disabled={$isConnecting}>
        {$isConnecting ? 'Connecting...' : 'Connect'}
      </button>
    {/if}
  </div>

  {#if $lastError}
    <div class="error-message">
      <span class="error-text">{$lastError}</span>
    </div>
  {/if}
</div>

<style>
  .connection-status {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.status-connected {
    background-color: #10b981;
  }

  .status-dot.status-connecting {
    background-color: #f59e0b;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status-dot.status-expired {
    background-color: #ef4444;
  }

  .status-dot.status-disconnected {
    background-color: #6b7280;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .status-text {
    font-weight: 500;
    color: #374151;
  }

  .session-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 16px;
    padding: 8px;
    background-color: #f9fafb;
    border-radius: 4px;
  }

  .info-item {
    display: flex;
    gap: 8px;
    font-size: 14px;
  }

  .label {
    font-weight: 500;
    color: #6b7280;
    min-width: 70px;
  }

  .value {
    color: #374151;
  }

  .actions {
    display: flex;
    justify-content: center;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background-color: #3b82f6;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #2563eb;
  }

  .btn-secondary {
    background-color: #6b7280;
    color: white;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #4b5563;
  }

  .error-message {
    margin-top: 12px;
    padding: 8px;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 4px;
  }

  .error-text {
    color: #dc2626;
    font-size: 14px;
  }
</style>
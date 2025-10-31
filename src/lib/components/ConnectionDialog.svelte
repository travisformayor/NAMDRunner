<script lang="ts">
  import { sessionActions } from '../stores/session';

  export let isOpen: boolean = false;
  export let onClose: () => void = () => {};

  let host = 'login.rc.colorado.edu';
  let username = '';
  let password = '';
  let isConnecting = false;
  let errorMessage = '';

  // Reset form when dialog opens/closes
  $: if (isOpen) {
    password = '';
    errorMessage = '';
  }

  async function handleConnect() {
    if (!host || !username || !password) {
      errorMessage = 'Please fill in all fields';
      return;
    }

    isConnecting = true;
    errorMessage = '';

    try {
      const success = await sessionActions.connect(host, username, password);
      if (success) {
        onClose();
        password = ''; // Clear password
      } else {
        errorMessage = 'Connection failed. Please check your credentials.';
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isConnecting = false;
    }
  }

  function handleCancel() {
    password = '';
    errorMessage = '';
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleCancel();
    } else if (event.key === 'Enter') {
      handleConnect();
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if isOpen}
  <div class="dialog-overlay" onclick={handleCancel}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <div class="dialog-header">
        <h2>Connect to Cluster</h2>
        <button class="close-btn" onclick={handleCancel} title="Close">×</button>
      </div>

      <div class="dialog-content">
        <form onsubmit={(e) => { e.preventDefault(); handleConnect(); }}>
          <div class="form-group">
            <label for="host">Host</label>
            <input
              id="host"
              type="text"
              bind:value={host}
              disabled={isConnecting}
              placeholder="login.rc.colorado.edu"
              required
            />
          </div>

          <div class="form-group">
            <label for="username">Username</label>
            <input
              id="username"
              type="text"
              bind:value={username}
              disabled={isConnecting}
              placeholder="Your cluster username"
              required
            />
          </div>

          <div class="form-group">
            <label for="password">Password</label>
            <input
              id="password"
              type="password"
              bind:value={password}
              disabled={isConnecting}
              placeholder="Your cluster password"
              required
              onkeydown={handleKeydown}
            />
          </div>


          {#if errorMessage}
            <div class="error-message">
              {errorMessage}
            </div>
          {/if}

          <div class="dialog-actions">
            <button type="button" class="btn btn-secondary" onclick={handleCancel} disabled={isConnecting}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary" disabled={isConnecting}>
              {isConnecting ? 'Connecting...' : 'Connect'}
            </button>
          </div>
        </form>
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
    background: white;
    border-radius: 8px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    max-width: 400px;
    width: 90%;
    max-height: 90vh;
    overflow: auto;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 20px 0 20px;
    border-bottom: 1px solid #e5e7eb;
    margin-bottom: 20px;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #111827;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    color: #6b7280;
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
    background-color: #f3f4f6;
    color: #374151;
  }

  .dialog-content {
    padding: 0 20px 20px 20px;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 500;
    color: #374151;
    font-size: 14px;
  }

  .form-group input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 14px;
    box-sizing: border-box;
  }

  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .form-group input:disabled {
    background-color: #f9fafb;
    color: #6b7280;
    cursor: not-allowed;
  }

  .error-message {
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 10px;
    border-radius: 6px;
    font-size: 14px;
    margin-bottom: 16px;
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
    background-color: #f3f4f6;
    color: #374151;
    border: 1px solid #d1d5db;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #e5e7eb;
  }
</style>
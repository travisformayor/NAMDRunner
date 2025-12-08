<script lang="ts">
  import { sessionActions } from '../stores/session';
  import Dialog from './ui/Dialog.svelte';

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

    const success = await sessionActions.connect(host, username, password);
    if (success) {
      onClose();
      password = ''; // Clear password
    } else {
      errorMessage = 'Connection failed. Please check your credentials.';
    }

    isConnecting = false;
  }

  function handleCancel() {
    password = '';
    errorMessage = '';
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleConnect();
    }
  }
</script>

<Dialog open={isOpen} size="sm" onClose={handleCancel}>
  <svelte:fragment slot="header">
    <h2 class="dialog-title">Connect to Cluster</h2>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <form on:submit|preventDefault={handleConnect}>
      <div class="form-group">
        <label for="host" class="namd-label">Host</label>
        <input
          id="host"
          type="text"
          class="namd-input"
          bind:value={host}
          disabled={isConnecting}
          placeholder="login.rc.colorado.edu"
          required
        />
      </div>

      <div class="form-group">
        <label for="username" class="namd-label">Username</label>
        <input
          id="username"
          type="text"
          class="namd-input"
          bind:value={username}
          disabled={isConnecting}
          placeholder="Your cluster username"
          required
        />
      </div>

      <div class="form-group">
        <label for="password" class="namd-label">Password</label>
        <input
          id="password"
          type="password"
          class="namd-input"
          bind:value={password}
          disabled={isConnecting}
          placeholder="Your cluster password"
          required
          on:keydown={handleKeydown}
        />
      </div>

      {#if errorMessage}
        <div class="error-message">
          {errorMessage}
        </div>
      {/if}
    </form>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button
      type="button"
      class="namd-button namd-button--secondary"
      on:click={handleCancel}
      disabled={isConnecting}
    >
      Cancel
    </button>
    <button
      type="button"
      class="namd-button namd-button--primary"
      on:click={handleConnect}
      disabled={isConnecting}
    >
      {isConnecting ? 'Connecting...' : 'Connect'}
    </button>
  </svelte:fragment>
</Dialog>

<style>
  .form-group {
    margin-bottom: var(--namd-spacing-md);
  }

  .error-message {
    background-color: var(--namd-error-bg);
    border: 1px solid var(--namd-error-border);
    color: var(--namd-error-fg);
    padding: var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius);
    font-size: var(--namd-font-size-base);
    margin-top: var(--namd-spacing-md);
  }
</style>

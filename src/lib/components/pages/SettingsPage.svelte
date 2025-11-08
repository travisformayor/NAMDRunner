<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore } from '$lib/stores/settings';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import { logger } from '$lib/utils/logger';
  import { jobsStore } from '$lib/stores/jobs';
  import { loadTemplates } from '$lib/stores/templateStore';

  // Store subscriptions
  $: databaseInfo = $settingsStore.databaseInfo;
  $: isLoading = $settingsStore.isLoading;

  // Dialog states
  let showRestoreWarning = false;
  let showResetWarning = false;

  // Load database info on mount
  onMount(() => {
    settingsStore.loadDatabaseInfo();
  });

  // Format file size
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  // Backup handler
  async function handleBackup() {
    logger.debug('Settings', 'Starting database backup');
    const result = await settingsStore.backupDatabase();

    if (result.success) {
      // Success feedback shown via logger
    } else if (result.error !== 'Backup cancelled') {
      // Show error (cancelled is user action, not an error)
      alert(`Backup failed: ${result.error}`);
    }
  }

  // Restore handlers
  function handleRestoreClick() {
    showRestoreWarning = true;
  }

  async function handleRestoreConfirm() {
    showRestoreWarning = false;
    logger.debug('Settings', 'Starting database restore');

    const result = await settingsStore.restoreDatabase();

    if (result.success) {
      // Reload all stores after restore
      await jobsStore.loadFromDatabase();
      await loadTemplates();
      alert('Database restored successfully. All data has been reloaded.');
    } else if (result.error !== 'Restore cancelled') {
      alert(`Restore failed: ${result.error}`);
    }
  }

  // Reset handlers
  function handleResetClick() {
    showResetWarning = true;
  }

  async function handleResetConfirm() {
    showResetWarning = false;
    logger.debug('Settings', 'Resetting database');

    const result = await settingsStore.resetDatabase();

    if (result.success) {
      // Reload all stores after reset
      await jobsStore.loadFromDatabase();
      await loadTemplates();
      alert('Database reset successfully. All data has been cleared.');
    } else {
      alert(`Reset failed: ${result.error}`);
    }
  }
</script>

<div class="settings-page">
  <div class="settings-section">
    <h2>Database</h2>

    {#if isLoading}
      <p class="loading">Loading database information...</p>
    {:else if databaseInfo}
      <div class="db-info">
        <div class="info-row">
          <span class="label">Location:</span>
          <code class="path">{databaseInfo.path}</code>
        </div>
        <div class="info-row">
          <span class="label">Size:</span>
          <span class="value">{formatBytes(databaseInfo.size_bytes)}</span>
        </div>
      </div>

      <div class="db-actions">
        <button class="btn btn-secondary" on:click={handleBackup}> Backup Database </button>

        <button class="btn btn-secondary" on:click={handleRestoreClick}> Restore Database </button>

        <button class="btn btn-destructive" on:click={handleResetClick}> Reset Database </button>
      </div>
    {:else}
      <p class="error">Failed to load database information</p>
    {/if}
  </div>
</div>

<!-- Restore Warning Dialog -->
<ConfirmDialog
  isOpen={showRestoreWarning}
  title="Restore Database?"
  message="<strong>Warning:</strong> This will replace your current database with the backup you select. All current data (jobs, templates, etc.) will be lost.<br><br>Are you sure you want to continue?"
  confirmText="Restore"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleRestoreConfirm}
  onCancel={() => (showRestoreWarning = false)}
/>

<!-- Reset Warning Dialog -->
<ConfirmDialog
  isOpen={showResetWarning}
  title="Reset Database?"
  message="<strong>Warning:</strong> This will delete all data in the database and create a fresh database. All jobs, templates, and other data will be permanently lost.<br><br>This action cannot be undone. Are you sure?"
  confirmText="Reset Database"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleResetConfirm}
  onCancel={() => (showResetWarning = false)}
/>

<style>
  .settings-page {
    padding: 2rem;
    max-width: 800px;
  }

  .settings-section {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .settings-section h2 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
    color: var(--color-text-primary);
  }

  .db-info {
    margin-bottom: 1.5rem;
  }

  .info-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    align-items: baseline;
  }

  .label {
    font-weight: 600;
    color: var(--color-text-secondary);
    min-width: 80px;
  }

  .path {
    background: var(--color-background);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9rem;
    word-break: break-all;
  }

  .value {
    color: var(--color-text-primary);
  }

  .db-actions {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: none;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .btn-secondary {
    background: var(--color-primary);
    color: white;
  }

  .btn-secondary:hover {
    background: var(--color-primary-hover);
  }

  .btn-destructive {
    background: var(--color-danger);
    color: white;
  }

  .btn-destructive:hover {
    background: var(--color-danger-hover);
  }

  .loading,
  .error {
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .error {
    color: var(--color-danger);
  }
</style>

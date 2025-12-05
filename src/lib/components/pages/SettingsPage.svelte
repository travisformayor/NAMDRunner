<script lang="ts">
  import { onMount } from 'svelte';
  import { getName, getVersion } from '@tauri-apps/api/app';
  import { settingsStore } from '$lib/stores/settings';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import AlertDialog from '../ui/AlertDialog.svelte';
  import { jobsStore } from '$lib/stores/jobs';
  import { templateStore } from '$lib/stores/templateStore';

  // Store subscriptions
  $: databaseInfo = $settingsStore.databaseInfo;
  $: isLoading = $settingsStore.loading;

  // App information state for about section
  let appName = '';
  let appVersion = '';

  // Dialog states
  let showRestoreWarning = false;
  let showResetWarning = false;

  // Alert dialog state
  let showAlert = false;
  let alertTitle = '';
  let alertMessage = '';
  let alertVariant: 'success' | 'error' | 'warning' | 'info' = 'info';

  function showAlertDialog(title: string, message: string, variant: 'success' | 'error' | 'warning' | 'info' = 'info') {
    alertTitle = title;
    alertMessage = message;
    alertVariant = variant;
    showAlert = true;
  }

  // Load database info and app metadata on mount
  onMount(async () => {
    settingsStore.loadDatabaseInfo();

    // Fetch app metadata from Tauri APIs
    appName = await getName();
    appVersion = await getVersion();
  });

  // Format file size
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`;
  }

  // Backup handler
  async function handleBackup() {
    const result = await settingsStore.backupDatabase();

    if (result.success) {
      // Success
    } else if (result.error !== 'Backup cancelled') {
      // Show error (cancelled is user action, not an error)
      showAlertDialog('Backup Failed', `Backup failed: ${result.error}`, 'error');
    }
  }

  // Restore handlers
  function handleRestoreClick() {
    showRestoreWarning = true;
  }

  async function handleRestoreConfirm() {
    showRestoreWarning = false;

    const result = await settingsStore.restoreDatabase();

    if (result.success) {
      // Reload all stores after restore
      await jobsStore.loadFromDatabase();
      await templateStore.loadTemplates();
      showAlertDialog('Database Restored', 'Database restored successfully. All data has been reloaded.', 'success');
    } else if (result.error !== 'Restore cancelled') {
      showAlertDialog('Restore Failed', `Restore failed: ${result.error}`, 'error');
    }
  }

  // Reset handlers
  function handleResetClick() {
    showResetWarning = true;
  }

  async function handleResetConfirm() {
    showResetWarning = false;

    const result = await settingsStore.resetDatabase();

    if (result.success) {
      // Reload all stores after reset
      await jobsStore.loadFromDatabase();
      await templateStore.loadTemplates();
      showAlertDialog('Database Reset', 'Database reset successfully. All data has been cleared.', 'success');
    } else {
      showAlertDialog('Reset Failed', `Reset failed: ${result.error}`, 'error');
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
        <button class="namd-button namd-button--secondary" on:click={handleBackup}> Backup Database </button>

        <button class="namd-button namd-button--secondary" on:click={handleRestoreClick}> Restore Database </button>

        <button class="namd-button namd-button--destructive" on:click={handleResetClick}> Reset Database </button>
      </div>
    {:else}
      <p class="error">Failed to load database information</p>
    {/if}
  </div>

  <div class="settings-section">
    <h2>About</h2>

    <div class="db-info">
      <div class="info-row">
        <span class="label">Name:</span>
        <span class="value">{appName}</span>
      </div>
      <div class="info-row">
        <span class="label">Version:</span>
        <span class="value">{appVersion}</span>
      </div>
    </div>
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

<!-- Alert Dialog -->
<AlertDialog
  open={showAlert}
  title={alertTitle}
  message={alertMessage}
  variant={alertVariant}
  onClose={() => (showAlert = false)}
/>

<style>
  .settings-page {
    padding: 2rem;
    max-width: 800px;
  }

  .settings-section {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
    margin-bottom: var(--namd-spacing-lg);
  }

  .settings-section h2 {
    margin: 0 0 var(--namd-spacing-md) 0;
    font-size: var(--namd-font-size-xl);
    color: var(--namd-text-primary);
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
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-secondary);
    min-width: 80px;
  }

  .path {
    background: var(--namd-bg-muted);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-sm);
    word-break: break-all;
  }

  .value {
    color: var(--namd-text-primary);
  }

  .db-actions {
    display: flex;
    gap: var(--namd-spacing-sm);
    flex-wrap: wrap;
  }

  .loading,
  .error {
    color: var(--namd-text-secondary);
    font-style: italic;
  }

  .error {
    color: var(--namd-error);
  }
</style>

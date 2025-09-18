<script lang="ts">
  import { isConnected } from '../../stores/session';
  import { jobsStore, jobCounts, jobs } from '../../stores/jobs';
  import { uiStore } from '../../stores/ui';
  import JobsTable from '../jobs/JobsTable.svelte';
  import SyncControls from '../jobs/SyncControls.svelte';

  async function handleSync() {
    try {
      await jobsStore.sync();
    } catch (error) {
      console.error('Sync failed:', error);
    }
  }

  function handleCreateJob() {
    uiStore.setView('create');
  }

  function handleJobSelect(event: CustomEvent<string>) {
    uiStore.selectJob(event.detail);
  }
</script>

<div class="jobs-page">
  <!-- Header - matches React mockup: simple title + button -->
  <div class="jobs-header">
    <h1>Jobs</h1>
    <button
      class="namd-button namd-button--primary create-job-button"
      on:click={handleCreateJob}
      disabled={!$isConnected}
      title={$isConnected ? 'Create a new job' : 'Connect to cluster first'}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
      Create New Job
    </button>
  </div>

  <!-- Sync Status - matches React mockup positioning -->
  <div class="sync-status-section">
    <SyncControls on:sync={handleSync} />
  </div>

  <div class="jobs-content">
    {#if $jobCounts.total === 0}
      <!-- Show appropriate empty state based on connection status -->
      {#if !$isConnected}
        <div class="empty-state">
          <div class="empty-icon">🔗</div>
          <h3 class="empty-title">Connect to Cluster</h3>
          <p class="empty-description">
            Connect to your SLURM cluster to view and manage your NAMD simulation jobs.
          </p>
        </div>
      {:else}
        <div class="empty-state">
          <div class="empty-icon">📋</div>
          <h3 class="empty-title">No Jobs Yet</h3>
          <p class="empty-description">
            You haven't created any jobs yet. Click "Create Job" to submit your first NAMD simulation.
          </p>
          <button
            class="namd-button namd-button--primary"
            on:click={handleCreateJob}
          >
            <span class="button-icon">➕</span>
            Create Your First Job
          </button>
        </div>
      {/if}
    {:else}
      <!-- Always show jobs table when jobs exist, regardless of connection status -->
      <JobsTable on:jobSelect={handleJobSelect} />
    {/if}
  </div>
</div>

<style>
  .jobs-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--namd-bg-secondary);
    padding: var(--namd-spacing-lg);
    gap: var(--namd-spacing-md);
  }

  .jobs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }


  .create-job-button {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .sync-status-section {
    flex-shrink: 0;
    margin-bottom: calc(-1 * var(--namd-spacing-sm));
  }

  .jobs-content {
    flex: 1;
    overflow: auto;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: var(--namd-spacing-2xl);
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: var(--namd-spacing-lg);
    opacity: 0.6;
  }

  .empty-title {
    margin: 0 0 var(--namd-spacing-md) 0;
    font-size: var(--namd-font-size-xl);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .empty-description {
    margin: 0 0 var(--namd-spacing-lg) 0;
    color: var(--namd-text-secondary);
    max-width: 400px;
    line-height: 1.6;
  }

  @media (max-width: 768px) {
    .jobs-header {
      flex-direction: column;
      align-items: stretch;
      gap: var(--namd-spacing-md);
    }

    .create-job-button {
      justify-content: center;
    }
  }
</style>
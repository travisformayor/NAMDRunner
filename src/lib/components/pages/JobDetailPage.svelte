<script lang="ts">
  import { selectedJobId, uiStore } from '../../stores/ui';
  import { jobs, jobsStore } from '../../stores/jobs';
  import { isConnected } from '../../stores/session';
  import { derived } from 'svelte/store';
  import JobSummary from '../job-detail/JobSummary.svelte';
  import JobTabs from '../job-detail/JobTabs.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';

  // Get the selected job
  const selectedJob = derived(
    [jobs, selectedJobId],
    ([$jobs, $selectedJobId]) => $selectedJobId ? $jobs.find(job => job.job_id === $selectedJobId) : null
  );

  let showDeleteDialog = false;
  let isDeleting = false;
  let deleteError = '';
  let isSyncingResults = false;
  let syncError = '';
  let isSubmitting = false;
  let submitError = '';

  function handleBack() {
    uiStore.selectJob(null);
  }

  function handleDeleteJob() {
    if (!$selectedJob) return;
    if (!$isConnected) return; // Should be disabled, but extra check

    showDeleteDialog = true;
  }

  async function handleSubmitJob() {
    if (!$selectedJob) return;
    if (!$isConnected) return;

    isSubmitting = true;
    submitError = '';

    try {
      const result = await jobsStore.submitJob($selectedJob.job_id);

      if (!result.success) {
        submitError = result.error || 'Failed to submit job';
      }
      // Success - job info will be updated in store automatically
    } catch (error) {
      submitError = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isSubmitting = false;
    }
  }

  async function handleSyncResults() {
    if (!$selectedJob) return;
    if (!$isConnected) return;

    isSyncingResults = true;
    syncError = '';

    try {
      const result = await jobsStore.syncResultsFromScratch($selectedJob.job_id);

      if (!result.success) {
        syncError = result.error || 'Failed to sync results from scratch';
      }
      // Success - job info will be updated in store automatically
    } catch (error) {
      syncError = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isSyncingResults = false;
    }
  }

  async function handleConfirmDelete() {
    if (!$selectedJob) return;

    showDeleteDialog = false;
    isDeleting = true;
    deleteError = '';

    try {
      // Call backend to delete job with remote deletion
      const result = await jobsStore.deleteJob($selectedJob.job_id);

      if (result.success) {
        // Navigate back to jobs list after successful deletion
        handleBack();
      } else {
        deleteError = result.error || 'Failed to delete job';
      }
    } catch (error) {
      deleteError = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isDeleting = false;
    }
  }

  function handleCancelDelete() {
    showDeleteDialog = false;
  }
</script>

<div class="job-detail-page">
  {#if $selectedJob}
    <!-- Back Button -->
    <button class="back-button" on:click={handleBack}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m12 19-7-7 7-7"/>
        <path d="M19 12H5"/>
      </svg>
      Back to Jobs
    </button>

    {#if deleteError}
      <div class="error-banner">
        <strong>Error deleting job:</strong> {deleteError}
      </div>
    {/if}

    {#if submitError}
      <div class="error-banner">
        <strong>Error submitting job:</strong> {submitError}
      </div>
    {/if}

    {#if syncError}
      <div class="error-banner">
        <strong>Error syncing results:</strong> {syncError}
      </div>
    {/if}

    <!-- Job Summary -->
    <JobSummary job={$selectedJob} />

    <!-- Tab Navigation -->
    <JobTabs job={$selectedJob} />

    <!-- Action Buttons -->
    <div class="action-buttons">
      {#if $selectedJob.status === 'CREATED' || $selectedJob.status === 'FAILED'}
        <button
          class="namd-button namd-button--primary submit-button"
          on:click={handleSubmitJob}
          disabled={!$isConnected || isSubmitting}
          title={!$isConnected ? "Connect to server to submit jobs" : "Submit job to SLURM scheduler"}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14"/>
            <path d="m12 5 7 7-7 7"/>
          </svg>
          {isSubmitting ? 'Submitting...' : 'Submit Job'}
        </button>
      {/if}

      {#if $selectedJob.status === 'COMPLETED'}
        <button
          class="namd-button namd-button--primary sync-button"
          on:click={handleSyncResults}
          disabled={!$isConnected || isSyncingResults}
          title={!$isConnected ? "Connect to server to sync results" : "Sync results from scratch directory"}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {isSyncingResults ? 'Syncing...' : 'Sync Results from Scratch'}
        </button>
      {/if}

      <button
        class="delete-button namd-button namd-button--destructive"
        on:click={handleDeleteJob}
        disabled={!$isConnected || isDeleting}
        title={!$isConnected ? "Connect to server to delete jobs" : "Delete this job"}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="m3 6 3 0"/>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        </svg>
        {isDeleting ? 'Deleting...' : 'Delete Job'}
      </button>
    </div>
  {:else}
    <div class="no-job-selected">
      <div class="no-job-icon">📋</div>
      <h3 class="no-job-title">No Job Selected</h3>
      <p class="no-job-description">
        Select a job from the jobs list to view its details.
      </p>
      <button class="namd-button namd-button--primary" on:click={handleBack}>
        View Jobs
      </button>
    </div>
  {/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  isOpen={showDeleteDialog}
  title="Delete Job?"
  message="Are you sure you want to delete this job? This will permanently delete:
  • The job record from your local database
  • All job files on the server (input files, output files, SLURM scripts)
  • All job metadata on the server

This action cannot be undone."
  confirmText="Delete Job"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleConfirmDelete}
  onCancel={handleCancelDelete}
/>

<style>
  .job-detail-page {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    background-color: var(--namd-bg-secondary);
    padding: var(--namd-spacing-sm) var(--namd-spacing-lg);
    gap: var(--namd-spacing-sm);
    overflow: auto;
  }

  .error-banner {
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 12px;
    border-radius: 6px;
    font-size: 14px;
  }

  .back-button {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    background: none;
    border: none;
    color: var(--namd-text-primary);
    cursor: pointer;
    font-size: var(--namd-font-size-sm);
    margin-bottom: var(--namd-spacing-md);
    padding: 0;
  }

  .back-button:hover {
    color: var(--namd-primary);
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    padding-top: var(--namd-spacing-sm);
    border-top: 1px solid var(--namd-border);
    margin-top: var(--namd-spacing-xs);
  }

  .submit-button,
  .sync-button,
  .delete-button {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .no-job-selected {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: var(--namd-spacing-2xl);
  }

  .no-job-icon {
    font-size: 4rem;
    margin-bottom: var(--namd-spacing-lg);
    opacity: 0.6;
  }

  .no-job-title {
    margin: 0 0 var(--namd-spacing-md) 0;
    font-size: var(--namd-font-size-xl);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .no-job-description {
    margin: 0 0 var(--namd-spacing-lg) 0;
    color: var(--namd-text-secondary);
    max-width: 400px;
    line-height: 1.6;
  }

  @media (max-width: 768px) {
    .action-buttons {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
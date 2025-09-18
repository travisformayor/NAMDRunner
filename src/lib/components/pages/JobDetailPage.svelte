<script lang="ts">
  import { selectedJobId, uiStore } from '../../stores/ui';
  import { jobs, jobsStore } from '../../stores/jobs';
  import { derived } from 'svelte/store';
  import JobSummary from '../job-detail/JobSummary.svelte';
  import JobTabs from '../job-detail/JobTabs.svelte';

  // Get the selected job
  const selectedJob = derived(
    [jobs, selectedJobId],
    ([$jobs, $selectedJobId]) => $selectedJobId ? $jobs.find(job => job.jobId === $selectedJobId) : null
  );

  function handleBack() {
    uiStore.selectJob(null);
  }

  function handleDeleteJob() {
    if (!$selectedJob) return;

    // Show confirmation dialog (for now, just confirm with browser dialog)
    const confirmed = confirm(`Are you sure you want to delete job "${$selectedJob.jobName}"?`);
    if (confirmed) {
      jobsStore.removeJob($selectedJob.jobId);
      handleBack();
    }
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

    <!-- Job Summary -->
    <JobSummary job={$selectedJob} />

    <!-- Tab Navigation -->
    <JobTabs job={$selectedJob} />

    <!-- Action Buttons -->
    <div class="action-buttons">
      {#if $selectedJob.status === 'COMPLETED'}
        <button class="namd-button namd-button--primary sync-button">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Sync Results from Scratch
        </button>
      {/if}

      <button
        class="delete-button namd-button namd-button--destructive"
        on:click={handleDeleteJob}
        title="Delete this job"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="m3 6 3 0"/>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        </svg>
        Delete Job
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
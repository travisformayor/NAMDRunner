<script lang="ts">
  import type { Job } from '../../types/api';
  import JobStatusBadge from '../jobs/JobStatusBadge.svelte';

  export let job: Job;

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString();
  }

  function formatWallTime(job: Job): string {
    if (!job.slurmConfig) return '--';
    const wallTime = job.slurmConfig.time;
    if (job.wallTimeRemaining && job.wallTimeRemaining !== '--') {
      return wallTime;
    }
    return wallTime;
  }

  function formatWallTimeLeft(job: Job): string | null {
    if (!job.wallTimeRemaining || job.wallTimeRemaining === '--') return null;
    if (job.status === 'COMPLETED' || job.status === 'FAILED') return null;
    return job.wallTimeRemaining;
  }
</script>

<div class="job-summary namd-card">
  <div class="namd-card-header">
    <div class="summary-header">
      <div class="title-section">
        <h2 class="job-title">{job.jobName}</h2>
        <div class="job-ids">
          <span class="job-id">Job ID: <span class="id-value">{job.jobId}</span></span>
          {#if job.slurmJobId}
            <span class="slurm-id">SLURM ID: <span class="id-value">{job.slurmJobId}</span></span>
          {/if}
        </div>
      </div>
      <JobStatusBadge status={job.status} />
    </div>
  </div>

  <div class="namd-card-content">
    <div class="summary-grid">
      <div class="grid-item">
        <div class="grid-label">Created</div>
        <div class="grid-value">{formatDate(job.createdAt)}</div>
      </div>

      <div class="grid-item">
        <div class="grid-label">Submitted</div>
        <div class="grid-value">{job.submittedAt ? formatDate(job.submittedAt) : '--'}</div>
      </div>

      <div class="grid-item">
        <div class="grid-label">Runtime</div>
        <div class="grid-value namd-text-mono">{job.runtime || '--'}</div>
      </div>

      <div class="grid-item">
        <div class="grid-label">Wall Time</div>
        <div class="grid-value">
          <div>{formatWallTime(job)}</div>
          {#if formatWallTimeLeft(job)}
            <div class="wall-time-left">{formatWallTimeLeft(job)}</div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .summary-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--namd-spacing-md);
  }

  .title-section {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .job-title {
    margin: 0;
    font-size: var(--namd-font-size-xl);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    word-break: break-word;
  }

  .job-ids {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-muted);
  }

  .id-value {
    font-family: var(--namd-font-mono);
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--namd-spacing-md);
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .grid-label {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-muted);
  }

  .grid-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
  }

  .wall-time-left {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
  }

  @media (max-width: 768px) {
    .summary-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--namd-spacing-sm);
    }

    .job-ids {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--namd-spacing-xs);
    }

    .summary-grid {
      grid-template-columns: 1fr;
      gap: var(--namd-spacing-md);
    }
  }

  @media (min-width: 768px) {
    .summary-grid {
      grid-template-columns: repeat(4, 1fr);
    }
  }
</style>
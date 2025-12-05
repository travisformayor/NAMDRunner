<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { jobs } from '../../stores/jobs';
  import JobStatusBadge from './JobStatusBadge.svelte';
  import type { JobInfo } from '../../types/api';

  const dispatch = createEventDispatcher<{ jobSelect: string }>();

  type SortField = 'job_name' | 'status' | 'created_at' | 'slurm_job_id';
  type SortDirection = 'asc' | 'desc';

  let sortField: SortField = 'created_at';
  let sortDirection: SortDirection = 'desc';

  // Sort jobs based on current sort settings
  $: sortedJobs = [...$jobs].sort((a, b) => {
    let aValue: any;
    let bValue: any;

    switch (sortField) {
      case 'job_name':
        aValue = a.job_name.toLowerCase();
        bValue = b.job_name.toLowerCase();
        break;
      case 'status':
        aValue = a.status;
        bValue = b.status;
        break;
      case 'created_at':
        aValue = new Date(a.created_at);
        bValue = new Date(b.created_at);
        break;
      case 'slurm_job_id':
        aValue = a.slurm_job_id || '';
        bValue = b.slurm_job_id || '';
        break;
      default:
        aValue = a.created_at;
        bValue = b.created_at;
    }

    if (typeof aValue === 'string') {
      const result = aValue.localeCompare(bValue);
      return sortDirection === 'asc' ? result : -result;
    } else if (aValue instanceof Date) {
      const result = aValue.getTime() - bValue.getTime();
      return sortDirection === 'asc' ? result : -result;
    } else {
      const result = aValue - bValue;
      return sortDirection === 'asc' ? result : -result;
    }
  });

  function handleSort(field: SortField) {
    if (sortField === field) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortField = field;
      sortDirection = 'asc';
    }
  }

  function handleJobClick(job: JobInfo) {
    dispatch('jobSelect', job.job_id);
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return `${date.toLocaleDateString()} ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
  }

  function formatWallTime(job: JobInfo): string {
    if (!job.slurm_config) return '--';
    return job.slurm_config.walltime;
  }
</script>

<div class="jobs-table-container">
  <table class="jobs-table">
    <thead>
      <tr>
        <th>
          <button
            class="sort-header"
            class:active={sortField === 'job_name'}
            on:click={() => handleSort('job_name')}
          >
            Job Name
            {#if sortField === 'job_name'}
              <span class="sort-indicator" class:desc={sortDirection === 'desc'}>▲</span>
            {/if}
          </button>
        </th>
        <th>
          <button
            class="sort-header"
            class:active={sortField === 'status'}
            on:click={() => handleSort('status')}
          >
            Status
            {#if sortField === 'status'}
              <span class="sort-indicator" class:desc={sortDirection === 'desc'}>▲</span>
            {/if}
          </button>
        </th>
        <th>Wall Time</th>
        <th>
          <button
            class="sort-header"
            class:active={sortField === 'created_at'}
            on:click={() => handleSort('created_at')}
          >
            Created
            {#if sortField === 'created_at'}
              <span class="sort-indicator" class:desc={sortDirection === 'desc'}>▲</span>
            {/if}
          </button>
        </th>
        <th>
          <button
            class="sort-header"
            class:active={sortField === 'slurm_job_id'}
            on:click={() => handleSort('slurm_job_id')}
          >
            Job ID
            {#if sortField === 'slurm_job_id'}
              <span class="sort-indicator" class:desc={sortDirection === 'desc'}>▲</span>
            {/if}
          </button>
        </th>
      </tr>
    </thead>
    <tbody>
      {#each sortedJobs as job (job.job_id)}
        <tr class="job-row" on:click={() => handleJobClick(job)}>
          <td class="job-name">
            <span class="name-text">{job.job_name}</span>
          </td>
          <td class="job-status">
            <JobStatusBadge status={job.status} />
          </td>
          <td class="job-walltime">
            <span class="walltime-text">{formatWallTime(job)}</span>
          </td>
          <td class="job-created">
            <span class="date-text">{formatDate(job.created_at)}</span>
          </td>
          <td class="job-id">
            <span class="id-text">{job.slurm_job_id || 'Not submitted'}</span>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if sortedJobs.length === 0}
    <div class="empty-table">
      <div class="empty-icon">📋</div>
      <p class="empty-text">No jobs found</p>
    </div>
  {/if}
</div>

<style>
  .jobs-table-container {
    background-color: var(--namd-bg-primary);
    border-radius: var(--namd-border-radius);
    box-shadow: var(--namd-shadow-sm);
    overflow: hidden;
  }

  .jobs-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--namd-font-size-sm);
  }

  .jobs-table thead {
    background-color: var(--namd-bg-muted);
    border-bottom: 1px solid var(--namd-border-muted);
  }

  .jobs-table th {
    text-align: left;
    padding: 0;
    font-weight: var(--namd-font-weight-medium);
  }

  .sort-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--namd-spacing-md);
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-secondary);
    transition: all 0.15s ease;
  }

  .sort-header:hover {
    background-color: var(--namd-bg-secondary);
  }

  .sort-header.active {
    color: var(--namd-text-primary);
  }

  .sort-indicator {
    transition: transform 0.15s ease;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-primary);
  }

  .sort-indicator.desc {
    transform: rotate(180deg);
  }

  .jobs-table tbody tr {
    border-bottom: 1px solid var(--namd-border-muted);
    transition: background-color 0.15s ease;
  }

  .job-row {
    cursor: pointer;
  }

  .job-row:hover {
    background-color: var(--namd-bg-muted);
  }

  .job-row:last-child {
    border-bottom: none;
  }

  .jobs-table td {
    padding: var(--namd-spacing-md);
    vertical-align: middle;
  }

  .job-name .name-text {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .id-text {
    font-family: var(--namd-font-mono);
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-xs);
  }

  .walltime-text {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .date-text {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .empty-table {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--namd-spacing-2xl);
    text-align: center;
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: var(--namd-spacing-md);
    opacity: 0.5;
  }

  .empty-text {
    margin: 0;
    color: var(--namd-text-muted);
    font-size: var(--namd-font-size-sm);
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .jobs-table {
      font-size: var(--namd-font-size-xs);
    }

    .jobs-table td,
    .sort-header {
      padding: var(--namd-spacing-sm);
    }

    .jobs-table th:nth-child(3),
    .jobs-table td:nth-child(3),
    .jobs-table th:nth-child(4),
    .jobs-table td:nth-child(4) {
      display: none;
    }
  }
</style>
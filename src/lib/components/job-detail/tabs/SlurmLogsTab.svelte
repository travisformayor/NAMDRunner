<script lang="ts">
  import type { JobInfo } from '../../../types/api';
  import { isConnected } from '../../../stores/session';

  export let job: JobInfo;

  let activeLogTab: 'stdout' | 'stderr' = 'stdout';
  let isRefetchingLogs = false;
  let refetchError = '';

  function getStdoutContent(): string {
    if (job.status === 'CREATED' || job.status === 'PENDING') {
      return 'Logs will be available once the job starts running.';
    }

    // Use cached logs from database
    if (job.slurm_stdout !== undefined && job.slurm_stdout !== null) {
      return job.slurm_stdout || '(No output)';
    }

    return 'Logs are being fetched from the server automatically...';
  }

  function getStderrContent(): string {
    if (job.status === 'CREATED' || job.status === 'PENDING') {
      return 'Logs will be available once the job starts running.';
    }

    // Use cached logs from database
    if (job.slurm_stderr !== undefined && job.slurm_stderr !== null) {
      return job.slurm_stderr || '(No errors)';
    }

    return 'Logs are being fetched from the server automatically...';
  }

  function copyLogs() {
    const content = activeLogTab === 'stdout' ? getStdoutContent() : getStderrContent();
    navigator.clipboard.writeText(content);
  }

  async function refetchLogs() {
    if (!job || !$isConnected) return;

    isRefetchingLogs = true;
    refetchError = '';

    try {
      const result = await CoreClientFactory.getClient().refetchSlurmLogs(job.job_id);

      if (!result.success) {
        refetchError = result.error || 'Failed to refetch logs';
      } else if (result.job_info) {
        // Update the job with new logs (parent component should handle this via reactive update)
        job = result.job_info;
      }
    } catch (error) {
      refetchError = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isRefetchingLogs = false;
    }
  }
</script>

<div class="namd-tab-panel">
  {#if job.status === 'CREATED'}
    <div class="empty-logs">
      Logs will be available once the job starts running.
    </div>
  {:else}
    <div class="logs-container">
      <div class="logs-header">
        <div class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-2">
          <button
            class="namd-tab-button"
            class:active={activeLogTab === 'stdout'}
            on:click={() => activeLogTab = 'stdout'}
          >
            Output
          </button>
          <button
            class="namd-tab-button"
            class:active={activeLogTab === 'stderr'}
            on:click={() => activeLogTab = 'stderr'}
          >
            Error
          </button>
        </div>

        <div class="log-actions">
          <button class="namd-button namd-button--outline log-action-btn" on:click={copyLogs}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
              <path d="M4 16c-1.1 0-2-.9-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
            </svg>
            Copy
          </button>
          <button
            class="namd-button namd-button--outline log-action-btn"
            on:click={refetchLogs}
            disabled={!$isConnected || isRefetchingLogs}
            title={!$isConnected ? "Connect to server to refetch logs" : "Re-fetch latest logs from server"}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 4 23 10 17 10"/>
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
            </svg>
            {isRefetchingLogs ? 'Refetching...' : 'Refetch Logs'}
          </button>
        </div>
      </div>

      <div class="log-content">
        {#if refetchError}
          <div class="namd-error-message" style="margin-bottom: 1rem;">
            {refetchError}
          </div>
        {/if}
        {#if activeLogTab === 'stdout'}
          <div class="log-viewer">
            <pre class="log-text">{getStdoutContent()}</pre>
          </div>
        {:else}
          <div class="log-viewer">
            <pre class="log-text">{getStderrContent()}</pre>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { JobInfo, DownloadResult } from '../../../types/api';
  import { getFileIcon, getTypeLabel, getTypeColor, getFileDescription, getFileExtension, formatFileSize } from '../../../utils/file-helpers';
  import { isConnected } from '../../../stores/session';

  export let job: JobInfo;

  function getOutputFiles() {
    // Use job.output_files from job info
    return job.output_files?.map(file => {
      const ext = getFileExtension(file.name);
      return {
        name: file.name,
        path: `outputs/${file.name}`,
        size: formatFileSize(file.size),
        type: ext,
        description: getFileDescription(ext),
        lastModified: file.modified_at,
        available: true
      };
    }) || [];
  }

  // File handling functions
  let downloadingFiles = new Set<string>();
  let downloadErrors = new Map<string, string>();

  async function downloadFile(file_path: string, file_name: string) {
    // Clear previous error for this file (matches app pattern)
    downloadErrors.delete(file_name);
    downloadErrors = downloadErrors; // Trigger reactivity

    if (!$isConnected) {
      downloadErrors.set(file_name, 'Connect to server to download files');
      downloadErrors = downloadErrors; // Trigger reactivity
      return;
    }

    downloadingFiles.add(file_name);
    downloadingFiles = downloadingFiles; // Trigger reactivity

    try {
      const result = await invoke<DownloadResult>('download_job_output', { job_id: job.job_id, file_path });

      if (!result.success) {
        downloadErrors.set(file_name, result.error || 'Failed to download file');
        downloadErrors = downloadErrors; // Trigger reactivity
      }
      // Success - file was saved to user's chosen location via native dialog
    } catch (error) {
      downloadErrors.set(file_name, error instanceof Error ? error.message : 'Download failed');
      downloadErrors = downloadErrors; // Trigger reactivity
    } finally {
      downloadingFiles.delete(file_name);
      downloadingFiles = downloadingFiles; // Trigger reactivity
    }
  }

  let isDownloadingAll = false;
  let downloadAllError = '';

  async function downloadAllOutputs() {
    // Clear previous error (matches app pattern)
    downloadAllError = '';

    if (!$isConnected) {
      downloadAllError = 'Connect to server to download files';
      return;
    }

    isDownloadingAll = true;

    try {
      const result = await invoke<DownloadResult>('download_all_outputs', { job_id: job.job_id });

      if (!result.success) {
        downloadAllError = result.error || 'Failed to download output files';
      }
      // Success - zip file was saved to user's chosen location via native dialog
    } catch (error) {
      downloadAllError = error instanceof Error ? error.message : 'Download failed';
    } finally {
      isDownloadingAll = false;
    }
  }
</script>

<div class="namd-tab-panel">
  <div class="output-files-content">
    {#if job.status === 'CREATED' || job.status === 'PENDING'}
      <div class="empty-files">
        Output files will be available once the job starts running.
      </div>
    {:else}
      <!-- Available Files -->
      {#if getOutputFiles().filter(f => f.available).length > 0}
        <div class="files-section">
          <div class="files-section-header">
            <h3>Available Files</h3>
            <button
              class="download-all-button"
              on:click={downloadAllOutputs}
              disabled={!$isConnected || isDownloadingAll}
              title={!$isConnected ? "Connect to server to download files" : "Download all output files as ZIP"}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7,10 12,15 17,10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
              {isDownloadingAll ? 'Downloading...' : 'Download All Outputs'}
            </button>
          </div>
          {#if downloadAllError}
            <div class="download-all-error">{downloadAllError}</div>
          {/if}
          <div class="files-grid">
            {#each getOutputFiles().filter(f => f.available) as file}
              <div class="file-card">
                <div class="file-card-content">
                  <div class="file-info">
                    <div class="file-icon-large">{getFileIcon(file.type)}</div>
                    <div class="file-details">
                      <div class="file-header-row">
                        <span class="file-name">{file.name}</span>
                        <span class="namd-file-type-badge {getTypeColor(file.type)}">
                          {getTypeLabel(file.type)}
                        </span>
                      </div>
                      <div class="file-description">{file.description}</div>
                      <div class="file-metadata">
                        Size: {file.size} • Modified: {file.lastModified}
                      </div>
                    </div>
                  </div>

                  <button
                    class="download-button"
                    on:click={() => downloadFile(file.path, file.name)}
                    disabled={!$isConnected || downloadingFiles.has(file.name)}
                    title={!$isConnected ? "Connect to server to download files" : "Download file"}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                      <polyline points="7,10 12,15 17,10"/>
                      <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    {downloadingFiles.has(file.name) ? 'Downloading...' : 'Download'}
                  </button>
                  {#if downloadErrors.has(file.name)}
                    <div class="file-error">{downloadErrors.get(file.name)}</div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Unavailable/Expected Files -->
      {#if getOutputFiles().filter(f => !f.available).length > 0}
        <div class="files-section">
          <h3>Expected Files</h3>
          <div class="namd-text-sm namd-text-muted expected-files-description">
            These files will be available once the simulation produces them.
          </div>
          <div class="files-grid">
            {#each getOutputFiles().filter(f => !f.available) as file}
              <div class="file-card unavailable">
                <div class="file-card-content">
                  <div class="file-info">
                    <div class="file-icon-large">{getFileIcon(file.type)}</div>
                    <div class="file-details">
                      <div class="file-header-row">
                        <span class="file-name">{file.name}</span>
                        <span class="namd-file-type-badge {getTypeColor(file.type)}">
                          {getTypeLabel(file.type)}
                        </span>
                      </div>
                      <div class="file-description">{file.description}</div>
                    </div>
                  </div>

                  <button class="download-button" disabled>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                      <polyline points="7,10 12,15 17,10"/>
                      <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    Pending
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Output Summary -->
      <div class="file-summary">
        <div class="summary-title">Output Summary</div>
        <div class="summary-grid">
          <div class="summary-item">
            <span class="summary-label">Trajectory Files:</span>
            <span class="summary-value">{getOutputFiles().filter(f => f.type === 'trajectory').length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Log Files:</span>
            <span class="summary-value">{getOutputFiles().filter(f => f.type === 'log').length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Analysis Files:</span>
            <span class="summary-value">{getOutputFiles().filter(f => f.type === 'analysis').length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Checkpoint Files:</span>
            <span class="summary-value">{getOutputFiles().filter(f => f.type === 'checkpoint').length}</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

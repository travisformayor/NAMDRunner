<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { JobInfo, DownloadInfo, ApiResult } from '../../../types/api';
  import { getFileIcon, getFileExtension } from '../../../utils/file-helpers';
  import { isConnected } from '../../../stores/session';

  export let job: JobInfo;

  function getInputFiles() {
    if (!job.input_files) return [];

    return job.input_files.map(fileName => ({
      name: fileName,
      path: `input_files/${fileName}`,
      ext: getFileExtension(fileName)
    }));
  }

  // File download state
  let downloadingFiles = new Set<string>();
  let downloadErrors = new Map<string, string>();

  async function downloadFile(file_path: string, file_name: string) {
    downloadErrors.delete(file_name);
    downloadErrors = new Map(downloadErrors);

    if (!$isConnected) {
      downloadErrors.set(file_name, 'Connect to server to download files');
      downloadErrors = new Map(downloadErrors);
      return;
    }

    downloadingFiles.add(file_name);
    downloadingFiles = new Set(downloadingFiles);

    try {
      const result = await invoke<ApiResult<DownloadInfo>>('download_job_input', { job_id: job.job_id, file_path });

      if (!result.success) {
        downloadErrors.set(file_name, result.error || 'Failed to download file');
        downloadErrors = new Map(downloadErrors);
      }
    } catch (error) {
      downloadErrors.set(file_name, error instanceof Error ? error.message : 'Download failed');
      downloadErrors = new Map(downloadErrors);
    } finally {
      downloadingFiles.delete(file_name);
      downloadingFiles = new Set(downloadingFiles);
    }
  }

  let isDownloadingAll = false;
  let downloadAllError = '';

  async function downloadAllInputs() {
    downloadAllError = '';

    if (!$isConnected) {
      downloadAllError = 'Connect to server to download files';
      return;
    }

    isDownloadingAll = true;

    try {
      const result = await invoke<ApiResult<DownloadInfo>>('download_all_inputs', { job_id: job.job_id });

      if (!result.success) {
        downloadAllError = result.error || 'Failed to download input files';
      }
    } catch (error) {
      downloadAllError = error instanceof Error ? error.message : 'Download failed';
    } finally {
      isDownloadingAll = false;
    }
  }
</script>

<div class="namd-tab-panel">
  <div class="files-section">
    <!-- Bulk download header -->
    {#if getInputFiles().length > 0}
      <div class="namd-file-list-header">
        <h3>Input Files</h3>
        <button
          class="namd-button namd-button--secondary namd-button--sm"
          on:click={downloadAllInputs}
          disabled={!$isConnected || isDownloadingAll}
          title={!$isConnected ? "Connect to server to download files" : "Download all input files as ZIP"}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {isDownloadingAll ? 'Downloading...' : 'Download All'}
        </button>
      </div>
    {/if}

    <!-- Bulk download error -->
    {#if downloadAllError}
      <div class="namd-file-list-error">{downloadAllError}</div>
    {/if}

    <!-- File list -->
    {#if getInputFiles().length > 0}
      <div class="namd-file-list">
        {#each getInputFiles() as file}
          <div class="namd-file-item">
            <div class="namd-file-content">
              <span class="namd-file-icon">{getFileIcon(file.ext)}</span>
              <span class="namd-file-name">{file.name}</span>
              <div class="namd-file-action">
                <button
                  class="namd-button namd-button--secondary namd-button--sm"
                  on:click={() => downloadFile(file.path, file.name)}
                  disabled={!$isConnected || downloadingFiles.has(file.name)}
                  title={!$isConnected ? "Connect to server to download file" : "Download file"}
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7,10 12,15 17,10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                  </svg>
                  {downloadingFiles.has(file.name) ? 'Downloading...' : 'Download'}
                </button>
              </div>
            </div>
            {#if downloadErrors.has(file.name)}
              <div class="namd-file-error">{downloadErrors.get(file.name)}</div>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div class="namd-file-list-empty">
        No input files available for this job.
      </div>
    {/if}
  </div>
</div>

<style>
  .files-section {
    max-width: 800px;
  }

  .files-section h3 {
    margin: 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-medium);
  }
</style>

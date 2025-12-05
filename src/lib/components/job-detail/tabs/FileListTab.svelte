<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { JobInfo, DownloadInfo, ApiResult } from '../../../types/api';
  import { getFileIcon, getFileExtension, formatFileSize } from '../../../utils/file-helpers';
  import { isConnected } from '../../../stores/session';

  export let job: JobInfo;
  export let type: 'input' | 'output';

  const config = type === 'input' ? {
    title: 'Input Files',
    downloadCommand: 'download_job_input',
    downloadAllCommand: 'download_all_inputs',
    pathPrefix: 'input_files/',
    showSize: false,
    checkStatus: false,
    emptyMessage: 'No input files available for this job.'
  } : {
    title: 'Output Files',
    downloadCommand: 'download_job_output',
    downloadAllCommand: 'download_all_outputs',
    pathPrefix: 'outputs/',
    showSize: true,
    checkStatus: true,
    statusMessage: 'Output files will be available once the job starts running.',
    emptyMessage: 'No output files available yet. Files will appear here once the simulation produces them.'
  };

  function getFiles() {
    if (type === 'input') {
      return job.input_files.map(fileName => ({
        name: fileName,
        path: `${config.pathPrefix}${fileName}`,
        ext: getFileExtension(fileName),
        size: undefined
      }));
    } else {
      return (job.output_files || []).map(file => ({
        name: file.name,
        path: `${config.pathPrefix}${file.name}`,
        ext: getFileExtension(file.name),
        size: formatFileSize(file.size)
      }));
    }
  }

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

    const result = await invoke<ApiResult<DownloadInfo>>(config.downloadCommand, {
      job_id: job.job_id,
      file_path,
    });

    if (!result.success) {
      downloadErrors.set(file_name, result.error || 'Failed to download file');
      downloadErrors = new Map(downloadErrors);
    }

    downloadingFiles.delete(file_name);
    downloadingFiles = new Set(downloadingFiles);
  }

  let isDownloadingAll = false;
  let downloadAllError = '';

  async function downloadAllFiles() {
    downloadAllError = '';

    if (!$isConnected) {
      downloadAllError = 'Connect to server to download files';
      return;
    }

    isDownloadingAll = true;

    const result = await invoke<ApiResult<DownloadInfo>>(config.downloadAllCommand, {
      job_id: job.job_id,
    });

    if (!result.success) {
      downloadAllError = result.error || `Failed to download ${type} files`;
    }

    isDownloadingAll = false;
  }
</script>

<div class="namd-tab-panel">
  <div class="files-section">
    {#if config.checkStatus && (job.status === 'CREATED' || job.status === 'PENDING')}
      <div class="namd-file-list-empty">
        {config.statusMessage}
      </div>
    {:else if getFiles().length > 0}
      <div class="namd-file-list-header">
        <h3>{config.title}</h3>
        <button
          class="namd-button namd-button--secondary namd-button--sm"
          on:click={downloadAllFiles}
          disabled={!$isConnected || isDownloadingAll}
          title={!$isConnected ? "Connect to server to download files" : `Download all ${type} files as ZIP`}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {isDownloadingAll ? 'Downloading...' : 'Download All'}
        </button>
      </div>

      {#if downloadAllError}
        <div class="namd-file-list-error">{downloadAllError}</div>
      {/if}

      <div class="namd-file-list">
        {#each getFiles() as file}
          <div class="namd-file-item">
            <div class="namd-file-content">
              <span class="namd-file-icon">{getFileIcon(file.ext)}</span>
              <span class="namd-file-name">{file.name}</span>
              {#if config.showSize && file.size}
                <span class="namd-file-metadata">{file.size}</span>
              {/if}
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
        {config.emptyMessage}
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

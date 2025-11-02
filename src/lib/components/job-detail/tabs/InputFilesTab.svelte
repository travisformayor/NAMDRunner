<script lang="ts">
  import type { JobInfo } from '../../../types/api';
  import { getFileIcon, getTypeLabel, getTypeColor, getFileDescription, getFileExtension, formatFileSize } from '../../../utils/file-helpers';
  import { CoreClientFactory } from '../../../ports/clientFactory';
  import { isConnected } from '../../../stores/session';
  import { mockInputFiles } from '../../../test/fixtures/mockJobData';

  export let job: JobInfo;
  export let isDemoMode: boolean = false;

  function getInputFiles() {
    if (isDemoMode) {
      return mockInputFiles;
    }
    // Real mode: use job.input_files from job info
    return job.input_files?.map(file => {
      const name = file.name || file.remote_name || 'unknown';
      const ext = file.file_type || getFileExtension(name);
      return {
        name,
        path: `input_files/${name}`,
        size: file.size ? formatFileSize(file.size) : '--',
        type: ext,
        description: getFileDescription(ext)
      };
    }) || [];
  }

  // File handling functions
  let downloadingFiles = new Set<string>();
  let downloadErrors = new Map<string, string>();

  async function downloadFile(file_path: string, file_name: string) {
    if (!$isConnected) {
      downloadErrors.set(file_name, 'Connect to server to download files');
      setTimeout(() => downloadErrors.delete(file_name), 3000);
      return;
    }

    downloadingFiles.add(file_name);
    downloadingFiles = downloadingFiles; // Trigger reactivity
    downloadErrors.delete(file_name);

    try {
      const result = await CoreClientFactory.getClient().downloadJobOutput(job.job_id, file_path);

      if (!result.success) {
        downloadErrors.set(file_name, result.error || 'Failed to download file');
        setTimeout(() => downloadErrors.delete(file_name), 5000);
      }
      // Success - file was saved to user's chosen location via native dialog
    } catch (error) {
      downloadErrors.set(file_name, error instanceof Error ? error.message : 'Download failed');
      setTimeout(() => downloadErrors.delete(file_name), 5000);
    } finally {
      downloadingFiles.delete(file_name);
      downloadingFiles = downloadingFiles; // Trigger reactivity
    }
  }
</script>

<div class="namd-tab-panel">
  <div class="input-files-content">
    <div class="files-header">
      <h3>Input Files</h3>
      <div class="namd-text-sm namd-text-muted">
        Input files used for job: {job.job_name}
      </div>
    </div>

    <div class="files-grid">
      {#each getInputFiles() as file}
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
                <div class="file-size">Size: {file.size}</div>
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

    <div class="file-summary">
      <div class="summary-title">File Summary</div>
      <div class="summary-grid">
        <div class="summary-item">
          <span class="summary-label">Structure Files:</span>
          <span class="summary-value">{getInputFiles().filter(f => f.type === 'structure').length}</span>
        </div>
        <div class="summary-item">
          <span class="summary-label">Parameter Files:</span>
          <span class="summary-value">{getInputFiles().filter(f => f.type === 'parameters').length}</span>
        </div>
        <div class="summary-item">
          <span class="summary-label">Configuration Files:</span>
          <span class="summary-value">{getInputFiles().filter(f => f.type === 'configuration').length}</span>
        </div>
      </div>
    </div>
  </div>
</div>

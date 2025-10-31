<script lang="ts">
  export let uploadedFiles: { name: string; size: number; type: string; path: string }[];
  export let errors: Record<string, string>;
  export let onFileUpload: () => void;
  export let onRemoveFile: (index: number) => void;
  export let formatFileSize: (bytes: number) => string;

  // File upload functionality
  let isDragOver = false;

  const acceptedExtensions = [".pdb", ".psf", ".prm", ".exb", ".enm.extra"];

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
    // Trigger the file dialog instead of using dropped files directly
    // (browser File objects don't have accessible absolute paths)
    onFileUpload();
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
  }

  function handleBrowseClick() {
    onFileUpload();
  }
</script>

<div class="namd-tab-panel">
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Input Files</h3>
    </div>

    <div class="file-upload-area">
      <!-- Drag and drop upload zone (triggers Tauri file dialog) -->
      <div
        class="file-upload-zone"
        class:drag-over={isDragOver}
        class:error={errors.files}
        on:drop={handleDrop}
        on:dragover={handleDragOver}
        on:dragleave={handleDragLeave}
        on:click={handleBrowseClick}
        role="button"
        tabindex="0"
        on:keydown={(e) => e.key === 'Enter' && handleBrowseClick()}
      >
        <div class="upload-content">
          <svg class="upload-icon" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <div class="upload-text">
            <div class="upload-primary">Drag & drop files here or</div>
            <button type="button" class="browse-button" on:click|stopPropagation={handleBrowseClick}>
              Click to Browse
            </button>
          </div>
          <div class="upload-hint">
            Accepted: {acceptedExtensions.join(", ")}
          </div>
        </div>
      </div>

      {#if errors.files}
        <div class="error-text">{errors.files}</div>
      {/if}

      <!-- Uploaded files list -->
      {#if uploadedFiles.length > 0}
        <div class="uploaded-files">
          <h4 class="uploaded-files-title">Uploaded Files:</h4>
          {#each uploadedFiles as file, index}
            <div class="file-item">
              <svg class="file-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                <polyline points="14,2 14,8 20,8"/>
              </svg>
              <div class="file-details">
                <div class="file-name">{file.name}</div>
                <div class="file-meta">
                  {formatFileSize(file.size)} • {file.type.toUpperCase()} file
                </div>
              </div>
              <button type="button" class="remove-file" on:click={() => onRemoveFile(index)} aria-label="Remove file">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <!-- File requirements -->
      <div class="file-requirements">
        <div class="requirements-title">Required files:</div>
        <ul class="requirements-list">
          <li>Structure file (.pdb or .psf)</li>
          <li>Parameter file(s) (.prm)</li>
          <li>Additional parameter files as needed</li>
        </ul>
      </div>
    </div>
  </div>
</div>

<style>

  .file-upload-area {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .file-upload-zone {
    border: 2px dashed var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-xl);
    text-align: center;
    cursor: pointer;
    transition: all 0.2s ease;
    background-color: var(--namd-bg-muted);
  }

  .file-upload-zone:hover,
  .file-upload-zone.drag-over {
    border-color: var(--namd-primary);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .file-upload-zone.error {
    border-color: var(--namd-error);
    background-color: var(--namd-error-bg);
  }

  .upload-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--namd-spacing-md);
  }

  .upload-icon {
    color: var(--namd-text-secondary);
  }

  .upload-text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--namd-spacing-xs);
  }

  .upload-primary {
    font-size: var(--namd-font-size-lg);
    color: var(--namd-text-primary);
  }

  .browse-button {
    background: none;
    border: none;
    color: var(--namd-primary);
    cursor: pointer;
    font-size: var(--namd-font-size-lg);
    text-decoration: underline;
    padding: 0;
  }

  .browse-button:hover {
    opacity: 0.8;
  }

  .upload-hint {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .uploaded-files {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .uploaded-files-title {
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0 0 var(--namd-spacing-sm) 0;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
  }

  .file-icon {
    color: var(--namd-text-secondary);
    flex-shrink: 0;
  }

  .file-details {
    flex: 1;
  }

  .file-name {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .file-meta {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-sm);
  }

  .remove-file {
    background: none;
    border: none;
    color: var(--namd-error);
    cursor: pointer;
    padding: var(--namd-spacing-xs);
    border-radius: var(--namd-border-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-file:hover {
    background-color: var(--namd-error-bg);
  }

  .file-requirements {
    padding: var(--namd-spacing-md);
    background-color: var(--namd-info-bg);
    border-radius: var(--namd-border-radius-sm);
    border: 1px solid var(--namd-info);
  }

  .requirements-title {
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-info-fg);
    margin-bottom: var(--namd-spacing-xs);
  }

  .requirements-list {
    margin: 0;
    padding-left: var(--namd-spacing-lg);
    color: var(--namd-info-fg);
  }

  .requirements-list li {
    margin-bottom: var(--namd-spacing-xs);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-sm);
    margin-top: var(--namd-spacing-xs);
  }

  .error-text {
    color: var(--namd-error);
    font-size: var(--namd-font-size-sm);
    margin-top: var(--namd-spacing-xs);
  }

</style>
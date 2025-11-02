<script lang="ts">
  // Debug: Component initialization started
  if (typeof window !== 'undefined' && window.sshConsole) {
    window.sshConsole.addDebug('[CreateJobPage] Script block executing - component initializing');
  }

  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { uiStore } from '../../stores/ui';
  import { jobsStore } from '../../stores/jobs';
  import { isConnected } from '../../stores/session';
  import { isLoaded, loadError, partitions, allQosOptions } from '../../stores/clusterConfig';
  import type { CreateJobParams, NAMDConfig } from '../../types/api';
  import CreateJobTabs from '../create-job/CreateJobTabs.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import { CoreClientFactory } from '../../ports/clientFactory';
  import { formatFileSize } from '../../utils/file-helpers';

  if (typeof window !== 'undefined' && window.sshConsole) {
    window.sshConsole.addDebug('[CreateJobPage] Imports complete');
  }

  // Get default partition and QoS from backend cluster config
  $: defaultPartition = $partitions.find(p => p.is_default) || $partitions[0];
  $: defaultQos = $allQosOptions.find(q => q.is_default) || $allQosOptions[0];

  // Resource configuration - use backend defaults (no hardcoded cluster info!)
  let resourceConfig = {
    cores: 24,
    memory: "16",
    wallTime: "04:00:00",
    partition: "",  // Will be set from backend default
    qos: ""  // Will be set from backend default
  };

  let unlistenUpload: (() => void) | undefined;

  onMount(async () => {
    if (window.sshConsole) {
      window.sshConsole.addDebug('[CreateJobPage] Mounting');
      window.sshConsole.addDebug(`[CreateJobPage] Cluster config state: isLoaded=${get(isLoaded)}, hasError=${!!get(loadError)}, error=${get(loadError)}`);
    }

    // Initialize with backend defaults once cluster config is loaded
    if (defaultPartition && defaultQos) {
      resourceConfig.partition = defaultPartition.id;
      resourceConfig.qos = defaultQos.id;
      if (window.sshConsole) {
        window.sshConsole.addDebug(`[CreateJobPage] Set defaults from backend: partition=${defaultPartition.id}, qos=${defaultQos.id}`);
      }
    }

    // Listen for file upload progress events
    unlistenUpload = await listen('file-upload-progress', (event) => {
      const progress = event.payload as any;
      uploadProgress.set(progress.file_name, { percentage: progress.percentage });
      uploadProgress = uploadProgress; // Trigger reactivity
    });
  });

  // Cleanup listener on component destroy
  onDestroy(() => {
    if (unlistenUpload) {
      unlistenUpload();
    }
  });

  // Uploaded files - now stores absolute paths from Tauri dialog
  let uploadedFiles: { name: string; size: number; type: string; path: string }[] = [];

  // Job name (separate from NAMD config)
  let job_name = "";

  // NAMD configuration (matches backend NAMDConfig type)
  let namdConfig: NAMDConfig = {
    outputname: "",
    temperature: 310,
    timestep: 2,
    execution_mode: 'run',
    steps: 1000000,
    pme_enabled: false,  // Default to false since cell basis vectors are not set
    npt_enabled: false,  // Default to false for vacuum simulations
    langevin_damping: 5.0,
    xst_freq: 1200,
    output_energies_freq: 1200,
    dcd_freq: 1200,
    restart_freq: 1200,
    output_pressure_freq: 1200
  };

  let errors: Record<string, string> = {};
  let isSubmitting = false;
  let selectedPresetId = '';
  let showConfirmDialog = false;
  let createError: string = '';
  let uploadProgress: Map<string, { percentage: number }> = new Map();

  // File type detection now calls backend for source of truth
  async function detectFileType(filename: string): Promise<'pdb' | 'psf' | 'prm' | 'exb' | 'other'> {
    const coreClient = CoreClientFactory.getClient();
    const result = await coreClient.detectFileType(filename);
    // Ensure the result matches our literal type
    if (result === 'pdb' || result === 'psf' || result === 'prm' || result === 'exb' || result === 'other') {
      return result;
    }
    return 'other';
  }

  async function handleSubmit() {
    // Validation is now handled in CreateJobTabs component
    if (Object.keys(errors).length > 0) {
      return;
    }

    // Show confirmation dialog
    showConfirmDialog = true;
  }

  async function handleConfirmCreate() {
    showConfirmDialog = false;
    isSubmitting = true;
    createError = '';

    try {
      // Build CreateJobParams for backend
      const params: CreateJobParams = {
        job_name: job_name,
        namd_config: namdConfig,  // Already matches backend type exactly
        slurm_config: {
          cores: resourceConfig.cores,
          memory: resourceConfig.memory,
          walltime: resourceConfig.wallTime,
          partition: resourceConfig.partition,
          qos: resourceConfig.qos,
        },
        input_files: await Promise.all(uploadedFiles.map(async file => ({
          name: file.name,
          local_path: file.path,  // Use absolute path from Tauri dialog
          remote_name: file.name,
          file_type: await detectFileType(file.name),
        }))),
      };

      // Call backend to create job (includes file upload and directory creation)
      const result = await jobsStore.createJob(params);

      if (result.success) {
        // Navigate to jobs view on success
        uiStore.setView('jobs');
      } else {
        // Show backend validation or creation errors
        createError = result.error || 'Failed to create job';
      }
    } catch (error) {
      createError = error instanceof Error ? error.message : 'An unexpected error occurred';
    } finally {
      isSubmitting = false;
    }
  }

  function handleCancelCreate() {
    showConfirmDialog = false;
  }

  function handleCancel() {
    uiStore.setView('jobs');
  }


  async function handleFileSelect() {
    // Call backend to show file dialog and get selected files via client abstraction
    try {
      const client = CoreClientFactory.getClient();
      const selected = await client.selectInputFiles();

      if (!selected || !Array.isArray(selected) || selected.length === 0) {
        return; // User cancelled or no files selected
      }

      // selected is already an array of SelectedFile objects with name, path, size, file_type
      const newFiles = selected.map((file: any) => ({
        name: file.name,
        size: file.size,
        type: file.file_type,
        path: file.path
      }));

      uploadedFiles = [...uploadedFiles, ...newFiles];
    } catch (error) {
      console.error('Failed to select files:', error);
    }
  }

  function handleFileUpload() {
    // Trigger file selection via dialog
    handleFileSelect();
  }

  function removeFile(index: number) {
    uploadedFiles = uploadedFiles.filter((_, i) => i !== index);
  }

  function handlePresetSelect(preset: any) {
    // Track selected preset
    selectedPresetId = preset.id;

    // Apply preset configuration
    resourceConfig.cores = preset.config.cores;
    resourceConfig.memory = preset.config.memory;
    resourceConfig.wallTime = preset.config.wall_time;
    resourceConfig.partition = preset.config.partition;
    resourceConfig.qos = preset.config.qos;

    // Clear any existing errors
    errors = {};
  }

  async function handlePartitionChange(partition: string) {
    resourceConfig.partition = partition;

    // Auto-adjust QOS based on partition using backend logic (no hardcoded cluster info!)
    const { suggestQos, walltimeToHours } = await import('../../stores/clusterConfig');
    const suggestedQos = await suggestQos(walltimeToHours(resourceConfig.wallTime), partition);
    resourceConfig.qos = suggestedQos;

    if (window.sshConsole) {
      window.sshConsole.addDebug(`[CreateJobPage] Partition changed to ${partition}, backend suggested QoS: ${suggestedQos}`);
    }

    // Clear partition-related errors
    if (errors.partition) {
      delete errors.partition;
      errors = { ...errors };
    }
  }

  function handleQosChange(qos: string) {
    resourceConfig.qos = qos;

    // Clear QOS-related errors
    if (errors.qos) {
      delete errors.qos;
      errors = { ...errors };
    }
  }
</script>

<div class="create-job-page">
  <!-- Back Button -->
  <button class="back-button" on:click={handleCancel}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="m12 19-7-7 7-7"/>
      <path d="M19 12H5"/>
    </svg>
    Back to Jobs
  </button>

  <h1 class="page-title">Create New Job</h1>

  {#if createError}
    <div class="error-banner">
      <strong>Error creating job:</strong> {createError}
    </div>
  {/if}

  <!-- Main Content -->
  <CreateJobTabs
    bind:job_name
    bind:resourceConfig
    bind:uploadedFiles
    bind:namdConfig
    bind:errors
    bind:selectedPresetId
    bind:isSubmitting
    uploadProgress={uploadProgress}
    onPresetSelect={handlePresetSelect}
    onPartitionChange={handlePartitionChange}
    onQosChange={handleQosChange}
    onFileUpload={handleFileUpload}
    onRemoveFile={removeFile}
    onSubmit={handleSubmit}
    onCancel={handleCancel}
    formatFileSize={formatFileSize}
  />
</div>

<!-- Confirmation Dialog -->
<ConfirmDialog
  isOpen={showConfirmDialog}
  title="Create Job?"
  message="Ready to create job? This will:
  • Upload all selected files to the server
  • Create job directories and metadata
  • Save the job in your local database

Note: The job will NOT be submitted to SLURM yet. You'll need to submit it manually from the Jobs page."
  confirmText="Create Job"
  cancelText="Cancel"
  confirmStyle="primary"
  onConfirm={handleConfirmCreate}
  onCancel={handleCancelCreate}
/>

<style>
  .create-job-page {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    background-color: var(--namd-bg-secondary);
    padding: var(--namd-spacing-lg);
    gap: var(--namd-spacing-lg);
    max-width: 1200px;
    margin: 0 auto;
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

  .page-title {
    font-size: var(--namd-font-size-2xl);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
    margin: 0;
  }

  .error-banner {
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 12px;
    border-radius: 6px;
    font-size: 14px;
  }
</style>
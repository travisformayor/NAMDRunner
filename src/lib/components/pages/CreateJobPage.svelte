<script lang="ts">
  import { uiStore } from '../../stores/ui';
  import { jobsStore } from '../../stores/jobs';
  import { isConnected } from '../../stores/session';
  import type { Job, NAMDConfig, SlurmConfig } from '../../types/api';
  import CreateJobTabs from '../create-job/CreateJobTabs.svelte';

  // Resource configuration matching the mockup
  let resourceConfig = {
    cores: 128,
    memory: "512",
    wallTime: "04:00:00",
    partition: "amilan",
    qos: "normal"
  };

  // Uploaded files
  let uploadedFiles: { name: string; size: number; type: string; file: File }[] = [];

  // NAMD configuration
  let namdConfig = {
    jobName: "",
    simulationSteps: 1000000,
    temperature: 310,
    timestep: 2,
    outputName: "",
    dcdFreq: 5000,
    restartFreq: 10000
  };

  let errors: Record<string, string> = {};
  let isSubmitting = false;
  let selectedPresetId = '';

  async function handleSubmit() {
    // Validation is now handled in CreateJobTabs component
    if (Object.keys(errors).length > 0) {
      return;
    }

    isSubmitting = true;

    try {
      // Simulate job creation
      await new Promise(resolve => setTimeout(resolve, 2000));

      console.log("Creating job with:", {
        resourceConfig,
        uploadedFiles: uploadedFiles.map(f => ({ name: f.name, size: f.size })),
        namdConfig
      });

      // Create mock job
      const newJob: Job = {
        jobId: `job_${Date.now()}`,
        jobName: namdConfig.jobName,
        status: 'CREATED',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        namdConfig: { ...namdConfig } as NAMDConfig,
        slurmConfig: {
          partition: resourceConfig.partition,
          nodes: 1,
          ntasks: resourceConfig.cores,
          time: resourceConfig.wallTime,
          mem: `${resourceConfig.memory}GB`,
          account: 'ucb-general'
        } as SlurmConfig,
        inputFiles: uploadedFiles.map(file => ({
          fileName: file.name,
          size: file.size,
          uploadedAt: new Date().toISOString()
        })),
        runtime: '--',
        wallTimeRemaining: '--'
      };

      jobsStore.addJob(newJob);
      uiStore.setView('jobs');
    } catch (error) {
      console.error("Error creating job:", error);
    } finally {
      isSubmitting = false;
    }
  }

  function handleCancel() {
    uiStore.setView('jobs');
  }


  function handleFileSelect(files: FileList | null) {
    if (!files) return;

    const newFiles: typeof uploadedFiles = [];
    const acceptedExtensions = [".pdb", ".psf", ".prm"];

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const extension = "." + file.name.split('.').pop()?.toLowerCase();

      if (acceptedExtensions.includes(extension)) {
        newFiles.push({
          name: file.name,
          size: file.size,
          type: extension,
          file: file
        });
      }
    }

    uploadedFiles = [...uploadedFiles, ...newFiles];
  }

  function handleFileUpload(event: Event) {
    const target = event.target as HTMLInputElement;
    handleFileSelect(target.files);
  }

  function removeFile(index: number) {
    uploadedFiles = uploadedFiles.filter((_, i) => i !== index);
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function handlePresetSelect(preset: any) {
    // Track selected preset
    selectedPresetId = preset.id;

    // Apply preset configuration
    resourceConfig.cores = preset.config.cores;
    resourceConfig.memory = preset.config.memory;
    resourceConfig.wallTime = preset.config.wallTime;
    resourceConfig.partition = preset.config.partition;
    resourceConfig.qos = preset.config.qos;

    // Clear any existing errors
    errors = {};
  }

  function handlePartitionChange(partition: string) {
    resourceConfig.partition = partition;

    // Auto-adjust QOS based on partition
    if (partition === 'amem') {
      resourceConfig.qos = 'mem';
    } else if (partition === 'atesting') {
      resourceConfig.qos = 'testing';
    } else {
      resourceConfig.qos = 'normal';
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

  <!-- Main Content -->
  <CreateJobTabs
    bind:resourceConfig
    bind:uploadedFiles
    bind:namdConfig
    bind:errors
    bind:selectedPresetId
    bind:isSubmitting
    onPresetSelect={handlePresetSelect}
    onPartitionChange={handlePartitionChange}
    onQosChange={handleQosChange}
    onFileUpload={handleFileUpload}
    onFileSelect={handleFileSelect}
    onRemoveFile={removeFile}
    onSubmit={handleSubmit}
    onCancel={handleCancel}
    formatFileSize={formatFileSize}
  />
</div>

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


</style>
<script lang="ts">
  import ResourcesTab from './ResourcesTab.svelte';
  import ConfigurationTab from './ConfigurationTab.svelte';
  import FilesTab from './FilesTab.svelte';
  import ReviewTab from './ReviewTab.svelte';
  import { getPartitionLimits } from '../../data/cluster-config';

  export let resourceConfig: {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
  };

  export let uploadedFiles: { name: string; size: number; type: string; file: File }[];

  export let namdConfig: {
    jobName: string;
    simulationSteps: number;
    temperature: number;
    timestep: number;
    outputName: string;
    dcdFreq: number;
    restartFreq: number;
  };

  export let errors: Record<string, string>;
  export let selectedPresetId: string;
  export let onPresetSelect: (preset: any) => void;
  export let onPartitionChange: (partition: string) => void;
  export let onQosChange: (qos: string) => void;
  export let onFileUpload: (event: Event) => void;
  export let onFileSelect: (files: FileList | null) => void;
  export let onRemoveFile: (index: number) => void;
  export let formatFileSize: (bytes: number) => string;
  export let onSubmit: () => void;
  export let onCancel: () => void;
  export let isSubmitting: boolean = false;

  type TabId = 'resources' | 'configuration' | 'files' | 'review';

  const tabs = [
    { id: 'resources', label: 'Resources' },
    { id: 'configuration', label: 'Configuration' },
    { id: 'files', label: 'Files' },
    { id: 'review', label: 'Review' }
  ];

  let activeTab: TabId = 'resources';

  // Validation function
  function validateConfiguration() {
    const newErrors: Record<string, string> = {};

    // Resource validation
    if (!resourceConfig.cores || resourceConfig.cores <= 0) {
      newErrors.cores = "Cores must be a positive number";
    }

    // Check partition limits
    const partitionLimits = getPartitionLimits(resourceConfig.partition);
    if (partitionLimits && resourceConfig.cores > partitionLimits.maxCores) {
      newErrors.cores = `Cores exceed partition limit of ${partitionLimits.maxCores}`;
    }

    if (!resourceConfig.memory || parseFloat(resourceConfig.memory) <= 0) {
      newErrors.memory = "Memory must be a positive number";
    }

    // Check memory limits
    if (partitionLimits) {
      const memoryPerCore = parseFloat(resourceConfig.memory) / resourceConfig.cores;
      if (memoryPerCore > partitionLimits.maxMemoryPerCore) {
        newErrors.memory = `Memory per core (${memoryPerCore.toFixed(1)}GB) exceeds partition limit of ${partitionLimits.maxMemoryPerCore}GB per core`;
      }
    }

    if (!resourceConfig.wallTime || !/^\d{2}:\d{2}:\d{2}$/.test(resourceConfig.wallTime)) {
      newErrors.wallTime = "Wall time must be in HH:MM:SS format";
    }

    // File validation
    const requiredFiles = [".pdb", ".psf", ".prm"];
    const fileExtensions = uploadedFiles.map(f => f.name.split('.').pop()?.toLowerCase());

    for (const ext of requiredFiles) {
      if (!fileExtensions.some(fe => fe === ext.substring(1))) {
        newErrors.files = `Missing required file type: ${ext}`;
        break;
      }
    }

    // NAMD validation
    if (!namdConfig.jobName.trim()) {
      newErrors.jobName = "Job name is required";
    }
    if (!namdConfig.simulationSteps || namdConfig.simulationSteps <= 0) {
      newErrors.simulationSteps = "Simulation steps must be a positive number";
    }
    if (!namdConfig.temperature || namdConfig.temperature <= 0) {
      newErrors.temperature = "Temperature must be a positive number";
    }
    if (!namdConfig.timestep || namdConfig.timestep <= 0) {
      newErrors.timestep = "Timestep must be a positive number";
    }
    if (!namdConfig.outputName.trim()) {
      newErrors.outputName = "Output name is required";
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  // Run validation when switching to review tab
  $: if (activeTab === 'review') {
    validateConfiguration();
  }
</script>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-4">
      {#each tabs as tab}
        <button
          class="namd-tab-button"
          class:active={activeTab === tab.id}
          on:click={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </div>

  <div class="namd-tab-content">
    {#if activeTab === 'resources'}
      <ResourcesTab
        {resourceConfig}
        {errors}
        {selectedPresetId}
        {onPresetSelect}
        {onPartitionChange}
        {onQosChange}
      />
    {:else if activeTab === 'configuration'}
      <ConfigurationTab
        {namdConfig}
        {errors}
      />
    {:else if activeTab === 'files'}
      <FilesTab
        {uploadedFiles}
        {errors}
        {onFileUpload}
        {onFileSelect}
        {onRemoveFile}
        {formatFileSize}
      />
    {:else if activeTab === 'review'}
      <ReviewTab
        {resourceConfig}
        {namdConfig}
        {uploadedFiles}
        {errors}
        {formatFileSize}
        {onSubmit}
        {onCancel}
        {isSubmitting}
      />
    {/if}
  </div>
</div>


<script lang="ts">
  import type { NAMDConfig } from '../../types/api';
  import ResourcesTab from './ResourcesTab.svelte';
  import ConfigurationTab from './ConfigurationTab.svelte';
  import FilesTab from './FilesTab.svelte';
  import ReviewTab from './ReviewTab.svelte';

  export let job_name: string;

  export let resourceConfig: {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
  };

  export let uploadedFiles: { name: string; size: number; type: string; path: string }[];

  export let namdConfig: NAMDConfig;

  export let errors: Record<string, string>;
  export let selectedPresetId: string;
  export let onPresetSelect: (preset: any) => void;
  export let onPartitionChange: (partition: string) => void;
  export let onQosChange: (qos: string) => void;
  export let onFileUpload: () => void;
  export let onRemoveFile: (index: number) => void;
  export let formatFileSize: (bytes: number) => string;
  export let onSubmit: () => void;
  export let onCancel: () => void;
  export let isSubmitting: boolean = false;
  export let uploadProgress: Map<string, { percentage: number }> = new Map();

  type TabId = 'resources' | 'configuration' | 'files' | 'review';

  const tabs = [
    { id: 'resources', label: 'Resources' },
    { id: 'configuration', label: 'Configuration' },
    { id: 'files', label: 'Files' },
    { id: 'review', label: 'Review' }
  ];

  let activeTab: TabId = 'resources';

  // UI-only validation (format checking only - business logic in backend)
  function validateConfiguration() {
    const newErrors: Record<string, string> = {};

    // Basic format validation only - backend will do comprehensive validation
    if (!resourceConfig.cores || resourceConfig.cores <= 0) {
      newErrors.cores = "Cores is required";
    }

    if (!resourceConfig.memory || !resourceConfig.memory.trim()) {
      newErrors.memory = "Memory is required";
    }

    if (!resourceConfig.wallTime || !/^\d{2}:\d{2}:\d{2}$/.test(resourceConfig.wallTime)) {
      newErrors.wallTime = "Wall time must be in HH:MM:SS format";
    }

    if (!job_name.trim()) {
      newErrors.job_name = "Job name is required";
    }

    if (!namdConfig.outputname.trim()) {
      newErrors.outputname = "Output name is required";
    }

    // Note: Backend will validate:
    // - Partition limits and resource constraints
    // - File types and requirements
    // - NAMD parameter ranges
    // - QOS compatibility

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  // Run validation when switching to review tab
  $: if (activeTab === 'review') {
    validateConfiguration();
  }
</script>

<style>
  /* All styling is handled by parent namd-card and namd-tabs classes in global CSS */
</style>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-4">
      {#each tabs as tab}
        <button
          class="namd-tab-button"
          class:active={activeTab === tab.id}
          on:click={() => {
            activeTab = tab.id as TabId;
            if (window.sshConsole) {
              window.sshConsole.addDebug(`[USER] Switched to tab: ${tab.label}`);
            }
          }}
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
        bind:job_name
        bind:namdConfig
        {errors}
      />
    {:else if activeTab === 'files'}
      <FilesTab
        {uploadedFiles}
        {errors}
        {onFileUpload}
        {onRemoveFile}
        {formatFileSize}
      />
    {:else if activeTab === 'review'}
      <ReviewTab
        {job_name}
        {resourceConfig}
        {namdConfig}
        {uploadedFiles}
        {errors}
        {formatFileSize}
        {onSubmit}
        {onCancel}
        {isSubmitting}
        {uploadProgress}
      />
    {/if}
  </div>
</div>


<script lang="ts">
  import { logger } from '$lib/utils/logger';
  import { invoke } from '@tauri-apps/api/core';
  import type { JobValidationResult } from '$lib/types/api';
  import ResourcesTab from './ResourcesTab.svelte';
  import ConfigureTab from './ConfigureTab.svelte';
  import ReviewTab from './ReviewTab.svelte';

  // Props from parent
  export let jobName: string;
  export let templateId: string;
  export let templateValues: Record<string, any>;
  export let resourceConfig: {
    cores: number;
    memory: string;
    walltime: string;
    partition: string;
    qos: string;
  };
  export let errors: Record<string, string>;
  export let onSubmit: () => void;
  export let onCancel: () => void;
  export let isSubmitting: boolean = false;
  export let uploadProgress: Map<string, { percentage: number }>;
  export let uploadFileList: string[] = [];

  type TabId = 'resources' | 'configure' | 'review';

  const tabs = [
    { id: 'resources', label: 'Resources' },
    { id: 'configure', label: 'Configure' },
    { id: 'review', label: 'Review' }
  ];

  let activeTab: TabId = 'resources';
  let validationTimer: number;

  // Debounced backend validation - triggers on any input change
  $: if (jobName || templateId || templateValues || resourceConfig) {
    triggerValidation();
  }

  function triggerValidation() {
    clearTimeout(validationTimer);
    validationTimer = window.setTimeout(async () => {
      await runBackendValidation();
    }, 500);
  }

  async function runBackendValidation() {
    try {
      const result = await invoke<JobValidationResult>('validate_job_config', {
        params: {
          job_name: jobName,
          template_id: templateId,
          template_values: templateValues,
          cores: resourceConfig.cores,
          memory: resourceConfig.memory,
          walltime: resourceConfig.walltime,
          partition: resourceConfig.partition || null,
          qos: resourceConfig.qos || null
        }
      });

      if (result.is_valid) {
        errors = {};
      } else {
        // Parse issues to extract field-specific errors
        const newErrors: Record<string, string> = {};
        for (const error of result.issues) {
          if (error.toLowerCase().includes('job name')) {
            newErrors.job_name = error;
          } else if (error.toLowerCase().includes('template')) {
            newErrors.template = error;
          } else if (error.toLowerCase().includes('cores')) {
            newErrors.cores = error;
          } else if (error.toLowerCase().includes('memory')) {
            newErrors.memory = error;
          } else if (error.toLowerCase().includes('wall time')) {
            newErrors.walltime = error;
          } else {
            // Template variable errors - extract field name from "FieldLabel: error" format
            const colonIndex = error.indexOf(':');
            if (colonIndex > 0) {
              const fieldLabel = error.substring(0, colonIndex).trim();
              // Field errors will be handled by DynamicJobForm's validation display
              if (!newErrors.general) {
                newErrors.general = error;
              }
            } else {
              newErrors.general = error;
            }
          }
        }
        errors = newErrors;
      }
    } catch (error) {
      logger.error('[CreateJobTabs]', 'Validation error', error);
    }
  }
</script>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-3">
      {#each tabs as tab}
        <button
          class="namd-tab-button"
          class:active={activeTab === tab.id}
          on:click={() => activeTab = tab.id as TabId}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </div>

  <div class="namd-tab-content">
    {#if activeTab === 'resources'}
      <ResourcesTab bind:resourceConfig {errors} />
    {:else if activeTab === 'configure'}
      <ConfigureTab bind:jobName bind:templateId bind:templateValues {errors} />
    {:else if activeTab === 'review'}
      <ReviewTab
        {jobName}
        {templateId}
        {templateValues}
        {resourceConfig}
        {errors}
        {uploadProgress}
        {uploadFileList}
        {onSubmit}
        {onCancel}
        {isSubmitting}
      />
    {/if}
  </div>
</div>

<style>
  /* All styling handled by global namd-tabs CSS */
</style>

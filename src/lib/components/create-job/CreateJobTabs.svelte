<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import type { ValidationResult } from '$lib/types/api';
  import type { Template } from '$lib/types/template';
  import ResourcesTab from './ResourcesTab.svelte';
  import ConfigureTab from './ConfigureTab.svelte';
  import ReviewTab from './ReviewTab.svelte';
  import ValidationDisplay from '../ui/ValidationDisplay.svelte';

  // Props from parent
  export let jobName: string;
  export let templateId: string;
  export let template: Template | null = null;
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

  type TabId = 'resources' | 'configure' | 'review';

  const tabs = [
    { id: 'resources', label: 'Resources' },
    { id: 'configure', label: 'Configure' },
    { id: 'review', label: 'Review' }
  ];

  let activeTab: TabId = 'resources';
  let validationTimer: number;
  let validationResult: ValidationResult = {
    is_valid: true,
    issues: [],
    warnings: [],
    suggestions: [],
  };

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
    const result = await invoke<ValidationResult>('validate_job_config', {
      params: {
        job_name: jobName,
        template_id: templateId,
        template_values: templateValues,
        cores: resourceConfig.cores,
        memory: resourceConfig.memory,
        walltime: resourceConfig.walltime,
        partition: resourceConfig.partition,
        qos: resourceConfig.qos,
      },
    });

    validationResult = result;

    if (result.is_valid) {
      errors = {};
    } else {
      // Use structured field_errors directly from backend
      errors = result.field_errors || {};
    }
  }

  onDestroy(() => {
    if (validationTimer) {
      clearTimeout(validationTimer);
    }
  });
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
      <ConfigureTab bind:jobName bind:templateId bind:templateValues bind:template {errors} />
    {:else if activeTab === 'review'}
      <ReviewTab
        {jobName}
        {templateId}
        {template}
        {templateValues}
        {resourceConfig}
        {errors}
        {uploadProgress}
        {onSubmit}
        {onCancel}
        {isSubmitting}
      />
    {/if}
  </div>

  <!-- General validation feedback -->
  <ValidationDisplay validation={validationResult} />
</div>

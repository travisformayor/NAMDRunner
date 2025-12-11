<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { uiStore } from '../../stores/ui';
  import { jobsStore } from '../../stores/jobs';
  import { isConnected } from '../../stores/session';
  import { partitions, allQosOptions } from '../../stores/clusterConfig';
  import CreateJobTabs from '../create-job/CreateJobTabs.svelte';
  import type { CreateJobParams } from '../../types/api';
  import type { Template } from '$lib/types/template';

  // Job configuration
  let jobName = '';
  let templateId = '';
  let template: Template | null = null;
  let templateValues: Record<string, any> = {};

  // Resource configuration
  let resourceConfig = {
    cores: 24,
    memory: '16GB',
    walltime: '04:00:00',
    partition: '',
    qos: ''
  };

  // UI state
  let errors: Record<string, string> = {};
  let isSubmitting = false;
  let uploadProgress: Map<string, { percentage: number }> = new Map();
  let unlistenUpload: (() => void) | undefined;

  // Get defaults from cluster config
  $: defaultPartition = $partitions.find(p => p.is_default) || $partitions[0];
  $: defaultQos = $allQosOptions.find(q => q.is_default) || $allQosOptions[0];

  onMount(async () => {
    // Set defaults from backend
    if (defaultPartition && defaultQos) {
      resourceConfig.partition = defaultPartition.name;
      resourceConfig.qos = defaultQos.name;
    }

    // Listen for file upload progress
    unlistenUpload = await listen('file-upload-progress', (event) => {
      const progress = event.payload as any;
      uploadProgress = new Map(
        uploadProgress.set(progress.file_name, { percentage: progress.percentage })
      );
    });
  });

  onDestroy(() => {
    if (unlistenUpload) {
      unlistenUpload();
    }
  });

  function handleCancel() {
    uiStore.setView('jobs');
  }

  async function handleSubmit() {
    isSubmitting = true;

    const params: CreateJobParams = {
      job_name: jobName,
      template_id: templateId,
      template_values: templateValues,
      slurm_config: {
        cores: resourceConfig.cores,
        memory: resourceConfig.memory,
        walltime: resourceConfig.walltime,
        ...(resourceConfig.partition && { partition: resourceConfig.partition }),
        ...(resourceConfig.qos && { qos: resourceConfig.qos }),
      },
    };

    const result = await jobsStore.createJob(params);

    if (result.success) {
      uiStore.setView('jobs');
    }
    // Errors handled by jobsStore and displayed in UI

    isSubmitting = false;
  }
</script>

<div class="create-job-page namd-page">
  {#if !$isConnected}
    <div class="warning-banner">
      <strong>Not Connected:</strong> Please connect to the cluster before creating jobs.
    </div>
  {:else}
    <CreateJobTabs
      bind:jobName
      bind:templateId
      bind:template
      bind:templateValues
      bind:resourceConfig
      bind:errors
      {uploadProgress}
      onSubmit={handleSubmit}
      onCancel={handleCancel}
      {isSubmitting}
    />
  {/if}
</div>

<style>
  .create-job-page {
    padding: var(--namd-spacing-xl);
    max-width: var(--namd-max-width-content);
    margin: 0 auto;
  }

  .warning-banner {
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius-sm);
    margin-bottom: var(--namd-spacing-lg);
    background: var(--namd-warning-bg);
    border: 1px solid var(--namd-warning-border);
    color: var(--namd-warning-fg);
  }
</style>

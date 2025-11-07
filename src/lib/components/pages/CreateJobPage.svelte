<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { logger } from '../../utils/logger';
  import { uiStore } from '../../stores/ui';
  import { jobsStore } from '../../stores/jobs';
  import { isConnected } from '../../stores/session';
  import { partitions, allQosOptions } from '../../stores/clusterConfig';
  import CreateJobTabs from '../create-job/CreateJobTabs.svelte';
  import type { CreateJobParams } from '../../types/api';

  // Job configuration
  let jobName = '';
  let templateId = '';
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
      resourceConfig.partition = defaultPartition.id;
      resourceConfig.qos = defaultQos.id;
    }

    // Listen for file upload progress
    unlistenUpload = await listen('file-upload-progress', (event) => {
      const progress = event.payload as any;
      uploadProgress.set(progress.file_name, { percentage: progress.percentage });
      uploadProgress = uploadProgress; // Trigger reactivity
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

    try {
      const params: CreateJobParams = {
        job_name: jobName,
        template_id: templateId,
        template_values: templateValues,
        slurm_config: {
          cores: resourceConfig.cores,
          memory: resourceConfig.memory,
          walltime: resourceConfig.walltime,
          partition: resourceConfig.partition || undefined,
          qos: resourceConfig.qos || undefined
        }
      };

      const result = await jobsStore.createJob(params);

      if (result.success) {
        uiStore.setView('jobs');
      }
      // Errors handled by jobsStore and displayed in UI
    } catch (error) {
      logger.error('CreateJob', 'Job creation error', error);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<div class="create-job-page">
  {#if !$isConnected}
    <div class="warning-banner">
      <strong>Not Connected:</strong> Please connect to the cluster before creating jobs.
    </div>
  {:else}
    <CreateJobTabs
      bind:jobName
      bind:templateId
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
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .warning-banner {
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1.5rem;
    background: var(--warning-light, #fff3cd);
    border: 1px solid var(--warning, #ffc107);
    color: var(--warning-dark, #856404);
  }
</style>

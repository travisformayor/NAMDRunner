<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getName, getVersion } from '@tauri-apps/api/app';
  import { settingsStore, databaseInfo, settingsLoading, settingsError } from '$lib/stores/settings';
  import { clusterConfig, partitions, allQosOptions, jobPresets, billingRates, saveClusterConfig, resetClusterConfig } from '$lib/stores/clusterConfig';
  import type { ApiResult, DatabaseOperationData, ClusterCapabilities, PartitionSpec, QosSpec, JobPreset, BillingRates as BillingRatesType } from '$lib/types/api';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import EditDialog from '../ui/EditDialog.svelte';
  import { jobsStore } from '$lib/stores/jobs';
  import { templateStore } from '$lib/stores/templateStore';

  // App information state
  let appName = '';
  let appVersion = '';

  // Database dialog states
  let showRestoreWarning = false;
  let showResetWarning = false;

  // Cluster config dialog states
  let showResetClusterWarning = false;
  let showPartitionDialog = false;
  let showQosDialog = false;
  let showPresetDialog = false;
  let showBillingDialog = false;
  let showHostDialog = false;
  let showDeletePartitionDialog = false;
  let showDeleteQosDialog = false;
  let showDeletePresetDialog = false;

  // Edit states
  let editingPartition: PartitionSpec | null = null;
  let editingQos: QosSpec | null = null;
  let editingPreset: JobPreset | null = null;
  let editingBilling: BillingRatesType | null = null;
  let editingHost: string = '';

  // Delete targets
  let deleteTargetPartition: PartitionSpec | null = null;
  let deleteTargetQos: QosSpec | null = null;
  let deleteTargetPreset: JobPreset | null = null;

  // Form errors
  let partitionErrors: Record<string, string> = {};
  let qosErrors: Record<string, string> = {};
  let presetErrors: Record<string, string> = {};

  // Alert dialog state
  let showAlert = false;
  let alertTitle = '';
  let alertMessage = '';
  let alertVariant: 'success' | 'error' | 'warning' | 'info' = 'info';

  function showAlertDialog(title: string, message: string, variant: 'success' | 'error' | 'warning' | 'info' = 'info') {
    alertTitle = title;
    alertMessage = message;
    alertVariant = variant;
    showAlert = true;
  }

  onMount(async () => {
    await settingsStore.loadDatabaseInfo();
    appName = await getName();
    appVersion = await getVersion();
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`;
  }

  // Database handlers
  async function handleBackup() {
    const result = await invoke<ApiResult<DatabaseOperationData>>('backup_database');
    if (!result.success && result.error !== 'Backup cancelled') {
      showAlertDialog('Backup Failed', `Backup failed: ${result.error}`, 'error');
    }
  }

  async function handleRestoreConfirm() {
    showRestoreWarning = false;
    const result = await settingsStore.restoreDatabase();
    if (result.success) {
      await jobsStore.loadFromDatabase();
      await templateStore.loadTemplates();
      showAlertDialog('Database Restored', 'Database restored successfully. All data has been reloaded.', 'success');
    } else if (result.error !== 'Restore cancelled') {
      showAlertDialog('Restore Failed', `Restore failed: ${result.error}`, 'error');
    }
  }

  async function handleResetConfirm() {
    showResetWarning = false;
    const result = await settingsStore.resetDatabase();
    if (result.success) {
      await jobsStore.loadFromDatabase();
      await templateStore.loadTemplates();
      await clusterConfig.refresh();
      showAlertDialog('Database Reset', 'Database reset successfully. All data has been cleared.', 'success');
    } else {
      showAlertDialog('Reset Failed', `Reset failed: ${result.error}`, 'error');
    }
  }

  // Partition handlers
  function handleAddPartition() {
    editingPartition = {
      name: '',
      title: '',
      description: '',
      max_cores: 0,
      max_memory_per_core_gb: 0,
      gpu_type: null,
      gpu_count: null,
      is_default: false
    };
    partitionErrors = {};
    showPartitionDialog = true;
  }

  function handleEditPartition(partition: PartitionSpec) {
    editingPartition = { ...partition };
    partitionErrors = {};
    showPartitionDialog = true;
  }

  async function handleSavePartition() {
    if (!editingPartition || !$clusterConfig) return;

    partitionErrors = {};
    if (!editingPartition.name.trim()) partitionErrors.name = 'Name is required';
    if (!editingPartition.title.trim()) partitionErrors.title = 'Title is required';
    if (editingPartition.max_cores <= 0) partitionErrors.max_cores = 'Max cores must be > 0';
    if (editingPartition.max_memory_per_core_gb <= 0) partitionErrors.max_memory_per_core_gb = 'Memory must be > 0';

    if (Object.keys(partitionErrors).length > 0) return;

    const updatedConfig = { ...$clusterConfig };

    // Enforce exclusive default: if this partition is being set as default, uncheck all others
    if (editingPartition.is_default) {
      updatedConfig.partitions = updatedConfig.partitions.map(p => ({
        ...p,
        is_default: false
      }));
    }

    const existingIndex = updatedConfig.partitions.findIndex(p => p.name === editingPartition!.name);

    if (existingIndex >= 0) {
      updatedConfig.partitions[existingIndex] = editingPartition;
    } else {
      updatedConfig.partitions.push(editingPartition);
    }

    const success = await saveClusterConfig(updatedConfig);
    if (success) {
      showPartitionDialog = false;
      editingPartition = null;
    }
  }

  function confirmDeletePartition(partition: PartitionSpec) {
    deleteTargetPartition = partition;
    showDeletePartitionDialog = true;
  }

  async function handleDeletePartitionConfirm() {
    if (!deleteTargetPartition || !$clusterConfig) return;
    showDeletePartitionDialog = false;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.partitions = updatedConfig.partitions.filter(p => p.name !== deleteTargetPartition!.name);
    updatedConfig.job_presets = updatedConfig.job_presets.filter(preset => preset.partition !== deleteTargetPartition!.name);

    await saveClusterConfig(updatedConfig);
    deleteTargetPartition = null;
  }

  // QoS handlers
  function handleAddQos() {
    editingQos = {
      name: '',
      title: '',
      description: '',
      max_walltime_hours: 24,
      valid_partitions: [],
      min_memory_gb: null,
      is_default: false
    };
    qosErrors = {};
    showQosDialog = true;
  }

  function handleEditQos(qos: QosSpec) {
    editingQos = { ...qos, valid_partitions: [...qos.valid_partitions] };
    qosErrors = {};
    showQosDialog = true;
  }

  async function handleSaveQos() {
    if (!editingQos || !$clusterConfig) return;

    qosErrors = {};
    if (!editingQos.name.trim()) qosErrors.name = 'Name is required';
    if (!editingQos.title.trim()) qosErrors.title = 'Title is required';
    if (editingQos.max_walltime_hours <= 0) qosErrors.max_walltime_hours = 'Max walltime must be > 0';
    if (editingQos.valid_partitions.length === 0) qosErrors.valid_partitions = 'Select at least one partition';

    if (Object.keys(qosErrors).length > 0) return;

    const updatedConfig = { ...$clusterConfig };

    // Enforce exclusive default: if this QoS is being set as default, uncheck all others
    if (editingQos.is_default) {
      updatedConfig.qos_options = updatedConfig.qos_options.map(q => ({
        ...q,
        is_default: false
      }));
    }

    const existingIndex = updatedConfig.qos_options.findIndex(q => q.name === editingQos!.name);

    if (existingIndex >= 0) {
      updatedConfig.qos_options[existingIndex] = editingQos;
    } else {
      updatedConfig.qos_options.push(editingQos);
    }

    const success = await saveClusterConfig(updatedConfig);
    if (success) {
      showQosDialog = false;
      editingQos = null;
    }
  }

  function confirmDeleteQos(qos: QosSpec) {
    deleteTargetQos = qos;
    showDeleteQosDialog = true;
  }

  async function handleDeleteQosConfirm() {
    if (!deleteTargetQos || !$clusterConfig) return;
    showDeleteQosDialog = false;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.qos_options = updatedConfig.qos_options.filter(q => q.name !== deleteTargetQos!.name);
    updatedConfig.job_presets = updatedConfig.job_presets.filter(preset => preset.qos !== deleteTargetQos!.name);

    await saveClusterConfig(updatedConfig);
    deleteTargetQos = null;
  }


  // Preset handlers (Add only, no Edit)
  function handleAddPreset() {
    editingPreset = {
      name: '',
      description: '',
      cores: 24,
      memory: '16',
      walltime: '04:00:00',
      partition: $partitions[0]?.name || '',
      qos: $allQosOptions[0]?.name || ''
    };
    presetErrors = {};
    showPresetDialog = true;
  }

  async function handleSavePreset() {
    if (!editingPreset || !$clusterConfig) return;

    presetErrors = {};
    if (!editingPreset.name.trim()) presetErrors.name = 'Name is required';
    if (editingPreset.cores <= 0) presetErrors.cores = 'Cores must be > 0';
    if (!editingPreset.memory.trim()) presetErrors.memory = 'Memory is required';
    if (!editingPreset.walltime.trim()) presetErrors.walltime = 'Walltime is required';

    if (Object.keys(presetErrors).length > 0) return;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.job_presets.push(editingPreset);

    const success = await saveClusterConfig(updatedConfig);
    if (success) {
      showPresetDialog = false;
      editingPreset = null;
    }
  }

  function confirmDeletePreset(preset: JobPreset) {
    deleteTargetPreset = preset;
    showDeletePresetDialog = true;
  }

  async function handleDeletePresetConfirm() {
    if (!deleteTargetPreset || !$clusterConfig) return;
    showDeletePresetDialog = false;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.job_presets = updatedConfig.job_presets.filter(p => p.name !== deleteTargetPreset!.name);

    await saveClusterConfig(updatedConfig);
    deleteTargetPreset = null;
  }

  // Billing handlers
  function handleEditBilling() {
    if ($billingRates) {
      editingBilling = { ...$billingRates };
      showBillingDialog = true;
    }
  }

  async function handleSaveBilling() {
    if (!editingBilling || !$clusterConfig) return;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.billing_rates = editingBilling;

    const success = await saveClusterConfig(updatedConfig);
    if (success) {
      showBillingDialog = false;
      editingBilling = null;
    }
  }

  // Default Host handlers
  function handleEditHost() {
    if ($clusterConfig) {
      editingHost = $clusterConfig.default_host;
      showHostDialog = true;
    }
  }

  async function handleSaveHost() {
    if (!editingHost.trim() || !$clusterConfig) return;

    const updatedConfig = { ...$clusterConfig };
    updatedConfig.default_host = editingHost.trim();

    const success = await saveClusterConfig(updatedConfig);
    if (success) {
      showHostDialog = false;
      editingHost = '';
    }
  }

  // Reset cluster config
  async function handleResetClusterConfirm() {
    showResetClusterWarning = false;
    const success = await resetClusterConfig();
    if (success) {
      showAlertDialog('Reset Complete', 'Cluster configuration has been reset to defaults.', 'success');
    } else {
      showAlertDialog('Reset Failed', 'Failed to reset cluster configuration.', 'error');
    }
  }
</script>

<div class="settings-page namd-page">
  <!-- About Section (Top) -->
  <div class="settings-section">
    <h2>About</h2>
    <div class="db-info">
      <div class="info-row">
        <span class="label">Name:</span>
        <span class="value">{appName}</span>
      </div>
      <div class="info-row">
        <span class="label">Version:</span>
        <span class="value">{appVersion}</span>
      </div>
    </div>
  </div>

  <!-- Cluster Configuration Section (Middle) -->
  <div class="settings-section">
    <h2>Cluster Configuration</h2>

    <!-- Default Host Subsection (First) -->
    <details>
      <summary class="config-subsection-summary">Default Host</summary>
      <div class="subsection-content">
        <div class="db-info">
          <div class="info-row">
            <span class="label">Hostname:</span>
            <span class="value">{$clusterConfig?.default_host || 'Not set'}</span>
          </div>
        </div>
        <button class="namd-button namd-button--secondary" on:click={handleEditHost}>Edit Default Host</button>
      </div>
    </details>

    <!-- Job Presets Subsection (Second) -->
    <details>
      <summary class="config-subsection-summary">Job Presets</summary>
      <div class="subsection-content">
        <div class="subsection-header">
          <button class="namd-button namd-button--primary namd-button--sm" on:click={handleAddPreset}>
            + Add Preset
          </button>
        </div>

        <div class="config-list">
          {#each $jobPresets as preset}
            <div class="namd-card">
              <div class="namd-card-header">
                <div class="config-item-info">
                  <h4>{preset.name}</h4>
                </div>
                <div class="config-item-actions">
                  <button class="namd-button namd-button--destructive namd-button--sm" on:click={() => confirmDeletePreset(preset)}>
                    Delete
                  </button>
                </div>
              </div>
              <div class="namd-card-content">
                <p>{preset.description}</p>
                <div class="config-meta">
                  <span>{preset.cores} cores</span>
                  <span>{preset.memory}GB</span>
                  <span>{preset.walltime}</span>
                  <span>{preset.partition} / {preset.qos}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </details>

    <!-- Partitions Subsection (Third) -->
    <details>
      <summary class="config-subsection-summary">Partitions</summary>
      <div class="subsection-content">
        <div class="subsection-header">
          <button class="namd-button namd-button--primary namd-button--sm" on:click={handleAddPartition}>
            + Add Partition
          </button>
        </div>

        <div class="config-list">
          {#each $partitions as partition}
            <div class="namd-card">
              <div class="namd-card-header">
                <div class="config-item-info">
                  <h4>{partition.title} {#if partition.is_default}<span class="default-badge">Default</span>{/if}</h4>
                  <code class="config-name">{partition.name}</code>
                </div>
                <div class="config-item-actions">
                  <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEditPartition(partition)}>
                    Edit
                  </button>
                  <button class="namd-button namd-button--destructive namd-button--sm" on:click={() => confirmDeletePartition(partition)}>
                    Delete
                  </button>
                </div>
              </div>
              <div class="namd-card-content">
                <p>{partition.description}</p>
                <div class="config-meta">
                  <span>Max Cores: {partition.max_cores}</span>
                  <span>Max Memory: {partition.max_memory_per_core_gb}GB/core</span>
                  {#if partition.gpu_type}
                    <span>GPU: {partition.gpu_type} (×{partition.gpu_count})</span>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </details>

    <!-- QoS Options Subsection (Fourth) -->
    <details>
      <summary class="config-subsection-summary">QoS Options</summary>
      <div class="subsection-content">
        <div class="subsection-header">
          <button class="namd-button namd-button--primary namd-button--sm" on:click={handleAddQos}>
            + Add QoS
          </button>
        </div>

        <div class="config-list">
          {#each $allQosOptions as qos}
            <div class="namd-card">
              <div class="namd-card-header">
                <div class="config-item-info">
                  <h4>{qos.title} {#if qos.is_default}<span class="default-badge">Default</span>{/if}</h4>
                  <code class="config-name">{qos.name}</code>
                </div>
                <div class="config-item-actions">
                  <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEditQos(qos)}>
                    Edit
                  </button>
                  <button class="namd-button namd-button--destructive namd-button--sm" on:click={() => confirmDeleteQos(qos)}>
                    Delete
                  </button>
                </div>
              </div>
              <div class="namd-card-content">
                <p>{qos.description}</p>
                <div class="config-meta">
                  <span>Max Walltime: {qos.max_walltime_hours}h</span>
                  {#if qos.min_memory_gb}
                    <span>Min Memory: {qos.min_memory_gb}GB</span>
                  {/if}
                  <span>Valid for: {qos.valid_partitions.join(', ')}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </details>

    <!-- Billing Rates Subsection (Fifth) -->
    <details>
      <summary class="config-subsection-summary">Billing Rates</summary>
      <div class="subsection-content">
        <div class="db-info">
          <div class="info-row">
            <span class="label">CPU Cost:</span>
            <span class="value">{$billingRates?.cpu_cost_per_core_hour} SU/core-hour</span>
          </div>
          <div class="info-row">
            <span class="label">GPU Cost:</span>
            <span class="value">{$billingRates?.gpu_cost_per_gpu_hour} SU/GPU-hour</span>
          </div>
        </div>
        <button class="namd-button namd-button--secondary" on:click={handleEditBilling}>Edit Billing Rates</button>
      </div>
    </details>

    <!-- Reset to Defaults -->
    <div class="reset-section">
      <button class="namd-button namd-button--destructive" on:click={() => showResetClusterWarning = true}>
        Reset Cluster Config to Defaults
      </button>
    </div>
  </div>

  <!-- Database Section (Bottom) -->
  <div class="settings-section">
    <h2>Database</h2>

    {#if $settingsLoading}
      <p class="loading">Loading database information...</p>
    {:else if $settingsError}
      <div class="error">
        <strong>Error:</strong> {$settingsError}
      </div>
    {:else if $databaseInfo}
      <div class="db-info">
        <div class="info-row">
          <span class="label">Location:</span>
          <code class="path">{$databaseInfo.path}</code>
        </div>
        <div class="info-row">
          <span class="label">Size:</span>
          <span class="value">{formatBytes($databaseInfo.size_bytes)}</span>
        </div>
      </div>

      <div class="db-actions">
        <button class="namd-button namd-button--secondary" on:click={handleBackup}>Backup Database</button>
        <button class="namd-button namd-button--secondary" on:click={() => showRestoreWarning = true}>Restore Database</button>
        <button class="namd-button namd-button--destructive" on:click={() => showResetWarning = true}>Reset Database</button>
      </div>
    {:else}
      <p class="error">No database information available</p>
    {/if}
  </div>
</div>

<!-- Partition Edit Dialog -->
<EditDialog
  isOpen={showPartitionDialog}
  title={editingPartition && $partitions.some(p => p.name === editingPartition?.name) ? 'Edit Partition' : 'Add Partition'}
  onSave={handleSavePartition}
  onClose={() => showPartitionDialog = false}
>
  <svelte:fragment slot="form">
    {#if editingPartition}
      <div class="namd-field-group">
        <label class="namd-label" for="partition-name">Name *</label>
        <input
          class="namd-input"
          id="partition-name"
          type="text"
          bind:value={editingPartition.name}
          class:error={partitionErrors.name}
          placeholder="amilan"
        />
        {#if partitionErrors.name}
          <span class="namd-error-text">{partitionErrors.name}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-title">Title *</label>
        <input
          class="namd-input"
          id="partition-title"
          type="text"
          bind:value={editingPartition.title}
          class:error={partitionErrors.title}
          placeholder="General Compute"
        />
        {#if partitionErrors.title}
          <span class="namd-error-text">{partitionErrors.title}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-description">Description *</label>
        <textarea
          class="namd-input"
          id="partition-description"
          bind:value={editingPartition.description}
          rows="2"
          placeholder="Standard CPU nodes for most NAMD simulations"
        ></textarea>
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-max-cores">Max Cores *</label>
        <input
          class="namd-input"
          id="partition-max-cores"
          type="number"
          bind:value={editingPartition.max_cores}
          class:error={partitionErrors.max_cores}
          min="1"
          placeholder="64"
        />
        {#if partitionErrors.max_cores}
          <span class="namd-error-text">{partitionErrors.max_cores}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-max-memory">Max Memory per Core (GB) *</label>
        <input
          class="namd-input"
          id="partition-max-memory"
          type="number"
          step="0.01"
          bind:value={editingPartition.max_memory_per_core_gb}
          class:error={partitionErrors.max_memory_per_core_gb}
          min="0.01"
          placeholder="3.75"
        />
        {#if partitionErrors.max_memory_per_core_gb}
          <span class="namd-error-text">{partitionErrors.max_memory_per_core_gb}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-gpu-type">GPU Type (optional)</label>
        <input
          class="namd-input"
          id="partition-gpu-type"
          type="text"
          bind:value={editingPartition.gpu_type}
          placeholder="NVIDIA A100"
        />
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="partition-gpu-count">GPUs per Node (optional)</label>
        <input
          class="namd-input"
          id="partition-gpu-count"
          type="number"
          bind:value={editingPartition.gpu_count}
          min="1"
        />
      </div>

      <div class="namd-field-group">
        <label class="namd-label">
          <input type="checkbox" bind:checked={editingPartition.is_default} />
          Set as default partition
        </label>
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>

<!-- QoS Edit Dialog -->
<EditDialog
  isOpen={showQosDialog}
  title={editingQos && $allQosOptions.some(q => q.name === editingQos?.name) ? 'Edit QoS Option' : 'Add QoS Option'}
  onSave={handleSaveQos}
  onClose={() => showQosDialog = false}
>
  <svelte:fragment slot="form">
    {#if editingQos}
      <div class="namd-field-group">
        <label class="namd-label" for="qos-name">Name *</label>
        <input
          class="namd-input"
          id="qos-name"
          type="text"
          bind:value={editingQos.name}
          class:error={qosErrors.name}
          placeholder="normal"
        />
        {#if qosErrors.name}
          <span class="namd-error-text">{qosErrors.name}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="qos-title">Title *</label>
        <input
          class="namd-input"
          id="qos-title"
          type="text"
          bind:value={editingQos.title}
          class:error={qosErrors.title}
          placeholder="Normal Priority"
        />
        {#if qosErrors.title}
          <span class="namd-error-text">{qosErrors.title}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="qos-description">Description *</label>
        <textarea
          class="namd-input"
          id="qos-description"
          bind:value={editingQos.description}
          rows="2"
        ></textarea>
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="qos-walltime">Max Walltime (hours) *</label>
        <input
          class="namd-input"
          id="qos-walltime"
          type="number"
          bind:value={editingQos.max_walltime_hours}
          class:error={qosErrors.max_walltime_hours}
          min="1"
        />
        {#if qosErrors.max_walltime_hours}
          <span class="namd-error-text">{qosErrors.max_walltime_hours}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="qos-min-memory">Min Memory (GB, optional)</label>
        <input
          class="namd-input"
          id="qos-min-memory"
          type="number"
          bind:value={editingQos.min_memory_gb}
          min="1"
        />
      </div>

      <fieldset class="namd-field-group">
        <legend class="namd-label">Valid Partitions *</legend>
        <div class="partition-checkboxes">
          {#each $partitions as partition}
            <label class="checkbox-label">
              <input
                type="checkbox"
                checked={editingQos.valid_partitions.includes(partition.name)}
                on:change={(e) => {
                  if (!editingQos) return;
                  if (e.currentTarget.checked) {
                    editingQos.valid_partitions = [...editingQos.valid_partitions, partition.name];
                  } else {
                    editingQos.valid_partitions = editingQos.valid_partitions.filter(p => p !== partition.name);
                  }
                }}
              />
              {partition.title}
            </label>
          {/each}
        </div>
        {#if qosErrors.valid_partitions}
          <span class="namd-error-text">{qosErrors.valid_partitions}</span>
        {/if}
      </fieldset>

      <div class="namd-field-group">
        <label class="namd-label">
          <input type="checkbox" bind:checked={editingQos.is_default} />
          Set as default QoS
        </label>
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>

<!-- Preset Add Dialog -->
<EditDialog
  isOpen={showPresetDialog}
  title="Add Job Preset"
  onSave={handleSavePreset}
  onClose={() => showPresetDialog = false}
>
  <svelte:fragment slot="form">
    {#if editingPreset}
      <div class="namd-field-group">
        <label class="namd-label" for="preset-name">Name *</label>
        <input
          class="namd-input"
          id="preset-name"
          type="text"
          bind:value={editingPreset.name}
          class:error={presetErrors.name}
          placeholder="Production Run"
        />
        {#if presetErrors.name}
          <span class="namd-error-text">{presetErrors.name}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-description">Description *</label>
        <input
          class="namd-input"
          id="preset-description"
          type="text"
          bind:value={editingPreset.description}
          placeholder="Standard production simulation"
        />
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-cores">Cores *</label>
        <input
          class="namd-input"
          id="preset-cores"
          type="number"
          bind:value={editingPreset.cores}
          class:error={presetErrors.cores}
          min="1"
        />
        {#if presetErrors.cores}
          <span class="namd-error-text">{presetErrors.cores}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-memory">Memory (GB) *</label>
        <input
          class="namd-input"
          id="preset-memory"
          type="text"
          bind:value={editingPreset.memory}
          class:error={presetErrors.memory}
          placeholder="32"
        />
        {#if presetErrors.memory}
          <span class="namd-error-text">{presetErrors.memory}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-walltime">Walltime (HH:MM:SS) *</label>
        <input
          class="namd-input"
          id="preset-walltime"
          type="text"
          bind:value={editingPreset.walltime}
          class:error={presetErrors.walltime}
          placeholder="24:00:00"
        />
        {#if presetErrors.walltime}
          <span class="namd-error-text">{presetErrors.walltime}</span>
        {/if}
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-partition">Partition *</label>
        <select class="namd-input" id="preset-partition" bind:value={editingPreset.partition}>
          {#each $partitions as partition}
            <option value={partition.name}>{partition.title}</option>
          {/each}
        </select>
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="preset-qos">QoS *</label>
        <select class="namd-input" id="preset-qos" bind:value={editingPreset.qos}>
          {#each $allQosOptions as qos}
            <option value={qos.name}>{qos.title}</option>
          {/each}
        </select>
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>

<!-- Billing Edit Dialog -->
<EditDialog
  isOpen={showBillingDialog}
  title="Edit Billing Rates"
  onSave={handleSaveBilling}
  onClose={() => {
    showBillingDialog = false;
    editingBilling = null;
  }}
>
  <svelte:fragment slot="form">
    {#if editingBilling}
      <div class="namd-field-group">
        <label class="namd-label" for="cpu-rate">CPU Cost per Core-Hour (SU)</label>
        <input
          class="namd-input"
          id="cpu-rate"
          type="number"
          step="0.1"
          bind:value={editingBilling.cpu_cost_per_core_hour}
          min="0"
        />
      </div>

      <div class="namd-field-group">
        <label class="namd-label" for="gpu-rate">GPU Cost per GPU-Hour (SU)</label>
        <input
          class="namd-input"
          id="gpu-rate"
          type="number"
          step="0.1"
          bind:value={editingBilling.gpu_cost_per_gpu_hour}
          min="0"
        />
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>

<!-- Default Host Edit Dialog -->
<EditDialog
  isOpen={showHostDialog}
  title="Edit Default Host"
  onSave={handleSaveHost}
  onClose={() => {
    showHostDialog = false;
    editingHost = '';
  }}
>
  <svelte:fragment slot="form">
    {#if editingHost !== ''}
      <div class="namd-field-group">
        <label class="namd-label" for="default-host">Hostname *</label>
        <input
          class="namd-input"
          id="default-host"
          type="text"
          bind:value={editingHost}
          placeholder="login.rc.colorado.edu"
        />
        <p class="help-text">Default SSH hostname for cluster connections</p>
      </div>
    {/if}
  </svelte:fragment>
</EditDialog>

<!-- Delete Confirmation Dialogs -->
<ConfirmDialog
  isOpen={showDeletePartitionDialog}
  title="Delete Partition?"
  message="<strong>Warning:</strong> Delete partition '{deleteTargetPartition?.title}'?<br><br>This will also delete any job presets that use this partition."
  confirmText="Delete"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleDeletePartitionConfirm}
  onCancel={() => {
    showDeletePartitionDialog = false;
    deleteTargetPartition = null;
  }}
/>

<ConfirmDialog
  isOpen={showDeleteQosDialog}
  title="Delete QoS Option?"
  message="<strong>Warning:</strong> Delete QoS '{deleteTargetQos?.title}'?<br><br>This will also delete any job presets that use this QoS option."
  confirmText="Delete"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleDeleteQosConfirm}
  onCancel={() => {
    showDeleteQosDialog = false;
    deleteTargetQos = null;
  }}
/>

<ConfirmDialog
  isOpen={showDeletePresetDialog}
  title="Delete Preset?"
  message="Delete preset '{deleteTargetPreset?.name}'?"
  confirmText="Delete"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleDeletePresetConfirm}
  onCancel={() => {
    showDeletePresetDialog = false;
    deleteTargetPreset = null;
  }}
/>

<!-- Database Confirmation Dialogs -->
<ConfirmDialog
  isOpen={showRestoreWarning}
  title="Restore Database?"
  message="<strong>Warning:</strong> This will replace your current database with the backup you select. All current data (jobs, templates, etc.) will be lost.<br><br>Are you sure you want to continue?"
  confirmText="Restore"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleRestoreConfirm}
  onCancel={() => (showRestoreWarning = false)}
/>

<ConfirmDialog
  isOpen={showResetWarning}
  title="Reset Database?"
  message="<strong>Warning:</strong> This will delete all data in the database and create a fresh database. All jobs, templates, and other data will be permanently lost.<br><br>This action cannot be undone. Are you sure?"
  confirmText="Reset Database"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleResetConfirm}
  onCancel={() => (showResetWarning = false)}
/>

<ConfirmDialog
  isOpen={showResetClusterWarning}
  title="Reset Cluster Configuration?"
  message="<strong>Warning:</strong> This will reset all cluster configuration (partitions, QoS options, presets, billing rates) to default values.<br><br>Any customizations you've made will be lost. Are you sure?"
  confirmText="Reset to Defaults"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleResetClusterConfirm}
  onCancel={() => (showResetClusterWarning = false)}
/>

<ConfirmDialog
  isOpen={showAlert}
  title={alertTitle}
  message={alertMessage}
  variant={alertVariant}
  showCancel={false}
  confirmText="OK"
  onConfirm={() => (showAlert = false)}
  onCancel={() => (showAlert = false)}
/>

<style>
  .settings-page {
    padding: var(--namd-spacing-xl);
    max-width: var(--namd-max-width-form);
  }

  .settings-section {
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
    margin-bottom: var(--namd-spacing-lg);
  }

  .settings-section h2 {
    margin: 0 0 var(--namd-spacing-md) 0;
    font-size: var(--namd-font-size-xl);
    color: var(--namd-text-primary);
  }

  .subsection-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--namd-spacing-md);
  }

  .config-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--namd-spacing-lg);
  }

  .config-item-info {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
  }

  .config-item-info h4 {
    margin: 0;
    font-size: var(--namd-font-size-base);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-primary);
  }

  .config-name {
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
  }

  .config-item-actions {
    display: flex;
    gap: var(--namd-spacing-xs);
  }

  .config-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-sm);
    margin-top: var(--namd-spacing-sm);
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-secondary);
  }

  .config-meta span {
    background: var(--namd-bg-muted);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
  }

  .reset-section {
    margin-top: var(--namd-spacing-lg);
    padding-top: var(--namd-spacing-lg);
    border-top: 1px solid var(--namd-border);
  }

  .db-info {
    margin-bottom: var(--namd-spacing-lg);
  }

  .info-row {
    display: flex;
    gap: var(--namd-spacing-sm);
    margin-bottom: var(--namd-spacing-sm);
    align-items: baseline;
  }

  .label {
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-secondary);
    min-width: 80px;
  }

  .path {
    background: var(--namd-bg-muted);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-base);
    word-break: break-all;
  }

  .value {
    color: var(--namd-text-primary);
  }

  .db-actions {
    display: flex;
    gap: var(--namd-spacing-sm);
    flex-wrap: wrap;
  }

  .loading,
  .error {
    color: var(--namd-text-secondary);
    font-style: italic;
  }

  .error {
    color: var(--namd-error);
  }

  .partition-checkboxes {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    cursor: pointer;
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }

  .default-badge {
    display: inline-block;
    background: var(--namd-primary-bg);
    color: var(--namd-primary);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    margin-left: var(--namd-spacing-sm);
  }

  .config-subsection-summary {
    cursor: pointer;
    font-weight: var(--namd-font-weight-semibold);
    font-size: var(--namd-font-size-lg);
    color: var(--namd-text-primary);
    padding: var(--namd-spacing-md) 0;
    margin: 0;
    list-style: none;
    user-select: none;
    border-bottom: 1px solid var(--namd-border);
  }

  .config-subsection-summary:hover {
    color: var(--namd-primary);
  }

  .config-subsection-summary::-webkit-details-marker {
    display: none;
  }

  .config-subsection-summary::before {
    content: '▶';
    display: inline-block;
    margin-right: var(--namd-spacing-sm);
    transition: transform 0.2s;
  }

  details[open] .config-subsection-summary::before {
    transform: rotate(90deg);
  }

  details[open] .config-subsection-summary {
    border-bottom-color: var(--namd-primary);
  }

  .subsection-content {
    padding: var(--namd-spacing-md) 0;
  }
</style>

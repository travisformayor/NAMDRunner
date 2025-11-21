<script lang="ts">
  import type { JobInfo } from '../../types/api';
  import OverviewTab from './tabs/OverviewTab.svelte';
  import InputFilesTab from './tabs/InputFilesTab.svelte';
  import OutputFilesTab from './tabs/OutputFilesTab.svelte';
  import SlurmLogsTab from './tabs/SlurmLogsTab.svelte';

  export let job: JobInfo;

  type TabId = 'overview' | 'input-files' | 'output-files' | 'slurm-logs';

  const tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'input-files', label: 'Input Files' },
    { id: 'output-files', label: 'Output Files' },
    { id: 'slurm-logs', label: 'SLURM Logs' }
  ];

  let activeTab: TabId = 'overview';
</script>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-4">
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
    {#if activeTab === 'overview'}
      <OverviewTab {job} />
    {:else if activeTab === 'input-files'}
      <InputFilesTab {job} />
    {:else if activeTab === 'output-files'}
      <OutputFilesTab {job} />
    {:else if activeTab === 'slurm-logs'}
      <SlurmLogsTab {job} />
    {/if}
  </div>
</div>

<style>
  /* All styles now come from global CSS - this component is purely a coordinator */
</style>

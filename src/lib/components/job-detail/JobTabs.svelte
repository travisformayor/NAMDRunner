<script lang="ts">
  import type { JobInfo } from '../../types/api';
  import { CoreClientFactory } from '../../ports/clientFactory';
  import OverviewTab from './tabs/OverviewTab.svelte';
  import SlurmLogsTab from './tabs/SlurmLogsTab.svelte';
  import InputFilesTab from './tabs/InputFilesTab.svelte';
  import OutputFilesTab from './tabs/OutputFilesTab.svelte';
  import ConfigurationTab from './tabs/ConfigurationTab.svelte';

  export let job: JobInfo;

  type TabId = 'overview' | 'slurm-logs' | 'input-files' | 'output-files' | 'configuration';

  const tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'slurm-logs', label: 'SLURM Logs' },
    { id: 'input-files', label: 'Input Files' },
    { id: 'output-files', label: 'Output Files' },
    { id: 'configuration', label: 'Configuration' }
  ];

  let activeTab: TabId = 'overview';

  // Determine if we're in demo mode
  $: isDemoMode = CoreClientFactory.getUserMode() === 'demo';
</script>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-5">
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
      <OverviewTab {job} {isDemoMode} />
    {:else if activeTab === 'slurm-logs'}
      <SlurmLogsTab {job} {isDemoMode} />
    {:else if activeTab === 'input-files'}
      <InputFilesTab {job} {isDemoMode} />
    {:else if activeTab === 'output-files'}
      <OutputFilesTab {job} {isDemoMode} />
    {:else if activeTab === 'configuration'}
      <ConfigurationTab {job} {isDemoMode} />
    {/if}
  </div>
</div>

<style>
  /* All styles now come from global CSS - this component is purely a coordinator */
</style>

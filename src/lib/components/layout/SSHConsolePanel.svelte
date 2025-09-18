<script lang="ts">
  import { consoleOpen, uiStore } from '../../stores/ui';

  // Mock console output matching the React mockup exactly
  const mockConsoleOutput = [
    '$ module load slurm/alpine',
    '$ squeue -u jsmith --format="%.10i %.20j %.8T %.10M %.6D %R"',
    '    JOBID                 NAME     STATE       TIME  NODES NODELIST(REASON)',
    '12345678     protein_folding_sim        R    2:15:30      4 compute-[001-004]',
    '12345679        drug_binding_ana       PD       0:00      2 (Resources)',
    '$ sbatch job.sbatch',
    'Submitted batch job 12345680',
    '$ scancel 12345676',
    'Job cancelled successfully',
    '$ scontrol show job 12345678',
    'JobId=12345678 JobName=protein_folding_sim',
    '   UserId=jsmith(1001) GroupId=research(1001) MCS_label=N/A',
    '   Priority=1000 Nice=0 Account=research QOS=normal',
    '   JobState=RUNNING Reason=None Dependency=(null)',
    '   Requeue=1 Restarts=0 BatchFlag=1 Reboot=0 ExitCode=0:0',
    '   RunTime=02:15:30 TimeLimit=04:00:00 TimeMin=N/A',
    '   SubmitTime=2024-01-15T09:35:00 EligibleTime=2024-01-15T09:35:00',
    '   AccrueTime=2024-01-15T09:35:00',
    '   StartTime=2024-01-15T09:35:00 EndTime=2024-01-15T13:35:00 Deadline=N/A',
    '   SuspendTime=None SecsPreSuspend=0 LastSchedEval=2024-01-15T09:35:00',
    '   Partition=amilan AllocNode:Sid=login01:12345',
    '   ReqNodeList=(null) ExcNodeList=(null)',
    '   NodeList=compute-[001-004]',
    '   BatchHost=compute-001',
    '   NumNodes=4 NumCPUs=128 NumTasks=128 CPUs/Task=1 ReqB:S:C:T=0:0:*:*',
    '$'
  ];

  let consoleOutput = mockConsoleOutput;

  function toggleConsole() {
    uiStore.toggleConsole();
  }

  function handleCopyAll() {
    navigator.clipboard.writeText(consoleOutput.join('\n'));
  }

  function handleClear() {
    consoleOutput = ['$'];
  }
</script>

{#if !$consoleOpen}
  <!-- Collapsed state - just a clickable bar -->
  <div class="console-collapsed">
    <button class="console-toggle-collapsed" on:click={toggleConsole}>
      <svg class="chevron-up" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="18,15 12,9 6,15"></polyline>
      </svg>
      SSH Console
    </button>
  </div>
{:else}
  <!-- Expanded state -->
  <div class="console-expanded">
    <!-- Header with controls -->
    <div class="console-header">
      <button class="console-toggle-expanded" on:click={toggleConsole}>
        <svg class="chevron-down" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6,9 12,15 18,9"></polyline>
        </svg>
        <span class="console-title">SSH Console</span>
      </button>

      <div class="console-actions">
        <button class="action-button" on:click={handleCopyAll} title="Copy All">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          Copy All
        </button>
        <button class="action-button" on:click={handleClear} title="Clear">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
          Clear
        </button>
      </div>
    </div>

    <!-- Console output area -->
    <div class="console-output">
      {#each consoleOutput as line}
        <div class="output-line">{line}</div>
      {/each}
      <!-- Blinking cursor -->
      <div class="cursor-line">
        <span class="cursor"></span>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Collapsed state */
  .console-collapsed {
    border-top: 1px solid var(--namd-border);
    background-color: var(--namd-bg-primary);
  }

  .console-toggle-collapsed {
    width: 100%;
    padding: 8px;
    background: none;
    border: none;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .console-toggle-collapsed:hover {
    background-color: var(--namd-bg-muted);
  }

  /* Expanded state */
  .console-expanded {
    border-top: 1px solid var(--namd-border);
    background-color: var(--namd-bg-primary);
    height: 33vh;
    display: flex;
    flex-direction: column;
  }

  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px;
    border-bottom: 1px solid var(--namd-border);
    background-color: var(--namd-bg-primary);
  }

  .console-toggle-expanded {
    display: flex;
    align-items: center;
    gap: 8px;
    background: none;
    border: none;
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
    cursor: pointer;
    padding: 0;
  }

  .console-toggle-expanded:hover {
    background-color: transparent;
  }

  .console-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: none;
    border: none;
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    cursor: pointer;
    border-radius: var(--namd-border-radius-sm);
    transition: background-color 0.15s ease;
    height: 32px;
  }

  .action-button:hover {
    background-color: var(--namd-accent);
  }

  .console-output {
    flex: 1;
    padding: 12px;
    overflow-y: auto;
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-sm);
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
    line-height: 1.4;
  }

  .output-line {
    margin: 0;
    white-space: pre-wrap;
  }

  .cursor-line {
    display: inline-block;
  }

  .cursor {
    display: inline-block;
    width: 8px;
    height: 16px;
    background-color: var(--namd-text-primary);
    animation: blink 1s infinite;
    margin-left: 4px;
  }

  @keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }

  /* SVG icons */
  .chevron-up,
  .chevron-down,
  .action-button svg {
    flex-shrink: 0;
  }
</style>
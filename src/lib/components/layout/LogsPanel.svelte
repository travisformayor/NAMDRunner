<script lang="ts">
  import { consoleOpen, uiStore } from '../../stores/ui';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  interface ConsoleEntry {
    timestamp: string;
    type: 'ssh-command' | 'ssh-output' | 'app-debug';
    content: string;
  }

  let consoleEntries: ConsoleEntry[] = [];

  function addEntry(type: ConsoleEntry['type'], content: string) {
    const timestamp = new Date().toLocaleTimeString();
    consoleEntries = [...consoleEntries, { timestamp, type, content }];
  }

  function addDebugEntry(content: string) {
    addEntry('app-debug', content);
  }

  function addSSHCommandEntry(command: string) {
    addEntry('ssh-command', `$ ${command}`);
  }

  function addSSHOutputEntry(output: string) {
    addEntry('ssh-output', output);
  }

  // Export functions globally so other components can use them
  if (typeof window !== 'undefined') {
    window.appLogger = {
      addCommand: addSSHCommandEntry,
      addOutput: addSSHOutputEntry,
      addDebug: addDebugEntry
    };
  }

  // Listen for backend logs via Tauri events
  onMount(() => {
    // Listen for Rust logs from the backend (async setup)
    let unlisten: (() => void) | undefined;

    (async () => {
      unlisten = await listen('rust-log', (event: any) => {
        const logData = event.payload;
        const logLevel = logData.level?.toLowerCase() || 'info';
        const timestamp = new Date(logData.timestamp).toLocaleTimeString() || new Date().toLocaleTimeString();
        const message = `[${logLevel.toUpperCase()}] [${logData.target}] ${logData.message}`;

        // Add to console with appropriate styling based on log level
        if (logLevel === 'error') {
          addDebugEntry(`🔴 ${message}`);
        } else if (logLevel === 'warn') {
          addDebugEntry(`🟡 ${message}`);
        } else if (logLevel === 'info') {
          addDebugEntry(`🔵 ${message}`);
        } else if (logLevel === 'debug') {
          addDebugEntry(`⚪ ${message}`);
        } else {
          addDebugEntry(message);
        }
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  });

  function toggleConsole() {
    uiStore.toggleConsole();
  }

  function handleCopyAll() {
    const output = consoleEntries.map(entry => {
      const prefix = entry.type === 'app-debug' ? `[${entry.timestamp}] ` : '';
      return `${prefix}${entry.content}`;
    }).join('\n');
    navigator.clipboard.writeText(output);
  }

  function handleClear() {
    consoleEntries = [];
  }
</script>

{#if !$consoleOpen}
  <!-- Collapsed state - just a clickable bar -->
  <div class="console-collapsed">
    <button class="console-toggle-collapsed" on:click={toggleConsole}>
      <svg class="chevron-up" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="18,15 12,9 6,15"></polyline>
      </svg>
      Logs
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
        <span class="console-title">Logs</span>
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
      {#each consoleEntries as entry}
        <div class="output-line {entry.type}">
          {#if entry.type === 'app-debug'}
            <span class="timestamp">[{entry.timestamp}]</span>
          {/if}
          <span class="content">{entry.content}</span>
        </div>
      {/each}
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
    margin: 2px 0;
    white-space: pre-wrap;
    display: flex;
    gap: 8px;
  }

  .output-line.ssh-command {
    color: var(--namd-text-primary);
    font-weight: 500;
  }

  .output-line.ssh-output {
    color: var(--namd-text-secondary);
  }

  .output-line.app-debug {
    color: var(--namd-text-secondary);
    font-size: 0.9em;
  }

  .timestamp {
    color: var(--namd-text-muted);
    font-size: 0.85em;
    flex-shrink: 0;
  }

  .content {
    flex: 1;
  }

  /* SVG icons */
  .chevron-up,
  .chevron-down,
  .action-button svg {
    flex-shrink: 0;
  }
</style>
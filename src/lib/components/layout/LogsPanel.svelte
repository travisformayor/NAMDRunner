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

  // Listen for backend logs via Tauri events
  onMount(() => {
    // Listen for Rust logs from the backend (async setup)
    let unlisten: (() => void) | undefined;

    (async () => {
      unlisten = await listen('app-log', (event: any) => {
        const logData = event.payload;
        const logLevel = logData.level?.toLowerCase() || 'info';
        const messageText = logData.details || logData.message;
        const formattedMessage = `[${logLevel.toUpperCase()}] [${logData.category}] ${messageText}`;

        // Add to console with appropriate styling based on log level
        if (logLevel === 'error') {
          addDebugEntry(`🔴 ${formattedMessage}`);
        } else if (logLevel === 'warn') {
          addDebugEntry(`🟡 ${formattedMessage}`);
        } else if (logLevel === 'info') {
          addDebugEntry(`🔵 ${formattedMessage}`);
        } else if (logLevel === 'debug') {
          addDebugEntry(`⚪ ${formattedMessage}`);
        } else {
          addDebugEntry(formattedMessage);
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
        <button class="namd-button namd-button--ghost namd-button--sm" on:click={handleCopyAll} title="Copy All">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          Copy All
        </button>
        <button class="namd-button namd-button--ghost namd-button--sm" on:click={handleClear} title="Clear">
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
    padding: var(--namd-spacing-sm);
    background: none;
    border: none;
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-primary);
    cursor: pointer;
    transition: background-color var(--namd-transition-fast);
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
    padding: var(--namd-spacing-sm);
    border-bottom: 1px solid var(--namd-border);
    background-color: var(--namd-bg-primary);
  }

  .console-toggle-expanded {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    background: none;
    border: none;
    font-size: var(--namd-font-size-base);
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
    gap: var(--namd-spacing-xs);
  }

  .console-output {
    flex: 1;
    padding: var(--namd-spacing-md);
    overflow-y: auto;
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-base);
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
    line-height: 1.4;
  }

  .output-line {
    margin: 2px 0;
    white-space: pre-wrap;
    display: flex;
    gap: var(--namd-spacing-sm);
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
    color: var(--namd-text-secondary);
    font-size: 0.85em;
    flex-shrink: 0;
  }

  .content {
    flex: 1;
  }

  /* SVG icons */
  .chevron-up,
  .chevron-down {
    flex-shrink: 0;
  }
</style>
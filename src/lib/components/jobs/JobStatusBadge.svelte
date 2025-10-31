<script lang="ts">
  import type { JobStatus } from '../../types/api';
  import { getStatusInfo } from '../../utils/file-helpers';

  export let status: JobStatus;

  $: statusInfo = getStatusInfo(status);
</script>

<span class="status-badge namd-status-badge--{statusInfo.class}">
  <span class="status-icon">{statusInfo.icon}</span>
  <span class="status-label">{statusInfo.label}</span>
</span>

<style>
  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border-radius: 9999px;
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.025em;
    white-space: nowrap;
  }

  .status-icon {
    font-size: 0.75rem;
    line-height: 1;
  }

  .status-label {
    line-height: 1;
  }

  /* Status-specific styles using CSS custom properties */
  .status-badge:global(.namd-status-badge--created) {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-secondary);
  }

  .status-badge:global(.namd-status-badge--pending) {
    background-color: var(--namd-warning-bg);
    color: var(--namd-warning-fg);
  }

  .status-badge:global(.namd-status-badge--running) {
    background-color: var(--namd-info-bg);
    color: var(--namd-info-fg);
  }

  .status-badge:global(.namd-status-badge--completed) {
    background-color: var(--namd-success-bg);
    color: var(--namd-success-fg);
  }

  .status-badge:global(.namd-status-badge--failed) {
    background-color: var(--namd-error-bg);
    color: var(--namd-error-fg);
  }

  .status-badge:global(.namd-status-badge--cancelled) {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-secondary);
  }

  .status-badge:global(.namd-status-badge--unknown) {
    background-color: var(--namd-bg-muted);
    color: var(--namd-text-muted);
  }
</style>
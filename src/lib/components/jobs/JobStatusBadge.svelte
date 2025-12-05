<script lang="ts">
  import type { JobStatus } from '../../types/api';
  import { getStatusInfo } from '../../utils/file-helpers';

  export let status: JobStatus;

  $: statusInfo = getStatusInfo(status);
</script>

<span class="namd-status-badge namd-status-badge--{statusInfo.class}">
  <span class="status-icon">{statusInfo.icon}</span>
  <span class="status-label">{statusInfo.label}</span>
</span>

<style>
  .status-icon {
    font-size: 0.75rem;
    line-height: 1;
  }

  .status-label {
    line-height: 1;
  }
</style>
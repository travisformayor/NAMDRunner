<script lang="ts">
  import type { JobInfo } from '../../../types/api';
  import { getStatusBadgeClass } from '../../../utils/file-helpers';

  export let job: JobInfo;
  export let isDemoMode: boolean = false;

  // Reactive computed values for SLURM config
  $: slurmConfig = {
    cores: job.slurm_config.cores,
    memory: job.slurm_config.memory,
    wallTime: job.slurm_config.walltime,
    partition: job.slurm_config.partition || 'N/A',
  };

  function getSimulationProgress(): number {
    if (job.status === 'CREATED' || job.status === 'PENDING') return 0;
    if (job.status === 'COMPLETED') return 100;
    // For running and failed jobs, we don't have real-time progress tracking yet
    // Show a static value for demo/visual purposes only
    if (isDemoMode) {
      if (job.status === 'FAILED') return 75;
      if (job.status === 'RUNNING') return 45;
    }
    return 0; // Real mode: progress tracking not yet implemented
  }

  function getCompletedSteps(): number {
    const total = getTotalSteps();
    return Math.floor(total * (getSimulationProgress() / 100));
  }

  function getTotalSteps(): number {
    return job.namd_config?.steps || 0;
  }

  function getEstimatedTimeRemaining(): string {
    if (job.status === 'COMPLETED') return 'Completed';
    if (job.status === 'FAILED') return 'Failed';
    if (job.status === 'CREATED' || job.status === 'PENDING') return 'Not started';
    if (job.status === 'RUNNING') return 'Real-time tracking not yet available';
    return '--';
  }
</script>

<div class="namd-tab-panel">
  <div class="overview-content">
    <!-- Simulation Progress -->
    <div class="overview-section">
      <h3>Simulation Progress</h3>
      <div class="progress-card">
        <div class="progress-header">
          <span class="progress-label">MD Steps Completed</span>
          <span class="progress-value">{getSimulationProgress().toFixed(1)}%</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {getSimulationProgress()}%"></div>
        </div>
        <div class="progress-details">
          <span class="namd-text-sm">{getCompletedSteps().toLocaleString()} / {getTotalSteps().toLocaleString()} steps</span>
          <span class="namd-text-sm">{getEstimatedTimeRemaining()}</span>
        </div>
      </div>
    </div>

    <!-- Resource Allocation -->
    <div class="overview-section">
      <h3>Resource Allocation</h3>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Cores</span>
          <span class="info-value">{slurmConfig.cores}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Memory</span>
          <span class="info-value">{slurmConfig.memory}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Wall Time</span>
          <span class="info-value">{slurmConfig.wallTime}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Partition</span>
          <span class="info-value">{slurmConfig.partition}</span>
        </div>
      </div>
    </div>

    <!-- Job Information -->
    <div class="overview-section">
      <h3>Job Information</h3>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Status</span>
          <span class="info-value namd-status-badge {getStatusBadgeClass(job.status)}">{job.status}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Simulation Steps</span>
          <span class="info-value">{job.namd_config.steps.toLocaleString()}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Temperature</span>
          <span class="info-value">{job.namd_config.temperature} K</span>
        </div>
        <div class="info-item">
          <span class="info-label">Timestep</span>
          <span class="info-value">{job.namd_config.timestep} fs</span>
        </div>
      </div>
    </div>
  </div>
</div>

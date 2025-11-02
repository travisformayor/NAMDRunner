<script lang="ts">
  import type { JobInfo } from '../../../types/api';
  import { mockNAMDConfig, mockSlurmConfig } from '../../../test/fixtures/mockJobData';

  export let job: JobInfo;
  export let isDemoMode: boolean = false;

  // Display configuration types that include both real fields and demo-only fields
  type DisplayNAMDConfig = {
    simulationSteps: number;
    temperature: number;
    timestep: number;
    outputName: string;
    dcdFreq: number;
    restartFreq: number;
    // Demo-only fields (not in actual NAMDConfig type)
    coordinates?: string;
    structure?: string;
    parameters?: string[];
    cutoff?: number;
    switchDist?: number;
    pairlistDist?: number;
    PME?: boolean;
    PMEGridSpacing?: number;
    langevin?: boolean;
    langevinDamping?: number;
    langevinTemp?: number;
    langevinHydrogen?: boolean;
    useGroupPressure?: boolean;
    useFlexibleCell?: boolean;
    useConstantArea?: boolean;
    langevinPiston?: boolean;
    langevinPistonTarget?: number;
    langevinPistonPeriod?: number;
    langevinPistonDecay?: number;
    langevinPistonTemp?: number;
  };

  type DisplaySlurmConfig = {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
    // Demo-only fields (not in actual SlurmConfig type)
    account?: string;
    nodes?: number;
    tasksPerNode?: number;
    cpusPerTask?: number;
    gpus?: number;
    gpuType?: string;
  };

  // Use imported mock configs from fixtures
  const demoNamdConfig = mockNAMDConfig;
  const demoSlurmConfig = mockSlurmConfig;

  // Get configuration based on mode - using explicit types to satisfy TypeScript
  let namdConfig: DisplayNAMDConfig;
  $: namdConfig = isDemoMode ? demoNamdConfig : {
    simulationSteps: job.namd_config.steps,
    temperature: job.namd_config.temperature,
    timestep: job.namd_config.timestep,
    outputName: job.namd_config.outputname,
    dcdFreq: job.namd_config.dcd_freq || 0,
    restartFreq: job.namd_config.restart_freq || 0,
  };

  let slurmConfig: DisplaySlurmConfig;
  $: slurmConfig = isDemoMode ? demoSlurmConfig : {
    cores: job.slurm_config.cores,
    memory: job.slurm_config.memory,
    wallTime: job.slurm_config.walltime,
    partition: job.slurm_config.partition || 'N/A',
    qos: job.slurm_config.qos || 'N/A',
  };
</script>

<div class="namd-tab-panel">
  <div class="configuration-content">
    <!-- NAMD Configuration -->
    <div class="config-section">
      <h3>NAMD Configuration</h3>

      <!-- Basic Settings -->
      <div class="config-subsection">
        <h4>Simulation Settings</h4>
        <div class="config-grid-3">
          <div class="config-item">
            <div class="config-label">Simulation Steps</div>
            <div class="config-value">{namdConfig.simulationSteps.toLocaleString()}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Temperature (K)</div>
            <div class="config-value">{namdConfig.temperature}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Timestep (fs)</div>
            <div class="config-value">{namdConfig.timestep}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Output Name</div>
            <div class="config-value">{namdConfig.outputName}</div>
          </div>
          <div class="config-item">
            <div class="config-label">DCD Frequency</div>
            <div class="config-value">{namdConfig.dcdFreq > 0 ? namdConfig.dcdFreq.toLocaleString() : 'Not set'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Restart Frequency</div>
            <div class="config-value">{namdConfig.restartFreq > 0 ? namdConfig.restartFreq.toLocaleString() : 'Not set'}</div>
          </div>
        </div>
      </div>

      {#if isDemoMode}
      <!-- Input Files (Demo mode only - shows detailed NAMD config) -->
      <div class="config-subsection">
        <h4>Input Files</h4>
        <div class="input-files-config">
          <div class="input-file-item">
            <span class="input-file-label">Coordinates:</span>
            <span class="input-file-value">{namdConfig.coordinates ?? 'N/A'}</span>
          </div>
          <div class="input-file-item">
            <span class="input-file-label">Structure:</span>
            <span class="input-file-value">{namdConfig.structure ?? 'N/A'}</span>
          </div>
          <div class="input-file-item">
            <span class="input-file-label">Parameters:</span>
            <div class="parameter-badges">
              {#each (namdConfig.parameters ?? []) as param}
                <span class="parameter-badge">{param}</span>
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- Force Field & Cutoffs (Demo mode only) -->
      <div class="config-subsection">
        <h4>Force Field & Cutoffs</h4>
        <div class="config-grid-3">
          <div class="config-item">
            <div class="config-label">Cutoff (Å)</div>
            <div class="config-value">{namdConfig.cutoff ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Switch Distance (Å)</div>
            <div class="config-value">{namdConfig.switchDist ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Pairlist Distance (Å)</div>
            <div class="config-value">{namdConfig.pairlistDist ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">PME</div>
            <div class="config-badge {namdConfig.PME ? 'enabled' : 'disabled'}">
              {namdConfig.PME ? 'Enabled' : 'Disabled'}
            </div>
          </div>
          <div class="config-item">
            <div class="config-label">PME Grid Spacing (Å)</div>
            <div class="config-value">{namdConfig.PMEGridSpacing ?? 'N/A'}</div>
          </div>
        </div>
      </div>

      <!-- Dynamics Settings (Demo mode only) -->
      <div class="config-subsection">
        <h4>Dynamics Settings</h4>
        <div class="config-grid-3">
          <div class="config-item">
            <div class="config-label">Langevin Dynamics</div>
            <div class="config-badge {namdConfig.langevin ? 'enabled' : 'disabled'}">
              {namdConfig.langevin ? 'Enabled' : 'Disabled'}
            </div>
          </div>
          <div class="config-item">
            <div class="config-label">Langevin Damping</div>
            <div class="config-value">{namdConfig.langevinDamping ?? 'N/A'} ps⁻¹</div>
          </div>
          <div class="config-item">
            <div class="config-label">Langevin Temperature (K)</div>
            <div class="config-value">{namdConfig.langevinTemp ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Langevin Hydrogen</div>
            <div class="config-badge {namdConfig.langevinHydrogen ? 'enabled' : 'disabled'}">
              {namdConfig.langevinHydrogen ? 'Enabled' : 'Disabled'}
            </div>
          </div>
        </div>
      </div>

      <!-- Pressure Control (Demo mode only) -->
      <div class="config-subsection">
        <h4>Pressure Control</h4>
        <div class="config-grid-3">
          <div class="config-item">
            <div class="config-label">Langevin Piston</div>
            <div class="config-badge {namdConfig.langevinPiston ? 'enabled' : 'disabled'}">
              {namdConfig.langevinPiston ? 'Enabled' : 'Disabled'}
            </div>
          </div>
          <div class="config-item">
            <div class="config-label">Target Pressure (bar)</div>
            <div class="config-value">{namdConfig.langevinPistonTarget ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Piston Period (fs)</div>
            <div class="config-value">{namdConfig.langevinPistonPeriod ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Piston Decay (fs)</div>
            <div class="config-value">{namdConfig.langevinPistonDecay ?? 'N/A'}</div>
          </div>
          <div class="config-item">
            <div class="config-label">Piston Temperature (K)</div>
            <div class="config-value">{namdConfig.langevinPistonTemp ?? 'N/A'}</div>
          </div>
        </div>
      </div>
      {/if}
    </div>

    <!-- SLURM Configuration -->
    <div class="config-section">
      <h3>SLURM Resource Allocation</h3>
      <div class="config-grid-3">
        <div class="config-item">
          <div class="config-label">Cores</div>
          <div class="config-value">{slurmConfig.cores}</div>
        </div>
        <div class="config-item">
          <div class="config-label">Memory</div>
          <div class="config-value">{slurmConfig.memory}</div>
        </div>
        <div class="config-item">
          <div class="config-label">Wall Time</div>
          <div class="config-value">{slurmConfig.wallTime}</div>
        </div>
        <div class="config-item">
          <div class="config-label">Partition</div>
          <div class="config-value">{slurmConfig.partition}</div>
        </div>
        <div class="config-item">
          <div class="config-label">QOS</div>
          <div class="config-value">{slurmConfig.qos}</div>
        </div>
        {#if isDemoMode}
        <!-- Demo-only fields (not in actual SlurmConfig type) -->
        <div class="config-item">
          <div class="config-label">Account</div>
          <div class="config-value">{slurmConfig.account ?? 'N/A'}</div>
        </div>
        <div class="config-item">
          <div class="config-label">Nodes</div>
          <div class="config-value">{slurmConfig.nodes ?? 'N/A'}</div>
        </div>
        <div class="config-item">
          <div class="config-label">Tasks per Node</div>
          <div class="config-value">{slurmConfig.tasksPerNode ?? 'N/A'}</div>
        </div>
        <div class="config-item">
          <div class="config-label">CPUs per Task</div>
          <div class="config-value">{slurmConfig.cpusPerTask ?? 'N/A'}</div>
        </div>
        <div class="config-item">
          <div class="config-label">GPUs</div>
          <div class="config-value">{slurmConfig.gpus ?? 0} × {slurmConfig.gpuType?.toUpperCase() ?? 'N/A'}</div>
        </div>
        {/if}
      </div>
    </div>
  </div>
</div>

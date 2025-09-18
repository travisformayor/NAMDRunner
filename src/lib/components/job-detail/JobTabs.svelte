<script lang="ts">
  import type { Job } from '../../types/api';
  import { getFileIcon, getTypeLabel, getTypeColor, formatFileSize, getStatusBadgeClass } from '../../utils/file-helpers';

  export let job: Job;

  type TabId = 'overview' | 'slurm-logs' | 'input-files' | 'output-files' | 'configuration';

  const tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'slurm-logs', label: 'SLURM Logs' },
    { id: 'input-files', label: 'Input Files' },
    { id: 'output-files', label: 'Output Files' },
    { id: 'configuration', label: 'Configuration' }
  ];

  let activeTab: TabId = 'overview';
  let activeLogTab: 'stdout' | 'stderr' = 'stdout';

  const mockStdout = `NAMD 3.0beta3 for LINUX-X86_64-multicore
Built Thu Aug 26 16:49:51 CDT 2021 by jim on belfast.ks.uiuc.edu

Running on 128 processors.
CHARM> Running on 128 cores
CHARM> Number of chares: 128

Info: NAMD 3.0beta3 for LINUX-X86_64-multicore
Info: Built Thu Aug 26 16:49:51 CDT 2021 by jim on belfast.ks.uiuc.edu
Info: 1 CUDA devices on 1 nodes
Info: CUDA device 0 is Tesla V100-SXM2-16GB

Info: Running on 128 processors, 128 cores, 1 hosts.
Info: CPU topology information available.
Info: Charm++/Converse parallel runtime startup completed at 0.0123 s
Info: NAMD running on 128 processors, 128 cores, 1 hosts.

Reading structure file protein.psf
protein.psf info: 92428 atoms, 7324 bonds, 13251 angles, 18746 dihedrals,
                  11874 impropers, 0 cross-terms

Reading parameter file par_all36_prot.prm
Reading parameter file par_all36_lipid.prm
Reading parameter file toppar_water_ions.str

Info: READING COORDINATES FROM DCD FILE protein_eq.dcd
Info: DCD file protein_eq.dcd at time 0.0

TIMING: 500  CPU: 12.45, 0.0249/step  Wall: 12.45, 0.0249/step, 34.57 hours remaining
TIMING: 1000  CPU: 24.89, 0.0249/step  Wall: 24.89, 0.0249/step, 34.56 hours remaining
ENERGY:    1000      7892.3456   23451.2341    -89234.5678       234.5671    -45612.3456     8234.5672       0.0000       0.0000       0.0000  -95834.2034

TIMING: 1500  CPU: 37.34, 0.0249/step  Wall: 37.34, 0.0249/step, 34.55 hours remaining
TIMING: 2000  CPU: 49.78, 0.0249/step  Wall: 49.78, 0.0249/step, 34.54 hours remaining

[... continuing simulation ...]

TIMING: 450000  CPU: 11234.56, 0.0249/step  Wall: 11234.56, 0.0249/step, 1.23 hours remaining
ENERGY:  450000     7834.2341   23892.1234    -89567.8901       245.6789    -45823.4567     8456.7890       0.0000       0.0000       0.0000  -95962.5214

TIMING: 450500  CPU: 11247.01, 0.0249/step  Wall: 11247.01, 0.0249/step, 1.22 hours remaining`;

  const mockStderr = `Info: Startup phase 0 took 0.00123 s, 128 KB of memory in use
Info: Startup phase 1 took 0.00456 s, 256 KB of memory in use
Info: Startup phase 2 took 0.00789 s, 512 KB of memory in use
Info: Startup phase 3 took 0.01234 s, 1024 KB of memory in use
Info: Startup phase 4 took 0.02345 s, 2048 KB of memory in use
Info: Startup phase 5 took 0.03456 s, 4096 KB of memory in use

Warning: Continuing with simulation despite potential energy instability
Warning: Large force detected at step 12345, atom 67890
Warning: Temperature spike detected at step 23456, T = 315.67 K

Info: Checkpoint written at step 100000
Info: Checkpoint written at step 200000
Info: Checkpoint written at step 300000
Info: Checkpoint written at step 400000

Warning: GPU memory usage approaching limit: 92%
Info: Automatic load balancing triggered at step 445000
Info: Load balancing completed, performance improved by 3.2%`;

  // Mock input files data
  const mockInputFiles = [
    {
      name: "protein.pdb",
      size: "2.3 MB",
      type: "structure",
      description: "Protein structure file"
    },
    {
      name: "protein.psf",
      size: "1.8 MB",
      type: "structure",
      description: "Protein structure file (PSF format)"
    },
    {
      name: "par_all36_prot.prm",
      size: "456 KB",
      type: "parameters",
      description: "CHARMM36 protein parameters"
    },
    {
      name: "par_all36_lipid.prm",
      size: "234 KB",
      type: "parameters",
      description: "CHARMM36 lipid parameters"
    },
    {
      name: "toppar_water_ions.str",
      size: "123 KB",
      type: "parameters",
      description: "Water and ion parameters"
    },
    {
      name: "simulation.conf",
      size: "4.2 KB",
      type: "configuration",
      description: "NAMD configuration file"
    }
  ];

  // Mock output files data
  const mockOutputFiles = [
    {
      name: "trajectory.dcd",
      size: "1.2 GB",
      type: "trajectory",
      description: "Molecular dynamics trajectory",
      lastModified: "2024-01-15 12:45",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "output.log",
      size: "45 MB",
      type: "log",
      description: "NAMD output log file",
      lastModified: "2024-01-15 12:45",
      available: job.status !== "CREATED" && job.status !== "PENDING"
    },
    {
      name: "energy.log",
      size: "23 MB",
      type: "analysis",
      description: "Energy analysis output",
      lastModified: "2024-01-15 12:45",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.coor",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart coordinates",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.vel",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart velocities",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.xsc",
      size: "1 KB",
      type: "checkpoint",
      description: "Extended system coordinates",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    }
  ];

  // Mock configuration data
  const namdConfig = {
    simulationSteps: 1000000,
    temperature: 310.0,
    timestep: 2.0,
    outputName: "protein_simulation",
    dcdFreq: 5000,
    restartFreq: 10000,
    coordinates: "protein.pdb",
    structure: "protein.psf",
    parameters: ["par_all36_prot.prm", "par_all36_lipid.prm", "toppar_water_ions.str"],
    cutoff: 12.0,
    switchDist: 10.0,
    pairlistDist: 14.0,
    PME: true,
    PMEGridSpacing: 1.0,
    langevin: true,
    langevinDamping: 1.0,
    langevinTemp: 310.0,
    langevinHydrogen: false,
    useGroupPressure: true,
    useFlexibleCell: false,
    useConstantArea: false,
    langevinPiston: true,
    langevinPistonTarget: 1.01325,
    langevinPistonPeriod: 100.0,
    langevinPistonDecay: 50.0,
    langevinPistonTemp: 310.0
  };

  const slurmConfig = {
    cores: 128,
    memory: "512GB",
    wallTime: "04:00:00",
    partition: "amilan",
    qos: "normal",
    account: "research",
    nodes: 4,
    tasksPerNode: 32,
    cpusPerTask: 1,
    gpus: 4,
    gpuType: "v100"
  };

  function getStdoutContent(): string {
    if (job.status === 'CREATED') return '';
    return mockStdout;
  }

  function getStderrContent(): string {
    if (job.status === 'CREATED') return '';
    return mockStderr;
  }

  function copyLogs() {
    const content = activeLogTab === 'stdout' ? getStdoutContent() : getStderrContent();
    navigator.clipboard.writeText(content);
  }

  function downloadLogs() {
    const content = activeLogTab === 'stdout' ? getStdoutContent() : getStderrContent();
    const filename = `${job.jobId}_${activeLogTab}.log`;
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // Overview tab helper functions
  function getSimulationProgress(): number {
    if (job.status === 'CREATED' || job.status === 'PENDING') return 0;
    if (job.status === 'COMPLETED') return 100;
    if (job.status === 'FAILED') return 75; // Assume it failed at 75%
    return 45; // Running jobs show 45% progress
  }

  function getCompletedSteps(): number {
    const total = getTotalSteps();
    return Math.floor(total * (getSimulationProgress() / 100));
  }

  function getTotalSteps(): number {
    return job.namdConfig?.numSteps || 1000000;
  }

  function getEstimatedTimeRemaining(): string {
    if (job.status === 'COMPLETED') return 'Completed';
    if (job.status === 'FAILED') return 'Failed';
    if (job.status === 'CREATED' || job.status === 'PENDING') return 'Not started';
    return '1.23 hours remaining';
  }

  function getCpuUsage(): number {
    if (job.status === 'RUNNING') return 92.5;
    if (job.status === 'COMPLETED') return 100;
    return 0;
  }

  function getMemoryUsage(): number {
    if (job.status === 'RUNNING') return 76.8;
    if (job.status === 'COMPLETED') return 100;
    return 0;
  }

  function getGpuUsage(): number {
    if (job.status === 'RUNNING') return 88.2;
    if (job.status === 'COMPLETED') return 100;
    return 0;
  }

  function getDiskUsage(): number {
    if (job.status === 'RUNNING') return 34.5;
    if (job.status === 'COMPLETED') return 45.2;
    return 0;
  }

  function getPerformance(): string {
    if (job.status === 'RUNNING') return '0.0249 s/step';
    if (job.status === 'COMPLETED') return '0.0245 s/step (avg)';
    return '--';
  }

  // File handling functions
  function downloadFile(fileName: string) {
    console.log('Download file:', fileName);
    // Mock download functionality
    alert(`Downloading ${fileName}...`);
  }

</script>

<div class="namd-tabs-container namd-card">
  <div class="namd-tabs-header">
    <nav class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-5">
      {#each tabs as tab}
        <button
          class="namd-tab-button"
          class:active={activeTab === tab.id}
          on:click={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </div>

  <div class="namd-tab-content">
    {#if activeTab === 'overview'}
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

          <!-- Resource Usage -->
          <div class="overview-section">
            <h3>Resource Usage</h3>
            <div class="resource-grid">
              <div class="resource-card">
                <div class="resource-header">
                  <span class="resource-title">CPU Usage</span>
                  <span class="resource-percentage">{getCpuUsage().toFixed(1)}%</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill progress-blue" style="width: {getCpuUsage()}%"></div>
                </div>
                <div class="resource-details">128 cores</div>
              </div>

              <div class="resource-card">
                <div class="resource-header">
                  <span class="resource-title">Memory Usage</span>
                  <span class="resource-percentage">{getMemoryUsage().toFixed(1)}%</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill progress-green" style="width: {getMemoryUsage()}%"></div>
                </div>
                <div class="resource-details">512 GB</div>
              </div>

              <div class="resource-card">
                <div class="resource-header">
                  <span class="resource-title">GPU Usage</span>
                  <span class="resource-percentage">{getGpuUsage().toFixed(1)}%</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill progress-purple" style="width: {getGpuUsage()}%"></div>
                </div>
                <div class="resource-details">4 × V100</div>
              </div>

              <div class="resource-card">
                <div class="resource-header">
                  <span class="resource-title">Disk Usage</span>
                  <span class="resource-percentage">{getDiskUsage().toFixed(1)}%</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill progress-yellow" style="width: {getDiskUsage()}%"></div>
                </div>
                <div class="resource-details">2.5 TB</div>
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
                <span class="info-label">Runtime</span>
                <span class="info-value">{job.runtime || '--'}</span>
              </div>
              <div class="info-item">
                <span class="info-label">Wall Time Remaining</span>
                <span class="info-value">{job.wallTimeRemaining || '--'}</span>
              </div>
              <div class="info-item">
                <span class="info-label">Performance</span>
                <span class="info-value">{getPerformance()}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    {:else if activeTab === 'slurm-logs'}
      <div class="namd-tab-panel">
        {#if job.status === 'CREATED'}
          <div class="empty-logs">
            Logs will be available once the job starts running.
          </div>
        {:else}
          <div class="logs-container">
            <div class="logs-header">
              <div class="namd-tabs-nav namd-tabs-nav--grid namd-tabs-nav--grid-2">
                <button
                  class="namd-tab-button"
                  class:active={activeLogTab === 'stdout'}
                  on:click={() => activeLogTab = 'stdout'}
                >
                  Standard Output
                </button>
                <button
                  class="namd-tab-button"
                  class:active={activeLogTab === 'stderr'}
                  on:click={() => activeLogTab = 'stderr'}
                >
                  Standard Error
                </button>
              </div>

              <div class="log-actions">
                <button class="namd-button namd-button--outline log-action-btn" on:click={copyLogs}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
                    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
                  </svg>
                  Copy
                </button>
                <button class="namd-button namd-button--outline log-action-btn" on:click={downloadLogs}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7,10 12,15 17,10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                  </svg>
                  Download
                </button>
              </div>
            </div>

            <div class="log-content">
              {#if activeLogTab === 'stdout'}
                <div class="log-viewer">
                  <pre class="log-text">{getStdoutContent()}</pre>
                </div>
              {:else}
                <div class="log-viewer">
                  <pre class="log-text">{getStderrContent()}</pre>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'input-files'}
      <div class="namd-tab-panel">
        <div class="input-files-content">
          <div class="files-header">
            <h3>Input Files</h3>
            <div class="namd-text-sm namd-text-muted">
              Input files used for job: {job.name}
            </div>
          </div>

          <div class="files-grid">
            {#each mockInputFiles as file}
              <div class="file-card">
                <div class="file-card-content">
                  <div class="file-info">
                    <div class="file-icon-large">{getFileIcon(file.type)}</div>
                    <div class="file-details">
                      <div class="file-header-row">
                        <span class="file-name">{file.name}</span>
                        <span class="namd-file-type-badge {getTypeColor(file.type)}">
                          {getTypeLabel(file.type)}
                        </span>
                      </div>
                      <div class="file-description">{file.description}</div>
                      <div class="file-size">Size: {file.size}</div>
                    </div>
                  </div>

                  <button
                    class="download-button"
                    on:click={() => downloadFile(file.name)}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                      <polyline points="7,10 12,15 17,10"/>
                      <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    Download
                  </button>
                </div>
              </div>
            {/each}
          </div>

          <div class="file-summary">
            <div class="summary-title">File Summary</div>
            <div class="summary-grid">
              <div class="summary-item">
                <span class="summary-label">Structure Files:</span>
                <span class="summary-value">{mockInputFiles.filter(f => f.type === 'structure').length}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">Parameter Files:</span>
                <span class="summary-value">{mockInputFiles.filter(f => f.type === 'parameters').length}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">Configuration Files:</span>
                <span class="summary-value">{mockInputFiles.filter(f => f.type === 'configuration').length}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    {:else if activeTab === 'output-files'}
      <div class="namd-tab-panel">
        <div class="output-files-content">
          {#if job.status === 'CREATED' || job.status === 'PENDING'}
            <div class="empty-files">
              Output files will be available once the job starts running.
            </div>
          {:else}
            <!-- Available Files -->
            {#if mockOutputFiles.filter(f => f.available).length > 0}
              <div class="files-section">
                <h3>Available Files</h3>
                <div class="files-grid">
                  {#each mockOutputFiles.filter(f => f.available) as file}
                    <div class="file-card">
                      <div class="file-card-content">
                        <div class="file-info">
                          <div class="file-icon-large">{getFileIcon(file.type)}</div>
                          <div class="file-details">
                            <div class="file-header-row">
                              <span class="file-name">{file.name}</span>
                              <span class="namd-file-type-badge {getTypeColor(file.type)}">
                                {getTypeLabel(file.type)}
                              </span>
                            </div>
                            <div class="file-description">{file.description}</div>
                            <div class="file-metadata">
                              Size: {file.size} • Modified: {file.lastModified}
                            </div>
                          </div>
                        </div>

                        <button
                          class="download-button"
                          on:click={() => downloadFile(file.name)}
                        >
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                            <polyline points="7,10 12,15 17,10"/>
                            <line x1="12" y1="15" x2="12" y2="3"/>
                          </svg>
                          Download
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Unavailable/Expected Files -->
            {#if mockOutputFiles.filter(f => !f.available).length > 0}
              <div class="files-section">
                <h3>Expected Files</h3>
                <div class="namd-text-sm namd-text-muted expected-files-description">
                  These files will be available once the simulation produces them.
                </div>
                <div class="files-grid">
                  {#each mockOutputFiles.filter(f => !f.available) as file}
                    <div class="file-card unavailable">
                      <div class="file-card-content">
                        <div class="file-info">
                          <div class="file-icon-large">{getFileIcon(file.type)}</div>
                          <div class="file-details">
                            <div class="file-header-row">
                              <span class="file-name">{file.name}</span>
                              <span class="namd-file-type-badge {getTypeColor(file.type)}">
                                {getTypeLabel(file.type)}
                              </span>
                            </div>
                            <div class="file-description">{file.description}</div>
                          </div>
                        </div>

                        <button class="download-button" disabled>
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                            <polyline points="7,10 12,15 17,10"/>
                            <line x1="12" y1="15" x2="12" y2="3"/>
                          </svg>
                          Pending
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Output Summary -->
            <div class="file-summary">
              <div class="summary-title">Output Summary</div>
              <div class="summary-grid">
                <div class="summary-item">
                  <span class="summary-label">Trajectory Files:</span>
                  <span class="summary-value">{mockOutputFiles.filter(f => f.type === 'trajectory').length}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">Log Files:</span>
                  <span class="summary-value">{mockOutputFiles.filter(f => f.type === 'log').length}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">Analysis Files:</span>
                  <span class="summary-value">{mockOutputFiles.filter(f => f.type === 'analysis').length}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">Checkpoint Files:</span>
                  <span class="summary-value">{mockOutputFiles.filter(f => f.type === 'checkpoint').length}</span>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {:else if activeTab === 'configuration'}
      <div class="namd-tab-panel">
        <div class="configuration-content">
          <!-- NAMD Configuration -->
          <div class="config-section">
            <h3>NAMD Configuration</h3>

            <!-- Basic Settings -->
            <div class="config-subsection">
              <h4>Basic Settings</h4>
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
                  <div class="config-value">{namdConfig.dcdFreq.toLocaleString()}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Restart Frequency</div>
                  <div class="config-value">{namdConfig.restartFreq.toLocaleString()}</div>
                </div>
              </div>
            </div>

            <!-- Input Files -->
            <div class="config-subsection">
              <h4>Input Files</h4>
              <div class="input-files-config">
                <div class="input-file-item">
                  <span class="input-file-label">Coordinates:</span>
                  <span class="input-file-value">{namdConfig.coordinates}</span>
                </div>
                <div class="input-file-item">
                  <span class="input-file-label">Structure:</span>
                  <span class="input-file-value">{namdConfig.structure}</span>
                </div>
                <div class="input-file-item">
                  <span class="input-file-label">Parameters:</span>
                  <div class="parameter-badges">
                    {#each namdConfig.parameters as param}
                      <span class="parameter-badge">{param}</span>
                    {/each}
                  </div>
                </div>
              </div>
            </div>

            <!-- Force Field & Cutoffs -->
            <div class="config-subsection">
              <h4>Force Field & Cutoffs</h4>
              <div class="config-grid-3">
                <div class="config-item">
                  <div class="config-label">Cutoff (Å)</div>
                  <div class="config-value">{namdConfig.cutoff}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Switch Distance (Å)</div>
                  <div class="config-value">{namdConfig.switchDist}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Pairlist Distance (Å)</div>
                  <div class="config-value">{namdConfig.pairlistDist}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">PME</div>
                  <div class="config-badge {namdConfig.PME ? 'enabled' : 'disabled'}">
                    {namdConfig.PME ? 'Enabled' : 'Disabled'}
                  </div>
                </div>
                <div class="config-item">
                  <div class="config-label">PME Grid Spacing (Å)</div>
                  <div class="config-value">{namdConfig.PMEGridSpacing}</div>
                </div>
              </div>
            </div>

            <!-- Dynamics Settings -->
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
                  <div class="config-value">{namdConfig.langevinDamping} ps⁻¹</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Langevin Temperature (K)</div>
                  <div class="config-value">{namdConfig.langevinTemp}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Langevin Hydrogen</div>
                  <div class="config-badge {namdConfig.langevinHydrogen ? 'enabled' : 'disabled'}">
                    {namdConfig.langevinHydrogen ? 'Enabled' : 'Disabled'}
                  </div>
                </div>
              </div>
            </div>

            <!-- Pressure Control -->
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
                  <div class="config-value">{namdConfig.langevinPistonTarget}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Piston Period (fs)</div>
                  <div class="config-value">{namdConfig.langevinPistonPeriod}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Piston Decay (fs)</div>
                  <div class="config-value">{namdConfig.langevinPistonDecay}</div>
                </div>
                <div class="config-item">
                  <div class="config-label">Piston Temperature (K)</div>
                  <div class="config-value">{namdConfig.langevinPistonTemp}</div>
                </div>
              </div>
            </div>
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
              <div class="config-item">
                <div class="config-label">Account</div>
                <div class="config-value">{slurmConfig.account}</div>
              </div>
              <div class="config-item">
                <div class="config-label">Nodes</div>
                <div class="config-value">{slurmConfig.nodes}</div>
              </div>
              <div class="config-item">
                <div class="config-label">Tasks per Node</div>
                <div class="config-value">{slurmConfig.tasksPerNode}</div>
              </div>
              <div class="config-item">
                <div class="config-label">CPUs per Task</div>
                <div class="config-value">{slurmConfig.cpusPerTask}</div>
              </div>
              <div class="config-item">
                <div class="config-label">GPUs</div>
                <div class="config-value">{slurmConfig.gpus} × {slurmConfig.gpuType.toUpperCase()}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .job-tabs {
    display: flex;
    flex-direction: column;
    min-height: 400px;
  }

  .tabs-header {
    border-bottom: 1px solid var(--namd-border-muted);
    flex-shrink: 0;
  }



  .progress-info p {
    margin: var(--namd-spacing-sm) 0;
    color: var(--namd-text-secondary);
  }

  /* Overview Tab Styles */
  .overview-content {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xl);
  }

  .overview-section h3 {
    margin: 0 0 var(--namd-spacing-md) 0;
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-medium);
  }

  .progress-card {
    padding: var(--namd-spacing-md);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--namd-spacing-sm);
  }

  .progress-label {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .progress-value {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-muted);
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background-color: var(--namd-bg-muted);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: var(--namd-spacing-sm);
  }

  .progress-fill {
    height: 100%;
    background-color: #3b82f6;
    transition: width 0.3s ease;
    border-radius: 4px;
  }

  .progress-blue { background-color: #3b82f6; }
  .progress-green { background-color: #10b981; }
  .progress-purple { background-color: #8b5cf6; }
  .progress-yellow { background-color: #f59e0b; }

  .progress-details {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
  }

  .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .resource-card {
    padding: var(--namd-spacing-md);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .resource-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--namd-spacing-sm);
  }

  .resource-title {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .resource-percentage {
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    color: var(--namd-text-muted);
  }

  .resource-details {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    margin-top: var(--namd-spacing-sm);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-md);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .info-label {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    font-weight: var(--namd-font-weight-medium);
  }

  .info-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
  }



  /* File components styles */
  .input-files-content {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .files-header h3 {
    margin: 0 0 var(--namd-spacing-xs) 0;
  }


  .files-grid {
    display: grid;
    gap: var(--namd-spacing-md);
  }

  .file-card {
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    background-color: var(--namd-bg-primary);
    transition: all 0.2s ease;
  }

  .file-card:hover {
    background-color: var(--namd-bg-muted);
  }

  .file-card-content {
    padding: var(--namd-spacing-md);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .file-info {
    display: flex;
    align-items: flex-start;
    gap: var(--namd-spacing-md);
    flex: 1;
  }

  .file-icon-large {
    font-size: 1.5rem;
    line-height: 1;
  }

  .file-details {
    flex: 1;
    min-width: 0;
  }

  .file-header-row {
    display: inline-flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
    margin-bottom: var(--namd-spacing-xs);
    min-height: 1.5rem;
  }

  .file-name {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    font-family: var(--namd-font-mono);
    color: var(--namd-text-primary);
    line-height: 1.5;
  }

  .namd-file-type-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    line-height: 1;
    white-space: nowrap;
  }

  .file-description {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    margin-bottom: var(--namd-spacing-xs);
  }

  .file-size {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
  }

  .download-button {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    background-color: transparent;
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-sm);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .download-button:hover {
    background-color: var(--namd-bg-muted);
    border-color: var(--namd-border-hover);
  }

  .file-summary {
    padding: var(--namd-spacing-md);
    background-color: rgba(243, 244, 246, 0.3);
    border-radius: var(--namd-border-radius);
  }

  .summary-title {
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
    margin-bottom: var(--namd-spacing-sm);
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .summary-item {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
  }

  .summary-label {
    font-weight: var(--namd-font-weight-medium);
  }

  .summary-value {
    color: var(--namd-text-primary);
  }


  /* Output files styles */
  .output-files-content {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xl);
  }

  .empty-files {
    text-align: center;
    padding: var(--namd-spacing-2xl);
    color: var(--namd-text-muted);
  }

  .files-section {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .files-section h3 {
    margin: 0 0 var(--namd-spacing-sm) 0;
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-primary);
  }

  .expected-files-description {
    margin-bottom: var(--namd-spacing-md);
  }

  .file-card.unavailable {
    opacity: 0.6;
  }

  .file-metadata {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
  }

  .download-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .download-button:disabled:hover {
    background-color: transparent;
    border-color: var(--namd-border);
  }

  /* Configuration tab styles */
  .configuration-content {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xl);
  }

  .config-section {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-lg);
  }

  .config-section h3 {
    margin: 0;
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-lg);
    font-weight: var(--namd-font-weight-medium);
    padding-bottom: var(--namd-spacing-sm);
    border-bottom: 1px solid var(--namd-border);
  }

  .config-subsection {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .config-subsection h4 {
    margin: 0;
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-md);
    font-weight: var(--namd-font-weight-medium);
  }

  .config-grid-3 {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--namd-spacing-md);
  }

  .config-item {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .config-label {
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-muted);
    font-weight: var(--namd-font-weight-medium);
  }

  .config-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
  }

  .config-badge {
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    text-transform: uppercase;
    width: fit-content;
  }

  .config-badge.enabled {
    background-color: #dcfce7;
    color: #166534;
  }

  .config-badge.disabled {
    background-color: #f3f4f6;
    color: #6b7280;
  }

  .input-files-config {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .input-file-item {
    display: flex;
    align-items: flex-start;
    gap: var(--namd-spacing-sm);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
  }

  .input-file-label {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-muted);
    font-weight: var(--namd-font-weight-medium);
    min-width: 100px;
  }

  .input-file-value {
    font-size: var(--namd-font-size-sm);
    color: var(--namd-text-primary);
    font-family: var(--namd-font-mono);
  }

  .parameter-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--namd-spacing-xs);
  }

  .parameter-badge {
    padding: 2px 6px;
    background-color: #f3e8ff;
    color: #6b21a8;
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    font-family: var(--namd-font-mono);
  }

  .empty-logs {
    text-align: center;
    padding: var(--namd-spacing-2xl);
    color: var(--namd-text-muted);
  }

  .logs-container {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-md);
  }

  .logs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }


  .log-actions {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .log-action-btn {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-xs);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    font-size: var(--namd-font-size-sm);
  }

  .log-content {
    margin-top: var(--namd-spacing-md);
  }

  .log-viewer {
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    background-color: rgba(243, 244, 246, 0.3); /* bg-muted/30 equivalent */
    height: 600px; /* Increased from 384px to 600px */
    overflow: auto;
    padding: var(--namd-spacing-md);
  }

  .log-text {
    margin: 0;
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-xs);
    color: var(--namd-text-primary);
    white-space: pre-wrap;
    line-height: 1.4;
  }

  .files-list {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .file-row {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
    padding: var(--namd-spacing-sm);
    background-color: var(--namd-bg-muted);
    border-radius: var(--namd-border-radius-sm);
  }

  .file-icon {
    font-size: 1.25rem;
  }

  .file-name {
    flex: 1;
    font-family: var(--namd-font-mono);
    font-size: var(--namd-font-size-sm);
  }

  .file-size {
    color: var(--namd-text-muted);
    font-size: var(--namd-font-size-xs);
  }

  .config-grid {
    display: flex;
    flex-direction: column;
    gap: var(--namd-spacing-sm);
  }

  .config-row {
    display: flex;
    justify-content: space-between;
    padding: var(--namd-spacing-sm) 0;
    border-bottom: 1px solid var(--namd-border-muted);
  }

  .config-label {
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-text-secondary);
  }

  .config-value {
    font-family: var(--namd-font-mono);
    color: var(--namd-text-primary);
  }

  .no-files, .no-config {
    color: var(--namd-text-muted);
    font-style: italic;
    text-align: center;
    padding: var(--namd-spacing-xl);
  }

  @media (max-width: 768px) {
    .tabs-nav {
      grid-template-columns: repeat(5, 1fr);
    }

    .tab-button {
      padding: var(--namd-spacing-sm);
      font-size: var(--namd-font-size-xs);
    }
  }
</style>
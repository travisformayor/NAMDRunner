/**
 * Centralized mock data for demo mode and testing
 * All tab components should import from here instead of defining mock data inline
 */

import type { JobInfo } from '../../types/api';

// Helper to create NAMD config with defaults
function createMockNAMDConfig(overrides: Partial<JobInfo['namd_config']> = {}) {
  return {
    outputname: 'output',
    temperature: 300.0,
    timestep: 2.0,
    execution_mode: 'run' as const,
    steps: 10000,
    cell_basis_vector1: undefined,
    cell_basis_vector2: undefined,
    cell_basis_vector3: undefined,
    pme_enabled: false,
    npt_enabled: false,
    langevin_damping: 5.0,
    xst_freq: 100,
    output_energies_freq: 100,
    dcd_freq: 100,
    restart_freq: 500,
    output_pressure_freq: 100,
    ...overrides
  };
}

// Mock job data for UI development and testing - one example of each status
export const mockJobs: JobInfo[] = [
  {
    job_id: 'job_001',
    job_name: 'protein_folding_simulation',
    status: 'RUNNING',
    slurm_job_id: '12345678',
    created_at: '2024-01-15T09:30:00Z',
    updated_at: '2024-01-15T09:35:00Z',
    submitted_at: '2024-01-15T09:35:00Z',
    namd_config: createMockNAMDConfig({
      steps: 100000,
      outputname: 'protein_output',
      dcd_freq: 1000,
      restart_freq: 1000
    }),
    slurm_config: {
      cores: 24,
      memory: '16GB',
      walltime: '04:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'protein.pdb', local_path: '/local/protein.pdb', file_type: 'pdb' },
      { name: 'protein.psf', local_path: '/local/protein.psf', file_type: 'psf' },
      { name: 'par_all36_prot.prm', local_path: '/local/par_all36_prot.prm', file_type: 'prm' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_001'
  },
  {
    job_id: 'job_002',
    job_name: 'membrane_dynamics',
    status: 'COMPLETED',
    slurm_job_id: '12345677',
    created_at: '2024-01-14T14:20:00Z',
    updated_at: '2024-01-14T18:50:00Z',
    submitted_at: '2024-01-14T14:25:00Z',
    completed_at: '2024-01-14T18:50:00Z',
    namd_config: createMockNAMDConfig({
      steps: 200000,
      temperature: 310,
      timestep: 2.0,
      outputname: 'membrane_output',
      dcd_freq: 1000,
      restart_freq: 1000
    }),
    slurm_config: {
      cores: 48,
      memory: '32GB',
      walltime: '06:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'membrane.pdb', local_path: '/local/membrane.pdb', file_type: 'pdb' },
      { name: 'membrane.psf', local_path: '/local/membrane.psf', file_type: 'psf' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_002'
  },
  {
    job_id: 'job_003',
    job_name: 'drug_binding_analysis',
    status: 'PENDING',
    slurm_job_id: '12345679',
    created_at: '2024-01-15T11:45:00Z',
    updated_at: '2024-01-15T11:50:00Z',
    submitted_at: '2024-01-15T11:50:00Z',
    namd_config: createMockNAMDConfig({
      steps: 500000,
      temperature: 300,
      timestep: 1.0,
      outputname: 'drug_analysis',
      dcd_freq: 2000,
      restart_freq: 2000
    }),
    slurm_config: {
      cores: 96,
      memory: '64GB',
      walltime: '08:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'complex.pdb', local_path: '/local/complex.pdb', file_type: 'pdb' },
      { name: 'complex.psf', local_path: '/local/complex.psf', file_type: 'psf' },
      { name: 'drug.pdb', local_path: '/local/drug.pdb', file_type: 'pdb' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_003'
  },
  {
    job_id: 'job_004',
    job_name: 'enzyme_study_draft',
    status: 'CREATED',
    created_at: '2024-01-15T14:30:00Z',
    updated_at: '2024-01-15T14:30:00Z',
    namd_config: createMockNAMDConfig({
      steps: 75000,
      temperature: 310,
      timestep: 2.0,
      outputname: 'enzyme_output',
      dcd_freq: 1000,
      restart_freq: 1000
    }),
    slurm_config: {
      cores: 16,
      memory: '12GB',
      walltime: '02:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'enzyme.pdb', local_path: '/local/enzyme.pdb', file_type: 'pdb' },
      { name: 'enzyme.psf', local_path: '/local/enzyme.psf', file_type: 'psf' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_004'
  },
  {
    job_id: 'job_005',
    job_name: 'lipid_bilayer_crashed',
    status: 'FAILED',
    slurm_job_id: '12345680',
    created_at: '2024-01-14T10:15:00Z',
    updated_at: '2024-01-14T14:20:00Z',
    submitted_at: '2024-01-14T10:20:00Z',
    completed_at: '2024-01-14T14:20:00Z',
    error_info: 'Simulation crashed due to atom overlap',
    namd_config: createMockNAMDConfig({
      steps: 150000,
      temperature: 310,
      timestep: 2.5,
      outputname: 'bilayer_output',
      dcd_freq: 1500,
      restart_freq: 1500
    }),
    slurm_config: {
      cores: 32,
      memory: '24GB',
      walltime: '03:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'bilayer.pdb', local_path: '/local/bilayer.pdb', file_type: 'pdb' },
      { name: 'bilayer.psf', local_path: '/local/bilayer.psf', file_type: 'psf' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_005'
  }
];

// Mock SLURM logs for SlurmLogsTab
export const mockStdout = `NAMD 3.0beta3 for LINUX-X86_64-multicore
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

export const mockStderr = `Info: Startup phase 0 took 0.00123 s, 128 KB of memory in use
Info: Startup phase 1 took 0.00456 s, 256 KB of memory in use
Info: Startup phase 2 took 0.00789 s, 512 KB of memory in use
Info: Startup phase 3 took 0.01234 s, 1024 KB of memory in use
Info: Startup phase 4 took 0.02345 s, 2048 KB of memory in use
Info: Startup phase 5 took 0.03456 s, 4096 KB of memory in use
Info: Startup completed at 0.0678 s, 8192 KB of memory in use`;

// Mock input files for InputFilesTab
export const mockInputFiles = [
  {
    name: "protein.pdb",
    path: "input_files/protein.pdb",
    size: "2.3 MB",
    type: "structure",
    description: "Protein structure file"
  },
  {
    name: "protein.psf",
    path: "input_files/protein.psf",
    size: "1.8 MB",
    type: "structure",
    description: "Protein structure file (PSF format)"
  },
  {
    name: "par_all36_prot.prm",
    path: "input_files/par_all36_prot.prm",
    size: "456 KB",
    type: "parameters",
    description: "CHARMM36 protein parameters"
  },
  {
    name: "par_all36_lipid.prm",
    path: "input_files/par_all36_lipid.prm",
    size: "234 KB",
    type: "parameters",
    description: "CHARMM36 lipid parameters"
  },
  {
    name: "toppar_water_ions.str",
    path: "input_files/toppar_water_ions.str",
    size: "123 KB",
    type: "parameters",
    description: "Water and ion parameters"
  },
  {
    name: "simulation.conf",
    path: "input_files/simulation.conf",
    size: "4.2 KB",
    type: "configuration",
    description: "NAMD configuration file"
  }
];

// Mock output files for OutputFilesTab (function to support job status-based availability)
export function getMockOutputFiles(jobStatus: string) {
  return [
    {
      name: "trajectory.dcd",
      path: "outputs/trajectory.dcd",
      size: "1.2 GB",
      type: "trajectory",
      description: "Molecular dynamics trajectory",
      lastModified: "2024-01-15 12:45",
      available: jobStatus === "COMPLETED" || jobStatus === "RUNNING"
    },
    {
      name: "output.log",
      path: "outputs/output.log",
      size: "45 MB",
      type: "log",
      description: "NAMD output log file",
      lastModified: "2024-01-15 12:45",
      available: jobStatus !== "CREATED" && jobStatus !== "PENDING"
    },
    {
      name: "energy.log",
      path: "outputs/energy.log",
      size: "23 MB",
      type: "analysis",
      description: "Energy analysis output",
      lastModified: "2024-01-15 12:45",
      available: jobStatus === "COMPLETED" || jobStatus === "RUNNING"
    },
    {
      name: "restart.coor",
      path: "outputs/restart.coor",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart coordinates",
      lastModified: "2024-01-15 12:30",
      available: jobStatus === "COMPLETED" || jobStatus === "RUNNING"
    },
    {
      name: "restart.vel",
      path: "outputs/restart.vel",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart velocities",
      lastModified: "2024-01-15 12:30",
      available: jobStatus === "COMPLETED" || jobStatus === "RUNNING"
    },
    {
      name: "restart.xsc",
      path: "outputs/restart.xsc",
      size: "1 KB",
      type: "checkpoint",
      description: "Extended system coordinates",
      lastModified: "2024-01-15 12:30",
      available: jobStatus === "COMPLETED" || jobStatus === "RUNNING"
    }
  ];
}

// Mock NAMD configuration for ConfigurationTab (demo mode only - shows detailed NAMD config)
export const mockNAMDConfig = {
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

// Mock SLURM configuration for ConfigurationTab (demo mode only)
export const mockSlurmConfig = {
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

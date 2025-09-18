import { writable, derived } from 'svelte/store';
import type { Job, JobStatus } from '../types/api';

// Mock job data for UI development and testing
const mockJobs: Job[] = [
  {
    jobId: 'job_001',
    jobName: 'protein_folding_simulation',
    status: 'RUNNING',
    slurmJobId: '12345678',
    createdAt: '2024-01-15T09:30:00Z',
    updatedAt: '2024-01-15T09:35:00Z',
    submittedAt: '2024-01-15T09:35:00Z',
    namdConfig: {
      temperature: 300,
      timestep: 2.0,
      numSteps: 100000,
      outputFreq: 1000,
      restartFreq: 1000,
      dcdFreq: 1000
    },
    slurmConfig: {
      partition: 'amilan',
      nodes: 1,
      ntasks: 24,
      time: '04:00:00',
      mem: '16GB',
      account: 'ucb-general'
    },
    inputFiles: [
      { fileName: 'protein.pdb', size: 524288, uploadedAt: '2024-01-15T09:30:00Z' },
      { fileName: 'protein.psf', size: 1048576, uploadedAt: '2024-01-15T09:30:00Z' },
      { fileName: 'par_all36_prot.prm', size: 262144, uploadedAt: '2024-01-15T09:30:00Z' }
    ],
    runtime: '02:15:30',
    wallTimeRemaining: '01:44:30'
  },
  {
    jobId: 'job_002',
    jobName: 'membrane_dynamics',
    status: 'COMPLETED',
    slurmJobId: '12345677',
    createdAt: '2024-01-14T14:20:00Z',
    updatedAt: '2024-01-14T18:50:00Z',
    submittedAt: '2024-01-14T14:25:00Z',
    completedAt: '2024-01-14T18:50:00Z',
    namdConfig: {
      temperature: 310,
      timestep: 2.0,
      numSteps: 200000,
      outputFreq: 1000,
      restartFreq: 1000,
      dcdFreq: 1000
    },
    slurmConfig: {
      partition: 'amilan',
      nodes: 2,
      ntasks: 48,
      time: '06:00:00',
      mem: '32GB',
      account: 'ucb-general'
    },
    inputFiles: [
      { fileName: 'membrane.pdb', size: 2097152, uploadedAt: '2024-01-14T14:20:00Z' },
      { fileName: 'membrane.psf', size: 4194304, uploadedAt: '2024-01-14T14:20:00Z' }
    ],
    runtime: '04:25:30',
    wallTimeRemaining: '00:00:00'
  },
  {
    jobId: 'job_003',
    jobName: 'drug_binding_analysis',
    status: 'PENDING',
    slurmJobId: '12345679',
    createdAt: '2024-01-15T11:45:00Z',
    updatedAt: '2024-01-15T11:50:00Z',
    submittedAt: '2024-01-15T11:50:00Z',
    namdConfig: {
      temperature: 300,
      timestep: 1.0,
      numSteps: 500000,
      outputFreq: 2000,
      restartFreq: 2000,
      dcdFreq: 2000
    },
    slurmConfig: {
      partition: 'amilan',
      nodes: 4,
      ntasks: 96,
      time: '08:00:00',
      mem: '64GB',
      account: 'ucb-general'
    },
    inputFiles: [
      { fileName: 'complex.pdb', size: 1572864, uploadedAt: '2024-01-15T11:45:00Z' },
      { fileName: 'complex.psf', size: 3145728, uploadedAt: '2024-01-15T11:45:00Z' },
      { fileName: 'drug.pdb', size: 32768, uploadedAt: '2024-01-15T11:45:00Z' }
    ],
    runtime: '--',
    wallTimeRemaining: '--'
  },
  {
    jobId: 'job_004',
    jobName: 'enzyme_kinetics',
    status: 'FAILED',
    slurmJobId: '12345676',
    createdAt: '2024-01-15T08:15:00Z',
    updatedAt: '2024-01-15T08:50:00Z',
    submittedAt: '2024-01-15T08:20:00Z',
    failedAt: '2024-01-15T08:50:00Z',
    namdConfig: {
      temperature: 298,
      timestep: 2.0,
      numSteps: 150000,
      outputFreq: 1500,
      restartFreq: 1500,
      dcdFreq: 1500
    },
    slurmConfig: {
      partition: 'amilan',
      nodes: 1,
      ntasks: 12,
      time: '02:00:00',
      mem: '8GB',
      account: 'ucb-general'
    },
    inputFiles: [
      { fileName: 'enzyme.pdb', size: 786432, uploadedAt: '2024-01-15T08:15:00Z' },
      { fileName: 'enzyme.psf', size: 1572864, uploadedAt: '2024-01-15T08:15:00Z' }
    ],
    runtime: '00:30:15',
    wallTimeRemaining: '00:00:00',
    errorMessage: 'NAMD configuration error: Invalid temperature setting'
  },
  {
    jobId: 'job_005',
    jobName: 'dna_replication_study',
    status: 'CREATED',
    createdAt: '2024-01-15T12:30:00Z',
    updatedAt: '2024-01-15T12:30:00Z',
    namdConfig: {
      temperature: 300,
      timestep: 2.0,
      numSteps: 250000,
      outputFreq: 1000,
      restartFreq: 1000,
      dcdFreq: 1000
    },
    slurmConfig: {
      partition: 'amilan',
      nodes: 2,
      ntasks: 48,
      time: '12:00:00',
      mem: '48GB',
      account: 'ucb-general'
    },
    inputFiles: [
      { fileName: 'dna.pdb', size: 3145728, uploadedAt: '2024-01-15T12:30:00Z' },
      { fileName: 'dna.psf', size: 6291456, uploadedAt: '2024-01-15T12:30:00Z' }
    ],
    runtime: '--',
    wallTimeRemaining: '--'
  }
];

// Jobs store state with sync timing
interface JobsState {
  jobs: Job[];
  lastSyncTime: Date;
  isSyncing: boolean;
}

// Initialize with realistic mock sync time (15 minutes ago)
const initialJobsState: JobsState = {
  jobs: mockJobs,
  lastSyncTime: new Date(Date.now() - 15 * 60 * 1000), // 15 minutes ago
  isSyncing: false
};

// Create jobs store
function createJobsStore() {
  const { subscribe, set, update } = writable<JobsState>(initialJobsState);

  return {
    subscribe,
    // For UI testing - add a new job
    addJob: (job: Job) => update(state => ({
      ...state,
      jobs: [...state.jobs, job]
    })),
    // For UI testing - update job status
    updateJobStatus: (jobId: string, status: JobStatus) => update(state => ({
      ...state,
      jobs: state.jobs.map(job => job.jobId === jobId ? {
        ...job,
        status,
        updatedAt: new Date().toISOString(),
        ...(status === 'COMPLETED' && { completedAt: new Date().toISOString() }),
        ...(status === 'FAILED' && { failedAt: new Date().toISOString() })
      } : job)
    })),
    // For UI testing - remove a job
    removeJob: (jobId: string) => update(state => ({
      ...state,
      jobs: state.jobs.filter(job => job.jobId !== jobId)
    })),
    // Sync with backend (mock for now)
    sync: async () => {
      // Set syncing state
      update(state => ({ ...state, isSyncing: true }));

      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Update sync time and clear syncing state
      update(state => ({
        ...state,
        lastSyncTime: new Date(),
        isSyncing: false
      }));

      console.log('Syncing jobs with backend...');
    },
    // Reset to initial mock data
    reset: () => set(initialJobsState)
  };
}

export const jobsStore = createJobsStore();

// Derived stores for convenience
export const jobs = derived(jobsStore, $store => $store.jobs);
export const lastSyncTime = derived(jobsStore, $store => $store.lastSyncTime);
export const isSyncing = derived(jobsStore, $store => $store.isSyncing);

export const selectedJob = derived(
  [jobs, writable<string | null>(null)],
  ([$jobs, $selectedId]) => $selectedId ? $jobs.find(job => job.jobId === $selectedId) : null
);

export const jobsByStatus = derived(jobs, $jobs => {
  const grouped = {
    CREATED: [] as Job[],
    PENDING: [] as Job[],
    RUNNING: [] as Job[],
    COMPLETED: [] as Job[],
    FAILED: [] as Job[],
    CANCELLED: [] as Job[]
  };

  $jobs.forEach(job => {
    grouped[job.status].push(job);
  });

  return grouped;
});

export const jobCounts = derived(jobsByStatus, $grouped => ({
  total: Object.values($grouped).flat().length,
  running: $grouped.RUNNING.length,
  pending: $grouped.PENDING.length,
  completed: $grouped.COMPLETED.length,
  failed: $grouped.FAILED.length
}));
import { writable, derived } from 'svelte/store';
import type { JobInfo, JobStatus, CreateJobParams } from '../types/api';
import { CoreClientFactory } from '../ports/clientFactory';
import { listen } from '@tauri-apps/api/event';

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
    namd_config: {
      steps: 100000,
      temperature: 300,
      timestep: 2.0,
      outputname: 'protein_output',
      dcd_freq: 1000,
      restart_freq: 1000
    },
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
    namd_config: {
      steps: 200000,
      temperature: 310,
      timestep: 2.0,
      outputname: 'membrane_output',
      dcd_freq: 1000,
      restart_freq: 1000
    },
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
    namd_config: {
      steps: 500000,
      temperature: 300,
      timestep: 1.0,
      outputname: 'drug_analysis',
      dcd_freq: 2000,
      restart_freq: 2000
    },
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
    namd_config: {
      steps: 75000,
      temperature: 310,
      timestep: 2.0,
      outputname: 'enzyme_output',
      dcd_freq: 1000,
      restart_freq: 1000
    },
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
    updated_at: '2024-01-14T11:45:00Z',
    submitted_at: '2024-01-14T10:20:00Z',
    error_info: 'Job failed during execution',
    namd_config: {
      steps: 300000,
      temperature: 300,
      timestep: 2.5,
      outputname: 'lipid_output',
      dcd_freq: 1500,
      restart_freq: 1500
    },
    slurm_config: {
      cores: 32,
      memory: '24GB',
      walltime: '05:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'lipid.pdb', local_path: '/local/lipid.pdb', file_type: 'pdb' },
      { name: 'lipid.psf', local_path: '/local/lipid.psf', file_type: 'psf' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_005'
  },
  {
    job_id: 'job_006',
    job_name: 'canceled_test_run',
    status: 'CANCELLED',
    slurm_job_id: '12345681',
    created_at: '2024-01-15T08:00:00Z',
    updated_at: '2024-01-15T08:15:00Z',
    submitted_at: '2024-01-15T08:05:00Z',
    namd_config: {
      steps: 50000,
      temperature: 298,
      timestep: 1.5,
      outputname: 'test_output',
      dcd_freq: 500,
      restart_freq: 500
    },
    slurm_config: {
      cores: 8,
      memory: '8GB',
      walltime: '01:00:00',
      partition: 'amilan'
    },
    input_files: [
      { name: 'test.pdb', local_path: '/local/test.pdb', file_type: 'pdb' },
      { name: 'test.psf', local_path: '/local/test.psf', file_type: 'psf' }
    ],
    remote_directory: '/projects/mockuser/namdrunner_jobs/job_006'
  }
];

// Progress tracking interface
interface JobProgress {
  message: string;
  isActive: boolean;
}

// Jobs store state with sync timing and progress tracking
interface JobsState {
  jobs: JobInfo[];
  lastSyncTime: Date;
  hasEverSynced: boolean;
  isSyncing: boolean;
  creationProgress: JobProgress;
  submissionProgress: JobProgress;
}

// Initialize with empty state - jobs will be loaded when connected
const initialJobsState: JobsState = {
  jobs: [],
  lastSyncTime: new Date(0), // No sync yet
  hasEverSynced: false,
  isSyncing: false,
  creationProgress: { message: '', isActive: false },
  submissionProgress: { message: '', isActive: false }
};

// Create jobs store
function createJobsStore() {
  const { subscribe, set, update } = writable<JobsState>(initialJobsState);

  return {
    subscribe,
    // Sync with backend
    sync: async () => {
      // Log user action to SSH console
      if (typeof window !== 'undefined' && window.sshConsole) {
        window.sshConsole.addDebug('[SYNC] User clicked Sync Now button');
      }

      // Set syncing state
      update(state => ({ ...state, isSyncing: true }));

      const currentMode = CoreClientFactory.getUserMode();

      try {
        if (currentMode === 'demo') {
          // Demo mode: simulate sync but keep the same mockJobs for consistency
          // Add a small delay to simulate network activity
          await new Promise(resolve => setTimeout(resolve, 300));

          update(state => ({
            ...state,
            jobs: mockJobs, // Always use the same mockJobs in demo mode
            lastSyncTime: new Date(),
            hasEverSynced: true,
            isSyncing: false
          }));
          // Demo mode: simulated sync completed
        } else {
          // Real mode: Call syncJobs to update job statuses from SLURM, then fetch updated jobs
          if (typeof window !== 'undefined' && window.sshConsole) {
            window.sshConsole.addDebug('[SYNC] Starting job status sync with SLURM cluster');
          }

          const syncResult = await CoreClientFactory.getClient().syncJobs();

          if (syncResult.success) {
            // Pure caching - backend returns complete job list (discovery happens automatically if DB empty)
            if (typeof window !== 'undefined' && window.sshConsole) {
              window.sshConsole.addDebug(`[SYNC] Sync completed - ${syncResult.jobs_updated} job(s) updated, ${syncResult.jobs.length} total jobs`);
            }

            update(state => ({
              ...state,
              jobs: syncResult.jobs || [],
              lastSyncTime: new Date(),
              hasEverSynced: true,
              isSyncing: false
            }));
          } else {
            // Sync failed - log error and keep existing jobs
            if (typeof window !== 'undefined' && window.sshConsole) {
              window.sshConsole.addDebug(`[SYNC] Job sync failed: ${syncResult.errors.join(', ')}`);
            }
            update(state => ({
              ...state,
              lastSyncTime: new Date(),
              hasEverSynced: true,
              isSyncing: false
            }));
          }
        }
      } catch (error) {
        // Keep existing jobs but update sync time on error
        update(state => ({
          ...state,
          lastSyncTime: new Date(),
          isSyncing: false
        }));
        // Sync error - maintaining existing jobs
        if (typeof window !== 'undefined' && window.sshConsole) {
          window.sshConsole.addDebug(`[JOBS] Sync error: ${error}`);
        }
      }
    },

    // Create a new job via backend with progress tracking
    createJob: async (params: CreateJobParams) => {
      // Set up progress tracking
      update(state => ({
        ...state,
        creationProgress: { message: 'Starting job creation...', isActive: true }
      }));

      // Listen for progress events from the automation system
      const unlisten = await listen('job-creation-progress', (event) => {
        const message = event.payload as string;
        update(state => ({
          ...state,
          creationProgress: { message, isActive: true }
        }));
      });

      try {
        const result = await CoreClientFactory.getClient().createJob(params);

        if (result.success && result.job_id && result.job) {
          // Update progress to completion
          update(state => ({
            ...state,
            creationProgress: { message: 'Job created successfully!', isActive: false }
          }));

          // Add the returned job directly to the store (no second backend call)
          update(state => ({
            ...state,
            jobs: [...state.jobs, result.job!]
          }));

          return result;
        } else {
          // Job creation failed - error shown in UI
          if (typeof window !== 'undefined' && window.sshConsole) {
            window.sshConsole.addDebug(`[JOBS] Job creation failed: ${result.error}`);
          }
          update(state => ({
            ...state,
            creationProgress: { message: `Job creation failed: ${result.error}`, isActive: false }
          }));
          return result;
        }
      } catch (error) {
        // Job creation error - error shown in UI
        if (typeof window !== 'undefined' && window.sshConsole) {
          window.sshConsole.addDebug(`[JOBS] Job creation error: ${error}`);
        }
        update(state => ({
          ...state,
          creationProgress: { message: 'Job creation failed due to unexpected error', isActive: false }
        }));
        return { success: false, error: 'Job creation failed' };
      } finally {
        // Clean up event listener
        unlisten();
      }
    },

    // Submit a job for execution via backend with progress tracking
    submitJob: async (job_id: string) => {
      // Set up progress tracking
      update(state => ({
        ...state,
        submissionProgress: { message: 'Starting job submission...', isActive: true }
      }));

      // Listen for progress events from the automation system
      const unlisten = await listen('job-submission-progress', (event) => {
        const message = event.payload as string;
        update(state => ({
          ...state,
          submissionProgress: { message, isActive: true }
        }));
      });

      try {
        const result = await CoreClientFactory.getClient().submitJob(job_id);

        if (result.success) {
          // Update progress to completion
          update(state => ({
            ...state,
            submissionProgress: { message: 'Job submitted successfully!', isActive: false },
            jobs: state.jobs.map(job => {
              if (job.job_id === job_id) {
                const updatedJob: JobInfo = {
                  ...job,
                  status: 'PENDING' as JobStatus,
                  submitted_at: result.submitted_at || new Date().toISOString(),
                  updated_at: new Date().toISOString()
                };
                if (result.slurm_job_id) {
                  updatedJob.slurm_job_id = result.slurm_job_id;
                }
                return updatedJob;
              }
              return job;
            })
          }));
        } else {
          update(state => ({
            ...state,
            submissionProgress: { message: `Job submission failed: ${result.error}`, isActive: false }
          }));
        }

        return result;
      } catch (error) {
        // Job submission error - handled by UI state
        if (typeof window !== 'undefined' && window.sshConsole) {
          window.sshConsole.addDebug(`[JOBS] Job submission error: ${error}`);
        }
        update(state => ({
          ...state,
          submissionProgress: { message: 'Job submission failed due to unexpected error', isActive: false }
        }));
        return { success: false, error: 'Job submission failed' };
      } finally {
        // Clean up event listener
        unlisten();
      }
    },

    // Delete a job via backend
    deleteJob: async (job_id: string) => {
      try {
        const result = await CoreClientFactory.getClient().deleteJob(job_id, true);

        if (result.success) {
          // Remove job from local state
          update(state => ({
            ...state,
            jobs: state.jobs.filter(job => job.job_id !== job_id)
          }));
        }

        return result;
      } catch (error) {
        // Job deletion error - handled by UI state
        if (typeof window !== 'undefined' && window.sshConsole) {
          window.sshConsole.addDebug(`[JOBS] Job deletion error: ${error}`);
        }
        return { success: false, error: 'Job deletion failed' };
      }
    },

    // Get detailed job status via backend
    getJobStatus: async (job_id: string) => {
      try {
        const result = await CoreClientFactory.getClient().getJobStatus(job_id);

        if (result.success && result.job_info) {
          // Update the specific job in local state
          update(state => ({
            ...state,
            jobs: state.jobs.map(job => job.job_id === job_id ? result.job_info! : job)
          }));
        }

        return result;
      } catch (error) {
        // Job status check error - handled by UI state
        return { success: false, error: 'Job status check failed' };
      }
    },

    // Reset to initial mock data
    reset: () => set(initialJobsState),

    // Load demo jobs for offline/demo mode
    loadDemoJobs: () => update(state => ({
      ...state,
      jobs: mockJobs,
      lastSyncTime: new Date(0), // Never synced - demo data
      hasEverSynced: false,
    })),

    // Clear all jobs
    clearJobs: () => update(state => ({
      ...state,
      jobs: [],
      lastSyncTime: new Date(0),
      hasEverSynced: false,
    }))
  };
}

export const jobsStore = createJobsStore();

// Derived stores for convenience
export const jobs = derived(jobsStore, $store => $store.jobs);
export const lastSyncTime = derived(jobsStore, $store => $store.lastSyncTime);
export const hasEverSynced = derived(jobsStore, $store => $store.hasEverSynced);
export const isSyncing = derived(jobsStore, $store => $store.isSyncing);

// Progress tracking stores
export const creationProgress = derived(jobsStore, $store => $store.creationProgress);
export const submissionProgress = derived(jobsStore, $store => $store.submissionProgress);

export const selectedJob = derived(
  [jobs, writable<string | null>(null)],
  ([$jobs, $selectedId]) => $selectedId ? $jobs.find(job => job.job_id === $selectedId) : null
);

export const jobsByStatus = derived(jobs, $jobs => {
  const grouped = {
    CREATED: [] as JobInfo[],
    PENDING: [] as JobInfo[],
    RUNNING: [] as JobInfo[],
    COMPLETED: [] as JobInfo[],
    FAILED: [] as JobInfo[],
    CANCELLED: [] as JobInfo[]
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
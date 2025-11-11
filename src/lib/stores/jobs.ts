import { logger } from '$lib/utils/logger';
import { writable, derived } from 'svelte/store';
import type {
  JobInfo,
  JobStatus,
  CreateJobParams,
  GetAllJobsResult,
  SyncJobsResult,
  CreateJobResult,
  SubmitJobResult,
  DeleteJobResult,
  JobStatusResult
} from '../types/api';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { sessionActions } from './session';

// Helper: Detect if error indicates connection failure
function isConnectionError(errorMessage: string): boolean {
  const msg = errorMessage.toLowerCase();
  return msg.includes('timeout') ||
         msg.includes('not connected') ||
         msg.includes('connection') ||
         msg.includes('broken pipe') ||
         msg.includes('network') ||
         msg.includes('ssh');
}

// Helper: Handle connection failure by updating session state
function handleConnectionFailure(error: string) {
  if (isConnectionError(error)) {
    sessionActions.markExpired(error);
  }
}

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

    // Load jobs from database (for offline/startup)
    loadFromDatabase: async () => {
      try {
        const result = await invoke<GetAllJobsResult>('get_all_jobs');

        if (result.success && result.jobs) {
          update(state => ({
            ...state,
            jobs: result.jobs || [],
            hasEverSynced: !!(result.jobs && result.jobs.length > 0)
          }));
        }
      } catch (error) {
        logger.error('[Jobs]', 'Failed to load from database', error);
      }
    },

    // Sync with backend
    sync: async () => {
      // Log user action to SSH console

      // Set syncing state
      update(state => ({ ...state, isSyncing: true }));

      try {
        // Call syncJobs to update job statuses from SLURM, then fetch updated jobs

        const syncResult = await invoke<SyncJobsResult>('sync_jobs');

        if (syncResult.success) {
          // Pure caching - backend returns complete job list (discovery happens automatically if DB empty)
          update(state => ({
            ...state,
            jobs: syncResult.jobs || [],
            lastSyncTime: new Date(),
            hasEverSynced: true,
            isSyncing: false
          }));
        } else {
          // Sync failed - check if it's a connection error
          const errorMsg = syncResult.errors.join(', ');
          handleConnectionFailure(errorMsg);

          update(state => ({
            ...state,
            lastSyncTime: new Date(),
            hasEverSynced: true,
            isSyncing: false
          }));
        }
      } catch (error) {
        // Check if exception indicates connection failure
        const errorMsg = error instanceof Error ? error.message : String(error);
        handleConnectionFailure(errorMsg);

        update(state => ({
          ...state,
          lastSyncTime: new Date(),
          isSyncing: false
        }));
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
        const result = await invoke<CreateJobResult>('create_job', { params });

        if (result.success && result.job_id && result.job) {
          // Update progress to completion
          update(state => ({
            ...state,
            creationProgress: { message: 'Job created successfully!', isActive: false }
          }));

          // Add the returned job directly to the store (no second backend call)
          if (result.job) {
            update(state => ({
              ...state,
              jobs: [...state.jobs, result.job as JobInfo]
            }));
          }

          return result;
        } else {
          // Job creation failed - check for connection error
          const errorMsg = result.error || 'Job creation failed';
          handleConnectionFailure(errorMsg);

          update(state => ({
            ...state,
            creationProgress: { message: `Job creation failed: ${errorMsg}`, isActive: false }
          }));
          return result;
        }
      } catch (error) {
        // Job creation error - check for connection error
        const errorMsg = error instanceof Error ? error.message : String(error);
        handleConnectionFailure(errorMsg);

        update(state => ({
          ...state,
          creationProgress: { message: 'Job creation failed due to unexpected error', isActive: false }
        }));
        return { success: false, error: errorMsg };
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
        const result = await invoke<SubmitJobResult>('submit_job', { job_id });

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
          // Submission failed - check for connection error
          const errorMsg = result.error || 'Job submission failed';
          handleConnectionFailure(errorMsg);

          update(state => ({
            ...state,
            submissionProgress: { message: `Job submission failed: ${errorMsg}`, isActive: false }
          }));
        }

        return result;
      } catch (error) {
        // Job submission error - check for connection error
        const errorMsg = error instanceof Error ? error.message : String(error);
        handleConnectionFailure(errorMsg);

        update(state => ({
          ...state,
          submissionProgress: { message: 'Job submission failed due to unexpected error', isActive: false }
        }));
        return { success: false, error: errorMsg };
      } finally {
        // Clean up event listener
        unlisten();
      }
    },

    // Delete a job via backend
    deleteJob: async (job_id: string) => {
      try {
        const result = await invoke<DeleteJobResult>('delete_job', { job_id, delete_remote: true });

        if (result.success) {
          // Remove job from local state
          update(state => ({
            ...state,
            jobs: state.jobs.filter(job => job.job_id !== job_id)
          }));
        } else {
          // Deletion failed - check for connection error
          const errorMsg = result.error || 'Job deletion failed';
          handleConnectionFailure(errorMsg);
        }

        return result;
      } catch (error) {
        // Job deletion error - check for connection error
        const errorMsg = error instanceof Error ? error.message : String(error);
        handleConnectionFailure(errorMsg);

        return { success: false, error: errorMsg };
      }
    },

    // Get detailed job status via backend
    getJobStatus: async (job_id: string) => {
      try {
        const result = await invoke<JobStatusResult>('get_job_status', { job_id });

        if (result.success && result.job_info) {
          // Update the specific job in local state
          update(state => ({
            ...state,
            jobs: state.jobs.map(job => job.job_id === job_id ? result.job_info as JobInfo : job)
          }));
        } else {
          // Status check failed - check for connection error
          const errorMsg = result.error || 'Job status check failed';
          handleConnectionFailure(errorMsg);
        }

        return result;
      } catch (error) {
        // Job status check error - check for connection error
        const errorMsg = error instanceof Error ? error.message : String(error);
        handleConnectionFailure(errorMsg);
        return { success: false, error: errorMsg };
      }
    },

    // Reset to initial state
    reset: () => set(initialJobsState),

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
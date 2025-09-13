import type { JobInfo, JobStatus, CreateJobParams } from '../../types/api';

/**
 * Job fixture utilities for consistent test data
 * Based on realistic NAMD simulation patterns and SLURM behavior
 */

export interface JobScenario {
  name: string;
  description: string;
  jobInfo: JobInfo;
  createParams?: CreateJobParams;
}

// Realistic timestamp helpers
function hoursAgo(hours: number): string {
  return new Date(Date.now() - hours * 60 * 60 * 1000).toISOString();
}

function minutesAgo(minutes: number): string {
  return new Date(Date.now() - minutes * 60 * 1000).toISOString();
}

function now(): string {
  return new Date().toISOString();
}

// Base job creation parameters for testing
export const baseCreateJobParams: CreateJobParams = {
  jobName: 'test_simulation',
  namdConfig: {
    steps: 50000,
    temperature: 310.0,
    timestep: 2.0,
    outputname: 'output',
    dcdFreq: 1000,
    restartFreq: 5000,
  },
  slurmConfig: {
    cores: 24,
    memory: '16GB',
    walltime: '02:00:00',
    partition: 'amilan',
    qos: 'normal',
  },
  inputFiles: [
    {
      name: 'structure.pdb',
      localPath: '/tmp/structure.pdb',
      remoteName: 'structure.pdb',
      type: 'pdb',
      fileType: 'pdb',
    },
    {
      name: 'structure.psf',
      localPath: '/tmp/structure.psf',
      remoteName: 'structure.psf',
      type: 'psf',
      fileType: 'psf',
    },
    {
      name: 'parameters.prm',
      localPath: '/tmp/parameters.prm',
      remoteName: 'parameters.prm',
      type: 'prm',
      fileType: 'prm',
    },
  ],
};

// Job fixture scenarios covering different lifecycle states
export const jobFixtures: Record<string, JobScenario> = {
  // Fresh job, just created locally
  freshJob: {
    name: 'Fresh Job',
    description: 'Newly created job, not yet submitted to SLURM',
    jobInfo: {
      jobId: 'job_001',
      jobName: 'fresh_simulation',
      status: 'CREATED' as JobStatus,
      createdAt: minutesAgo(5),
      updatedAt: minutesAgo(5),
      projectDir: '/projects/testuser/namdrunner_jobs/job_001',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_001',
    },
    createParams: {
      ...baseCreateJobParams,
      jobName: 'fresh_simulation',
    },
  },

  // Job submitted and waiting in SLURM queue
  pendingJob: {
    name: 'Pending Job',
    description: 'Job submitted to SLURM, waiting for resources',
    jobInfo: {
      jobId: 'job_002',
      jobName: 'pending_simulation',
      status: 'PENDING' as JobStatus,
      slurmJobId: '12345678',
      createdAt: hoursAgo(2),
      updatedAt: minutesAgo(10),
      submittedAt: minutesAgo(15),
      projectDir: '/projects/testuser/namdrunner_jobs/job_002',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_002',
    },
  },

  // Currently running job
  runningJob: {
    name: 'Running Job',
    description: 'Job currently executing on cluster',
    jobInfo: {
      jobId: 'job_003',
      jobName: 'running_simulation',
      status: 'RUNNING' as JobStatus,
      slurmJobId: '12345679',
      createdAt: hoursAgo(3),
      updatedAt: minutesAgo(5),
      submittedAt: hoursAgo(2),
      projectDir: '/projects/testuser/namdrunner_jobs/job_003',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_003',
    },
  },

  // Successfully completed job
  completedJob: {
    name: 'Completed Job',
    description: 'Job finished successfully',
    jobInfo: {
      jobId: 'job_004',
      jobName: 'completed_simulation',
      status: 'COMPLETED' as JobStatus,
      slurmJobId: '12345680',
      createdAt: hoursAgo(6),
      updatedAt: hoursAgo(1),
      submittedAt: hoursAgo(5),
      completedAt: hoursAgo(1),
      projectDir: '/projects/testuser/namdrunner_jobs/job_004',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_004',
    },
  },

  // Failed job with error information
  failedJob: {
    name: 'Failed Job',
    description: 'Job that failed during execution',
    jobInfo: {
      jobId: 'job_005',
      jobName: 'failed_simulation',
      status: 'FAILED' as JobStatus,
      slurmJobId: '12345681',
      createdAt: hoursAgo(8),
      updatedAt: hoursAgo(2),
      submittedAt: hoursAgo(7),
      completedAt: hoursAgo(2),
      projectDir: '/projects/testuser/namdrunner_jobs/job_005',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_005',
    },
  },

  // Cancelled job
  cancelledJob: {
    name: 'Cancelled Job',
    description: 'Job cancelled by user or system',
    jobInfo: {
      jobId: 'job_006',
      jobName: 'cancelled_simulation',
      status: 'CANCELLED' as JobStatus,
      slurmJobId: '12345682',
      createdAt: hoursAgo(4),
      updatedAt: minutesAgo(30),
      submittedAt: hoursAgo(3),
      completedAt: minutesAgo(30),
      projectDir: '/projects/testuser/namdrunner_jobs/job_006',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_006',
    },
  },

  // Large simulation job with different parameters
  largeJob: {
    name: 'Large Job',
    description: 'Large-scale simulation with many cores and long runtime',
    jobInfo: {
      jobId: 'job_007',
      jobName: 'large_membrane_system',
      status: 'RUNNING' as JobStatus,
      slurmJobId: '12345683',
      createdAt: hoursAgo(12),
      updatedAt: minutesAgo(2),
      submittedAt: hoursAgo(10),
      projectDir: '/projects/testuser/namdrunner_jobs/job_007',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_007',
    },
    createParams: {
      ...baseCreateJobParams,
      jobName: 'large_membrane_system',
      namdConfig: {
        ...baseCreateJobParams.namdConfig,
        steps: 1000000, // 1M steps
        dcdFreq: 5000,
      },
      slurmConfig: {
        cores: 64,
        memory: '128GB',
        walltime: '7-00:00:00', // 7 days with long QoS
        partition: 'amilan',
        qos: 'long',
      },
    },
  },

  // Job with special characters in name (edge case testing)
  specialNameJob: {
    name: 'Special Characters Job',
    description: 'Job with special characters and Unicode in name',
    jobInfo: {
      jobId: 'job_008',
      jobName: 'test-job_v2.1_α-helix',
      status: 'CREATED' as JobStatus,
      createdAt: minutesAgo(1),
      updatedAt: minutesAgo(1),
      projectDir: '/projects/testuser/namdrunner_jobs/job_008',
      scratchDir: '/scratch/alpine/testuser/namdrunner_jobs/job_008',
    },
  },
};

// Job progression scenarios for testing state transitions
export const jobProgressionScenarios = {
  // Scenario: Job progresses from CREATED → PENDING → RUNNING → COMPLETED
  successfulProgression: [
    { ...jobFixtures.freshJob!.jobInfo, status: 'CREATED' as JobStatus },
    { 
      ...jobFixtures.freshJob!.jobInfo, 
      status: 'PENDING' as JobStatus,
      slurmJobId: '98765432',
      submittedAt: now(),
      updatedAt: now(),
    },
    {
      ...jobFixtures.freshJob!.jobInfo,
      status: 'RUNNING' as JobStatus,
      slurmJobId: '98765432',
      submittedAt: minutesAgo(10),
      updatedAt: now(),
    },
    {
      ...jobFixtures.freshJob!.jobInfo,
      status: 'COMPLETED' as JobStatus,
      slurmJobId: '98765432',
      submittedAt: hoursAgo(2),
      completedAt: now(),
      updatedAt: now(),
    },
  ],

  // Scenario: Job fails during execution
  failureProgression: [
    { ...jobFixtures.freshJob!.jobInfo, status: 'CREATED' as JobStatus },
    {
      ...jobFixtures.freshJob!.jobInfo,
      status: 'PENDING' as JobStatus,
      slurmJobId: '98765433',
      submittedAt: now(),
      updatedAt: now(),
    },
    {
      ...jobFixtures.freshJob!.jobInfo,
      status: 'RUNNING' as JobStatus,
      slurmJobId: '98765433',
      submittedAt: minutesAgo(30),
      updatedAt: now(),
    },
    {
      ...jobFixtures.freshJob!.jobInfo,
      status: 'FAILED' as JobStatus,
      slurmJobId: '98765433',
      submittedAt: hoursAgo(1),
      completedAt: now(),
      updatedAt: now(),
    },
  ],
};

// Helper functions for testing
export function getJobsByStatus(status: JobStatus): JobInfo[] {
  return Object.values(jobFixtures)
    .map(scenario => scenario.jobInfo)
    .filter(job => job.status === status);
}

export function getRandomJob(): JobInfo {
  const scenarios = Object.values(jobFixtures);
  const randomIndex = Math.floor(Math.random() * scenarios.length);
  return scenarios[randomIndex].jobInfo;
}

export function createJobWithTimestamp(baseJob: JobInfo, hoursAgo: number): JobInfo {
  const timestamp = new Date(Date.now() - hoursAgo * 60 * 60 * 1000).toISOString();
  return {
    ...baseJob,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

// Realistic SLURM job ID generator for testing
export function generateRealisticSlurmId(): string {
  const base = 10000000; // SLURM jobs start around this range
  const random = Math.floor(Math.random() * 90000000);
  return (base + random).toString();
}

// Generate a batch of test jobs for bulk operations testing
export function generateTestJobBatch(count: number): JobInfo[] {
  const jobs: JobInfo[] = [];
  const scenarios = Object.values(jobFixtures);
  
  for (let i = 0; i < count; i++) {
    const baseScenario = scenarios[i % scenarios.length];
    const job: JobInfo = {
      ...baseScenario.jobInfo,
      jobId: `job_batch_${(i + 1).toString().padStart(3, '0')}`,
      jobName: `batch_test_${i + 1}`,
      createdAt: hoursAgo(Math.random() * 24), // Random creation time in last 24h
      updatedAt: minutesAgo(Math.random() * 60), // Random update time in last hour
    };
    
    jobs.push(job);
  }
  
  return jobs;
}
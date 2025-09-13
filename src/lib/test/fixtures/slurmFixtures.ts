import type { JobStatus, JobId } from '../../types/api';

/**
 * SLURM command response fixtures for testing
 * Based on real SLURM output patterns from Colorado Research Computing
 */

export interface SlurmCommandResponse {
  command: string;
  exitCode: number;
  stdout: string;
  stderr: string;
  delay?: number; // Simulated network delay in ms
}

export interface SlurmJobInfo {
  jobId: string;
  name: string;
  state: string;
  nodes: string;
  cpus: string;
  memory: string;
  partition: string;
  qos: string;
  user: string;
  workdir: string;
  runtime?: string;
  submitTime?: string;
  startTime?: string;
  endTime?: string;
  exitCode?: string;
}

// Realistic SLURM job IDs (Colorado RC typically uses 8-digit numbers)
export const slurmJobIds = {
  pending: '12345678',
  running: '12345679',
  completed: '12345680',
  failed: '12345681',
  cancelled: '12345682',
  timeout: '12345683',
  out_of_memory: '12345684',
};

// Alpine cluster partitions and QOS (see docs/cluster-guide.md for more details)
export const slurmPartitions = {
  amilan: 'amilan',         // General compute nodes (32/48/64 cores)
  amilan128c: 'amilan128c', // High-core compute (128 cores)
  amem: 'amem',            // High-memory nodes (48/64/128 cores)
  aa100: 'aa100',          // NVIDIA A100 GPU nodes
  ami100: 'ami100',        // AMD MI100 GPU nodes
  al40: 'al40',            // NVIDIA L40 GPU nodes
  csu: 'csu',              // CSU contributed nodes
  atesting: 'atesting',    // CPU testing partition
  atesting_a100: 'atesting_a100',   // A100 GPU testing
  atesting_mi100: 'atesting_mi100', // MI100 GPU testing
  acompile: 'acompile',    // Compilation partition
};

export const slurmQOS = {
  normal: 'normal',     // Default QoS (1 day max)
  long: 'long',         // Extended runtime (7 days max)
  mem: 'mem',           // High-memory QoS (amem partition, 7 days)
  testing: 'testing',   // Testing partitions (1 hour max)
  compile: 'compile',   // Compilation QoS (12 hours max)
  csu: 'csu',          // CSU nodes
};

// SBATCH command responses - simplified for practical testing
export const sbatchFixtures: Record<string, SlurmCommandResponse> = {
  successfulSubmission: {
    command: 'sbatch job.sbatch',
    exitCode: 0,
    stdout: 'Submitted batch job 12345678',
    stderr: '',
    delay: 300,
  },

  submissionFailure: {
    command: 'sbatch job.sbatch',
    exitCode: 1,
    stdout: '',
    stderr: 'sbatch: error: Batch job submission failed',
    delay: 200,
  },
};

// SQUEUE command responses - simplified for practical testing
export const squeueFixtures: Record<string, SlurmCommandResponse> = {
  pendingJob: {
    command: `squeue --job=${slurmJobIds.pending} --format="%A|%j|%T|%M|%L|%D|%C|%m|%P|%Z" --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.pending}|test_simulation|PENDING|0:00|2:00:00|1|24|16384M|amilan|/mock/job/dir`,
    stderr: '',
    delay: 100,
  },

  runningJob: {
    command: `squeue --job=${slurmJobIds.running} --format="%A|%j|%T|%M|%L|%D|%C|%m|%P|%Z" --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.running}|test_simulation|RUNNING|0:15:30|1:44:30|1|24|16384M|amilan|/mock/job/dir`,
    stderr: '',
    delay: 100,
  },

  jobNotFound: {
    command: `squeue --job=${slurmJobIds.completed} --format="%A|%j|%T|%M|%L|%D|%C|%m|%P|%Z" --parsable2 --noheader`,
    exitCode: 1,
    stdout: '',
    stderr: 'squeue: error: Invalid job id specified',
    delay: 100,
  },
};

// SACCT command responses (for job history and completion status)
export const sacctFixtures: Record<string, SlurmCommandResponse> = {
  // Successfully completed job
  completedJob: {
    command: `sacct --job=${slurmJobIds.completed} --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.completed}|test_simulation|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T10:05:00|2025-01-15T11:00:00|00:55:00|/mock/job/dir`,
    stderr: '',
    delay: 150,
  },

  // Failed job with non-zero exit code
  failedJob: {
    command: `sacct --job=${slurmJobIds.failed} --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.failed}|failed_simulation|FAILED|1:0|2025-01-15T08:00:00|2025-01-15T08:10:00|2025-01-15T08:25:00|00:15:00|/scratch/alpine/testuser/namdrunner_jobs/job_005`,
    stderr: '',
    delay: 350,
  },

  // Cancelled job
  cancelledJob: {
    command: `sacct --job=${slurmJobIds.cancelled} --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.cancelled}|cancelled_simulation|CANCELLED|0:15|2025-01-15T12:00:00|2025-01-15T12:05:00|2025-01-15T12:30:00|00:25:00|/scratch/alpine/testuser/namdrunner_jobs/job_006`,
    stderr: '',
    delay: 300,
  },

  // Job timed out
  timeoutJob: {
    command: `sacct --job=${slurmJobIds.timeout} --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.timeout}|timeout_simulation|TIMEOUT|0:0|2025-01-15T06:00:00|2025-01-15T06:10:00|2025-01-15T08:10:00|02:00:00|/scratch/alpine/testuser/namdrunner_jobs/job_007`,
    stderr: '',
    delay: 250,
  },

  // Out of memory job
  outOfMemoryJob: {
    command: `sacct --job=${slurmJobIds.out_of_memory} --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 0,
    stdout: `${slurmJobIds.out_of_memory}|oom_simulation|OUT_OF_MEMORY|0:125|2025-01-15T14:00:00|2025-01-15T14:05:00|2025-01-15T14:45:00|00:40:00|/scratch/alpine/testuser/namdrunner_jobs/job_008`,
    stderr: '',
    delay: 200,
  },

  // Job not found in history
  jobNotFoundHistory: {
    command: `sacct --job=99999999 --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader`,
    exitCode: 1,
    stdout: '',
    stderr: 'SLURM accounting storage is unavailable',
    delay: 100,
  },

  // Multiple jobs query
  multipleJobsHistory: {
    command: 'sacct --user=testuser --starttime=2025-01-15 --format=JobID,JobName,State,ExitCode,Submit,Start,End,Elapsed,WorkDir --parsable2 --noheader',
    exitCode: 0,
    stdout: [
      `${slurmJobIds.completed}|completed_simulation|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T10:05:00|2025-01-15T11:00:00|00:55:00|/scratch/alpine/testuser/namdrunner_jobs/job_004`,
      `${slurmJobIds.failed}|failed_simulation|FAILED|1:0|2025-01-15T08:00:00|2025-01-15T08:10:00|2025-01-15T08:25:00|00:15:00|/scratch/alpine/testuser/namdrunner_jobs/job_005`,
      `${slurmJobIds.cancelled}|cancelled_simulation|CANCELLED|0:15|2025-01-15T12:00:00|2025-01-15T12:05:00|2025-01-15T12:30:00|00:25:00|/scratch/alpine/testuser/namdrunner_jobs/job_006`,
    ].join('\n'),
    stderr: '',
    delay: 600,
  },
};

// SCANCEL command responses (for job cancellation)
export const scancelFixtures: Record<string, SlurmCommandResponse> = {
  successfulCancel: {
    command: `scancel ${slurmJobIds.running}`,
    exitCode: 0,
    stdout: '',
    stderr: '',
    delay: 300,
  },

  jobNotFound: {
    command: `scancel 99999999`,
    exitCode: 1,
    stdout: '',
    stderr: 'scancel: error: Kill job error on job id 99999999: Invalid job id specified',
    delay: 200,
  },

  permissionDenied: {
    command: `scancel ${slurmJobIds.running}`,
    exitCode: 1,
    stdout: '',
    stderr: 'scancel: error: Kill job error on job id 12345679: Access/permission denied',
    delay: 250,
  },

  alreadyCompleted: {
    command: `scancel ${slurmJobIds.completed}`,
    exitCode: 1,
    stdout: '',
    stderr: 'scancel: error: Kill job error on job id 12345680: Job/step already completing or completed',
    delay: 150,
  },
};

// Helper functions to parse SLURM output
export function parseSqueueOutput(stdout: string): SlurmJobInfo[] {
  if (!stdout.trim()) return [];
  
  return stdout.trim().split('\n').map(line => {
    const [jobId, name, state, runtime, timelimit, nodes, cpus, memory, partition, workdir] = line.split('|');
    return {
      jobId,
      name,
      state,
      nodes,
      cpus,
      memory,
      partition,
      qos: 'normal', // Default, not in squeue output
      user: 'testuser',
      workdir,
      runtime,
    };
  });
}

export function parseSacctOutput(stdout: string): SlurmJobInfo[] {
  if (!stdout.trim()) return [];
  
  return stdout.trim().split('\n').map(line => {
    const [jobId, name, state, exitCode, submitTime, startTime, endTime, runtime, workdir] = line.split('|');
    return {
      jobId,
      name,
      state,
      nodes: '1', // Not in sacct output, default
      cpus: '24', // Not in sacct output, default
      memory: '16384M', // Not in sacct output, default
      partition: 'amilan', // Not in sacct output, default
      qos: 'normal',
      user: 'testuser',
      workdir,
      submitTime,
      startTime,
      endTime,
      exitCode,
      runtime,
    };
  });
}

// Map SLURM states to NAMDRunner job statuses
export function mapSlurmStateToJobStatus(slurmState: string): JobStatus {
  const stateMap: Record<string, JobStatus> = {
    'PENDING': 'PENDING',
    'RUNNING': 'RUNNING',
    'COMPLETED': 'COMPLETED',
    'FAILED': 'FAILED',
    'CANCELLED': 'CANCELLED',
    'TIMEOUT': 'FAILED',
    'OUT_OF_MEMORY': 'FAILED',
    'NODE_FAIL': 'FAILED',
    'PREEMPTED': 'CANCELLED',
  };
  
  return stateMap[slurmState] || 'FAILED';
}

// Generate realistic SLURM job ID
export function generateSlurmJobId(): string {
  const base = 10000000; // Base SLURM job ID
  const random = Math.floor(Math.random() * 90000000);
  return (base + random).toString();
}

// Create SLURM command scenarios for testing
export const slurmCommandScenarios = {
  // Successful job submission and monitoring cycle
  successfulJobLifecycle: [
    sbatchFixtures.successfulSubmission,
    squeueFixtures.pendingJob,
    squeueFixtures.runningJob,
    squeueFixtures.jobNotFound, // Job completed, no longer in queue
    sacctFixtures.completedJob,
  ],

  // Job submission failure
  submissionFailure: [
    sbatchFixtures.insufficientResources,
  ],

  // Job fails during execution
  jobFailureDuringExecution: [
    sbatchFixtures.successfulSubmission,
    squeueFixtures.pendingJob,
    squeueFixtures.runningJob,
    squeueFixtures.jobNotFound,
    sacctFixtures.failedJob,
  ],

  // Job gets cancelled
  jobCancellation: [
    sbatchFixtures.successfulSubmission,
    squeueFixtures.pendingJob,
    scancelFixtures.successfulCancel,
    squeueFixtures.jobNotFound,
    sacctFixtures.cancelledJob,
  ],

  // System errors during monitoring
  systemErrorScenario: [
    sbatchFixtures.successfulSubmission,
    squeueFixtures.systemError,
    sacctFixtures.jobNotFoundHistory,
  ],
};

// Validate SLURM job parameters
export function validateSlurmJobParameters(params: {
  cores: number;
  memory: string;
  walltime: string;
  partition?: string;
  qos?: string;
}): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (params.cores < 1 || params.cores > 128) {
    errors.push('Cores must be between 1 and 128 (varies by partition)');
  }
  
  const memoryMatch = params.memory.match(/^(\d+)(GB?|MB?)$/i);
  if (!memoryMatch) {
    errors.push('Memory must be in format "16GB" or "1024MB"');
  }
  
  const walltimeMatch = params.walltime.match(/^(\d{1,2}):(\d{2}):(\d{2})$/);
  if (!walltimeMatch) {
    errors.push('Walltime must be in format "HH:MM:SS"');
  }
  
  if (params.partition && !Object.values(slurmPartitions).includes(params.partition)) {
    errors.push(`Invalid partition: ${params.partition}`);
  }
  
  if (params.qos && !Object.values(slurmQOS).includes(params.qos)) {
    errors.push(`Invalid QOS: ${params.qos}`);
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}

// Generate realistic SLURM script content for testing
export function generateSlurmScript(params: {
  jobName: string;
  cores: number;
  memory: string;
  walltime: string;
  partition: string;
  qos: string;
  workdir: string;
}): string {
  return `#!/bin/bash
#SBATCH --job-name=${params.jobName}
#SBATCH --nodes=1
#SBATCH --ntasks-per-node=${params.cores}
#SBATCH --mem=${params.memory}
#SBATCH --time=${params.walltime}
#SBATCH --partition=${params.partition}
#SBATCH --qos=${params.qos}
#SBATCH --output=${params.jobName}_%j.out
#SBATCH --error=${params.jobName}_%j.err

# Change to job directory
cd ${params.workdir}

# Load required modules
module load namd/3.0alpha13

# Run NAMD simulation
namd3 +p${params.cores} config.namd > namd_output.log 2>&1

echo "Job completed at $(date)"
`;
}
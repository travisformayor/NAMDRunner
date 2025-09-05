import type { ICoreClient } from './coreClient';
import type {
  ConnectParams,
  ConnectResult,
  DisconnectResult,
  ConnectionStatusResult,
  CreateJobParams,
  CreateJobResult,
  SubmitJobResult,
  JobStatusResult,
  GetAllJobsResult,
  SyncJobsResult,
  DeleteJobResult,
  FileUpload,
  UploadResult,
  DownloadResult,
  ListFilesResult,
  JobId,
  JobInfo,
  ConnectionState,
  JobStatus,
} from '../types/api';

/**
 * Mock implementation of the core client for offline development
 * Simulates realistic responses without requiring actual backend connection
 */
export class MockCoreClient implements ICoreClient {
  private connectionState: ConnectionState = 'Disconnected';
  private jobs: Map<JobId, JobInfo> = new Map();
  private jobCounter = 0;

  // Helper to simulate network delay
  private async delay(ms: number = 500): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // Generate a new job ID
  private generateJobId(): JobId {
    this.jobCounter++;
    return `job_${this.jobCounter.toString().padStart(3, '0')}`;
  }

  // Connection management
  async connect(params: ConnectParams): Promise<ConnectResult> {
    await this.delay(1000); // Simulate connection time

    // Simulate connection validation
    if (!params.host || !params.username || !params.password) {
      return {
        success: false,
        error: 'Missing required connection parameters',
      };
    }

    // Simulate connection failure for certain hosts
    if (params.host.includes('invalid')) {
      return {
        success: false,
        error: 'Connection failed: Host unreachable',
      };
    }

    this.connectionState = 'Connected';
    return {
      success: true,
      sessionInfo: {
        host: params.host,
        username: params.username,
        connectedAt: new Date().toISOString(),
      },
    };
  }

  async disconnect(): Promise<DisconnectResult> {
    await this.delay(200);
    this.connectionState = 'Disconnected';
    return {
      success: true,
    };
  }

  async getConnectionStatus(): Promise<ConnectionStatusResult> {
    await this.delay(100);
    return {
      state: this.connectionState,
      sessionInfo:
        this.connectionState === 'Connected'
          ? {
              host: 'login.rc.colorado.edu',
              username: 'testuser',
              connectedAt: new Date().toISOString(),
            }
          : undefined,
    };
  }

  // Job management
  async createJob(params: CreateJobParams): Promise<CreateJobResult> {
    await this.delay(300);

    // Validate job parameters
    if (!params.jobName || !params.namdConfig || !params.slurmConfig) {
      return {
        success: false,
        error: 'Missing required job parameters',
      };
    }

    const jobId = this.generateJobId();
    const now = new Date().toISOString();

    const jobInfo: JobInfo = {
      jobId,
      jobName: params.jobName,
      status: 'CREATED',
      createdAt: now,
      updatedAt: now,
      projectDir: `/projects/testuser/namdrunner_jobs/${jobId}`,
      scratchDir: `/scratch/alpine/testuser/namdrunner_jobs/${jobId}`,
    };

    this.jobs.set(jobId, jobInfo);

    return {
      success: true,
      jobId,
    };
  }

  async submitJob(jobId: JobId): Promise<SubmitJobResult> {
    await this.delay(800);

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (this.connectionState !== 'Connected') {
      return {
        success: false,
        error: 'Not connected to cluster',
      };
    }

    // Simulate SLURM job submission
    const slurmJobId = Math.floor(Math.random() * 10000000).toString();
    const submittedAt = new Date().toISOString();

    job.status = 'PENDING';
    job.slurmJobId = slurmJobId;
    job.submittedAt = submittedAt;
    job.updatedAt = submittedAt;

    return {
      success: true,
      slurmJobId,
      submittedAt,
    };
  }

  async getJobStatus(jobId: JobId): Promise<JobStatusResult> {
    await this.delay(200);

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    return {
      success: true,
      jobInfo: { ...job },
    };
  }

  async getAllJobs(): Promise<GetAllJobsResult> {
    await this.delay(150);

    return {
      success: true,
      jobs: Array.from(this.jobs.values()),
    };
  }

  async syncJobs(): Promise<SyncJobsResult> {
    await this.delay(1200);

    if (this.connectionState !== 'Connected') {
      return {
        success: false,
        jobsUpdated: 0,
        errors: ['Not connected to cluster'],
      };
    }

    // Simulate job status updates
    let updatesCount = 0;
    const now = new Date().toISOString();

    for (const job of this.jobs.values()) {
      if (job.status === 'PENDING' && Math.random() > 0.7) {
        job.status = 'RUNNING';
        job.updatedAt = now;
        updatesCount++;
      } else if (job.status === 'RUNNING' && Math.random() > 0.8) {
        job.status = 'COMPLETED';
        job.completedAt = now;
        job.updatedAt = now;
        updatesCount++;
      }
    }

    return {
      success: true,
      jobsUpdated: updatesCount,
      errors: [],
    };
  }

  async deleteJob(jobId: JobId, deleteRemote: boolean): Promise<DeleteJobResult> {
    await this.delay(deleteRemote ? 800 : 200);

    if (!this.jobs.has(jobId)) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    this.jobs.delete(jobId);

    return {
      success: true,
    };
  }

  // File management
  async uploadJobFiles(jobId: JobId, files: FileUpload[]): Promise<UploadResult> {
    await this.delay(files.length * 200); // Simulate upload time per file

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        failedUploads: [
          {
            fileName: 'unknown',
            error: `Job ${jobId} not found`,
          },
        ],
      };
    }

    if (this.connectionState !== 'Connected') {
      return {
        success: false,
        failedUploads: files.map((file) => ({
          fileName: file.remoteName,
          error: 'Not connected to cluster',
        })),
      };
    }

    // Simulate some successful and some failed uploads
    const uploadedFiles: string[] = [];
    const failedUploads: Array<{ fileName: string; error: string }> = [];

    for (const file of files) {
      if (Math.random() > 0.1) {
        // 90% success rate
        uploadedFiles.push(file.remoteName);
      } else {
        failedUploads.push({
          fileName: file.remoteName,
          error: 'Simulated upload failure',
        });
      }
    }

    return {
      success: failedUploads.length === 0,
      uploadedFiles,
      failedUploads: failedUploads.length > 0 ? failedUploads : undefined,
    };
  }

  async downloadJobOutput(jobId: JobId, fileName: string): Promise<DownloadResult> {
    await this.delay(600);

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (this.connectionState !== 'Connected') {
      return {
        success: false,
        error: 'Not connected to cluster',
      };
    }

    // Simulate file content
    const mockContent = `Mock content for ${fileName} from job ${jobId}\nGenerated at ${new Date().toISOString()}`;

    return {
      success: true,
      content: mockContent,
      filePath: `/tmp/${fileName}`,
      fileSize: mockContent.length,
    };
  }

  async listJobFiles(jobId: JobId): Promise<ListFilesResult> {
    await this.delay(400);

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (this.connectionState !== 'Connected') {
      return {
        success: false,
        error: 'Not connected to cluster',
      };
    }

    // Simulate file listing
    const mockFiles = [
      {
        name: 'config.namd',
        size: 2048,
        modifiedAt: new Date().toISOString(),
        fileType: 'config' as const,
      },
      {
        name: 'job.sbatch',
        size: 1024,
        modifiedAt: new Date().toISOString(),
        fileType: 'config' as const,
      },
      {
        name: 'output.log',
        size: 15360,
        modifiedAt: new Date().toISOString(),
        fileType: 'log' as const,
      },
    ];

    return {
      success: true,
      files: mockFiles,
    };
  }
}
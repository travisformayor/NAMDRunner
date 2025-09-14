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
  SessionInfo,
} from '../types/api';
import type { ConnectionStateManager, SessionManager } from '../types/connection';
import { getServiceContainer, setServiceContainer, createMockServiceContainer, type ServiceDependencies } from '../services/serviceContainer';
import { ErrorBuilder, CONNECTION_ERRORS } from '../types/errors';

/**
 * Enhanced mock implementation with realistic connection state management
 * Simulates complex connection scenarios for comprehensive testing
 */
export class MockCoreClient implements ICoreClient {
  private services: ServiceDependencies;
  private jobs: Map<JobId, JobInfo> = new Map();
  private jobCounter = 0;
  private errorInjectionEnabled = false;
  private errorRate = 0.05; // 5% default error rate

  constructor(services: ServiceDependencies) {
    this.services = services;
    
    // Initialize with some test data
    this.loadTestData();
  }

  // Configuration methods for testing
  enableErrorInjection(enabled: boolean, errorRate: number = 0.1): void {
    this.errorInjectionEnabled = enabled;
    this.errorRate = errorRate;
  }



  // Error injection helper
  private shouldInjectError(): boolean {
    return this.errorInjectionEnabled && Math.random() < this.errorRate;
  }

  // Generate a new job ID
  private generateJobId(): JobId {
    this.jobCounter++;
    return `job_${this.jobCounter.toString().padStart(3, '0')}`;
  }

  // Connection management with enhanced state handling
  async connect(params: ConnectParams): Promise<ConnectResult> {
    // Transition to connecting state
    const transitionResult = this.services.stateManager.transitionTo('Connecting', 'User initiated connection');
    if (!transitionResult.success) {
      return {
        success: false,
        error: `Cannot connect: ${transitionResult.error!.message}`,
      };
    }


    // Validate parameters
    if (!params.host || !params.username || !params.password) {
      this.services.stateManager.transitionTo('Disconnected', 'Invalid parameters');
      return {
        success: false,
        error: 'Missing required connection parameters',
      };
    }

    // Simulate various connection failure scenarios
    if (this.shouldInjectError()) {
      this.services.stateManager.transitionTo('Disconnected', 'Random error injection');
      const errorTemplates = [
        CONNECTION_ERRORS.NETWORK_UNREACHABLE,
        CONNECTION_ERRORS.CONNECTION_TIMEOUT,
        CONNECTION_ERRORS.AUTH_FAILED,
      ];
      const randomError = errorTemplates[Math.floor(Math.random() * errorTemplates.length)];
      return {
        success: false,
        error: randomError.message,
      };
    }

    // Host-specific failure patterns
    if (params.host.includes('invalid') || params.host.includes('unreachable')) {
      this.services.stateManager.transitionTo('Disconnected', 'Host unreachable');
      return {
        success: false,
        error: CONNECTION_ERRORS.NETWORK_UNREACHABLE.message,
      };
    }

    if (params.host.includes('timeout')) {
      this.services.stateManager.transitionTo('Disconnected', 'Connection timeout');
      return {
        success: false,
        error: CONNECTION_ERRORS.CONNECTION_TIMEOUT.message,
      };
    }

    if (params.password === 'wrongpassword') {
      this.services.stateManager.transitionTo('Disconnected', 'Authentication failed');
      return {
        success: false,
        error: CONNECTION_ERRORS.AUTH_FAILED.message,
      };
    }

    // Successful connection
    const sessionInfo: SessionInfo = {
      host: params.host,
      username: params.username,
      connectedAt: new Date().toISOString(),
    };

    // Update state and session
    this.services.stateManager.transitionTo('Connected', 'Authentication successful');
    await this.services.sessionManager.saveSession(sessionInfo);

    return {
      success: true,
      sessionInfo,
    };
  }

  async disconnect(): Promise<DisconnectResult> {

    // Simulate occasional disconnect failures
    if (this.shouldInjectError()) {
      return {
        success: false,
        error: 'Failed to properly close connection: Network error',
      };
    }

    // Clear session and update state
    await this.services.sessionManager.clearSession();
    this.services.stateManager.transitionTo('Disconnected', 'User initiated disconnect');

    return {
      success: true,
    };
  }

  async getConnectionStatus(): Promise<ConnectionStatusResult> {

    const currentState = this.services.stateManager.getCurrentState();
    let sessionInfo: SessionInfo | undefined = undefined;
    
    // Check for session expiration using session manager
    if (currentState === 'Connected') {
      const sessionResult = await this.services.sessionManager.loadSession();
      if (!sessionResult.success || !sessionResult.data) {
        // Session expired
        this.services.stateManager.transitionTo('Expired', 'Session expired');
        
        return {
          state: 'Expired',
          sessionInfo: undefined,
        };
      } else {
        sessionInfo = sessionResult.data;
      }
    }

    return {
      state: currentState,
      sessionInfo,
    };
  }

  // Job management with connection state validation
  async createJob(params: CreateJobParams): Promise<CreateJobResult> {

    // Check connection state
    if (!this.services.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to create jobs',
      };
    }

    // Error injection
    if (this.shouldInjectError()) {
      return {
        success: false,
        error: 'Failed to create job directory on cluster',
      };
    }

    // Validate job parameters
    if (!params.jobName || !params.namdConfig || !params.slurmConfig) {
      return {
        success: false,
        error: 'Missing required job parameters',
      };
    }

    const jobId = this.generateJobId();
    const now = new Date().toISOString();
    
    // Get username from session manager
    const sessionResult = await this.services.sessionManager.loadSession();
    const username = sessionResult.data?.username || 'testuser';

    const jobInfo: JobInfo = {
      jobId,
      jobName: params.jobName,
      status: 'CREATED',
      createdAt: now,
      updatedAt: now,
      projectDir: `/projects/${username}/namdrunner_jobs/${jobId}`,
      scratchDir: `/scratch/alpine/${username}/namdrunner_jobs/${jobId}`,
    };

    this.jobs.set(jobId, jobInfo);

    return {
      success: true,
      jobId,
    };
  }

  async submitJob(jobId: JobId): Promise<SubmitJobResult> {

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (!this.services.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to submit jobs',
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

    return {
      success: true,
      jobs: Array.from(this.jobs.values()),
    };
  }

  async syncJobs(): Promise<SyncJobsResult> {

    if (!this.services.stateManager.isConnected()) {
      return {
        success: false,
        jobsUpdated: 0,
        errors: ['Must be connected to cluster to sync jobs'],
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

    if (!this.services.stateManager.isConnected()) {
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

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (!this.services.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to download files',
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

    const job = this.jobs.get(jobId);
    if (!job) {
      return {
        success: false,
        error: `Job ${jobId} not found`,
      };
    }

    if (!this.services.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to list files',
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

  // Test data management
  private loadTestData(): void {
    // Initialize with some test jobs
    const testJobs: JobInfo[] = [
      {
        jobId: 'test_001',
        jobName: 'Basic NAMD Test',
        status: 'COMPLETED',
        createdAt: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(), // 1 day ago
        updatedAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(), // 2 hours ago
        completedAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        projectDir: '/projects/testuser/namdrunner_jobs/test_001',
        slurmJobId: '12345678',
      },
      {
        jobId: 'test_002', 
        jobName: 'Structure Optimization',
        status: 'RUNNING',
        createdAt: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(), // 6 hours ago
        updatedAt: new Date(Date.now() - 30 * 60 * 1000).toISOString(), // 30 minutes ago
        submittedAt: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
        projectDir: '/projects/testuser/namdrunner_jobs/test_002',
        slurmJobId: '12345679',
      }
    ];

    testJobs.forEach(job => {
      this.jobs.set(job.jobId, job);
    });
    
    this.jobCounter = testJobs.length;
  }

  // Enhanced testing and diagnostic methods
  
  /**
   * Get state manager for testing purposes
   */
  getStateManager(): ConnectionStateManager {
    return this.services.stateManager;
  }

  /**
   * Get session manager for testing purposes  
   */
  getSessionManager(): SessionManager {
    return this.services.sessionManager;
  }

  /**
   * Force a specific connection state (for testing)
   */
  forceConnectionState(state: ConnectionState, reason?: string): void {
    // Use transitionTo instead of forceState for proper state management
    if (state === 'Disconnected' || state === 'Expired') {
      this.services.stateManager.transitionTo(state, reason || 'Forced for testing');
      this.services.sessionManager.clearSession();
    } else {
      this.services.stateManager.transitionTo(state, reason || 'Forced for testing');
    }
  }

  /**
   * Simulate session expiration (for testing)
   */
  async expireSession(): Promise<void> {
    const sessionResult = await this.services.sessionManager.loadSession();
    if (sessionResult.success && sessionResult.data) {
      // Create an old session
      const expiredSession = {
        ...sessionResult.data,
        connectedAt: new Date(Date.now() - 10 * 60 * 60 * 1000).toISOString() // 10 hours ago
      };
      await this.services.sessionManager.saveSession(expiredSession);
      this.services.stateManager.transitionTo('Expired', 'Session expired for testing');
    }
  }

  /**
   * Reset to clean state (for testing)
   */
  resetToCleanState(): void {
    this.services.stateManager.transitionTo('Disconnected', 'Reset to clean state');
    this.services.sessionManager.clearSession();
    this.jobs.clear();
    this.jobCounter = 0;
    this.errorInjectionEnabled = false;
  }

  /**
   * Get diagnostics for debugging
   */
  getDiagnostics() {
    return {
      connectionState: this.services.stateManager.getCurrentState(),
      stateHistory: this.services.stateManager.getStateHistory(),
      sessionInfo: this.services.sessionManager.getSessionInfo(),
      sessionDiagnostics: this.services.sessionManager.getSessionDiagnostics(),
      jobCount: this.jobs.size,
      errorInjectionEnabled: this.errorInjectionEnabled,
      errorRate: this.errorRate,
      lastError: this.services.stateManager.getLastError && this.services.stateManager.getLastError(),
      canRetry: this.services.stateManager.canRetry && this.services.stateManager.canRetry()
    };
  }
}
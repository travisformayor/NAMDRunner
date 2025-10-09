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
  DiscoverJobsResult,
  CompleteJobResult,
  FileUpload,
  UploadResult,
  DownloadResult,
  ListFilesResult,
  GetClusterCapabilitiesResult,
  JobId,
  JobInfo,
  ConnectionState,
  JobStatus,
  SessionInfo,
  RemoteFile,
} from '../types/api';
import type { ConnectionStateManager, SessionManager, StateTransition, Result } from '../types/connection';
import { ErrorBuilder, CONNECTION_ERRORS } from '../types/errors';
import { mockJobs } from '../stores/jobs';

// Simple in-memory state manager for demo mode
// Replaces the complex ConnectionStateMachine that was only used here
class SimpleStateManager implements ConnectionStateManager {
  private currentState: ConnectionState = 'Disconnected';
  private stateHistory: StateTransition[] = [];

  transitionTo(newState: ConnectionState, reason?: string): Result<void> {
    const old = this.currentState;
    this.currentState = newState;
    this.stateHistory.push({
      from: old,
      to: newState,
      timestamp: new Date().toISOString(),
      reason: reason,
      success: true
    });
    return { success: true, data: undefined };
  }

  getCurrentState(): ConnectionState {
    return this.currentState;
  }

  isConnected(): boolean {
    return this.currentState === 'Connected';
  }

  canRetry(): boolean {
    return this.currentState === 'Disconnected' || this.currentState === 'Expired';
  }

  getStateHistory(): StateTransition[] {
    return [...this.stateHistory];
  }
}

// Simple in-memory session manager for demo mode
// Replaces the complex SSHSessionManager that was only used here
class SimpleSessionManager implements SessionManager {
  private currentSession: SessionInfo | null = null;

  async saveSession(session_info: SessionInfo): Promise<Result<void>> {
    this.currentSession = session_info;
    return { success: true, data: undefined };
  }

  async loadSession(): Promise<Result<SessionInfo | null>> {
    return { success: true, data: this.currentSession };
  }

  async clearSession(): Promise<Result<void>> {
    this.currentSession = null;
    return { success: true, data: undefined };
  }

  getSessionInfo(): SessionInfo | null {
    return this.currentSession;
  }
}

/**
 * Enhanced mock implementation with realistic connection state management
 * Simulates complex connection scenarios for comprehensive testing
 */
export class MockCoreClient implements ICoreClient {
  private stateManager: ConnectionStateManager;
  private sessionManager: SessionManager;
  private jobs: Map<JobId, JobInfo> = new Map();
  private jobCounter = 0;
  private errorInjectionEnabled = false;
  private errorRate = 0.05; // 5% default error rate

  constructor() {
    // Create simple in-memory services for demo mode
    this.stateManager = new SimpleStateManager();
    this.sessionManager = new SimpleSessionManager();

    // Initialize with frontend mockJobs to ensure consistency
    this.loadFrontendMockJobs();
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
    const transitionResult = this.stateManager.transitionTo('Connecting', 'User initiated connection');
    if (!transitionResult.success) {
      return {
        success: false,
        error: `Cannot connect: ${transitionResult.error!.message}`,
      };
    }


    // Validate parameters
    if (!params.host || !params.username || !params.password) {
      this.stateManager.transitionTo('Disconnected', 'Invalid parameters');
      return {
        success: false,
        error: 'Missing required connection parameters',
      };
    }

    // Simulate various connection failure scenarios
    if (this.shouldInjectError()) {
      this.stateManager.transitionTo('Disconnected', 'Random error injection');
      const errorTemplates = [
        CONNECTION_ERRORS.NETWORK_UNREACHABLE,
        CONNECTION_ERRORS.CONNECTION_TIMEOUT,
        CONNECTION_ERRORS.AUTH_FAILED,
      ];
      const randomError = errorTemplates[Math.floor(Math.random() * errorTemplates.length)];
      if (!randomError) {
        return {
          success: false,
          error: 'Unknown connection error',
        };
      }
      return {
        success: false,
        error: randomError.message,
      };
    }

    // Host-specific failure patterns
    if (params.host.includes('invalid') || params.host.includes('unreachable')) {
      this.stateManager.transitionTo('Disconnected', 'Host unreachable');
      return {
        success: false,
        error: CONNECTION_ERRORS.NETWORK_UNREACHABLE.message,
      };
    }

    if (params.host.includes('timeout')) {
      this.stateManager.transitionTo('Disconnected', 'Connection timeout');
      return {
        success: false,
        error: CONNECTION_ERRORS.CONNECTION_TIMEOUT.message,
      };
    }

    if (params.password === 'wrongpassword') {
      this.stateManager.transitionTo('Disconnected', 'Authentication failed');
      return {
        success: false,
        error: CONNECTION_ERRORS.AUTH_FAILED.message,
      };
    }

    // Successful connection
    const session_info: SessionInfo = {
      host: params.host,
      username: params.username,
      connected_at: new Date().toISOString(),
    };

    // Update state and session
    this.stateManager.transitionTo('Connected', 'Authentication successful');
    await this.sessionManager.saveSession(session_info);

    return {
      success: true,
      session_info,
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
    await this.sessionManager.clearSession();
    this.stateManager.transitionTo('Disconnected', 'User initiated disconnect');

    return {
      success: true,
    };
  }

  async getConnectionStatus(): Promise<ConnectionStatusResult> {

    const currentState = this.stateManager.getCurrentState();
    let session_info: SessionInfo | undefined = undefined;

    // Check for session expiration using session manager
    if (currentState === 'Connected') {
      const sessionResult = await this.sessionManager.loadSession();
      if (!sessionResult.success || !sessionResult.data) {
        // Session expired
        this.stateManager.transitionTo('Expired', 'Session expired');

        return {
          state: 'Expired',
          session_info: undefined,
        };
      } else {
        session_info = sessionResult.data;
      }
    }

    return {
      state: currentState,
      session_info,
    };
  }

  // Job management with connection state validation
  async createJob(params: CreateJobParams): Promise<CreateJobResult> {

    // Check connection state
    if (!this.stateManager.isConnected()) {
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
    if (!params.job_name || !params.namd_config || !params.slurm_config) {
      return {
        success: false,
        error: 'Missing required job parameters',
      };
    }

    const job_id = this.generateJobId();
    const now = new Date().toISOString();

    // Get username from session manager
    const sessionResult = await this.sessionManager.loadSession();
    const username = (sessionResult.success && sessionResult.data) ? sessionResult.data.username : 'testuser';

    const jobInfo: JobInfo = {
      job_id: job_id,
      job_name: params.job_name,
      status: 'CREATED',
      created_at: now,
      updated_at: now,
      namd_config: params.namd_config,
      slurm_config: params.slurm_config,
      input_files: params.input_files || [],
      remote_directory: `/projects/${username}/namdrunner_jobs/${job_id}`,
      project_dir: `/projects/${username}/namdrunner_jobs/${job_id}`,
      scratch_dir: `/scratch/alpine/${username}/namdrunner_jobs/${job_id}`,
    };

    this.jobs.set(job_id, jobInfo);

    return {
      success: true,
      job_id: job_id,
      job: jobInfo,
    };
  }

  async submitJob(job_id: JobId): Promise<SubmitJobResult> {

    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    if (!this.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to submit jobs',
      };
    }

    // Simulate SLURM job submission
    const slurmJobId = Math.floor(Math.random() * 10000000).toString();
    const submittedAt = new Date().toISOString();

    job.status = 'PENDING';
    job.slurm_job_id = slurmJobId;
    job.submitted_at = submittedAt;
    job.updated_at = submittedAt;

    return {
      success: true,
      slurm_job_id: slurmJobId,
      submitted_at: submittedAt,
    };
  }

  async getJobStatus(job_id: JobId): Promise<JobStatusResult> {

    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    return {
      success: true,
      job_info: { ...job },
    };
  }

  async getAllJobs(): Promise<GetAllJobsResult> {

    return {
      success: true,
      jobs: Array.from(this.jobs.values()),
    };
  }

  async syncJobs(): Promise<SyncJobsResult> {

    if (!this.stateManager.isConnected()) {
      return {
        success: false,
        jobs_updated: 0,
        errors: ['Must be connected to cluster to sync jobs'],
      };
    }

    // Simulate job status updates
    let updatesCount = 0;
    const now = new Date().toISOString();

    for (const job of this.jobs.values()) {
      if (job.status === 'PENDING' && Math.random() > 0.7) {
        job.status = 'RUNNING';
        job.updated_at = now;
        updatesCount++;
      } else if (job.status === 'RUNNING' && Math.random() > 0.8) {
        job.status = 'COMPLETED';
        job.completed_at = now;
        job.updated_at = now;
        updatesCount++;
      }
    }

    return {
      success: true,
      jobs_updated: updatesCount,
      errors: [],
    };
  }

  async deleteJob(job_id: JobId, delete_remote: boolean): Promise<DeleteJobResult> {

    if (!this.jobs.has(job_id)) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    this.jobs.delete(job_id);

    return {
      success: true,
    };
  }

  async discoverJobsFromServer(): Promise<DiscoverJobsResult> {
    // Mock implementation: simulate finding 3 jobs on server, but 0 imported (already in DB)
    return {
      success: true,
      jobs_found: 3,
      jobs_imported: 0,
    };
  }

  async completeJob(job_id: JobId): Promise<CompleteJobResult> {
    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    if (job.status !== 'COMPLETED') {
      return {
        success: false,
        error: `Job ${job_id} is not completed (status: ${job.status})`,
      };
    }

    // Mock: Return success with updated job info
    return {
      success: true,
      job_info: job,
    };
  }

  // File management
  async uploadJobFiles(job_id: JobId, files: FileUpload[]): Promise<UploadResult> {

    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        failed_uploads: [
          {
            file_name: 'unknown',
            error: `Job ${job_id} not found`,
          },
        ],
      };
    }

    if (!this.stateManager.isConnected()) {
      return {
        success: false,
        failed_uploads: files.map((file) => ({
          file_name: file.remote_name,
          error: 'Not connected to cluster',
        })),
      };
    }

    // Simulate some successful and some failed uploads
    const uploadedFiles: string[] = [];
    const failedUploads: Array<{ file_name: string; error: string }> = [];

    for (const file of files) {
      if (Math.random() > 0.1) {
        // 90% success rate
        uploadedFiles.push(file.remote_name);
      } else {
        failedUploads.push({
          file_name: file.remote_name,
          error: 'Simulated upload failure',
        });
      }
    }

    const result: UploadResult = {
      success: failedUploads.length === 0,
      uploaded_files: uploadedFiles,
    };

    if (failedUploads.length > 0) {
      result.failed_uploads = failedUploads;
    }

    return result;
  }

  async downloadJobOutput(job_id: JobId, file_name: string): Promise<DownloadResult> {

    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    if (!this.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to download files',
      };
    }

    // Simulate file content
    const mockContent = `Mock content for ${file_name} from job ${job_id}\nGenerated at ${new Date().toISOString()}`;

    return {
      success: true,
      content: mockContent,
      file_path: `/tmp/${file_name}`,
      file_size: mockContent.length,
    };
  }

  async listJobFiles(job_id: JobId): Promise<ListFilesResult> {

    const job = this.jobs.get(job_id);
    if (!job) {
      return {
        success: false,
        error: `Job ${job_id} not found`,
      };
    }

    if (!this.stateManager.isConnected()) {
      return {
        success: false,
        error: 'Must be connected to cluster to list files',
      };
    }

    // Simulate file listing
    const mockFiles: RemoteFile[] = [
      {
        name: 'config.namd',
        size: 2048,
        modified_at: new Date().toISOString(),
        file_type: 'config' as const,
      },
      {
        name: 'job.sbatch',
        size: 1024,
        modified_at: new Date().toISOString(),
        file_type: 'config' as const,
      },
      {
        name: 'output.log',
        size: 15360,
        modified_at: new Date().toISOString(),
        file_type: 'log' as const,
      },
    ];

    return {
      success: true,
      files: mockFiles,
    };
  }

  // Load frontend mockJobs to ensure consistency with the jobs store
  private loadFrontendMockJobs(): void {
    // Use the same mockJobs from frontend store to avoid data redundancy
    mockJobs.forEach(job => {
      this.jobs.set(job.job_id, { ...job }); // Clone to avoid mutations
    });

    this.jobCounter = mockJobs.length;
  }

  // Enhanced testing and diagnostic methods

  /**
   * Get state manager for testing purposes
   */
  getStateManager(): ConnectionStateManager {
    return this.stateManager;
  }

  /**
   * Get session manager for testing purposes
   */
  getSessionManager(): SessionManager {
    return this.sessionManager;
  }

  /**
   * Force a specific connection state (for testing)
   */
  forceConnectionState(state: ConnectionState, reason?: string): void {
    // Use transitionTo instead of forceState for proper state management
    if (state === 'Disconnected' || state === 'Expired') {
      this.stateManager.transitionTo(state, reason || 'Forced for testing');
      this.sessionManager.clearSession();
    } else {
      this.stateManager.transitionTo(state, reason || 'Forced for testing');
    }
  }

  /**
   * Simulate session expiration (for testing)
   */
  async expireSession(): Promise<void> {
    const sessionResult = await this.sessionManager.loadSession();
    if (sessionResult.success && sessionResult.data) {
      // Create an old session
      const expiredSession = {
        ...sessionResult.data,
        connected_at: new Date(Date.now() - 10 * 60 * 60 * 1000).toISOString() // 10 hours ago
      };
      await this.sessionManager.saveSession(expiredSession);
      this.stateManager.transitionTo('Expired', 'Session expired for testing');
    }
  }

  /**
   * Reset to clean state (for testing)
   */
  resetToCleanState(): void {
    this.stateManager.transitionTo('Disconnected', 'Reset to clean state');
    this.sessionManager.clearSession();
    this.jobs.clear();
    this.jobCounter = 0;
    this.errorInjectionEnabled = false;
  }

  /**
   * Get diagnostics for debugging
   */
  async getClusterCapabilities(): Promise<GetClusterCapabilitiesResult> {
    // Cluster capabilities are static configuration data
    // Return real backend data in both demo and production mode
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('get_cluster_capabilities');
  }

  async validateResourceAllocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<import('../types/api').ValidateResourceAllocationResult> {
    // Use real backend validation in both demo and production mode
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('validate_resource_allocation', {
      cores,
      memory,
      walltime,
      partition_id,
      qos_id
    });
  }

  async suggestQosForPartition(walltime_hours: number, partition_id: string): Promise<string> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('suggest_qos_for_partition', {
      walltime_hours,
      partition_id
    });
  }

  async estimateQueueTimeForJob(cores: number, partition_id: string): Promise<string> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('estimate_queue_time_for_job', {
      cores,
      partition_id
    });
  }

  async calculateJobCost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('calculate_job_cost', {
      cores,
      walltime_hours,
      has_gpu,
      gpu_count
    });
  }

  getDiagnostics() {
    return {
      connectionState: this.stateManager.getCurrentState(),
      stateHistory: this.stateManager.getStateHistory?.() || [],
      session_info: this.sessionManager.getSessionInfo?.() || null,
      sessionDiagnostics: null,
      jobCount: this.jobs.size,
      errorInjectionEnabled: this.errorInjectionEnabled,
      errorRate: this.errorRate,
      lastError: null,
      canRetry: this.stateManager.canRetry?.() || false
    };
  }
}
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
  ValidateResourceAllocationResult,
  JobId,
} from '../types/api';

/**
 * Core IPC client interface for NAMDRunner
 * This defines the contract between the frontend and backend
 */
export interface ICoreClient {
  // Connection management
  connect(params: ConnectParams): Promise<ConnectResult>;
  disconnect(): Promise<DisconnectResult>;
  getConnectionStatus(): Promise<ConnectionStatusResult>;

  // Cluster configuration and validation
  getClusterCapabilities(): Promise<GetClusterCapabilitiesResult>;
  validateResourceAllocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<ValidateResourceAllocationResult>;
  suggestQosForPartition(walltime_hours: number, partition_id: string): Promise<string>;
  estimateQueueTimeForJob(cores: number, partition_id: string): Promise<string>;
  calculateJobCost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number>;

  // Job management
  createJob(params: CreateJobParams): Promise<CreateJobResult>;
  submitJob(job_id: JobId): Promise<SubmitJobResult>;
  getJobStatus(job_id: JobId): Promise<JobStatusResult>;
  getAllJobs(): Promise<GetAllJobsResult>;
  syncJobs(): Promise<SyncJobsResult>;
  deleteJob(job_id: JobId, delete_remote: boolean): Promise<DeleteJobResult>;
  discoverJobsFromServer(): Promise<DiscoverJobsResult>;

  // File management
  uploadJobFiles(job_id: JobId, files: FileUpload[]): Promise<UploadResult>;
  downloadJobOutput(job_id: JobId, file_name: string): Promise<DownloadResult>;
  listJobFiles(job_id: JobId): Promise<ListFilesResult>;
}
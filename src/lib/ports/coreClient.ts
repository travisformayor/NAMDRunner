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
  RefetchLogsResult,
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
  syncJobs(): Promise<SyncJobsResult>;  // Discovery happens automatically during sync if DB empty
  deleteJob(job_id: JobId, delete_remote: boolean): Promise<DeleteJobResult>;
  refetchSlurmLogs(job_id: JobId): Promise<RefetchLogsResult>;

  // File management
  selectInputFile(): Promise<any>; // Returns single SelectedFile object or null
  detectFileType(filename: string): Promise<string>;
  uploadJobFiles(job_id: JobId, files: FileUpload[]): Promise<UploadResult>;
  downloadJobOutput(job_id: JobId, file_path: string): Promise<DownloadResult>;
  downloadAllOutputs(job_id: JobId): Promise<DownloadResult>;
  listJobFiles(job_id: JobId): Promise<ListFilesResult>;
}
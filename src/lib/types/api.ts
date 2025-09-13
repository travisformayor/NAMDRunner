// Core type definitions matching Rust types
export type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
export type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';
export type JobId = string;
export type SlurmJobId = string;
export type Timestamp = string;

// Basic interfaces
export interface SessionInfo {
  host: string;
  username: string;
  connectedAt: Timestamp;
}

export interface JobInfo {
  jobId: JobId;
  jobName: string;
  status: JobStatus;
  slurmJobId?: SlurmJobId;
  createdAt: Timestamp;
  updatedAt?: Timestamp;
  submittedAt?: Timestamp;
  completedAt?: Timestamp;
  projectDir?: string;
  scratchDir?: string;
  errorInfo?: string;
}

export interface NAMDConfig {
  steps: number;
  temperature: number;
  timestep: number;
  outputname: string;
  dcdFreq?: number;
  restartFreq?: number;
}

export interface SlurmConfig {
  cores: number;
  memory: string;
  walltime: string;
  partition?: string;
  qos?: string;
}

export interface InputFile {
  name: string;
  localPath: string;
  remoteName?: string;
  type?: 'pdb' | 'psf' | 'prm' | 'other';
  fileType?: 'pdb' | 'psf' | 'prm' | 'other';
}

export interface FileUpload {
  localPath: string;
  remoteName: string;
}

export interface RemoteFile {
  name: string;
  size: number;
  modifiedAt: Timestamp;
  fileType: 'input' | 'output' | 'config' | 'log';
}

// Command parameters and results
export interface ConnectParams {
  host: string;
  username: string;
  password: string;
}

export interface ConnectResult {
  success: boolean;
  sessionInfo?: SessionInfo;
  error?: string;
}

export interface DisconnectResult {
  success: boolean;
  error?: string;
}

export interface ConnectionStatusResult {
  state: ConnectionState;
  sessionInfo: SessionInfo | undefined;
}

export interface CreateJobParams {
  jobName: string;
  namdConfig: NAMDConfig;
  slurmConfig: SlurmConfig;
  inputFiles: InputFile[];
}

export interface CreateJobResult {
  success: boolean;
  jobId?: JobId;
  error?: string;
}

export interface SubmitJobResult {
  success: boolean;
  slurmJobId?: SlurmJobId;
  submittedAt?: Timestamp;
  error?: string;
}

export interface JobStatusResult {
  success: boolean;
  jobInfo?: JobInfo;
  error?: string;
}

export interface GetAllJobsResult {
  success: boolean;
  jobs?: JobInfo[];
  error?: string;
}

export interface SyncJobsResult {
  success: boolean;
  jobsUpdated: number;
  errors: string[];
}

export interface DeleteJobResult {
  success: boolean;
  error?: string;
}

export interface UploadResult {
  success: boolean;
  uploadedFiles?: string[];
  failedUploads?: Array<{
    fileName: string;
    error: string;
  }>;
}

export interface DownloadResult {
  success: boolean;
  content?: string;
  filePath?: string;
  fileSize?: number;
  error?: string;
}

export interface ListFilesResult {
  success: boolean;
  files?: RemoteFile[];
  error?: string;
}

// Error handling
export interface NAMDRunnerError {
  category: 'Network' | 'Authentication' | 'Validation' | 'FileSystem' | 'SLURM' | 'Internal';
  message: string;
  details?: string;
  retryable: boolean;
}
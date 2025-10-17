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
  connected_at: Timestamp;
}

export interface JobInfo {
  job_id: JobId;
  job_name: string;
  status: JobStatus;
  slurm_job_id?: SlurmJobId;
  created_at: Timestamp;
  updated_at?: Timestamp;
  submitted_at?: Timestamp;
  completed_at?: Timestamp;
  project_dir?: string;
  scratch_dir?: string;
  error_info?: string;
  slurm_stdout?: string;
  slurm_stderr?: string;
  namd_config: NAMDConfig;
  slurm_config: SlurmConfig;
  input_files: InputFile[];
  remote_directory: string;
}

export interface NAMDConfig {
  steps: number;
  temperature: number;
  timestep: number;
  outputname: string;
  dcd_freq?: number;
  restart_freq?: number;
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
  local_path: string;
  remote_name?: string;
  file_type?: 'pdb' | 'psf' | 'prm' | 'other';
}

export interface FileUpload {
  local_path: string;
  remote_name: string;
}

export interface RemoteFile {
  name: string;           // Display name (just filename)
  path: string;           // Full relative path from job root (e.g., "outputs/sim.dcd")
  size: number;
  modified_at: Timestamp;
  file_type: 'input' | 'output' | 'config' | 'log';
}

// Command parameters and results
export interface ConnectParams {
  host: string;
  username: string;
  password: string;
}

export interface ConnectResult {
  success: boolean;
  session_info?: SessionInfo;
  error?: string;
}

export interface DisconnectResult {
  success: boolean;
  error?: string;
}

export interface ConnectionStatusResult {
  state: ConnectionState;
  session_info: SessionInfo | undefined;
}

export interface CreateJobParams {
  job_name: string;
  namd_config: NAMDConfig;
  slurm_config: SlurmConfig;
  input_files: InputFile[];
}

export interface CreateJobResult {
  success: boolean;
  job_id?: JobId;
  job?: JobInfo;
  error?: string;
}

export interface SubmitJobResult {
  success: boolean;
  slurm_job_id?: SlurmJobId;
  submitted_at?: Timestamp;
  error?: string;
}

export interface JobStatusResult {
  success: boolean;
  job_info?: JobInfo;
  error?: string;
}

export interface GetAllJobsResult {
  success: boolean;
  jobs?: JobInfo[];
  error?: string;
}

export interface SyncJobsResult {
  success: boolean;
  jobs: JobInfo[];           // Complete job list after sync
  jobs_updated: number;       // Number of jobs updated during sync
  errors: string[];
}

export interface DeleteJobResult {
  success: boolean;
  error?: string;
}

export interface DiscoverJobsResult {
  success: boolean;
  jobs_found: number;
  jobs_imported: number;
  error?: string;
}

export interface RefetchLogsResult {
  success: boolean;
  job_info?: JobInfo;
  error?: string;
}

export interface UploadResult {
  success: boolean;
  uploaded_files?: string[];
  failed_uploads?: Array<{
    file_name: string;
    error: string;
  }>;
}

export interface DownloadResult {
  success: boolean;
  saved_to?: string;  // Local path where file was saved
  file_size?: number;
  error?: string;
}

export interface ListFilesResult {
  success: boolean;
  files?: RemoteFile[];
  error?: string;
}

// Cluster Capabilities (from backend)
export type PartitionCategory = 'Compute' | 'GPU' | 'HighMemory' | 'Development' | 'Compile';
export type QosPriority = 'High' | 'Normal' | 'Low';

export interface PartitionSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  nodes: string;
  cores_per_node: string;
  ram_per_core: string;
  max_walltime: string;
  gpu_type: string | null;
  gpu_count: number | null;
  category: PartitionCategory;
  use_cases: string[];
  is_standard: boolean;
  is_default: boolean;
}

export interface QosSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  max_walltime_hours: number;
  max_jobs: number;
  node_limit: number;
  valid_partitions: string[];
  requirements: string[];
  priority: QosPriority;
  is_default: boolean;
}

export interface JobPresetConfig {
  cores: number;
  memory: string;
  wall_time: string;
  partition: string;
  qos: string;
}

export interface JobPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  config: JobPresetConfig;
  estimated_cost: string;
  estimated_queue: string;
  use_cases: string[];
  requires_gpu: boolean;
}

export interface BillingRates {
  cpu_cost_per_core_hour: number;
  gpu_cost_per_gpu_hour: number;
}

export interface ClusterCapabilities {
  partitions: PartitionSpec[];
  qos_options: QosSpec[];
  job_presets: JobPreset[];
  billing_rates: BillingRates;
}

export interface GetClusterCapabilitiesResult {
  success: boolean;
  data?: ClusterCapabilities;
  error?: string;
}

export interface ValidateResourceAllocationResult {
  is_valid: boolean;
  issues: string[];
  warnings: string[];
  suggestions: string[];
}

// Error handling
export interface NAMDRunnerError {
  category: 'Network' | 'Authentication' | 'Validation' | 'FileSystem' | 'SLURM' | 'Internal';
  message: string;
  details?: string;
  retryable: boolean;
}
import type { TemplateSummary } from './template';

// JSON value type matching Rust serde_json::Value
// Can be any valid JSON value: string, number, boolean, null, object, or array
export type JsonValue = string | number | boolean | null | JsonObject | JsonArray;
export interface JsonObject {
  [key: string]: JsonValue;
}
export type JsonArray = JsonValue[];

// Generic API result type matching Rust ApiResult<T>
export interface ApiResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

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
  template_id: string;
  template_values: Record<string, JsonValue>;
  slurm_config: SlurmConfig;
  input_files: string[];
  output_files: OutputFile[];
}

export interface SlurmConfig {
  cores: number;
  memory: string;
  walltime: string;
  partition?: string;
  qos?: string;
}

export interface OutputFile {
  name: string;
  size: number;
  modified_at: string;
}

export interface FileUpload {
  local_path: string;
  remote_name: string;
}


// Response DTOs for multi-field command responses
export interface DownloadInfo {
  saved_to: string;
  file_size: number;
}

export interface DatabaseInfo {
  path: string;
  size_bytes: number;
  job_count: number;
}

export interface DatabaseOperationData {
  path: string;
  message: string;
}

export interface ConnectionStatus {
  state: ConnectionState;
  session_info?: SessionInfo;
}

// Command parameters
export interface ConnectParams {
  host: string;
  username: string;
  password: string;
}

export interface CreateJobParams {
  job_name: string;
  template_id: string;
  template_values: Record<string, JsonValue>;
  slurm_config: SlurmConfig;
}

// Complex batch operation results (domain-specific)
export interface SyncJobsResult {
  success: boolean;
  jobs: JobInfo[];           // Complete job list after sync
  jobs_updated: number;       // Number of jobs updated during sync
  errors: string[];
}

// Cluster Capabilities (from backend)

export interface PartitionSpec {
  name: string;
  title: string;
  description: string;
  max_cores: number;
  max_memory_per_core_gb: number;
  gpu_type: string | null;
  gpu_count: number | null;
  is_default: boolean;
}

export interface QosSpec {
  name: string;
  title: string;
  description: string;
  max_walltime_hours: number;
  valid_partitions: string[];
  min_memory_gb: number | null;
  is_default: boolean;
}

export interface JobPreset {
  name: string;
  description: string;
  cores: number;
  memory: string;
  walltime: string;
  partition: string;
  qos: string;
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
  default_host: string;
}

// Unified validation result type matching Rust ValidationResult
export interface ValidationResult {
  is_valid: boolean;
  issues: string[];
  warnings: string[];
  suggestions: string[];
  field_errors?: Record<string, string>;
}

// App initialization data
export interface AppInitializationData {
  capabilities: ClusterCapabilities;
  templates: TemplateSummary[];
  jobs: JobInfo[];
}

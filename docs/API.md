# API & Data Contracts

This document defines all interfaces and data contracts for NAMDRunner, consolidating IPC communication patterns and data schemas.

## Table of Contents
- [Implementation Notes](#implementation-notes)
- [Core Type Definitions](#core-type-definitions)
  - [Connection State Management](#connection-state-management)
  - [Job Lifecycle States](#job-lifecycle-states)
  - [Basic Types](#basic-types)
- [Application Configuration Commands](#application-configuration-commands)
  - [IPC Interface](#ipc-interface)
- [Connection Management Commands](#connection-management-commands)
  - [IPC Interface](#ipc-interface-1)
- [Job Management Commands](#job-management-commands)
  - [IPC Interface](#ipc-interface-2)
  - [Request/Response Types](#requestresponse-types)
- [File Management Commands](#file-management-commands)
  - [IPC Interface](#ipc-interface-3)
- [Error Handling Strategy](#error-handling-strategy)
  - [Error Categories](#error-categories)
  - [Common Error Examples](#common-error-examples)
- [Rust Type Definitions](#rust-type-definitions)
  - [Core Types](#core-types)
  - [Command Result Types](#command-result-types)
- [SLURM Integration](#slurm-integration)
- [Related Documentation](#related-documentation)

## Implementation Notes

1. **Always use full paths** for working directories
2. **Module commands must be sourced properly** with `/etc/profile`
3. **Parse both stdout and stderr** for error detection
4. **Handle queue wait times** - jobs may be PENDING for hours
5. **Account for 90-day scratch purge policy**
6. **Never log or persist passwords** - memory only
7. **Validate SSH connection** before SLURM operations
8. **Use working directory pattern** to identify NAMDRunner jobs: `/scratch/alpine/$USER/namdrunner_jobs/*`
9. **IPC parameter naming convention**: **All Tauri commands use `#[tauri::command(rename_all = "snake_case")]`** to maintain consistent naming across the API boundary. This ensures frontend and backend use identical parameter names (snake_case), eliminating conversion logic and improving code searchability.

   **Why this matters**: By default, Tauri v2 automatically converts Rust's snake_case parameters to JavaScript's camelCase convention. For example, a Rust parameter `job_id: String` would normally require JavaScript to pass `{jobId: "test1_123"}`. This creates confusion because:
   - The same concept has different names in frontend vs backend
   - Searching for `job_id` across the codebase misses frontend uses
   - Developers must mentally translate between naming conventions
   - Conversion bugs can occur (as seen with the submit_job parameter order issue)

   **Our approach**: We add `rename_all = "snake_case"` to every command, so both sides use the same names:
   ```rust
   // Rust backend
   #[tauri::command(rename_all = "snake_case")]
   pub async fn validate_resource_allocation(
       partition_id: String,
       qos_id: String
   ) -> ValidationResult

   // TypeScript frontend - same names!
   invoke('validate_resource_allocation', {
       partition_id: "amilan",
       qos_id: "normal"
   })
   ```

   This consistency was a key goal of Phase 6.4, where 49 `#[serde(rename)]` attributes were removed for the same reason.

10. **IPC parameter serialization for structs**: Commands that accept struct parameters (like `connect_to_cluster`) expect the struct wrapped: `{params: {host, username, password}}`. Commands with individual parameters send them directly: `{job_id: "job_001"}`.
11. **Demo mode synchronization**: Frontend mode preference must be synced to backend via `set_app_mode` command with `{is_demo: boolean}` parameter

These patterns are proven to work with the CURC Alpine cluster and provide the foundation for reliable SLURM integration in the Tauri implementation.

> **For detailed SSH/SFTP implementation patterns, security practices, and performance optimizations, see [`docs/SSH.md`](SSH.md)**.

## Core Type Definitions

### Connection State Management
```typescript
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
```

### Job Lifecycle States
```typescript
type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';
```

### Basic Types
```typescript
type JobId = string;        // Format: job_001, job_002, etc.
type SlurmJobId = string;   // SLURM's job ID (numbers)
type Timestamp = string;    // ISO 8601 format
```

## Application Configuration Commands

### IPC Interface
```typescript
interface IApplicationCommands {
  // Set application mode (demo/real)
  set_app_mode(is_demo: boolean): Promise<{success: boolean, data?: string, error?: string}>;
}
```

**Actual Rust Command**: `set_app_mode(is_demo: bool) -> ApiResult<String>`
Location: [src-tauri/src/commands/system.rs](../src-tauri/src/commands/system.rs)

## Connection Management Commands

### IPC Interface
```typescript
interface IConnectionCommands {
  // Establish SSH connection to cluster
  connect_to_cluster(params: ConnectParams): Promise<ConnectResult>;

  // Close SSH connection
  disconnect(): Promise<DisconnectResult>;

  // Check current connection status
  get_connection_status(): Promise<ConnectionStatusResult>;

  // Get cluster capabilities (partitions, QoS options, etc.)
  get_cluster_capabilities(): Promise<ApiResult<ClusterCapabilities>>;

  // Helper functions for cluster resource planning
  suggest_qos_for_partition(walltime_hours: number, partition_id: string): Promise<string>;
  estimate_queue_time_for_job(cores: number, partition_id: string): Promise<string>;
  calculate_job_cost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number>;
}

interface ConnectResult {
  success: boolean;
  session_info?: {
    host: string;
    username: string;
    connected_at: Timestamp;
  };
  error?: string;
}

interface DisconnectResult {
  success: boolean;
  error?: string;
}

interface ConnectionStatusResult {
  state: ConnectionState;
  session_info?: {
    host: string;
    username: string;
    connected_at: Timestamp;
  };
}

interface GetClusterCapabilitiesResult {
  success: boolean;
  data?: ClusterCapabilities;
  error?: string;
}

interface ClusterCapabilities {
  partitions: PartitionSpec[];
  qos_options: QosSpec[];
  job_presets: JobPreset[];
  billing_rates: BillingRates;
}

// See src/lib/types/cluster.ts for complete type definitions
```


## Job Management Commands

### IPC Interface

**Important**: All command parameters use snake_case naming per Implementation Note #9. Example invocation:
```typescript
// Correct - snake_case matches Rust backend
await invoke('submit_job', { job_id: "test1_123" });

// Wrong - would fail with "missing required key job_id"
await invoke('submit_job', { jobId: "test1_123" });
```

```typescript
interface IJobCommands {
  // Create new job (local only, not submitted yet)
  createJob(params: CreateJobParams): Promise<CreateJobResult>;

  // Submit job to SLURM cluster
  submitJob(job_id: JobId): Promise<SubmitJobResult>;

  // Get status of specific job
  getJobStatus(job_id: JobId): Promise<JobStatusResult>;

  // Get all jobs from local cache
  getAllJobs(): Promise<GetAllJobsResult>;

  // Sync job statuses with cluster
  syncJobs(): Promise<SyncJobsResult>;

  // Discover jobs from server and import to local database
  discoverJobs(): Promise<DiscoverJobsResult>;

  // Delete job (local and optionally remote)
  deleteJob(job_id: JobId, delete_remote: boolean): Promise<DeleteJobResult>;

  // Refetch SLURM logs from server (overwrites cached logs)
  refetchSlurmLogs(job_id: JobId): Promise<RefetchLogsResult>;

  // Validate resource allocation for cluster constraints
  validate_resource_allocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<ValidationResult>;
}
```

**Note**: Command names use `snake_case` per Rust convention. The TypeScript client should maintain this naming.

### Request/Response Types
```typescript
interface CreateJobParams {
  job_name: string;
  namd_config: {
    steps: number;
    temperature: number;
    timestep: number;
    outputname: string;
    dcd_freq?: number;
    restart_freq?: number;
  };
  slurm_config: {
    cores: number;
    memory: string;      // e.g., "16GB"
    walltime: string;    // e.g., "02:00:00"
    partition?: string;  // default: "amilan"
    qos?: string;        // default: "normal"
  };
  input_files: InputFile[];
}

interface InputFile {
  name: string;
  local_path: string;
  remote_name?: string;
  file_type?: 'pdb' | 'psf' | 'prm' | 'other';
}

interface CreateJobResult {
  success: boolean;
  job_id?: JobId;
  error?: string;
}

interface SubmitJobResult {
  success: boolean;
  slurm_job_id?: SlurmJobId;
  submitted_at?: Timestamp;
  error?: string;
}

interface JobStatusResult {
  success: boolean;
  job_info?: JobInfo;
  error?: string;
}

interface JobInfo {
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
  namd_config: NAMDConfig;
  slurm_config: SlurmConfig;
  input_files: InputFile[];
  remote_directory: string;
}

interface GetAllJobsResult {
  success: boolean;
  jobs?: JobInfo[];
  error?: string;
}

interface SyncJobsResult {
  success: boolean;
  jobs_updated: number;
  errors: string[];
}

interface DiscoverJobsResult {
  success: boolean;
  jobs_found: number;
  jobs_imported: number;
  error?: string;
}

interface DeleteJobResult {
  success: boolean;
  error?: string;
}

interface RefetchLogsResult {
  success: boolean;
  slurm_stdout?: string;  // Fetched SLURM stdout
  slurm_stderr?: string;  // Fetched SLURM stderr
  error?: string;
}
```

## File Management Commands

### IPC Interface
```typescript
interface IFileCommands {
  // Upload files to job directory on cluster
  uploadJobFiles(job_id: JobId, files: FileUpload[]): Promise<UploadResult>;

  // Download single job output file (shows native save dialog)
  downloadJobOutput(job_id: JobId, file_path: string): Promise<DownloadResult>;

  // Download all output files as ZIP archive (shows native save dialog)
  downloadAllOutputs(job_id: JobId): Promise<DownloadResult>;

  // List files in job directory
  listJobFiles(job_id: JobId): Promise<ListFilesResult>;
}

interface FileUpload {
  local_path: string;
  remote_name: string;
}

interface UploadResult {
  success: boolean;
  uploaded_files?: string[];
  failed_uploads?: Array<{
    file_name: string;
    error: string;
  }>;
}

interface DownloadResult {
  success: boolean;
  saved_to?: string;     // Local path where file was saved (via native dialog)
  file_size?: number;
  error?: string;
}

interface RemoteFile {
  name: string;          // Display name (just filename)
  path: string;          // Full relative path from job root (e.g., "outputs/sim.dcd")
  size: number;
  modified_at: Timestamp;
  file_type: 'input' | 'output' | 'config' | 'log';
}

interface ListFilesResult {
  success: boolean;
  files?: RemoteFile[];
  error?: string;
}
```

## Error Handling Strategy

### Error Categories
```typescript
interface NAMDRunnerError {
  category: 'Network' | 'Authentication' | 'Validation' | 'FileSystem' | 'SLURM' | 'Internal';
  message: string;
  details?: string;
  retryable: boolean;
}
```

### Common Error Examples

#### Network Errors
```typescript
const NETWORK_ERROR: NAMDRunnerError = {
  category: 'Network',
  message: 'Failed to connect to cluster',
  details: 'Connection timed out after 30 seconds',
  retryable: true
};
```

#### Validation Errors
```typescript
const VALIDATION_ERROR: NAMDRunnerError = {
  category: 'Validation',
  message: 'Invalid NAMD parameters',
  details: 'Temperature must be set and a number',
  retryable: false
};
```

### Best Practices

**Error message quality:**
Provide actionable error messages. Instead of "Job failed", show "Job failed: Out of memory. Requested 32GB, consider increasing to 64GB."

**Retry behavior:**
Use the `retryable` field to determine whether operations should be retried with exponential backoff or fail immediately.

> **For retry implementation patterns**, see [`docs/SSH.md#retry-strategies`](SSH.md#retry-strategies)
> **For SLURM error messages**, see [`docs/reference/slurm-commands-reference.md#error-handling`](reference/slurm-commands-reference.md#error-handling)
> **For NAMD error messages**, see [`docs/reference/namd-commands-reference.md#error-handling`](reference/namd-commands-reference.md#error-handling)


## Rust Type Definitions

### Core Types
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Created,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
    pub slurm_job_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub project_dir: Option<String>,
    pub scratch_dir: Option<String>,
    pub error_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NAMDConfig {
    pub steps: u32,
    pub temperature: f64,
    pub timestep: f64,
    pub outputname: String,
    pub dcd_freq: Option<u32>,
    pub restart_freq: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmConfig {
    pub cores: u32,
    pub memory: String,
    pub walltime: String,
    pub partition: Option<String>,
    pub qos: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFile {
    pub name: String,
    pub local_path: String,
    pub file_type: String,
}
```

### Command Result Types
```rust
#[derive(Debug, Serialize)]
pub struct ConnectResult {
    pub success: bool,
    pub session_info: Option<SessionInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateJobResult {
    pub success: bool,
    pub job_id: Option<String>,
    pub error: Option<String>,
}

// Additional result types follow same pattern...
```

## SLURM Integration

> **For complete SLURM integration patterns including job submission, status monitoring, error handling, and command examples, see [`docs/reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md) and [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md)**.

## Related Documentation

For SSH/SFTP connection management, security patterns, and file transfer implementation details, see [`docs/SSH.md`](SSH.md).

For architectural principles, clean architecture patterns, and development best practices, see [`CONTRIBUTING.md#developer-standards--project-philosophy`](CONTRIBUTING.md#developer-standards--project-philosophy).

For testing strategies and infrastructure setup, see [`docs/CONTRIBUTING.md#testing-strategy`](CONTRIBUTING.md#testing-strategy).

For data schemas, SQLite database design, and persistence patterns, see [`docs/DB.md`](DB.md).
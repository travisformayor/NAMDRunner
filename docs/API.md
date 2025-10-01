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
9. **IPC parameter serialization**: Tauri commands expect specific parameter structures - `connect_to_cluster` takes `{params: {host, username, password}}` and `set_app_mode` takes `{isDemo: boolean}`
10. **Demo mode synchronization**: Frontend mode preference must be synced to backend via `set_app_mode` command with `{isDemo: boolean}` parameter

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
  setAppMode(isDemo: boolean): Promise<void>;
}
```

## Connection Management Commands

### IPC Interface
```typescript
interface IConnectionCommands {
  // Establish SSH connection to cluster
  connect(host: string, username: string, password: string): Promise<ConnectResult>;

  // Close SSH connection
  disconnect(): Promise<DisconnectResult>;

  // Check current connection status
  getConnectionStatus(): Promise<ConnectionStatusResult>;
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
```


## Job Management Commands

### IPC Interface
```typescript
interface IJobCommands {
  // Create new job (local only, not submitted yet)
  createJob(params: CreateJobParams): Promise<CreateJobResult>;
  
  // Submit job to SLURM cluster
  submitJob(jobId: JobId): Promise<SubmitJobResult>;
  
  // Get status of specific job
  getJobStatus(jobId: JobId): Promise<JobStatusResult>;
  
  // Get all jobs from local cache
  getAllJobs(): Promise<GetAllJobsResult>;
  
  // Sync job statuses with cluster
  syncJobs(): Promise<SyncJobsResult>;
  
  // Delete job (local and optionally remote)
  deleteJob(jobId: JobId, deleteRemote: boolean): Promise<DeleteJobResult>;
}
```

### Request/Response Types
```typescript
interface CreateJobParams {
  jobName: string;
  namdConfig: {
    steps: number;
    temperature: number;
    timestep: number;
    outputname: string;
    dcdFreq?: number;
    restartFreq?: number;
  };
  slurmConfig: {
    cores: number;
    memory: string;      // e.g., "16GB"
    walltime: string;    // e.g., "02:00:00"
    partition?: string;  // default: "amilan"
    qos?: string;        // default: "normal"
  };
  inputFiles: InputFile[];
}

interface InputFile {
  name: string;
  localPath: string;
  fileType: 'pdb' | 'psf' | 'prm' | 'other';
}

interface CreateJobResult {
  success: boolean;
  jobId?: JobId;
  error?: string;
}

interface SubmitJobResult {
  success: boolean;
  slurmJobId?: SlurmJobId;
  submittedAt?: Timestamp;
  error?: string;
}

interface JobStatusResult {
  success: boolean;
  jobInfo?: JobInfo;
  error?: string;
}

interface JobInfo {
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

interface GetAllJobsResult {
  success: boolean;
  jobs?: JobInfo[];
  error?: string;
}

interface SyncJobsResult {
  success: boolean;
  jobsUpdated: number;
  errors: string[];
}

interface DeleteJobResult {
  success: boolean;
  error?: string;
}
```

## File Management Commands

### IPC Interface
```typescript
interface IFileCommands {
  // Upload files to job directory on cluster
  uploadJobFiles(jobId: JobId, files: FileUpload[]): Promise<UploadResult>;
  
  // Download job output file
  downloadJobOutput(jobId: JobId, fileName: string): Promise<DownloadResult>;
  
  // List files in job directory
  listJobFiles(jobId: JobId): Promise<ListFilesResult>;
}

interface FileUpload {
  localPath: string;
  remoteName: string;
}

interface UploadResult {
  success: boolean;
  uploadedFiles?: string[];
  failedUploads?: Array<{
    fileName: string;
    error: string;
  }>;
}

interface DownloadResult {
  success: boolean;
  content?: string;      // For text files
  filePath?: string;     // Local path for downloaded file
  fileSize?: number;
  error?: string;
}

interface RemoteFile {
  name: string;
  size: number;
  modifiedAt: Timestamp;
  fileType: 'input' | 'output' | 'config' | 'log';
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
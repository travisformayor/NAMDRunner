# API Specification

This document defines the complete API interface for NAMDRunner, including IPC commands between the frontend and backend, and proven SLURM integration patterns.

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

### SSH Connection Best Practices

> **For complete Alpine cluster integration patterns, see [`docs/cluster-guide.md`](cluster-guide.md)**.

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

## SLURM Integration

> **For complete SLURM integration patterns including job submission, status monitoring, error handling, and command examples, see [`docs/cluster-guide.md`](cluster-guide.md)**.

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

#### SLURM Errors

> **For complete SLURM error patterns and handling strategies, see [`docs/cluster-guide.md`](cluster-guide.md)**.

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

## Resource Limits

> **For current resource limits, partition details, QoS specifications, and allocation recommendations, see [`docs/cluster-guide.md`](cluster-guide.md)**.

## Integration Best Practices

### Command Reliability
1. Always load modules before SLURM commands
2. Use full paths for working directories
3. Check command exit codes
4. Parse stderr for error messages
5. Handle network timeouts gracefully

### Interaction Optimization
1. Batch SLURM queries when possible
2. Cache job status to avoid repeated queries
3. Use background threads for long operations
4. Limit concurrent SSH connections

### Error Recovery
1. Retry failed commands with exponential backoff
2. Validate SSH connection before SLURM commands
3. Handle partial failures in batch operations
4. Provide clear error messages to users

## Mock Implementation Data

### Fixture Responses for Testing
```rust
// Mock squeue response
const MOCK_SQUEUE_RUNNING: &str = "12345678|test_job|R|00:15:30|01:44:30|1|24|16GB|amilan|/scratch/alpine/testuser/namdrunner_jobs/test_job";

// Mock sacct response
const MOCK_SACCT_COMPLETED: &str = "12345678|test_job|COMPLETED|0:0|2025-01-15T10:00:00|2025-01-15T11:00:00|01:00:00|/scratch/alpine/testuser/namdrunner_jobs/test_job";

// Mock sbatch response
const MOCK_SBATCH_SUCCESS: &str = "Submitted batch job 12345678";
```

> **For complete SLURM command patterns and response formats, see [`docs/cluster-guide.md`](cluster-guide.md)**.

## Important Implementation Notes

1. **Always use full paths** for working directories
2. **Module commands must be sourced properly** with `/etc/profile`
3. **Parse both stdout and stderr** for error detection
4. **Handle queue wait times** - jobs may be PENDING for hours
5. **Account for 90-day scratch purge policy**
6. **Never log or persist passwords** - memory only
7. **Validate SSH connection** before SLURM operations
8. **Use working directory pattern** to identify NAMDRunner jobs: `/scratch/alpine/$USER/namdrunner_jobs/*`

These patterns are proven to work with the CURC Alpine cluster and provide the foundation for reliable SLURM integration in the Tauri implementation.
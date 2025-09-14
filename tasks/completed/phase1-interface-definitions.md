# Phase 1 Interface Definitions

This document contains the fundamental interfaces and schemas that must be defined in Phase 1 before implementation begins. These serve as contracts between the frontend and backend.

## IPC Command Interface

### Core Type Definitions

```typescript
// Connection state management
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';

// Job lifecycle states
type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

// Basic types
type JobId = string;        // Format: job_001, job_002, etc.
type SlurmJobId = string;   // SLURM's job ID (numbers)
type Timestamp = string;    // ISO 8601 format
```

### Connection Management Commands

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

### Job Management Commands

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

### File Management Commands

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

## JSON Metadata Schema (Phase 1 Version)

### job_info.json (Single Job)

```json
{
  "schema_version": "1.0",
  "job_id": "job_001",
  "job_name": "test_simulation",
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T11:45:00Z",
  "status": "COMPLETED",
  "slurm_job_id": "12345678",
  "submitted_at": "2025-01-15T10:35:00Z",
  "completed_at": "2025-01-15T11:00:00Z",
  "config": {
    "namd": {
      "steps": 50000,
      "temperature": 310.0,
      "timestep": 2.0,
      "outputname": "output",
      "dcd_freq": 1000,
      "restart_freq": 5000
    },
    "slurm": {
      "cores": 24,
      "memory": "16GB", 
      "walltime": "02:00:00",
      "partition": "amilan",
      "qos": "normal"
    }
  },
  "input_files": [
    {
      "name": "structure.pdb",
      "path": "input_files/structure.pdb",
      "type": "pdb"
    },
    {
      "name": "structure.psf", 
      "path": "input_files/structure.psf",
      "type": "psf"
    },
    {
      "name": "parameters.prm",
      "path": "input_files/parameters.prm", 
      "type": "prm"
    }
  ],
  "generated_files": {
    "namd_config": "config.namd",
    "slurm_script": "job.sbatch"
  },
  "output_files": {
    "slurm_stdout": "test_simulation_12345678.out",
    "slurm_stderr": "test_simulation_12345678.err",
    "namd_log": "namd_output.log"
  },
  "directories": {
    "project_dir": "/projects/username/namdrunner_jobs/job_001",
    "scratch_dir": "/scratch/alpine/username/namdrunner_jobs/job_001"
  },
  "error_info": null
}
```

## SQLite Schema (Phase 1 Version)

```sql
-- Jobs table for local cache
CREATE TABLE jobs (
    job_id TEXT PRIMARY KEY,               -- Our internal job ID
    job_name TEXT NOT NULL,
    status TEXT NOT NULL,                  -- JobStatus enum values
    slurm_job_id TEXT,                     -- SLURM's job ID
    created_at TEXT NOT NULL,              -- ISO 8601 timestamp
    updated_at TEXT,
    submitted_at TEXT,
    completed_at TEXT,
    project_dir TEXT,
    scratch_dir TEXT,
    
    -- JSON columns for complex data
    namd_config_json TEXT,                 -- NAMD configuration as JSON
    slurm_config_json TEXT,                -- SLURM configuration as JSON
    input_files_json TEXT,                 -- Input file list as JSON
    output_files_json TEXT,                -- Output file list as JSON
    error_info TEXT                        -- Error information
);

-- Indexes for performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_slurm_id ON jobs(slurm_job_id);
CREATE INDEX idx_jobs_updated ON jobs(updated_at);

-- Application metadata table
CREATE TABLE app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT
);

-- Insert schema version
INSERT INTO app_metadata (key, value, updated_at) 
VALUES ('schema_version', '1.0', datetime('now'));
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

// ... other result types follow same pattern
```

## Error Handling Strategy

### Error Categories
```typescript
// Frontend error types
interface NAMDRunnerError {
  category: 'Network' | 'Authentication' | 'Validation' | 'FileSystem' | 'SLURM' | 'Internal';
  message: string;
  details?: string;
  retryable: boolean;
}

// Example error responses
const NETWORK_ERROR: NAMDRunnerError = {
  category: 'Network',
  message: 'Failed to connect to cluster',
  details: 'Connection timed out after 30 seconds',
  retryable: true
};

const VALIDATION_ERROR: NAMDRunnerError = {
  category: 'Validation', 
  message: 'Invalid NAMD parameters',
  details: 'Temperature must be between 200 and 400 Kelvin',
  retryable: false
};
```

## File Organization Requirements

### Directory Structure
```
/projects/$USER/namdrunner_jobs/
└── {job_id}/
    ├── job_info.json           # This schema
    ├── input_files/
    │   ├── structure.pdb
    │   ├── structure.psf
    │   └── parameters.prm
    ├── config.namd             # Generated NAMD config
    ├── job.sbatch              # Generated SLURM script
    └── outputs/                # After job completion
        ├── {job_name}_{slurm_job_id}.out
        ├── {job_name}_{slurm_job_id}.err
        └── namd_output.log

/scratch/alpine/$USER/namdrunner_jobs/
└── {job_id}/                   # Working directory during execution
```

These definitions provide the foundation for implementing the NAMDRunner application. All Phase 1 implementation should adhere to these interfaces to ensure consistency and maintainability.
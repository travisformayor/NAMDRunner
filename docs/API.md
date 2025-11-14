# API & Data Contracts

This document defines all interfaces and data contracts for NAMDRunner, consolidating IPC communication patterns and data schemas.

## Table of Contents
- [Implementation Notes](#implementation-notes)
- [Core Type Definitions](#core-type-definitions)
  - [Connection State Management](#connection-state-management)
  - [Job Lifecycle States](#job-lifecycle-states)
  - [Basic Types](#basic-types)
  - [ApiResult Pattern](#apiresult-pattern)
- [Connection Management Commands](#connection-management-commands)
  - [IPC Interface](#ipc-interface)
- [Job Management Commands](#job-management-commands)
  - [IPC Interface](#ipc-interface-1)
  - [Request/Response Types](#requestresponse-types)
- [File Management Commands](#file-management-commands)
  - [IPC Interface](#ipc-interface-2)
- [Database Management Commands](#database-management-commands)
  - [IPC Interface](#ipc-interface-3)
- [Template Management Commands](#template-management-commands)
  - [IPC Interface](#ipc-interface-4)
  - [Template Rendering](#template-rendering)
  - [Default Templates](#default-templates)
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

### ApiResult Pattern

Most commands return `ApiResult<T>`, a generic wrapper providing consistent success/error handling:

```typescript
interface ApiResult<T> {
  success: boolean;
  data?: T;         // Present when success is true
  error?: string;   // Present when success is false
}

// Usage examples:
// - ApiResult<JobInfo> for get_job_status
// - ApiResult<JobInfo[]> for get_all_jobs
// - ApiResult<string> for preview_slurm_script
// - ApiResult<void> for disconnect, delete operations
```

Commands with unique multi-field responses use specialized result types (e.g., `SyncJobsResult`, `UploadResult`, `ValidationResult`).


## Connection Management Commands

### IPC Interface

All commands return `ApiResult<T>` which wraps the result with success/error information (see [ApiResult Pattern](#apiresult-pattern)).

```typescript
interface IConnectionCommands {
  // Establish SSH connection to cluster
  connect_to_cluster(params: ConnectParams): Promise<ApiResult<SessionInfo>>;

  // Close SSH connection
  disconnect(): Promise<ApiResult<void>>;

  // Check current connection status
  get_connection_status(): Promise<ApiResult<ConnectionStatus>>;

  // Get cluster capabilities (partitions, QoS options, etc.)
  get_cluster_capabilities(): Promise<ApiResult<ClusterCapabilities>>;

  // Helper functions for cluster resource planning
  suggest_qos_for_partition(walltime_hours: number, partition_id: string): Promise<string>;
  estimate_queue_time_for_job(cores: number, partition_id: string): Promise<string>;
  calculate_job_cost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number>;

  // Validate resource allocation against cluster constraints
  validate_resource_allocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<ValidationResult>;
}

interface SessionInfo {
  host: string;
  username: string;
  connected_at: Timestamp;
}

interface ConnectionStatus {
  state: ConnectionState;
  session_info?: SessionInfo;
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

All commands return `ApiResult<T>` or specialized result types. Commands use snake_case naming per Implementation Note #9.

Example invocation:
```typescript
// Correct - snake_case matches Rust backend
await invoke('submit_job', { job_id: "test1_123" });

// Wrong - would fail with "missing required key job_id"
await invoke('submit_job', { jobId: "test1_123" });
```

```typescript
interface IJobCommands {
  // Create new job (local only, not submitted yet)
  create_job(params: CreateJobParams): Promise<ApiResult<JobInfo>>;

  // Submit job to SLURM cluster
  submit_job(job_id: JobId): Promise<ApiResult<JobSubmissionData>>;

  // Get status of specific job
  get_job_status(job_id: JobId): Promise<ApiResult<JobInfo>>;

  // Get all jobs from local cache
  get_all_jobs(): Promise<ApiResult<JobInfo[]>>;

  // Sync job statuses with cluster (includes automatic discovery if database empty)
  sync_jobs(): Promise<SyncJobsResult>;

  // Delete job (local and optionally remote)
  delete_job(job_id: JobId, delete_remote: boolean): Promise<ApiResult<void>>;

  // Refetch SLURM logs from server (overwrites cached logs)
  refetch_slurm_logs(job_id: JobId): Promise<ApiResult<JobInfo>>;

  // Preview SLURM script before submission
  preview_slurm_script(job_name: string, cores: number, memory: string, walltime: string, partition?: string, qos?: string): Promise<ApiResult<string>>;

  // Validate complete job configuration
  validate_job_config(params: ValidateJobConfigParams): Promise<ValidationResult>;
}
```

**Job Discovery Integration:**

The `sync_jobs()` command automatically handles job discovery when the local database is empty (e.g., first connection after database reset). This eliminates the need for separate discovery commands and multi-step frontend orchestration.

**Workflow:**
1. Query SLURM for active job status updates
2. If database is empty, automatically discover jobs from `/projects/$USER/namdrunner_jobs/`
3. Return complete job list (including discovered and synced jobs)

Frontend receives complete state in a single call. See [`AUTOMATIONS.md`](AUTOMATIONS.md#3-status-synchronization-automation-chain) for implementation details.

### Request/Response Types
```typescript
interface CreateJobParams {
  job_name: string;
  template_id: string;                     // Template to use (e.g., "explicit_solvent_npt_v1")
  template_values: Record<string, any>;    // Variable values for template
  slurm_config: {
    cores: number;
    memory: string;      // e.g., "16GB"
    walltime: string;    // e.g., "02:00:00"
    partition?: string;  // default: "amilan"
    qos?: string;        // default: "normal"
  };
}

// Example template_values:
// {
//   "structure_file": "/path/to/hextube.psf",  // Local path, will be uploaded
//   "coordinates_file": "/path/to/hextube.pdb",
//   "parameters_file": "/path/to/par_all36_na.prm",
//   "temperature": 300.0,
//   "timestep": 2.0,
//   "steps": 4800,
//   "execution_command": "minimize"
// }

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
  slurm_stdout?: string;
  slurm_stderr?: string;
  template_id: string;                     // Template used for this job
  template_values: Record<string, any>;    // Variable values for template
  slurm_config: SlurmConfig;
  input_files?: string[];                  // List of uploaded input files
  output_files?: OutputFile[];
  remote_directory: string;
}

interface JobSubmissionData {
  slurm_job_id: SlurmJobId;
  submitted_at: Timestamp;
  job_id: JobId;             // Local job ID
}

interface SyncJobsResult {
  success: boolean;
  jobs: JobInfo[];           // Complete job list (discovery + sync results)
  jobs_updated: number;      // Count of jobs that had status updates (u32 in Rust)
  errors: string[];
}

interface ValidateJobConfigParams {
  job_name: string;
  template_id: string;
  template_values: Record<string, any>;
  cores: number;
  memory: string;
  walltime: string;
  partition?: string;
  qos?: string;
}

interface ValidationResult {
  is_valid: boolean;
  issues: string[];
  warnings: string[];
  suggestions: string[];
}
```

## File Management Commands

### IPC Interface
```typescript
interface IFileCommands {
  // Detect NAMD file type from filename
  detect_file_type(filename: string): Promise<string>;

  // Open file dialog to select a single NAMD input file
  select_input_file(): Promise<SelectedFile | null>;

  // Upload files to job directory on cluster
  upload_job_files(job_id: JobId, files: FileUpload[]): Promise<UploadResult>;

  // Download single job output file (shows native save dialog)
  download_job_output(job_id: JobId, file_path: string): Promise<ApiResult<DownloadInfo>>;

  // Download all output files as ZIP archive (shows native save dialog)
  download_all_outputs(job_id: JobId): Promise<ApiResult<DownloadInfo>>;

  // Download single job input file (shows native save dialog)
  download_job_input(job_id: JobId, file_path: string): Promise<ApiResult<DownloadInfo>>;

  // Download all input files as ZIP archive (shows native save dialog)
  download_all_inputs(job_id: JobId): Promise<ApiResult<DownloadInfo>>;

  // List files in job directory
  list_job_files(job_id: JobId): Promise<ApiResult<RemoteFile[]>>;
}

interface FileUpload {
  local_path: string;
  remote_name: string;
}

interface SelectedFile {
  name: string;          // Filename
  path: string;          // Full local path
  size: number;          // File size in bytes
  file_type: string;     // File extension (e.g., ".pdb")
}

interface UploadResult {
  success: boolean;
  uploaded_files?: string[];
  failed_uploads?: Array<{
    file_name: string;
    error: string;
  }>;
}

interface DownloadInfo {
  saved_to: string;      // Local path where file was saved (via native dialog)
  file_size: number;     // File size in bytes
}

interface RemoteFile {
  name: string;          // Display name (just filename)
  path: string;          // Full relative path from job root (e.g., "outputs/sim.dcd")
  size: number;
  modified_at: Timestamp;
  file_type: 'input' | 'output' | 'config' | 'log';
}
```

## Database Management Commands

### IPC Interface
```typescript
interface IDatabaseCommands {
  // Get database path, size, and job count
  get_database_info(): Promise<ApiResult<DatabaseInfo>>;

  // Backup database to user-selected location
  backup_database(): Promise<ApiResult<DatabaseOperationData>>;

  // Restore database from user-selected backup file
  restore_database(): Promise<ApiResult<DatabaseOperationData>>;

  // Reset database (delete all data and recreate schema)
  reset_database(): Promise<ApiResult<DatabaseOperationData>>;
}

interface DatabaseInfo {
  path: string;           // Full path to database file
  size_bytes: number;     // Database file size in bytes
  job_count: number;      // Number of jobs in database
}

interface DatabaseOperationData {
  path: string;           // Path to database file
  message: string;        // Operation result message
}
```

**Implementation**: [src-tauri/src/commands/database.rs](../src-tauri/src/commands/database.rs)

> **For platform-specific database paths, operational details, and connection management**, see [`docs/DB.md`](DB.md)

## Template Management Commands

### IPC Interface
```typescript
interface ITemplateCommands {
  // List all templates (returns summary for template selection)
  list_templates(): Promise<ApiResult<TemplateSummary[]>>;

  // Get full template definition (for editing or job creation)
  get_template(template_id: string): Promise<ApiResult<Template>>;

  // Create new user template
  create_template(template: Template): Promise<ApiResult<string>>;

  // Update existing template
  update_template(template_id: string, template: Template): Promise<ApiResult<void>>;

  // Delete template (blocked if jobs exist using it)
  delete_template(template_id: string): Promise<ApiResult<void>>;

  // Validate template values against template definition
  validate_template_values(
    template_id: string,
    values: Record<string, any>
  ): Promise<ValidationResult>;

  // Preview rendered NAMD config with user values
  preview_namd_config(
    template_id: string,
    values: Record<string, any>
  ): Promise<ApiResult<string>>;

  // Preview template with default/sample values (for template editor testing)
  preview_template_with_defaults(template_id: string): Promise<ApiResult<string>>;
}

interface Template {
  id: string;
  name: string;
  description: string;
  namd_config_template: string;            // NAMD config with {{variables}}
  variables: Record<string, VariableDefinition>;
  is_builtin: boolean;                     // True for embedded templates, false for user-created
  created_at: string;
  updated_at: string;
}

interface VariableDefinition {
  key: string;
  label: string;
  var_type: VariableType;
  help_text?: string;
}

type VariableType =
  | { Number: { min?: number; max?: number; default?: number } }
  | { Text: { default?: string } }
  | { Boolean: { default: boolean } }
  | { FileUpload: { extensions: string[] } };

interface TemplateSummary {
  id: string;
  name: string;
  description: string;
  is_builtin: boolean;
}
```

**Actual Rust Commands**: Located in [src-tauri/src/commands/templates.rs](../src-tauri/src/commands/templates.rs)

### Template Rendering

Templates use `{{variable}}` syntax for variable substitution. During job creation:

1. Template loaded from database
2. Files extracted from template_values (FileUpload variables)
3. Files uploaded to cluster's `input_files/` directory
4. template_values updated with filenames (not full paths)
5. Template rendered: `{{variable}}` replaced with values
6. Type-specific rendering:
   - **FileUpload**: `{{structure_file}}` → `input_files/hextube.psf`
   - **Boolean**: `{{pme_enabled}}` → `yes` or `no`
   - **Number**: `{{temperature}}` → `300` (integers without .0)
   - **Text**: `{{output_name}}` → `npt_equilibration`

### Default Templates

**Built-in templates** are auto-loaded from `src-tauri/templates/*.json` on first app startup:
- `vacuum_optimization_v1` - Vacuum simulation with large periodic box
- `explicit_solvent_npt_v1` - NPT ensemble with PME electrostatics

Templates are stored in SQLite database after initial load. Embedded templates have `is_builtin: true` to communicate their origin to the user.

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
    pub created_at: String,  // RFC3339 timestamp
    pub updated_at: Option<String>,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub project_dir: Option<String>,
    pub scratch_dir: Option<String>,
    pub error_info: Option<String>,
    pub slurm_stdout: Option<String>,
    pub slurm_stderr: Option<String>,

    // Template-based configuration
    pub template_id: String,
    pub template_values: HashMap<String, serde_json::Value>,

    pub slurm_config: SlurmConfig,
    pub input_files: Option<Vec<String>>,   // List of uploaded input files
    pub output_files: Option<Vec<OutputFile>>,
    pub remote_directory: String,
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
pub struct OutputFile {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

// Template types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub namd_config_template: String,
    pub variables: HashMap<String, VariableDefinition>,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub key: String,
    pub label: String,
    pub var_type: VariableType,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Number { min: f64, max: f64, default: f64 },
    Text { default: String },
    Boolean { default: bool },
    FileUpload { extensions: Vec<String> },
}
```

**Location**: [src-tauri/src/types/core.rs](../src-tauri/src/types/core.rs), [src-tauri/src/templates/types.rs](../src-tauri/src/templates/types.rs)

### Command Result Types

**Generic Result Pattern**: Most commands return `ApiResult<T>`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Usage examples:
// - ApiResult<JobInfo> for get_job_status
// - ApiResult<Vec<JobInfo>> for get_all_jobs
// - ApiResult<String> for preview_slurm_script
// - ApiResult<()> for disconnect, delete operations
// - ApiResult<SessionInfo> for connect_to_cluster
// - ApiResult<ConnectionStatus> for get_connection_status
// - ApiResult<DownloadInfo> for download commands
// - ApiResult<Vec<RemoteFile>> for list_job_files
// - ApiResult<DatabaseInfo> for get_database_info
// - ApiResult<DatabaseOperationData> for backup/restore/reset operations
```

**Specialized Result Types** (for commands with unique multi-field responses):

```rust
// Session information (wrapped in ApiResult by connect_to_cluster)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    pub connected_at: String,  // RFC3339 timestamp
}

// Connection status (wrapped in ApiResult by get_connection_status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub state: ConnectionState,
    pub session_info: Option<SessionInfo>,
}

// Job submission response (wrapped in ApiResult by submit_job)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSubmissionData {
    pub slurm_job_id: String,
    pub submitted_at: String,
    pub job_id: String,
}

// Job sync result (not wrapped - contains its own success field)
#[derive(Debug, Serialize)]
pub struct SyncJobsResult {
    pub success: bool,
    pub jobs: Vec<JobInfo>,
    pub jobs_updated: u32,
    pub errors: Vec<String>,
}

// Validation result (not wrapped - contains its own is_valid field)
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

// File upload result (not wrapped - contains its own success field)
#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub success: bool,
    pub uploaded_files: Option<Vec<String>>,
    pub failed_uploads: Option<Vec<FailedUpload>>,
}

#[derive(Debug, Serialize)]
pub struct FailedUpload {
    pub file_name: String,
    pub error: String,
}

// Download info (wrapped in ApiResult by download commands)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub saved_to: String,
    pub file_size: u64,
}

// Database info (wrapped in ApiResult by get_database_info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub job_count: usize,
}

// Database operation result (wrapped in ApiResult by backup/restore/reset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOperationData {
    pub path: String,
    pub message: String,
}
```

**Location**: [src-tauri/src/types/core.rs](../src-tauri/src/types/core.rs), [src-tauri/src/types/commands.rs](../src-tauri/src/types/commands.rs), [src-tauri/src/types/response_data.rs](../src-tauri/src/types/response_data.rs), [src-tauri/src/validation/job_validation.rs](../src-tauri/src/validation/job_validation.rs)

## SLURM Integration

> **For complete SLURM integration patterns including job submission, status monitoring, error handling, and command examples, see [`docs/reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md) and [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md)**.

## Related Documentation

For SSH/SFTP connection management, security patterns, and file transfer implementation details, see [`docs/SSH.md`](SSH.md).

For architectural principles, clean architecture patterns, and development best practices, see [`CONTRIBUTING.md#developer-standards--project-philosophy`](CONTRIBUTING.md#developer-standards--project-philosophy).

For testing strategies and infrastructure setup, see [`docs/CONTRIBUTING.md#testing-strategy`](CONTRIBUTING.md#testing-strategy).

For data schemas, SQLite database design, and persistence patterns, see [`docs/DB.md`](DB.md).
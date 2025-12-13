# API & Data Contracts

**IPC command interfaces and data contracts for the TypeScript ↔ Rust boundary.**

> See project README for project overview.
>
> **Related Docs:**
>
> - [SSH.md](SSH.md) - Connection management
> - [AUTOMATIONS.md](AUTOMATIONS.md) - Job automation
> - [DB.md](DB.md) - Database schemas

## Naming Conventions

**All Tauri commands use `#[tauri::command(rename_all = "snake_case")]`:**

- Frontend and backend use identical parameter names (snake_case)
- Eliminates conversion logic and naming confusion
- Improves code searchability across the codebase

```typescript
// Correct - snake_case matches Rust backend
await invoke('submit_job', { job_id: "test1_123" });

// Wrong - would fail with "missing required key job_id"
await invoke('submit_job', { jobId: "test1_123" });
```

**Struct parameters are wrapped:**

- Commands accepting structs: `{params: {host, username, password}}`
- Commands with individual parameters: `{job_id: "job_001"}`

## ApiResult Pattern

Most commands return `ApiResult<T>`:

```typescript
interface ApiResult<T> {
  success: boolean;
  data?: T;         // Present when success is true
  error?: string;   // Present when success is false
}

// Common usages:
// ApiResult<JobInfo> - Single job
// ApiResult<JobInfo[]> - Job list
// ApiResult<string> - Preview/path results
// ApiResult<void> - Operations with no return data
```

Commands with unique multi-field responses use specialized types: `SyncJobsResult`, `UploadResult`, `ValidationResult`.

## Connection Management

```typescript
interface IConnectionCommands {
  connect_to_cluster(params: ConnectParams): Promise<ApiResult<SessionInfo>>;
  disconnect(): Promise<ApiResult<void>>;
  get_connection_status(): Promise<ApiResult<ConnectionStatus>>;
  get_cluster_capabilities(): Promise<ApiResult<ClusterCapabilities>>;

  // Resource helpers
  suggest_qos_for_partition(walltime_hours: number, partition_id: string): Promise<string>;
  estimate_queue_time_for_job(cores: number, partition_id: string): Promise<string>;
  calculate_job_cost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number>;
  validate_resource_allocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<ValidationResult>;
}

type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';

interface SessionInfo {
  host: string;
  username: string;
  connected_at: string;  // ISO 8601
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
```

See `src/lib/types/cluster.ts` for complete cluster type definitions.

## Job Management

```typescript
interface IJobCommands {
  create_job(params: CreateJobParams): Promise<ApiResult<JobInfo>>;
  submit_job(job_id: string): Promise<ApiResult<JobInfo>>;
  get_job_status(job_id: string): Promise<ApiResult<JobInfo>>;
  get_all_jobs(): Promise<ApiResult<JobInfo[]>>;
  sync_jobs(): Promise<SyncJobsResult>;
  delete_job(job_id: string, delete_remote: boolean): Promise<ApiResult<void>>;
  refetch_slurm_logs(job_id: string): Promise<ApiResult<JobInfo>>;
  preview_slurm_script(job_name: string, cores: number, memory: string, walltime: string, partition?: string, qos?: string): Promise<ApiResult<string>>;
  validate_job_config(params: ValidateJobConfigParams): Promise<ValidationResult>;
}

type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

interface CreateJobParams {
  job_name: string;
  template_id: string;
  template_values: Record<string, any>;  // Template variable values
  slurm_config: {
    cores: number;
    memory: string;      // e.g., "16GB"
    walltime: string;    // e.g., "02:00:00"
    partition?: string;
    qos?: string;
  };
}

interface JobInfo {
  job_id: string;
  job_name: string;
  status: JobStatus;
  slurm_job_id?: string;
  created_at: string;
  updated_at?: string;
  submitted_at?: string;
  completed_at?: string;
  project_dir?: string;
  scratch_dir?: string;
  error_info?: string;
  slurm_stdout?: string;
  slurm_stderr?: string;
  template_id: string;
  template_values: Record<string, any>;
  slurm_config: SlurmConfig;
  input_files: string[];
  output_files: OutputFile[];
  remote_directory: string;
}

interface SlurmConfig {
  cores: number;
  memory: string;
  walltime: string;
  partition?: string;
  qos?: string;
}

interface SyncJobsResult {
  success: boolean;
  jobs: JobInfo[];       // Complete job list
  jobs_updated: number;  // Count of status updates
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

**sync_jobs() behavior:**

- Queries SLURM for status updates
- Auto-discovers jobs from `/projects/$USER/namdrunner_jobs/` if database empty
- Returns complete job list in single call

See [`AUTOMATIONS.md`](AUTOMATIONS.md#3-status-synchronization-automation-chain) for workflow details.

## File Management

```typescript
interface IFileCommands {
  detect_file_type(filename: string): Promise<string>;
  select_input_file(): Promise<SelectedFile | null>;
  upload_job_files(job_id: string, files: FileUpload[]): Promise<UploadResult>;
  download_file(job_id: string, file_type: 'input' | 'output', file_path: string): Promise<ApiResult<DownloadInfo>>;
  download_all_files(job_id: string, file_type: 'input' | 'output'): Promise<ApiResult<DownloadInfo>>;
  list_job_files(job_id: string): Promise<ApiResult<RemoteFile[]>>;
}

interface FileUpload {
  local_path: string;
  remote_name: string;
}

interface SelectedFile {
  name: string;       // Filename
  path: string;       // Full local path
  size: number;       // Bytes
  file_type: string;  // Extension (e.g., ".pdb")
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
  saved_to: string;   // Local path (via native dialog)
  file_size: number;  // Bytes
}

interface RemoteFile {
  name: string;       // Filename only
  path: string;       // Relative path from job root
  size: number;
  modified_at: string;
  file_type: 'input' | 'output' | 'config' | 'log';
}
```

## Database Management

```typescript
interface IDatabaseCommands {
  get_database_info(): Promise<ApiResult<DatabaseInfo>>;
  backup_database(): Promise<ApiResult<DatabaseOperationData>>;
  restore_database(): Promise<ApiResult<DatabaseOperationData>>;
  reset_database(): Promise<ApiResult<DatabaseOperationData>>;
}

interface DatabaseInfo {
  path: string;        // Full path to database file
  size_bytes: number;  // Database file size
  job_count: number;   // Number of jobs
}

interface DatabaseOperationData {
  path: string;        // Database file path
  message: string;     // Operation result
}
```

**Implementation:** `src-tauri/src/commands/database.rs`

See [`DB.md`](DB.md) for platform paths and operational details.

## Template Management

```typescript
interface ITemplateCommands {
  list_templates(): Promise<ApiResult<TemplateSummary[]>>;
  get_template(template_id: string): Promise<ApiResult<Template>>;
  create_template(template: Template): Promise<ApiResult<string>>;
  update_template(template_id: string, template: Template): Promise<ApiResult<void>>;
  delete_template(template_id: string): Promise<ApiResult<void>>;
  export_template(template_id: string): Promise<ApiResult<string>>;
  import_template(): Promise<ApiResult<Template>>;
  validate_template_values(template_id: string, values: Record<string, any>): Promise<ValidationResult>;
  preview_namd_config(template_id: string, values: Record<string, any>): Promise<ApiResult<string>>;
  preview_template_with_defaults(template_id: string): Promise<ApiResult<string>>;
}

interface Template {
  id: string;
  name: string;
  description: string;
  namd_config_template: string;  // NAMD config with {{variables}}
  variables: Record<string, VariableDefinition>;
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
}
```

**Implementation:** `src-tauri/src/commands/templates.rs`

### Template Rendering

Templates use `{{variable}}` syntax. During job creation:

1. Template loaded from database
2. Files extracted from template_values (FileUpload variables)
3. Files uploaded to `input_files/` directory
4. template_values updated with filenames only
5. Variables replaced with type-specific rendering:
   - **FileUpload**: `{{structure_file}}` → `input_files/hextube.psf`
   - **Boolean**: `{{pme_enabled}}` → `yes` or `no`
   - **Number**: `{{temperature}}` → `300` (integers without .0)
   - **Text**: `{{output_name}}` → `npt_equilibration`

### Default Templates

Built-in templates auto-load from `src-tauri/templates/*.json` on first startup:

- `vacuum_optimization_v1` - Vacuum simulation with large periodic box
- `explicit_solvent_npt_v1` - NPT ensemble with PME electrostatics

Templates stored in SQLite after initial load.

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
    pub created_at: String,  // RFC3339
    pub updated_at: Option<String>,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub project_dir: Option<String>,
    pub scratch_dir: Option<String>,
    pub error_info: Option<String>,
    pub slurm_stdout: Option<String>,
    pub slurm_stderr: Option<String>,
    pub template_id: String,
    pub template_values: HashMap<String, serde_json::Value>,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<String>,
    pub output_files: Vec<OutputFile>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub namd_config_template: String,
    pub variables: HashMap<String, VariableDefinition>,
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

**Location:** `src-tauri/src/types/core.rs`, `src-tauri/src/templates/types.rs`

### Result Types

**Generic Pattern:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**Specialized Types:**

```rust
// Session info (wrapped in ApiResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    pub connected_at: String,  // RFC3339
}

// Connection status (wrapped in ApiResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub state: ConnectionState,
    pub session_info: Option<SessionInfo>,
}

// Job sync (not wrapped - has own success field)
#[derive(Debug, Serialize)]
pub struct SyncJobsResult {
    pub success: bool,
    pub jobs: Vec<JobInfo>,
    pub jobs_updated: u32,
    pub errors: Vec<String>,
}

// Validation (not wrapped - has own is_valid field)
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

// Upload (not wrapped - has own success field)
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

// Download info (wrapped in ApiResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub saved_to: String,
    pub file_size: u64,
}

// Database info (wrapped in ApiResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub job_count: usize,
}

// Database operations (wrapped in ApiResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOperationData {
    pub path: String,
    pub message: String,
}
```

**Location:** `src-tauri/src/types/core.rs`, `src-tauri/src/types/commands.rs`, `src-tauri/src/types/response_data.rs`, `src-tauri/src/validation/job.rs`

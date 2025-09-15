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

## SSH/Network Operations Implementation
*Implementation patterns for SSH/SFTP development*

### Security Implementation Guidelines
*Critical security lessons for secure implementation*

#### Secure Credential Handling
Always use SecStr for passwords and sensitive data with proper memory management.

```rust
use crate::security::SecurePassword;
use secstr::SecStr;

// ✅ Complete secure password implementation
#[derive(Clone)]
pub struct SecurePassword(SecStr);

impl SecurePassword {
    pub fn new(password: String) -> Self {
        Self(SecStr::from(password))
    }

    pub fn with_password<F, R>(&self, f: F) -> R
    where F: FnOnce(&str) -> R
    {
        let bytes = self.0.unsecure();
        let password_str = std::str::from_utf8(bytes).unwrap_or("");
        f(password_str)
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        // SecStr handles secure memory clearing automatically
    }
}

impl std::fmt::Debug for SecurePassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecurePassword([REDACTED])")
    }
}

// ✅ Secure connection implementation
impl ConnectionManager {
    pub async fn connect(&self, host: String, username: String, password: &SecurePassword) -> Result<ConnectionInfo> {
        // Use password only within closure to minimize exposure time
        password.with_password(|pwd| {
            // Connect using the password string temporarily
            connection.authenticate_password(&username, pwd)
        })?;
        // Password is automatically cleared from memory
        Ok(connection_info)
    }
}

// ❌ Insecure password handling
fn connect_insecure(password: String) {
    // Don't store passwords as String - no automatic cleanup
}
```

**Safe Logging**: Never log credentials or sensitive data.

```typescript
// ✅ Safe logging
logger.info('Connecting to cluster', {
  host: config.host,
  username: config.username
  // Never log password
});

// ❌ Dangerous logging
logger.info('Connection attempt', { config }); // May contain password
```

#### Path Security & Input Validation
**Sanitize All User Inputs**: Never use user input directly in path construction or shell commands.

```rust
// ❌ DANGEROUS - Direct user input in path construction
pub fn create_job_directory(username: &str, job_name: &str) -> String {
    format!("/projects/{}/jobs/{}", username, job_name)
    // Risk: username="../../../etc" or job_name="important_file; rm -rf /"
}

// ✅ SECURE - Sanitize all inputs before use
pub fn create_job_directory(username: &str, job_name: &str) -> Result<String> {
    let clean_username = sanitize_path_component(username)?;
    let clean_job_name = sanitize_path_component(job_name)?;

    validate_no_traversal(&clean_username)?;
    validate_no_traversal(&clean_job_name)?;

    Ok(format!("/projects/{}/jobs/{}", clean_username, clean_job_name))
}

fn sanitize_path_component(input: &str) -> Result<String> {
    // Remove dangerous characters
    let cleaned = input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    if cleaned.is_empty() {
        return Err(anyhow::anyhow!("Invalid path component"));
    }

    Ok(cleaned)
}

fn validate_no_traversal(path: &str) -> Result<()> {
    if path.contains("..") || path.contains('/') || path.contains('\\') {
        return Err(anyhow::anyhow!("Path traversal detected"));
    }
    Ok(())
}
```

**Command Injection Prevention**: Always escape shell parameters when executing remote commands.

```rust
// ❌ DANGEROUS - Command injection vulnerability
pub async fn run_slurm_command(job_name: &str) -> Result<String> {
    let command = format!("sbatch {}.slurm", job_name);
    ssh_manager.execute_command(&command).await
}

// ✅ SECURE - Properly escape shell parameters
pub async fn run_slurm_command(job_name: &str) -> Result<String> {
    let sanitized_name = sanitize_filename(job_name)?;
    let escaped_name = shell_escape::escape(std::borrow::Cow::Borrowed(&sanitized_name));
    let command = format!("sbatch {}.slurm", escaped_name);
    ssh_manager.execute_command(&command).await
}

fn sanitize_filename(filename: &str) -> Result<String> {
    let cleaned = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect::<String>();

    if cleaned.is_empty() || cleaned.starts_with('.') {
        return Err(anyhow::anyhow!("Invalid filename"));
    }

    Ok(cleaned)
}
```

**Path Validation**: Implement comprehensive validation for all user inputs.

```rust
// ✅ Essential path validation
pub fn validate_path_component(input: &str) -> Result<()> {
    if input.contains('\0') || input.contains("..") || input.starts_with('/') {
        return Err(anyhow::anyhow!("Invalid path component"));
    }

    if input.len() > 64 || input.is_empty() {
        return Err(anyhow::anyhow!("Path component length invalid"));
    }

    let allowed = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    if !input.chars().all(allowed) {
        return Err(anyhow::anyhow!("Invalid characters in path"));
    }

    Ok(())
}
```

### Connection Lifecycle Management
Always clean up connections properly.

```rust
// ✅ Proper connection cleanup
impl ConnectionManager {
    pub async fn disconnect(&self) -> Result<()> {
        let mut conn = self.connection.lock().await;
        if let Some(mut connection) = conn.take() {
            // Explicitly disconnect and clean up
            connection.disconnect().await?;
        }
        Ok(())
    }

    pub async fn connect(&self, params: ConnectParams) -> Result<ConnectionInfo> {
        // Always clean up existing connection first
        self.disconnect().await?;

        // Create new connection
        let connection = SSHConnection::new(params.host, params.port, params.username, config);
        // ... rest of connection logic
    }
}
```

### Error Mapping for User Experience
Convert technical errors to actionable user messages.

```rust
// ✅ User-friendly error mapping
pub fn map_ssh_error(error: &SSHError) -> ConnectionError {
    match error {
        SSHError::NetworkError(msg) => ConnectionError {
            category: "Network".to_string(),
            message: "Cannot reach cluster host".to_string(),
            retryable: true,
            suggestions: vec!["Check network connection", "Verify hostname"]
        },
        SSHError::AuthenticationError(_) => ConnectionError {
            category: "Authentication".to_string(),
            message: "Authentication failed".to_string(),
            retryable: false,
            suggestions: vec!["Check username and password"]
        },
        // ... other mappings
    }
}
```

### Mock vs Real Implementation Switching
Environment-based service selection.

```rust
// ✅ Clean mock/real switching
fn use_mock_mode() -> bool {
    if let Ok(val) = env::var("USE_MOCK_SSH") {
        return val.to_lowercase() == "true" || val == "1";
    }

    #[cfg(debug_assertions)]
    { true }  // Default to mock in debug

    #[cfg(not(debug_assertions))]
    { false } // Default to real in release
}

#[tauri::command]
pub async fn connect_to_cluster(params: ConnectParams) -> ConnectResult {
    if use_mock_mode() {
        return connect_mock(params).await;
    }
    connect_real(params).await
}
```

### Async Operations with Blocking Libraries
Handle ssh2's blocking nature.

```rust
// ✅ Proper async/blocking integration
impl ConnectionManager {
    pub async fn connect(&self, password: &SecurePassword) -> Result<ConnectionInfo> {
        password.with_password(|pwd| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { connection.connect(pwd).await })
        })
    }
}
```

### Retry Logic Implementation
Implement exponential backoff for retryable operations.

```rust
// ✅ Complete retry implementation
pub async fn connect_with_retry(params: ConnectParams) -> Result<ConnectionInfo> {
    let mut attempts = 0;
    let max_attempts = 3;
    let mut delay = Duration::from_millis(1000);

    loop {
        match connection_manager.connect(params.clone()).await {
            Ok(info) => return Ok(info),
            Err(e) => {
                let conn_error = map_ssh_error(&e);

                attempts += 1;
                if !conn_error.retryable || attempts >= max_attempts {
                    return Err(e);
                }

                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
}
```

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

## Related Documentation

For architectural principles, clean architecture patterns, and development best practices, see [`docs/developer-guidelines.md`](developer-guidelines.md).

For testing strategies and infrastructure setup, see [`docs/testing-spec.md`](testing-spec.md).
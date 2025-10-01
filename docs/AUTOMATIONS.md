# NAMDRunner Automation Architecture

**Centralized automation workflows for job management and cluster operations**

This document defines NAMDRunner's automation system architecture, current automation chains, and design principles for future extensibility including an in-app automation builder.

> **Implementation Status**: ✅ **IMPLEMENTED** - Phase 6.1 automation architecture completed
> **Current State**: Simple automation functions with progress callbacks and Tauri event emission

## Table of Contents
- [Overview](#overview)
- [Architecture Principles](#architecture-principles)
- [Core Automation Chains](#core-automation-chains)
- [Implementation Architecture](#implementation-architecture)
- [Event System](#event-system)
- [Verification Framework](#verification-framework)
- [Future Automation Builder](#future-automation-builder)

## Overview

NAMDRunner uses **automation functions** to orchestrate complex multi-step operations like job creation, submission, and management. Each automation function consists of discrete, verifiable steps that provide real-time progress feedback to users.

### Core Design Goals
- **Transparency**: Users understand exactly what happens when
- **Reliability**: Each step is verified before proceeding
- **Modularity**: Steps can be composed into different workflows
- **Simplicity**: Direct async functions following CONTRIBUTING.md philosophy
- **Future-Proofing**: Architecture supports eventual visual automation builder

## Architecture Principles

### 1. Simple Async Functions
Following CONTRIBUTING.md principles of direct functions and simple patterns:

```rust
pub async fn execute_job_creation_with_progress(
    _app_handle: AppHandle,
    params: CreateJobParams,
    progress_callback: impl Fn(&str),
) -> Result<(String, JobInfo)> {
    progress_callback("Starting job creation...");

    // Validate inputs
    progress_callback("Validating connection...");

    // Generate job paths
    progress_callback("Generating job paths...");

    // Create directories
    progress_callback("Creating project directories...");

    // Upload files
    progress_callback("Uploading input files...");

    // Save to database
    progress_callback("Saving job to database...");

    progress_callback("Job creation completed successfully");

    Ok((job_id, job_info))
}
```

### 2. Event-Driven Progress Tracking
Real-time UI updates via Tauri event system:

```rust
// Backend: Direct event emission in command handlers
let handle_clone = app_handle.clone();
match automations::execute_job_creation_with_progress(
    app_handle,
    params,
    move |msg| {
        let _ = handle_clone.emit("job-creation-progress", msg);
    }
).await {
    Ok((job_id, job_info)) => { /* success */ }
    Err(e) => { /* error handling */ }
}
```

```typescript
// Frontend: Event listening in stores
const unlisten = await listen('job-creation-progress', (event) => {
    const message = event.payload as string;
    update(state => ({
        ...state,
        creationProgress: { message, isActive: true }
    }));
});
```

### 3. Integrated File Operations
File uploads are integrated directly into job creation for atomic operations:

```rust
// Upload input files if any are provided
if !params.input_files.is_empty() {
    let input_files_dir = format!("{}/input_files", project_dir);

    for (i, file) in params.input_files.iter().enumerate() {
        progress_callback(&format!(
            "Uploading file {} of {}: {}",
            i + 1,
            params.input_files.len(),
            remote_name
        ));

        // Upload file using ConnectionManager
        connection_manager.upload_file(&file.local_path, &remote_path).await?;
    }
}
```

### 4. Security First
All automation steps follow NAMDRunner security principles:
- Input sanitization before any operations
- Path safety validation to prevent traversal attacks
- No credential logging or persistence
- Secure memory handling for sensitive data

## Core Automation Chains

### 1. Job Creation Automation Chain

**Trigger**: User clicks "Create Job" button in UI after completing job wizard
**Purpose**: Set up job in project directory with all necessary files and metadata
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/automations/job_creation.rs`

#### Implementation:

The `execute_job_creation_with_progress` function handles the complete job creation workflow:

1. **Validate Inputs**
   - Sanitize job name using `crate::validation::input::sanitize_job_id()`
   - Validate SSH connection is active
   - Get cluster username

2. **Create Project Directory Structure**
   - Create: `/projects/$USER/namdrunner_jobs/{job_id}/`
   - Create subdirectories using `paths::job_subdirectories()`
   - Only creates project directories (scratch directories created during submission)

3. **Upload Input Files** (Integrated)
   - Upload each user-provided file to `input_files/` subdirectory
   - Progress tracking per file with validation
   - Uses existing ConnectionManager with retry logic

4. **Create Job Metadata**
   - Generate JobInfo struct with complete job specification
   - Set only project directory (scratch_dir remains None until submission)

5. **Save to Local Database**
   - Store job record in local SQLite database using existing `with_database` utility
   - Set initial status as "CREATED" (not submitted)

**Result**: Job appears in UI with "CREATED" status, ready for submission

---

### 2. Job Submission Automation Chain

**Trigger**: User clicks "Submit Job" button for a created job
**Purpose**: Copy project to scratch directory and submit to SLURM scheduler
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/automations/job_submission.rs`

#### Implementation:

The `execute_job_submission_with_progress` function handles the complete job submission workflow:

1. **Load Job Information**:
   - Retrieve job from database by ID
   - Validate job status is CREATED or FAILED (eligible for submission)

2. **Create Scratch Directory Structure**:
   - Create main scratch directory: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/`
   - Create subdirectories: `input/`, `output/`, `logs/`

3. **Copy Files to Scratch**:
   - Copy all input files from `{project_dir}/input_files/` to `{scratch_dir}/input/`
   - Copy SLURM script (`job.sbatch`) and NAMD config (`config.namd`)
   - Uses secure file copy commands with proper path escaping

4. **Submit to SLURM**:
   - Execute `sbatch job.sbatch` in scratch directory
   - Parse SLURM job ID from output
   - Handle submission errors with proper timeout

5. **Update Job Status**:
   - Set status to PENDING, add SLURM job ID and submission timestamp
   - Update both database and remote JSON metadata file

**Result**: Job status changes from CREATED to PENDING with SLURM job ID assigned

---

### 3. Status Synchronization Automation Chain

**Trigger**: User clicks "Sync Status" button or "Sync All Jobs" button
**Purpose**: Query SLURM for current job status and update local database
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/commands/jobs.rs` and `src-tauri/src/slurm/status.rs`

#### Implementation:

The status synchronization automation is implemented through two main commands:

1. **Single Job Status Sync** (`sync_job_status`)
   - **Validate Inputs**: Sanitize job ID using `crate::validation::input::sanitize_job_id()`
   - **Load Job from Database**: Retrieve job information from local SQLite database
   - **Check for SLURM Job ID**: Only sync jobs that have been submitted to SLURM
   - **Get Cluster Username**: Retrieve username from environment for SLURM queries
   - **Query SLURM Status**: Use `SlurmStatusSync::sync_job_status()` to query squeue/sacct
   - **Update Job Status**: If status changed, update job with timestamp using `helpers::update_job_status_with_timestamps()`
   - **Return Updated Job**: Provide updated job information to UI

2. **All Jobs Status Sync** (`sync_all_jobs`)
   - **Load All Jobs**: Retrieve all jobs from local database
   - **Filter Submitted Jobs**: Only sync jobs that have SLURM job IDs
   - **Batch Status Updates**: Iterate through jobs and sync each one
   - **Aggregate Results**: Collect success/failure statistics for batch operation

**Result**: Jobs display current SLURM status (PENDING → RUNNING → COMPLETED/FAILED/CANCELLED) with accurate timestamps

---

### 4. Job Completion and Results Retrieval Automation Chain

**Trigger**: Manual user request to preserve completed job results (NOT automatic)
**Purpose**: Copy important output files from temporary scratch directory to permanent project directory for long-term storage
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/automations/job_completion.rs`

#### Implementation:

The `execute_job_completion_with_progress` function preserves critical simulation results:

1. **Validate Job Status**:
   - Verify job is in COMPLETED state
   - Ensure both project and scratch directories exist

2. **Create Results Directory**:
   - Create `{project_dir}/results/` directory for preserved files

3. **Scan and Copy Critical Files**:
   - **NAMD trajectory files**: `*.dcd`, `*.coor`, `*.vel`, `*.xsc`
   - **NAMD output logs**: `namd_output.log`, `*.energy`, `*.restart.*`
   - **SLURM output logs**: `*.out`, `*.err`
   - **Checkpoint files**: `*.restart`, `*.cpt`, `*.chk`
   - **Final coordinates**: `final.coor`, `final.vel`, `final.xsc`

4. **Optimization**:
   - Batches find operations per category to reduce SSH round-trips
   - Uses secure file copy commands with proper path escaping
   - Tracks file count and total size for user feedback

5. **Create Completion Summary**:
   - Generates summary with job details, files preserved, and timestamps
   - Stores summary as `completion_summary.txt` in results directory

**IMPORTANT**: This is **NOT** automatic on job completion. Results preservation is triggered manually by user request. Scratch directories remain until explicit cleanup or 90-day auto-purge.

**Result**: Critical simulation results are preserved in `/projects/$USER/namdrunner_jobs/{job_id}/results/` with completion summary

---

### 5. Job Cleanup Automation Chain

**Trigger**: User clicks "Delete Job" button with "Delete Remote Files" option
**Purpose**: Safely remove job directories from cluster while preserving database record option
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/commands/jobs.rs`

#### Implementation:

The job deletion automation performs safe cleanup with multiple validation steps:

1. **Input Validation**: Sanitize job ID to prevent injection attacks
2. **Load Job Information**: Retrieve job details from local database
3. **Connection Verification**: Ensure active SSH connection to cluster
4. **Directory Safety Validation**:
   - Verify paths contain "namdrunner_jobs" to prevent accidental deletion
   - Block dangerous patterns (.., /, /etc, /usr)
   - Validate both project and scratch directory paths
5. **Safe Directory Removal**:
   - Delete project directory: `/projects/$USER/namdrunner_jobs/{job_id}/`
   - Delete scratch directory: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/`
   - Use ConnectionManager with retry logic for network resilience
6. **Database Cleanup**: Remove job record from local SQLite database

**Safety Features**:
- Multiple path validation layers prevent directory traversal attacks
- Connection verification prevents orphaned database records
- Atomic operation - either fully succeeds or cleanly fails
- Detailed error messages for troubleshooting

**Result**: Job completely removed from both cluster storage and local database, with full safety validation

## Implementation Architecture

### Module Structure

```
src-tauri/src/automations/
├── mod.rs                   # Module exports
├── job_creation.rs         # ✅ Job creation automation (implemented)
├── job_submission.rs       # ✅ Job submission automation (implemented)
├── job_completion.rs       # ✅ Job completion and results retrieval (implemented)
└── progress.rs             # ✅ Progress tracking utilities (implemented)

# Status synchronization implemented in commands/jobs.rs and slurm/status.rs
# Job cleanup implemented in commands/jobs.rs (delete_job function)
```

### Command Integration

Current implementation in `src-tauri/src/commands/jobs.rs`:

```rust
use crate::automations;

async fn create_job_real(app_handle: tauri::AppHandle, params: CreateJobParams) -> CreateJobResult {
    let handle_clone = app_handle.clone();
    match automations::execute_job_creation_with_progress(
        app_handle,
        params,
        move |msg| {
            let _ = handle_clone.emit("job-creation-progress", msg);
        }
    ).await {
        Ok((job_id, job_info)) => CreateJobResult {
            success: true,
            job_id: Some(job_id),
            error: None,
        },
        Err(e) => CreateJobResult {
            success: false,
            job_id: None,
            error: Some(e.to_string()),
        },
    }
}
```

## Event System

### Progress Events

The automation system emits real-time progress events that the UI subscribes to:

```typescript
// Frontend stores integration
interface JobProgress {
  message: string;
  isActive: boolean;
}

// Listen for progress events from automation system
const unlisten = await listen('job-creation-progress', (event) => {
    const message = event.payload as string;
    update(state => ({
        ...state,
        creationProgress: { message, isActive: true }
    }));
});
```

### Event Examples

```rust
// Emitted during job creation automation
progress_callback("Starting job creation...");
progress_callback("Validating connection...");
progress_callback("Generating job paths...");
progress_callback("Creating project directories...");
progress_callback("Uploading file 1 of 3: protein.pdb");
progress_callback("Uploading file 2 of 3: protein.psf");
progress_callback("Uploading file 3 of 3: parameters.prm");
progress_callback("Creating job metadata...");
progress_callback("Saving job to database...");
progress_callback("Job creation completed successfully");
```

## Verification Framework

### Current Verification

The automation system includes basic verification:

```rust
fn validate_upload_file(file: &InputFile) -> Result<()> {
    // Check local file exists
    let local_path = Path::new(&file.local_path);
    if !local_path.exists() {
        return Err(anyhow!("Local file does not exist: {}", file.local_path));
    }

    // Check file is readable
    if let Err(e) = fs::File::open(local_path) {
        return Err(anyhow!("Cannot read local file: {}", e));
    }

    // Basic file size check (limit to 1GB)
    let metadata = fs::metadata(local_path)?;
    if metadata.len() > 1_073_741_824 {
        return Err(anyhow!("File too large: {} bytes (max 1GB)", metadata.len()));
    }

    // Validate remote filename
    if let Some(remote_name) = &file.remote_name {
        if remote_name.contains('/') || remote_name.contains('\\') {
            return Err(anyhow!("Remote filename cannot contain path separators"));
        }
    }

    Ok(())
}
```

### Planned Verification Enhancements

- File transfer verification with size/checksum validation
- Directory creation verification
- JSON metadata write verification
- Database save verification

## Future Automation Builder

### Design Principles for Extensibility

The current simple function architecture can be enhanced for future automation builder:

#### 1. Function-Based Steps
The current approach of simple async functions with progress callbacks provides a clean foundation for future serializable steps.

#### 2. Visual Workflow Builder
Future UI features will include:
- **Drag-and-drop step composition**: Visual workflow designer
- **Template management**: Save and share automation workflows
- **Conditional logic**: Branching based on step results
- **Custom parameters**: User-defined variables and inputs
- **Validation preview**: Test workflows before execution

#### 3. Automation Templates
Future serializable format building on current function structure:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<SerializableStep>,
    pub variables: HashMap<String, String>,
}
```

## Current Implementation Status

### ✅ Implemented (Phase 6.2 Complete)
- **Job Creation Automation**: Complete workflow with file uploads and progress tracking
- **Job Submission Automation**: Scratch workspace setup, file copying, SLURM submission
- **Status Synchronization**: Real-time SLURM queue monitoring with database updates
- **Job Completion Automation**: Results preservation from scratch to project directories
- **Job Cleanup Automation**: Safe deletion with comprehensive path validation
- **Security Improvements**: All operations use centralized command builders with proper escaping
- **Performance Optimization**: Batched find operations, reduced SSH round-trips
- **Progress Tracking**: Real-time UI updates via Tauri events for all automation chains
- **Error Handling**: Consistent `Result<T>` patterns with proper error classification

### 📋 Future Features (Phase 8+)
- Visual automation builder UI
- Serializable automation templates
- Conditional workflow logic
- Community template sharing
- Custom automation step creation

## Security Considerations

All automation functions follow NAMDRunner security principles:

- **Input Sanitization**: All user inputs sanitized before use
- **Path Safety**: Prevent directory traversal attacks
- **Credential Security**: No logging or persistence of sensitive data
- **Permission Validation**: Verify user has appropriate cluster access
- **Audit Trail**: Log automation actions for security review

## Testing Strategy

Automation testing follows NAMDRunner's 3-tier architecture:

- **Unit Tests**: Individual function logic and verification functions
- **Integration Tests**: Complete automation functions with mock dependencies
- **E2E Tests**: Full workflows against mock cluster environment

## Related Documentation

- [`docs/CONTRIBUTING.md`](CONTRIBUTING.md) - Development principles and security requirements
- [`docs/API.md`](API.md) - IPC interfaces for automation commands
- [`docs/SSH.md`](SSH.md) - File operations and security patterns
- [`docs/reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md) - Cluster-specific requirements

---

*This document describes the current automation architecture implemented in NAMDRunner Phase 6.1.*
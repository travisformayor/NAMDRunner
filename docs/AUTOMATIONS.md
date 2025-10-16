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

4. **Generate SLURM Batch Script**
   - Use `SlurmScriptGenerator::generate_namd_script()` to create job.sbatch
   - Upload script to `scripts/job.sbatch`
   - Script configures SLURM resources, modules, and NAMD execution

5. **Generate NAMD Configuration**
   - Use `SlurmScriptGenerator::generate_namd_config()` to create config.namd
   - Upload config to `scripts/config.namd`
   - Config includes simulation parameters, input files, and output settings

6. **Create Job Metadata**
   - Generate JobInfo struct with complete job specification
   - Set only project directory (scratch_dir remains None until submission)

7. **Save to Local Database**
   - Store job record in local SQLite database using existing `with_database` utility
   - Set initial status as "CREATED" (not submitted)

8. **Upload Job Metadata to Server**
   - Create `job_info.json` in project directory with complete job specification
   - Enables job discovery even for jobs that haven't been submitted yet
   - Serves as single source of truth for job parameters on cluster

**Result**: Job appears in UI with "CREATED" status, ready for submission. Job folder on cluster contains complete metadata for discovery and sharing.

---

### 2. Job Submission Automation Chain

**Trigger**: User clicks "Submit Job" button for a created job
**Purpose**: Mirror job directory to scratch and submit to SLURM scheduler
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/automations/job_submission.rs`

#### Implementation:

The `execute_job_submission_with_progress` function handles the complete job submission workflow:

1. **Load Job Information**:
   - Retrieve job from database by ID
   - Validate job status is CREATED or FAILED (eligible for submission)

2. **Mirror Job Directory to Scratch** (using rsync):
   - NAMDRunner app -> SLURM execution (crosses project/scratch data boundary)
   - Source: `/projects/$USER/namdrunner_jobs/{job_id}/` (app's work area)
   - Destination: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/` (SLURM's work area)
   - Uses single `rsync -az` command to sync entire directory structure
   - Preserves complete directory layout with all subdirectories:
     - `input_files/` - All NAMD input files
     - `scripts/` - SLURM batch script and NAMD configuration
     - `outputs/` - Empty initially, populated during job execution
     - `job_info.json` - Job metadata
   - **Why rsync**: Single cluster-side operation is much faster than per-file SFTP transfers, supports delta sync for resubmissions

3. **Submit to SLURM**:
   - Execute `sbatch` using the mirrored script in scratch location: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/scripts/job.sbatch`
   - SLURM job now owns scratch directory - app does not touch it during execution
   - Parse SLURM job ID from output
   - Handle submission errors with proper timeout

4. **Update Job Status**:
   - Set status to PENDING, add SLURM job ID, submission timestamp, and scratch directory path
   - Update local database
   - Update `job_info.json` in **project directory** with submission details

**Directory Structure After Submission**:
```
/projects/$USER/namdrunner_jobs/job_123/          /scratch/alpine/$USER/namdrunner_jobs/job_123/
├── input_files/                                   ├── input_files/
│   ├── structure.pdb                              │   ├── structure.pdb
│   ├── structure.psf                              │   ├── structure.psf
│   └── parameters.prm                             │   └── parameters.prm
├── scripts/                                       ├── scripts/
│   ├── job.sbatch                                 │   ├── job.sbatch
│   └── config.namd                                │   └── config.namd
├── outputs/ (empty)                               ├── outputs/ (SLURM writes here during execution)
└── job_info.json                                  └── job_info.json

        ↑ APP AREA                                         ↑ SLURM AREA
    (App reads/writes here)                        (App hands off, SLURM owns during execution)
```

**Result**: Job status changes from CREATED to PENDING with SLURM job ID assigned. Complete job directory mirrored to scratch for execution. App no longer interacts with scratch until job completes (see Job Completion chain).

---

### 3. Status Synchronization Automation Chain

**Trigger**: User clicks "Sync Status" button or automatic periodic sync (if implemented)
**Purpose**: Query SLURM for current job status, update local database, trigger job discovery if needed, and return complete job list
**Status**: 🔨 **REFACTORING IN PROGRESS** (Phase 6.6) - Current implementation has architecture violations

**Planned Architecture** (Backend-Owned Workflow):

The `sync_all_jobs()` function will handle complete synchronization workflow, including automatic job discovery:

1. **Validate Connection**:
   - Verify SSH connection is active
   - Get cluster username for SLURM queries

2. **Load Active Jobs**:
   - Retrieve all jobs from database
   - Filter to only Pending or Running jobs (others don't need syncing)

3. **Query SLURM for Active Jobs** (Batch Operation):
   - Use `SlurmStatusSync::sync_all_jobs()` with job SLURM IDs
   - Query `squeue` for active jobs (PENDING/RUNNING) in single batch command
   - Query `sacct` for completed jobs as fallback
   - Parse SLURM status output to JobStatus enum
   - Update job status, timestamps in database
   - **NO server metadata updates during execution** (Metadata-at-Boundaries principle)

4. **Automatic Job Discovery** (If Database Empty):
   - Check if database has zero jobs (first connection after DB reset)
   - If empty, trigger automatic discovery from cluster:
     - Scan `/projects/$USER/namdrunner_jobs/` for job directories
     - Download `job_info.json` from each directory
     - Parse and import to database
     - For jobs with status=PENDING/RUNNING, they'll sync on next cycle
   - If discovery fails, log error but continue with sync

5. **Trigger Job Completion if Terminal State**:
   - If status changed to terminal state (COMPLETED, FAILED, CANCELLED, TIMEOUT, OUT_OF_MEMORY):
     - Set `completed_at` timestamp
     - **Trigger Job Completion Automation Chain** (see Section 4 below)
     - Completion chain handles: rsync scratch→project FIRST, then cache logs, then update metadata
   - If status is still PENDING/RUNNING:
     - Save to database only (no metadata update)
     - Wait for next sync

6. **Return Complete Job List**:
   - Load all jobs from database (includes discovered + synced jobs)
   - Return complete list to frontend
   - Frontend simply caches the result (no orchestration logic)

**Architecture Principle (Metadata-at-Boundaries)**:
Server metadata (`job_info.json`) is only updated at lifecycle boundaries, not during execution:
- **Job Creation**: Update project metadata (job created)
- **Job Submission**: Update project metadata (slurm_job_id, submitted_at added)
- **During Execution**: Database ONLY (metadata shows submission state, becomes stale)
- **Job Completion**: Rsync scratch→project FIRST, then update metadata (final status)

This prevents rsync conflicts and keeps metadata updates simple and predictable.

**Frontend Integration**:
```typescript
// Frontend store - pure caching (no business logic)
const syncResult = await invoke('sync_jobs');
// syncResult.jobs contains complete list (discovery + sync happened automatically)
jobsStore.set(syncResult.jobs);
```

**Result**: Jobs display current SLURM status with accurate timestamps. Terminal state jobs automatically trigger completion chain. Database-empty scenario handled automatically. Frontend makes single backend call with no orchestration logic.

**Known Issues (To Be Fixed in Phase 6.6)**:
- ❌ Current: Frontend orchestrates discovery (3 backend calls)
- ❌ Current: Server metadata updated during execution (violates Metadata-at-Boundaries)
- ❌ Current: Rsync scratch→project missing on completion
- ❌ Current: Logs fetched from scratch instead of project
- ✅ Planned: Backend handles all workflows, frontend is pure caching

---

### 4. Job Completion and Results Retrieval Automation Chain

**Trigger**: Automatic when job status transitions to terminal state (COMPLETED, FAILED, CANCELLED, TIMEOUT, OUT_OF_MEMORY) during status synchronization
**Purpose**: Mirror scratch directory back to project directory to preserve all job results, then cache SLURM logs for offline viewing
**Status**: 🔨 **REFACTORING IN PROGRESS** (Phase 6.6) - Rsync missing, logs fetched from wrong location (scratch instead of project)

**Architecture Principle**: Scratch directory is for SLURM execution only. NAMDRunner app only interacts with project directory. Rsync keeps them in sync at job submission (project→scratch) and completion (scratch→project).

**Planned Implementation** (Correct Order):

When job status changes to terminal state during `sync_job_status()`:

1. **Automatic Rsync Scratch→Project** (FIRST AND CRITICAL):
   - Triggered immediately when terminal state detected in status sync
   - **DATA BOUNDARY CROSSED**: SLURM execution → NAMDRunner app
   - Source: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/` (append trailing slash)
   - Destination: `/projects/$USER/namdrunner_jobs/{job_id}/` (no trailing slash)
   - Uses single `rsync -az` command to sync entire directory structure
   - **Delta sync**: Only copies files that changed or were added
   - Preserves all job results:
     - `outputs/` - All NAMD trajectory files (.dcd), final coordinates, restart files
     - `{job_name}_{slurm_job_id}.out` - SLURM stdout log file
     - `{job_name}_{slurm_job_id}.err` - SLURM stderr log file
     - `namd_output.log` - NAMD execution log
     - Any other files written during execution
   - **Why rsync**: Efficient delta transfer, handles all files in single operation
   - **Error handling**: Log error if rsync fails but continue (don't block status sync)
   - **CRITICAL**: This MUST happen before any log fetching or file operations

2. **Cache SLURM Logs from Project Directory** (AFTER RSYNC):
   - After rsync completes, logs are now in project directory
   - Construct paths: `{project_dir}/{job_name}_{slurm_job_id}.out` and `.err`
   - Read log contents from **project directory** (not scratch!)
   - Store in database `slurm_stdout` and `slurm_stderr` columns for offline viewing
   - Implements "fetch once, cache forever" pattern
   - Manual "Refetch Logs" button in UI allows re-fetching if needed

3. **Update Job Metadata in Project Directory** (FINAL STEP):
   - Update job_info.json with final status and completion timestamp
   - This is the only time metadata is updated during job lifecycle after submission
   - Now metadata matches reality again (was stale during execution per Metadata-at-Boundaries principle)

**Data Flow**:
```
Terminal State Detected
         ↓
   1. Rsync scratch→project (entire job directory)
         ↓
   2. Fetch logs from project directory
         ↓
   3. Cache logs in database
         ↓
   4. Update project metadata with final status
         ↓
   Complete - App now interacts with project directory only
```

**Result**: All job data automatically syncs from scratch to project when job finishes. SLURM logs cached in database for offline viewing. All file operations (downloads, listings) use project directory. Users can download individual output files or bulk download all outputs as a zip. No manual sync button needed.

**Known Issues (To Be Fixed in Phase 6.6)**:
- ❌ Current: Rsync scratch→project never happens
- ❌ Current: Logs fetched from scratch directory (wrong location)
- ❌ Current: Discovered jobs fetch logs from scratch (before rsync)
- ✅ Planned: Rsync FIRST, then logs from project, then metadata update

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

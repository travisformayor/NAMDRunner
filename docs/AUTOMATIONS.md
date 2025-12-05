# NAMDRunner Automation Architecture

**Centralized automation workflows for job management and cluster operations**

This document defines NAMDRunner's automation system architecture, current automation chains, and design principles for future extensibility including an in-app automation builder.

> **Implementation Status**: ✅ **IMPLEMENTED** - Phase 7.1 with template system integration
> **Current State**: Template-based job creation with automatic file uploads, connection failure detection, and simplified automation chains

## Table of Contents
- [Overview](#overview)
- [Architecture Principles](#architecture-principles)
- [Core Automation Chains](#core-automation-chains)
- [Implementation Architecture](#implementation-architecture)
- [Event System](#event-system)
- [Verification Framework](#verification-framework)
- [Connection Failure Detection](#connection-failure-detection)
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
   - Create subdirectories: `input_files/`, `outputs/`
   - Only creates project directories (scratch directories created during submission)

3. **Load and Process Template**
   - Load template from database using `template_id` from CreateJobParams
   - Extract FileUpload variables from template definition
   - Process template_values to identify files that need uploading

4. **Upload Input Files** (Template-Driven)
   - Upload files for FileUpload variables to `input_files/` subdirectory
   - Extract filename from local path and update template_values with just filename
   - Progress tracking per file with validation
   - Uses ConnectionManager with retry logic
   - Store uploaded filenames in `input_files` field for job tracking

5. **Render NAMD Configuration from Template**
   - Use `crate::templates::render_template()` with template and values
   - FileUpload variables: filenames automatically get "input_files/" prepended
   - Number variables: converted to numeric strings
   - Boolean variables: converted to "yes"/"no" for NAMD
   - Text variables: used as-is
   - Validates all variables are replaced (no unreplaced {{variables}})

6. **Generate SLURM Batch Script**
   - Use `SlurmScriptGenerator::generate_namd_script(&job_info, &scratch_dir)` to create job.sbatch
   - Scratch directory path passed as parameter (not stored in JobInfo until submission)
   - Upload script to job root: `job.sbatch`
   - Script configures SLURM resources, modules, and NAMD execution

7. **Upload NAMD Configuration**
   - Upload rendered config to job root: `config.namd`
   - Config includes simulation parameters, input files, and output settings

8. **Create Job Metadata**
   - Generate JobInfo struct using factory function `create_job_info()`
   - Store template_id and template_values for job reconstruction
   - JobInfo.project_dir set to project directory path
   - JobInfo.scratch_dir remains None until job submission
   - Initial status set to "CREATED" (not submitted)

9. **Save to Local Database**
   - Store job record in local SQLite database using `with_database` utility
   - JobInfo includes template_id and template_values for template-based job tracking

10. **Upload Job Metadata to Server**
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
   - Template validation completed before submission (happens during job creation)

2. **Mirror Job Directory to Scratch** (using rsync):
   - NAMDRunner app -> SLURM execution (crosses project/scratch data boundary)
   - Source: `/projects/$USER/namdrunner_jobs/{job_id}/` (app's work area)
   - Destination: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/` (SLURM's work area)
   - Uses single `rsync -az` command to sync entire directory structure
   - Preserves complete directory layout with all subdirectories:
     - `input_files/` - All NAMD input files
     - `config.namd` - NAMD configuration (in job root)
     - `job.sbatch` - SLURM batch script (in job root)
     - `outputs/` - Empty initially, populated during job execution
     - `job_info.json` - Job metadata
   - **Why rsync**: Single cluster-side operation is much faster than per-file SFTP transfers, supports delta sync for resubmissions

3. **Submit to SLURM**:
   - Execute `sbatch` using the mirrored script in scratch location: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/job.sbatch`
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
├── job_info.json                                  ├── job_info.json
├── config.namd                                    ├── config.namd
├── job.sbatch                                     ├── job.sbatch
├── input_files/                                   ├── input_files/
│   ├── structure.pdb                              │   ├── structure.pdb
│   ├── structure.psf                              │   ├── structure.psf
│   └── parameters.prm                             │   └── parameters.prm
└── outputs/ (empty)                               └── outputs/ (SLURM writes here during execution)

        ↑ APP AREA                                         ↑ SLURM AREA
    (App reads/writes here)                        (App hands off, SLURM owns during execution)
```

**Result**: Job status changes from CREATED to PENDING with SLURM job ID assigned. Complete job directory mirrored to scratch for execution. App no longer interacts with scratch until job completes (see Job Completion chain).

---

### 3. Status Synchronization Automation Chain

**Trigger**: User clicks "Sync Status" button or automatic periodic sync (if implemented)
**Purpose**: Query SLURM for current job status, update local database, trigger job discovery if needed, and return complete job list
**Status**: ✅ **IMPLEMENTED** (Phase 6.6) - Complete backend-owned workflow with automatic job discovery and connection failure detection

**Implementation** (Backend-Owned Workflow):

The `sync_all_jobs()` function handles complete synchronization workflow, including automatic job discovery:

1. **Validate Connection**:
   - Verify SSH connection is active
   - Get cluster username for SLURM queries
   - Connection failures automatically handled by frontend (transitions to Expired state)

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
     - Returns `DiscoveryReport` struct containing:
       - `imported_jobs: Vec<JobSummary>` - Successfully imported jobs with key details
       - `failed_imports: Vec<FailedImport>` - Jobs that failed with specific error reasons
     - Logs detailed results: which jobs imported successfully, which failed and why
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
   - Return `SyncJobsResult` with complete job list to frontend
   - Frontend simply caches the result (no orchestration logic)

**Architecture Principle: Metadata-at-Boundaries**
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

---

### 4. Job Completion and Results Retrieval Automation Chain

**Trigger**: Automatic when job status transitions to terminal state (COMPLETED, FAILED, CANCELLED, TIMEOUT, OUT_OF_MEMORY) during status synchronization
**Purpose**: Mirror scratch directory back to project directory to preserve all job results, then cache SLURM logs for offline viewing
**Status**: ✅ **IMPLEMENTED** (Phase 6.6) - Automatic rsync on completion with correct log fetching from project directory

**Architecture Principle**: Scratch directory is for SLURM execution only. NAMDRunner app only interacts with project directory. Rsync keeps them in sync at job submission (project→scratch) and completion (scratch→project).

**Implementation** (Correct Order):

When job status changes to terminal state during `sync_all_jobs()`, the `execute_job_completion_internal()` function automatically triggers:

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

---

### 5. Job Deletion Automation Chain

**Trigger**: User clicks "Delete Job" button with optional "Delete Remote Files" checkbox
**Purpose**: Cancel active SLURM jobs, safely remove job directories from cluster, and clean up local database
**Status**: ✅ **IMPLEMENTED** in `src-tauri/src/automations/job_deletion.rs`

#### Implementation:

The `execute_job_deletion` function handles complete job cleanup with safety validation and progress tracking:

1. **Load Job Information**
   - Retrieve job from local database by job ID
   - Validate job exists
   - Extract status, SLURM job ID, project_dir, scratch_dir for cleanup operations

2. **Cancel SLURM Job** (if active)
   - Check if job status is Pending or Running
   - If active and has SLURM job ID:
     - Verify SSH connection is active
     - Get cluster username for SLURM commands
     - Cancel job using `scancel` via SlurmStatusSync
     - Prevents orphaned cluster processes consuming resources
   - If job already completed or has no SLURM ID, skip cancellation

3. **Delete Remote Directories** (if requested)
   - Only executes if user checked "Delete Remote Files" option
   - Verify SSH connection is active before any operations
   - Collect directories to delete:
     - Project directory: `/projects/$USER/namdrunner_jobs/{job_id}/` (if present)
     - Scratch directory: `/scratch/alpine/$USER/namdrunner_jobs/{job_id}/` (if present)
   - **Directory Safety Validation** (for each directory):
     - Verify path contains `JOB_BASE_DIRECTORY` constant ("namdrunner_jobs")
     - Block dangerous patterns: `..` (traversal), `/` (root), `/etc`, `/usr`
     - Return error and abort if any validation fails
   - Delete each validated directory using ConnectionManager
   - Uses retry logic for network resilience

4. **Remove from Database**
   - Delete job record from local SQLite database
   - Verify deletion succeeded (returns error if job not found)
   - Atomic operation - database transaction ensures consistency

**Safety Features**:
- **SLURM job cancellation**: Prevents orphaned cluster processes
- **Multi-layer path validation**: Prevents directory traversal attacks and accidental system directory deletion
- **Connection verification**: Ensures cluster access before remote operations to prevent partial deletions
- **Atomic database operation**: Either fully succeeds or cleanly fails
- **Progress reporting**: Real-time status updates at each step for user transparency
- **Opt-in remote deletion**: User must explicitly request remote file deletion

**Result**: Active SLURM job cancelled (if running), job directories removed from cluster storage (if requested), and job record deleted from local database. Operation is safe, transparent, and atomic.

## Implementation Architecture

### Module Structure

```
src-tauri/src/automations/
├── mod.rs                   # Module exports
├── job_creation.rs         # ✅ Template-based job creation with file uploads
├── job_submission.rs       # ✅ Simplified submission (validation in job_creation)
├── job_sync.rs             # ✅ Status sync, job discovery, connection detection
├── job_completion.rs       # ✅ Automatic completion on terminal state
├── job_deletion.rs         # ✅ Job deletion with SLURM cancellation
└── common.rs               # Shared helpers for automations

src-tauri/src/templates/
├── mod.rs                   # Template system module exports
├── types.rs                # Template, VariableDefinition, VariableType
├── renderer.rs             # Template rendering with variable substitution
└── validation.rs           # Template value validation (all variables required)
```

### Command Integration

Current implementation in `src-tauri/src/commands/jobs.rs`:

```rust
use crate::automations;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(app_handle: tauri::AppHandle, params: CreateJobParams) -> ApiResult<JobInfo> {
    // Validate inputs
    let clean_job_name = input::sanitize_job_id(&params.job_name)?;
    let validated_params = CreateJobParams { job_name: clean_job_name, ..params };

    // Call automation with progress tracking
    let handle_clone = app_handle.clone();
    match automations::execute_job_creation_with_progress(
        app_handle,
        validated_params,
        move |msg| {
            let _ = handle_clone.emit("job-creation-progress", msg);
        }
    ).await {
        Ok((_job_id, job_info)) => ApiResult::success(job_info),
        Err(e) => ApiResult::error(e.to_string()),
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

## Connection Failure Detection

NAMDRunner implements automatic connection failure detection without additional network calls, ensuring users are promptly notified when their SSH session expires.

### Implementation Approach

**Zero Extra Network Calls**: Connection failures are detected by pattern-matching error messages from existing job operations (sync, create, submit, delete). No dedicated "connection health check" commands are needed.

**Automatic State Transition**: When any automation function encounters a connection-related error (timeout, connection reset, authentication failure), the frontend automatically transitions the connection state to `Expired`.

**Error Patterns Detected**:
- SSH connection timeouts
- Connection reset errors
- Authentication failures
- Network unreachable errors
- Session termination errors

**User Experience**:
- User sees "Connection Expired" status in UI
- All action buttons disabled until reconnection
- User can click "Connect" to re-establish session
- No data loss - local database maintains offline cache

**Implementation Location**:
- Frontend: `src/lib/stores/jobs.ts` - Pattern matches errors in job operation results
- Backend: Automation functions return standard error messages that frontend can parse

## Verification Framework

### Current Verification

The automation system includes verification at multiple stages:

**Template Validation** (`src-tauri/src/templates/validation.rs`):
- All variables required: Every variable defined in template must have a value provided
- Type checking: Ensures template values match variable definitions
- Range validation: Number variables respect min/max constraints
- File extensions: FileUpload variables validate against allowed extensions

**Input Sanitization** (`src-tauri/src/validation/input.rs`):
- Job name sanitization: Removes unsafe characters from user input
- Path safety validation: Prevents directory traversal attacks
- Command injection prevention: Sanitizes all shell command inputs

**Connection Validation**:
- SSH connection state verification before operations
- Username retrieval and validation
- Connection failure detection (see Connection Failure Detection section)

**Directory Safety** (`src-tauri/src/validation/paths.rs`):
- Path validation: Ensures paths contain "namdrunner_jobs"
- Dangerous pattern blocking: Prevents access to system directories
- Remote path construction using validated functions

### Verification During Automation

**Job Creation**:
1. Template loaded and validated before rendering
2. File upload variables validated against template constraints
3. Template rendering validates all variables are replaced
4. Directory creation verified before file uploads
5. File uploads validated (size, path safety)
6. Database save verification

**Job Submission**:
1. Job status validated (must be CREATED or FAILED)
2. Connection verified before operations
3. Rsync command validated and sanitized
4. SLURM job ID parsed and validated from sbatch output
5. Database update verification

**Job Sync**:
1. Connection verified before batch query
2. SLURM status parsed and validated
3. Terminal state detection triggers completion chain
4. Database updates verified

**Job Completion**:
1. Terminal state validated (COMPLETED, FAILED, CANCELLED)
2. Project and scratch directories verified before rsync
3. Rsync source/destination paths validated
4. Log file fetch handles missing files gracefully
5. Output file metadata parsed and validated

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

### ✅ Implemented (Phase 7 Complete)
- **Job Creation Automation**: Template-based workflow with dynamic file uploads, input_files tracking, and progress reporting
- **Template System Integration**: Template loading, rendering, and validation integrated into job creation
- **Job Submission Automation**: Simplified workflow with pre-validated templates
- **Status Synchronization**: Real-time SLURM queue monitoring with automatic job discovery and detailed DiscoveryReport
- **Job Completion Automation**: Automatic rsync of results from scratch to project directories
- **Job Deletion Automation**: SLURM cancellation, safe directory removal, and database cleanup with progress tracking
- **Connection Failure Detection**: Automatic state transition to Expired on connection errors
- **Security Improvements**: Input sanitization, path validation, command injection prevention
- **Performance Optimization**: Batched SLURM queries, reduced SSH round-trips
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

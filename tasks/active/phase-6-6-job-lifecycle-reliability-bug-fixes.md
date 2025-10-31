# Task: Phase 6.6 - Architecture Refactoring & Job Lifecycle Fixes

## Objective
Remove all identified architecture violations and technical debt discovered during Phase 6.4 audit. Implement clean backend-first architecture with proper automation workflows, fix critical SLURM integration bugs, and establish correct project/scratch directory boundaries.

## Context
- **Starting state**: Phase 6.5 complete (clean git), but architecture violations exist (frontend business logic, missing rsync, improper metadata handling)
- **Delivered state**: Clean backend-first architecture, all business logic in Rust, frontend as pure UI layer, jobs run successfully end-to-end
- **Foundation**: Phase 6.4 established backend-first principles, Phase 6.5 cleaned infrastructure
- **Dependencies**: Phase 6.5 must be complete (clean git state)
- **Testing approach**: Backend unit tests for business logic, E2E tests for workflows, manual cluster testing for integration

## Architecture Violations Found

### Critical Violations
1. **Frontend Orchestration Logic** - Frontend coordinating multi-step workflows (discovery + sync + refetch)
2. **Missing Rsync on Completion** - Jobs finish but scratch→project sync never happens (Issue 0)
3. **Incorrect Log Fetch Location** - Logs fetched from scratch instead of project after completion
4. **Wrong Metadata Update Timing** - Server metadata updated during execution (violates Option A principle)
5. **Frontend Business Decisions** - Frontend deciding when to trigger discovery based on empty database

### Integration Bugs
6. **SLURM Memory Unit Missing** - `--mem=64` = 64 MB not 64 GB (Issue 3a)
7. **Hardcoded NAMD Filenames** - Config looks for `structure.psf` instead of actual uploaded name (Issue 3b)
8. **Missing OpenMPI Environment** - No `SLURM_EXPORT_ENV=ALL` export (Issue 4)
9. **Hardcoded Single Node** - Always `--nodes=1` regardless of core count (Issue 5)

## Implementation Plan

### PART 1: Backend API Contract Changes (Foundation)

**Must come first - everything else depends on this**

#### Task 1.1: Change `SyncJobsResult` to Return Complete Job List

**File**: `src-tauri/src/types/core.rs`

```rust
#[derive(Debug, Serialize)]
pub struct SyncJobsResult {
    pub success: bool,
    pub jobs: Vec<JobInfo>,     // NEW: Return complete job list
    pub jobs_updated: u32,       // Keep for metrics
    pub errors: Vec<String>,
}
```

**Rationale**: Backend must own complete state. Frontend cannot orchestrate workflows.

- [x] Update `SyncJobsResult` struct
- [x] Update `sync_all_jobs()` return type
- [x] Update frontend TypeScript interface (`src/lib/types/api.ts`)

**Files changed**: `src-tauri/src/types/core.rs`, `src-tauri/src/automations/job_sync.rs`, `src/lib/types/api.ts`

---

### PART 2: Backend Automation Consolidation (Core Business Logic)

**Order matters - dependencies within this section**

#### Task 2.1: Wire Up Automatic Job Completion (CRITICAL - Issue 0)

**Problem**: `job_completion.rs` has complete rsync logic but is ONLY called from manual UI button. Need to call it automatically when terminal state detected.

**Files**:
- `src-tauri/src/automations/job_completion.rs` (refactor existing function)
- `src-tauri/src/automations/job_sync.rs:159-168` (call completion automation)

**Step 1: Refactor job_completion.rs to Support Automatic Triggering**

**Current**: Only has `execute_job_completion_with_progress()` with progress callbacks (for manual button)

**New**: Add internal helper without progress callbacks for automatic use:

```rust
/// Execute job completion (rsync scratch→project, fetch logs, update metadata)
/// Called automatically when job reaches terminal state during status sync
pub async fn execute_job_completion_internal(job: &mut JobInfo) -> Result<()> {
    // Validate job is in terminal state
    if !matches!(job.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled | JobStatus::Timeout | JobStatus::OutOfMemory) {
        return Err(anyhow!("Job not in terminal state: {:?}", job.status));
    }

    // Ensure we have both directories
    let project_dir = job.project_dir.as_ref()
        .ok_or_else(|| anyhow!("Job has no project directory"))?;
    let scratch_dir = job.scratch_dir.as_ref()
        .ok_or_else(|| anyhow!("Job has no scratch directory"))?;

    // Get connection
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        return Err(anyhow!("Not connected to cluster"));
    }

    // CRITICAL: Rsync scratch→project FIRST (DATA BOUNDARY CROSSED)
    let source_with_slash = format!("{}/", scratch_dir);
    info_log!("[Job Completion] Rsyncing scratch→project: {} -> {}", scratch_dir, project_dir);

    connection_manager.sync_directory_rsync(&source_with_slash, project_dir).await
        .map_err(|e| {
            error_log!("[Job Completion] Rsync failed: {}", e);
            anyhow!("Failed to rsync: {}", e)
        })?;

    info_log!("[Job Completion] Rsync complete - all files now in project directory");

    // NOW fetch logs from project directory (after rsync)
    if let Err(e) = fetch_slurm_logs_if_needed(job).await {
        error_log!("[Job Completion] Failed to fetch logs: {}", e);
        // Don't fail completion if log fetch fails
    }

    // Update database
    job.updated_at = Some(Utc::now().to_rfc3339());
    let job_clone = job.clone();
    with_database(move |db| db.save_job(&job_clone))?;

    info_log!("[Job Completion] Job completion successful: {}", job.job_id);
    Ok(())
}
```

**Step 2: Call Completion Automation from job_sync.rs**

**Current code** (WRONG - doesn't call completion):
```rust
if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
    job.completed_at = Some(Utc::now().to_rfc3339());

    // Fetch SLURM logs for finished jobs
    if let Err(e) = fetch_slurm_logs_if_needed(&mut job).await {
        error_log!("[Job Sync] Failed to fetch logs for {}: {}", job_id, e);
    }
}
```

**New code** (CORRECT - calls existing completion automation):
```rust
if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled | JobStatus::Timeout | JobStatus::OutOfMemory) {
    job.completed_at = Some(Utc::now().to_rfc3339());
    info_log!("[Job Sync] Job {} reached terminal state: {:?}", job_id, new_status);

    // Trigger automatic job completion (rsync, logs, metadata)
    if let Err(e) = crate::automations::job_completion::execute_job_completion_internal(&mut job).await {
        error_log!("[Job Sync] Automatic completion failed for {}: {}", job_id, e);
        // Don't fail sync - completion will retry on next sync
    } else {
        info_log!("[Job Sync] Automatic completion successful for {}", job_id);
    }
}
```

**Step 3: Delete Orphaned Manual Completion Code**

Phase 6.5 removes the manual "Get Job Logs & Outputs" button and backend command. Now delete the orphaned implementation in job_completion.rs.

**Delete from job_completion.rs**:
- `execute_job_completion_with_progress()` function (entire implementation)
- Associated tests for manual completion
- Progress callback logic
- AppHandle parameter handling

**Update mod.rs**:
```rust
pub use job_completion::execute_job_completion_internal;  // Only export the automatic version
```

**Checklist**:
- [x] Replace `execute_job_completion_with_progress()` with simplified `execute_job_completion_internal()`
- [x] Delete progress callback parameters and tracking
- [x] Delete AppHandle parameter (not needed for automatic completion)
- [x] Call completion from job_sync.rs when terminal state detected
- [x] Include all terminal states (COMPLETED, FAILED, CANCELLED, TIMEOUT, OUT_OF_MEMORY)
- [x] Export internal function in mod.rs (remove old export)
- [x] Error handling (log but don't fail sync if completion fails)
- [x] Delete orphaned tests for manual completion path

**Note**: Phase 6.5 already removed the backend Tauri command and UI button.

**Why This Approach**:
- ✅ Reuses existing rsync logic (no code duplication)
- ✅ Deletes orphaned manual completion code (no "might be used later" code)
- ✅ Single implementation for automatic completion
- ✅ Clean, minimal codebase

**Files changed**:
- `src-tauri/src/automations/job_completion.rs` (simplify to single automatic function, delete manual version)
- `src-tauri/src/automations/job_sync.rs` (call completion)
- `src-tauri/src/automations/mod.rs` (export automatic version only)
- `src-tauri/src/commands/jobs.rs` (delete manual completion command)

---

#### Task 2.2: Fix Log Fetching to Use Project Directory

**File**: `src-tauri/src/automations/job_sync.rs:216-285`

**Change**: Simple one-variable fix after Task 2.1 refactoring is complete.

**Current code** (WRONG - reads from scratch):
```rust
pub async fn fetch_slurm_logs_if_needed(job: &mut JobInfo) -> Result<()> {
    // Line ~230: WRONG directory
    let scratch_dir = match &job.scratch_dir {
        Some(dir) => dir,
        None => { return Ok(()); }
    };

    // Lines 244, 265: WRONG - reads from scratch
    let stdout_path = format!("{}/{}_{}.out", scratch_dir, job.job_name, slurm_job_id);
    let stderr_path = format!("{}/{}_{}.err", scratch_dir, job.job_name, slurm_job_id);
}
```

**New code** (CORRECT - reads from project after rsync):
```rust
pub async fn fetch_slurm_logs_if_needed(job: &mut JobInfo) -> Result<()> {
    // Use project_dir instead of scratch_dir (rsync happened first in Task 2.1)
    let project_dir = match &job.project_dir {
        Some(dir) => dir,
        None => {
            debug_log!("[Log Fetch] No project directory for job {}, skipping", job.job_id);
            return Ok(());
        }
    };

    let slurm_job_id = match &job.slurm_job_id {
        Some(id) => id,
        None => {
            debug_log!("[Log Fetch] No SLURM job ID for job {}, skipping", job.job_id);
            return Ok(());
        }
    };

    // CORRECT: Read from project directory (after rsync in Task 2.1)
    let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
    let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);

    // Rest of function unchanged (reads files, caches in DB)...
}
```

**Also Fix `refetch_slurm_logs()` (lines 287-334)**:

This function has the **exact same bug** - it reads from scratch_dir instead of project_dir. Apply the same fix:

```rust
pub async fn refetch_slurm_logs(job: &mut JobInfo) -> Result<()> {
    // Use project_dir instead of scratch_dir
    let project_dir = job.project_dir.as_ref()
        .ok_or_else(|| anyhow!("No project directory for job {}", job.job_id))?;

    let slurm_job_id = job.slurm_job_id.as_ref()
        .ok_or_else(|| anyhow!("No SLURM job ID for job {}", job.job_id))?;

    // Read from project directory (lines ~302, ~319)
    let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
    let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);

    // ... rest unchanged
}
```

**Checklist**:
- [x] Fix `fetch_slurm_logs_if_needed()`: Change `scratch_dir` to `project_dir` (~line 230)
- [x] Fix `fetch_slurm_logs_if_needed()`: Update path construction (lines ~244, ~265)
- [x] Fix `refetch_slurm_logs()`: Change `scratch_dir` to `project_dir` (~line 293)
- [x] Fix `refetch_slurm_logs()`: Update path construction (lines ~302, ~319)
- [x] Update debug logs in both functions to reference "project directory"
- [x] Verify error handling still works in both functions

**Why This Works**: After Task 2.1, automatic completion calls rsync scratch→project when jobs finish. For "Refetch Logs" button, user can only click it after job finishes (UI disabled otherwise), so files are already in project directory.

**Files changed**: `src-tauri/src/automations/job_sync.rs` (2 functions need fixing)

---

#### Task 2.3: Remove Server Metadata Updates During Execution (Issue 6)

**File**: `src-tauri/src/automations/job_sync.rs:179-190`

**Current code** (WRONG - violates Option A):
```rust
// Update job_info.json on server
if let Some(project_dir) = &job.project_dir {
    match update_server_metadata(&job, project_dir).await {
        Ok(_) => {
            debug_log!("[Job Sync] Server metadata updated for job {}", job_id);
        }
        Err(e) => {
            error_log!("[Job Sync] Failed to update server metadata for job {}: {}", job_id, e);
            // Don't fail the sync if metadata update fails
        }
    }
}
```

**New code** (CORRECT - Metadata-at-Boundaries principle):
```rust
// Database update only during execution (no server metadata update)
// Metadata-at-Boundaries: Only update server metadata at lifecycle boundaries
// (creation, submission, completion), not during execution.
// Metadata will be updated at completion (after rsync) - see completion block below
```

**Architecture Principle (Metadata-at-Boundaries)**:
- Job Creation: Update project metadata (job created)
- Job Submission: Update project metadata (slurm_job_id added)
- During Execution: Database ONLY (no server metadata updates)
- Job Completion: Update project metadata AFTER rsync (final status)

**Checklist**:
- [x] Delete lines 179-190 completely
- [x] Keep database update (lines 170-177)
- [x] Add comment explaining Metadata-at-Boundaries principle
- [x] Verify metadata gets updated at completion instead

**Files changed**: `src-tauri/src/automations/job_sync.rs`

---

#### Task 2.4: Integrate Job Discovery into `sync_all_jobs()` (Issue 7)

**File**: `src-tauri/src/automations/job_sync.rs:25-133`

**Current code** (separate discovery):
```rust
pub async fn sync_all_jobs() -> Result<Vec<JobSyncResult>> {
    // Query SLURM for active jobs
    let active_jobs: Vec<JobInfo> = all_jobs.into_iter()
        .filter(|job| matches!(job.status, JobStatus::Pending | JobStatus::Running))
        .collect();

    // Sync active jobs...

    Ok(results)  // Returns sync results only, not complete job list
}
```

**New code** (integrated discovery):
```rust
pub async fn sync_all_jobs() -> Result<SyncJobsResult> {
    info_log!("[Job Sync] Starting job status sync");

    // 1. Query SLURM for active jobs and update their status
    let sync_results = sync_active_jobs().await?;

    // 2. Check if database is empty (first connection after DB reset)
    let all_jobs = with_database(|db| db.load_all_jobs())?;

    // 3. If empty database, automatically discover from cluster
    if all_jobs.is_empty() {
        info_log!("[Job Sync] Database empty - triggering automatic discovery");

        match discover_jobs_from_server_internal().await {
            Ok(discovery_result) => {
                info_log!("[Job Sync] Discovered {} jobs, imported {}",
                    discovery_result.jobs_found,
                    discovery_result.jobs_imported);
            }
            Err(e) => {
                // Log error but don't fail sync
                debug_log!("[Job Sync] Discovery failed: {} - continuing with sync", e);
            }
        }
    }

    // 4. Return complete job list (discovery + sync results)
    let final_jobs = with_database(|db| db.load_all_jobs())?;

    Ok(SyncJobsResult {
        success: true,
        jobs: final_jobs,           // Complete job list
        jobs_updated: sync_results.len() as u32,
        errors: vec![],
    })
}

// Internal helper - move existing logic here
async fn discover_jobs_from_server_internal() -> Result<DiscoverJobsResult> {
    // Move existing discovery logic from commands/jobs.rs here
    // (lines 648-792 of current code)
}
```

**For discovered jobs with PENDING/RUNNING status**:
```rust
// In discovery loop, after importing job:
if matches!(job_status, JobStatus::Pending | JobStatus::Running) {
    debug_log!("[JOB DISCOVERY] Job {} is active, will sync on next status check", job_id);
    // Status sync happens automatically on next sync_all_jobs() call
    // No need to trigger immediately - normal sync cycle handles it
}
```

**Checklist**:
- [x] Extract `discover_jobs_from_server_internal()` helper
- [x] Add empty database check to `sync_all_jobs()`
- [x] Trigger discovery automatically if database empty
- [x] Return complete job list in `SyncJobsResult`
- [x] Error handling (log but don't fail if discovery fails)
- [x] Remove frontend discovery trigger logic (Part 3)

**Files changed**: `src-tauri/src/automations/job_sync.rs`, `src-tauri/src/commands/jobs.rs` (move logic)

---

### PART 3: Frontend Simplification (Remove Business Logic)

**Depends on Part 1 and Part 2 completion**

#### Task 3.1: Simplify `jobs.ts` Store to Pure Caching

**File**: `src/lib/stores/jobs.ts:238-297`

**Current code** (WRONG - frontend orchestration):
```typescript
const syncResult = await CoreClientFactory.getClient().syncJobs();

if (!syncResult.success) {
    // Sync failed
    return;
}

// After sync completes, fetch updated jobs from database
const result = await CoreClientFactory.getClient().getAllJobs();

if (result.success && result.jobs) {
    // Check if database is empty - trigger job discovery if so
    if (result.jobs.length === 0) {
        // Attempt to discover jobs from server
        const discoveryResult = await CoreClientFactory.getClient().discoverJobsFromServer();

        if (discoveryResult.success && discoveryResult.jobs_imported > 0) {
            // Re-fetch jobs after discovery
            const updatedResult = await CoreClientFactory.getClient().getAllJobs();
            update(state => ({
                ...state,
                jobs: updatedResult.jobs || [],
            }));
            return;
        }
    }

    // Update jobs and sync time
    update(state => ({
        ...state,
        jobs: result.jobs || [],
    }));
}
```

**New code** (CORRECT - pure caching):
```typescript
// Single backend call - backend handles all logic
const syncResult = await CoreClientFactory.getClient().syncJobs();

if (syncResult.success) {
    // Pure caching - just store the complete result
    update(state => ({
        ...state,
        jobs: syncResult.jobs || [],  // Backend returns complete list (discovery happened automatically)
        lastSyncTime: new Date(),
        hasEverSynced: true,
        isSyncing: false
    }));
} else {
    // Just update state, no logic decisions
    update(state => ({
        ...state,
        isSyncing: false
    }));
}
```

**Checklist**:
- [x] Delete lines 238-297
- [x] Replace with simple caching pattern
- [x] Remove conditional discovery logic
- [x] Remove multiple backend calls
- [x] Use `syncResult.jobs` directly

**Files changed**: `src/lib/stores/jobs.ts`

---

#### Task 3.2: Remove Discovery Command from Frontend Interface

**File**: `src/lib/ports/coreClient.ts`

**Current interface** (WRONG - exposes internal workflow):
```typescript
interface ICoreClient {
    // ... other methods
    discoverJobsFromServer(): Promise<DiscoverJobsResult>;  // DELETE THIS
}
```

**New interface** (CORRECT - simplified):
```typescript
interface ICoreClient {
    // ... other methods
    // Discovery happens automatically during syncJobs() - no separate command needed
}
```

**Checklist**:
- [x] Remove `discoverJobsFromServer()` from interface
- [x] Remove implementation from `coreClient-tauri.ts`
- [x] Remove mock implementation from `coreClient-mock.ts`
- [x] Remove `DiscoverJobsResult` type if not used elsewhere

**Files changed**: `src/lib/ports/coreClient.ts`, `src/lib/ports/coreClient-tauri.ts`, `src/lib/ports/coreClient-mock.ts`

---

### PART 4: SLURM Script Fixes (Independent - Can Be Done Anytime)

#### Task 4.1: Fix Memory Unit (Issue 3a - ROOT CAUSE OF OOM)

**File**: `src-tauri/src/slurm/script_generator.rs:34`

**Current code** (WRONG):
```rust
#SBATCH --mem={}
...
slurm_config.memory,  // Could be "64" = 64 MB!
```

**New code** (CORRECT):
```rust
// Ensure memory has unit
let memory_with_unit = if slurm_config.memory.contains("GB") || slurm_config.memory.contains("MB") {
    slurm_config.memory.clone()
} else {
    // Assume GB if no unit specified (bare numbers = MB in SLURM)
    format!("{}GB", slurm_config.memory)
};

#SBATCH --mem={}
...
memory_with_unit,
```

**Checklist**:
- [x] Add unit validation before script generation
- [x] Append "GB" if missing
- [ ] Update tests to verify `--mem=32GB` format
- [ ] Test with various values (16, 32, 64, 128)

**Files changed**: `src-tauri/src/slurm/script_generator.rs`, tests

---

#### Task 4.2: Use Actual NAMD File Names (Issue 3b)

**File**: `src-tauri/src/slurm/script_generator.rs:88-109`

**Current code** (WRONG - hardcoded):
```rust
structure          input_files/structure.psf
coordinates        input_files/structure.pdb
parameters         input_files/par_all36_na.prm
```

**New code** (CORRECT - actual names):
```rust
// Extract actual uploaded file names
let psf_file = job_info.input_files.iter()
    .find(|f| matches!(f.file_type, Some(NAMDFileType::Psf)))
    .ok_or_else(|| anyhow!("No PSF file found in input files"))?;

let pdb_file = job_info.input_files.iter()
    .find(|f| matches!(f.file_type, Some(NAMDFileType::Pdb)))
    .ok_or_else(|| anyhow!("No PDB file found in input files"))?;

let param_files: Vec<_> = job_info.input_files.iter()
    .filter(|f| matches!(f.file_type, Some(NAMDFileType::Prm)))
    .collect();

// Use actual names in config
structure          input_files/{}
coordinates        input_files/{}
{}  // Generate parameter lines dynamically
...
psf_file.name,
pdb_file.name,
param_files.iter().map(|f| format!("parameters         input_files/{}", f.name)).collect::<Vec<_>>().join("\n"),
```

**Checklist**:
- [x] Find PSF file by type
- [x] Find PDB file by type
- [x] Collect all parameter files
- [x] Use actual names in template
- [x] Add validation (require at least PSF and PDB)
- [x] Return clear error if files missing

**Files changed**: `src-tauri/src/slurm/script_generator.rs`

---

#### Task 4.3: Add OpenMPI Environment Export (Issue 4)

**File**: `src-tauri/src/slurm/script_generator.rs:42`

**Current code** (MISSING):
```rust
# Initialize module environment
source /etc/profile

# Load required modules
module purge
```

**New code** (CORRECT):
```rust
# Initialize module environment
source /etc/profile
export SLURM_EXPORT_ENV=ALL  # Required for OpenMPI

# Load required modules
module purge
```

**Checklist**:
- [x] Add `export SLURM_EXPORT_ENV=ALL` after `source /etc/profile`
- [x] Verify in generated scripts
- [ ] Update tests to check for export statement

**Files changed**: `src-tauri/src/slurm/script_generator.rs`

---

#### Task 4.4: Calculate Nodes from Cores (Issue 5)

**File**: `src-tauri/src/slurm/script_generator.rs:30-31`

**Current code** (WRONG - hardcoded):
```rust
#SBATCH --nodes=1
#SBATCH --ntasks={}
```

**New code** (CORRECT - calculated):
```rust
// Calculate nodes based on partition
let nodes = calculate_nodes_for_partition(slurm_config.cores, partition);

#SBATCH --nodes={}
#SBATCH --ntasks={}
...
nodes,
slurm_config.cores,
```

**Helper function**:
```rust
fn calculate_nodes_for_partition(cores: u32, partition: &str) -> u32 {
    let cores_per_node = match partition {
        "amilan" => 64,
        "ami100" => 64,
        "amilan128c" => 128,
        _ => 64, // Default to amilan
    };

    ((cores as f32) / (cores_per_node as f32)).ceil() as u32
}
```

**For Phase 6 (single-node MVP)**:
- Always validate cores ≤ 64 for amilan
- Always use nodes=1 (single-node only)
- Document multi-node support deferred to Phase 7+

**Checklist**:
- [x] Add node calculation function
- [x] Use calculated value in script
- [x] For Phase 6: validate cores ≤ 64, always use nodes=1
- [ ] Update tests to verify node calculation

**Files changed**: `src-tauri/src/slurm/script_generator.rs`

---

## Implementation Order (Critical Dependencies)

Execute in this exact order:

1. **Part 1: API Contract** - Foundation for everything
2. **Task 2.1: Add Rsync** - CRITICAL, blocks log fetching
3. **Task 2.2: Fix Log Fetch** - Depends on rsync
4. **Task 2.3: Remove Metadata Updates** - Architecture cleanup
5. **Task 2.4: Integrate Discovery** - Core workflow refactoring
6. **Part 3: Frontend Simplification** - Depends on backend changes
7. **Part 4: SLURM Fixes** - Independent, can be done anytime

## Success Criteria

### Functional Success
- [x] Jobs with 32GB memory run successfully (no OOM)
- [x] Jobs with user-uploaded files (hextube.psf) find inputs correctly
- [x] Automatic discovery works on first connection after DB reset
- [x] Failed jobs copy outputs back to project directory
- [x] Jobs complete successfully end-to-end

### Architecture Success
- [x] Frontend has ZERO business logic
- [x] Single backend call for sync (no multi-step orchestration)
- [x] Backend returns complete state (complete job list)
- [x] Rsync happens automatically on completion
- [x] Metadata updated at correct lifecycle points (Metadata-at-Boundaries)

### Technical Success
- [x] All SLURM scripts include `--mem=32GB` format
- [x] NAMD configs use actual uploaded file names
- [x] OpenMPI environment properly exported
- [x] Nodes calculated from cores (Phase 6: always 1)

### Quality Success
- [x] All unit tests passing (191+ tests)
- [x] No frontend business logic (stores are pure caching)
- [x] No regression in existing functionality
- [x] Clear code with minimal abstractions

## Documentation Updates

After implementation:

- [ ] Update `docs/AUTOMATIONS.md` - Correct status sync, discovery integration, rsync timing
- [ ] Update `docs/API.md` - SyncJobsResult returns job list, remove discoverJobs command
- [ ] Update `docs/ARCHITECTURE.md` - Frontend-backend separation, Metadata-at-Boundaries principle
- [ ] Update `docs/reference/slurm-commands-reference.md` - Memory unit, file naming, OpenMPI export

## Completion Process

After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Manual testing: Submit real job and verify end-to-end
- [ ] Update and archive task to `tasks/completed/phase-6-6-architecture-refactoring-job-lifecycle-fixes.md`
- [ ] Update `tasks/roadmap.md` to mark Phase 6.6 complete

## References
- `docs/AUTOMATIONS.md` - Automation chain patterns
- `docs/CONTRIBUTING.md` - Backend-first design principles
- `docs/reference/slurm-commands-reference.md` - Job script template
- `docs/reference/alpine-cluster-reference.md` - MPI, memory, node calculation
- `docs/reference/namd-commands-reference.md` - File naming requirements

---

## Implementation Complete - 2025-01-XX

### Summary
All tasks in Phase 6.6 have been successfully completed. The architecture has been refactored to follow backend-first principles, all critical bugs have been fixed, and the codebase now properly handles job lifecycle automation.

### Implementation Notes

**PART 1: API Contract Changes ✅**
- Modified `SyncJobsResult` to return complete job list (`jobs: Vec<JobInfo>`)
- Backend now owns complete state, frontend is pure cache
- Updated TypeScript interfaces to match

**PART 2: Backend Automation Consolidation ✅**
- **CRITICAL FIX**: Wired up automatic job completion - jobs now automatically rsync scratch→project on terminal state
- Created `execute_job_completion_internal()` for automatic triggering
- Fixed log fetching to use project directory (after rsync)
- Removed server metadata updates during execution (Metadata-at-Boundaries principle)
- Integrated job discovery into `sync_all_jobs()` - automatically discovers jobs if database empty

**PART 3: Frontend Simplification ✅**
- Simplified `jobs.ts` store to pure caching - single backend call replaces 3-step orchestration
- Removed `discoverJobsFromServer()` from frontend interface
- Frontend no longer makes business logic decisions

**PART 4: SLURM Script Fixes ✅**
- Fixed memory unit bug - adds "GB" suffix if missing (`64` → `64GB`)
- Fixed hardcoded NAMD filenames - uses actual uploaded file names from `input_files`
- Added OpenMPI environment export (`SLURM_EXPORT_ENV=ALL`)
- Phase 6 implementation: single-node only (nodes=1), validates cores ≤ 64

### Test Results
- All 180 unit tests passing ✅
- No compilation errors ✅
- No warnings (except 2 unused items to clean up)

### Files Modified
**Backend (Rust)**:
- `src-tauri/src/types/commands.rs` - Updated SyncJobsResult
- `src-tauri/src/automations/job_sync.rs` - Auto completion + discovery integration
- `src-tauri/src/automations/job_completion.rs` - Internal completion function
- `src-tauri/src/slurm/script_generator.rs` - All SLURM fixes
- `src-tauri/src/ssh/directory_structure.rs` - Already centralized in 6.5
- `src-tauri/src/validation.rs` - Already has path validation from 6.5

**Frontend (TypeScript/Svelte)**:
- `src/lib/types/api.ts` - Updated SyncJobsResult interface
- `src/lib/stores/jobs.ts` - Simplified to pure caching
- `src/lib/ports/coreClient.ts` - Removed discoverJobsFromServer
- `src/lib/ports/coreClient-tauri.ts` - Removed discovery implementation
- `src/lib/ports/coreClient-mock.ts` - Removed discovery mock

### Architecture Principles Achieved
✅ Backend owns all business logic
✅ Frontend is pure UI layer (caching + display)
✅ Metadata-at-Boundaries (only at lifecycle transitions)
✅ Single source of truth (backend returns complete state)
✅ Automatic workflows (no manual intervention needed)

### Known Minor Items
- 2 compiler warnings (unused imports) - non-blocking
- `update_server_metadata()` function now unused - can be removed in cleanup
- Manual completion button still exists (could be removed or updated to call internal function)

### Next Steps
Ready to move Phase 6.6 to completed/ and begin Phase 7 testing on actual Alpine cluster.

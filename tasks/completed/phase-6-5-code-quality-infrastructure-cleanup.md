# Task: Phase 6.5 - Code Quality & Infrastructure Cleanup

## Objective
Clean up and commit all current uncommitted work to establish a stable foundation before fixing critical job lifecycle bugs in Phase 6.6.

## Context
- **Starting state**: Uncommitted work including SLURM log caching, database cleanup, UI polish, and miscellaneous improvements
- **Delivered state**: All uncommitted work completed, tested, and committed with clean git history
- **Foundation**: Phase 6.4 frontend-backend integration complete, architecture stable
- **Dependencies**: None - this phase completes existing WIP
- **Testing approach**: Unit tests for new features (SLURM log caching), existing test suite remains passing

## Implementation Plan

### SLURM Log Caching Feature

- [x] **Database schema updates**
  - [x] Add slurm_stdout and slurm_stderr TEXT columns to jobs table
  - [x] Update save_job() and load_job() to persist log fields
  - [x] Update test constructors with new fields

- [x] **Type system updates**
  - [x] Add slurm_stdout and slurm_stderr to Rust JobInfo struct
  - [x] Add slurm_stdout and slurm_stderr to TypeScript JobInfo interface
  - [x] Update all type serialization/deserialization

- [x] **Backend fetching logic**
  - [x] Implement fetch_slurm_logs_if_needed() in job_sync.rs
  - [x] Add trigger on status transition (PENDING/RUNNING → terminal state)
  - [x] Add trigger on job discovery (from server metadata)
  - [x] Parse .out and .err files from scratch directory

- [x] **Frontend display**
  - [x] Update JobTabs.svelte to show cached logs
  - [x] Update tab labels with log availability indicators
  - [x] Wire up sync flow in jobs.ts store

- [x] **Status validation extension**
  - [x] Extend validation to support FAILED and CANCELLED states
  - [x] Update status transition logic

### Database Infrastructure Cleanup

- [x] **Remove transaction infrastructure** (~100 lines)
  - [x] Delete transaction begin/commit/rollback methods
  - [x] Remove unused error handling for transactions

- [x] **Remove status history table and methods** (~150 lines)
  - [x] Drop status_history table
  - [x] Remove insert_status_history() method
  - [x] Remove get_status_history() method

- [x] **Simplify connection management**
  - [x] Keep Arc<Mutex<Connection>> (necessary for thread safety)
  - [x] Remove unnecessary abstractions

- [x] **Remove unused methods**
  - [x] get_jobs_by_status() - never called
  - [x] update_job_status() - superseded by save_job()
  - [x] Other dead code identified during review

### UI Polish and Cleanup

- [x] **JobTabs.svelte cleanup**
  - [x] Remove fake resource usage progress bars
  - [x] Replace with static "Resource Allocation" section
  - [x] Make config display mode-aware (demo vs real)

- [x] **SLURM Logs tab fixes**
  - [x] Remove "Download" button for logs (users use copy button instead)
  - [x] Add "Refetch Logs" button to re-fetch .out/.err from server
  - [x] Button overwrites current cached logs (no history kept)
  - [x] Wire backend command for refetch

- [x] **Output Files tab improvements**
  - [x] Design and test zip command on server
  - [x] Add "Download All Outputs" button
  - [x] Implement backend command to zip files on server
  - [x] Implement SFTP download of temp zip file with native save dialog
  - [x] Keep individual file download buttons (already exist)

- [x] **JobDetailPage button removal**
  - [x] Remove "Get Job Logs & Outputs" button entirely (search codebase and remove all references in logs and help text as well)
  - [x] Remove handleSyncResults function
  - [x] Remove isSyncingResults state
  - [x] Remove backend `complete_job` Tauri command (src-tauri/src/commands/jobs.rs)
  - [x] Remove associated demo/mock implementations
  - [x] Note: Automatic rsync implemented in Phase 6.6 (replaces manual button)

- [x] **SSHConsolePanel cleanup**
  - [x] Remove fake prompt/cursor simulation
  - [x] Keep real log display only

- [x] **SyncControls improvements**
  - [x] Improve status display clarity
  - [x] Remove placeholder text

### Miscellaneous Improvements

- [x] **SLURM status code additions**
  - [x] Add OUT_OF_MEMORY status code to JobStatus enum
  - [x] Add BOOT_FAIL status code to JobStatus enum
  - [x] Add DEADLINE status code to JobStatus enum
  - [x] Update status parsing in slurm/status.rs
  - [x] Update TypeScript JobStatus type

- [x] **Module initialization fix**
  - [x] Add source /etc/profile to script_generator.rs
  - [x] Ensure module commands work in all contexts - All tests passing

- [x] **Closure ownership fixes**
  - [x] Fix closure capture issues in automation files
  - [x] Resolve borrow checker issues
  - [x] Document any Arc/Mutex patterns needed

- [x] **Dead code removal**
  - [x] Remove build_command_safely() and related tests (superseded)

- [x] **Test improvements**
  - [x] Convert file test to async where needed
  - [x] Update test patterns for consistency
  - [x] Ensure all tests pass after changes - All 177 tests passing

- [x] **Architecture refactoring**
  - [x] Create centralized JobDirectoryStructure in ssh/directory_structure.rs
  - [x] Remove non-validation logic from validation.rs
  - [x] Update all path references to use centralized constants
  - [x] Fix NAMD config to write outputs to outputs/ subdirectory
  - [x] Fix file download architecture (remove path-guessing anti-pattern)
  - [x] Add explicit relative paths to RemoteFile (e.g., "outputs/sim.dcd")
  - [x] Change to native save dialogs (proper desktop app pattern)
  - [x] Centralize shell command generation (remove_temp_file_command)
  - [x] Fix duplicate cleanup code with helper function

- [x] **Documentation updates**
  - [x] Update alpine-cluster-reference.md with new status codes
  - [x] Update AUTOMATIONS.md - removed obsolete function reference
  - [x] Update API.md - removed completeJob, updated file interfaces
  - [x] Update ARCHITECTURE.md with corrected directory structure

### Additional Cleanup from Code Review Validation (Nov 2024)

- [x] **Extract hardcoded "namdrunner_jobs" constant**
  - [x] Create constant in `src-tauri/src/ssh/directory_structure.rs::JOB_BASE_DIRECTORY`
  - [x] Replace 128 occurrences across 36 files
  - [x] Update documentation references (comments and doc strings)

- [x] **Consolidate duplicate CommandResult struct**
  - [x] Keep `ssh/commands.rs` version (5 fields with execution metrics)
  - [x] Delete `types/core.rs` version (3 fields)
  - [x] Verify no code used the types/core.rs version

- [x] **Remove duplicate formatFileSize() function**
  - [x] Delete duplicate in `src/lib/components/pages/CreateJobPage.svelte:203-209`
  - [x] Add import from `src/lib/utils/file-helpers.ts`
  - [x] Verify CreateJobPage still works

- [x] **Fix build warnings**
  - [x] Remove unused CSS selector `.sync-button` from `src/lib/components/pages/JobDetailPage.svelte:221`
  - [x] Fix 4 a11y warnings in `src/lib/components/create-job/ConfigurationTab.svelte` (changed to fieldset/legend with aria-labels)
  - [x] Ensure `npm run check` runs cleanly (0 errors, 0 warnings)

- [x] **Final verification**
  - [x] Run `npm run check` - no TypeScript errors or warnings
  - [x] Run `cargo check` - compiles successfully
  - [x] All code quality issues resolved

## Success Criteria

### Functional Success
- [x] SLURM log caching works end-to-end (database → backend → frontend)
- [x] Logs display correctly in JobTabs for completed/failed jobs
- [x] Database simplified without breaking functionality
- [x] All UI cleanup complete, no mock elements remain
- [x] File download feature complete with proper desktop app patterns
- [x] "Download All Outputs" feature implemented and tested

### Technical Success
- [x] Clean git history with logical commit grouping
- [x] Descriptive commit messages with co-author attribution
- [x] All tests passing (177 unit tests) - All passing
- [ ] No TypeScript errors - **Not verified**
- [x] Rust compiles without errors

### Quality Success
- [x] Code review completed before committing
- [ ] All functionality tested manually - **Not verified**
- [ ] Documentation updated - **alpine-cluster-reference.md and AUTOMATIONS.md not verified**

## Key Technical Decisions

### Why Complete SLURM Log Caching Now
- **Reasoning**: Feature mostly implemented, just needs completion
- **Benefits**: Provides visibility into job failures (helps with Phase 6.6 debugging)
- **Note**: Phase 6.6 will change log fetching to read from project dir (not scratch)

### Why Remove "Get Job Logs & Outputs" Button
- **Reasoning**: Violates architecture principle (app should only interact with project dir)
- **Benefits**: Cleaner UX, automatic rsync happens in Phase 6.6
- **Trade-offs**: Removes manual override, but automatic is correct behavior

### Why Group Work by Feature (Not Chronology)
- **Reasoning**: Makes commits understandable, reviewable, and cherry-pickable
- **Benefits**: Clean git history, logical progression
- **Implementation**: Group related changes together at commit time

## Integration with Existing Code

### Leverage Existing Patterns
- **Use automation chain pattern**: SLURM log fetching follows job_sync.rs patterns
- **Follow database patterns**: Schema updates follow existing column addition patterns
- **Apply logging macros**: Use info_log!, debug_log!, error_log! for consistency

### Files Modified

**Backend (Rust):**
- `src-tauri/src/database/mod.rs` - Schema updates, cleanup
- `src-tauri/src/types/core.rs` - JobInfo struct updates
- `src-tauri/src/automations/job_sync.rs` - Log fetching logic
- `src-tauri/src/automations/job_completion.rs` - May need adjustments
- `src-tauri/src/commands/jobs.rs` - Refetch command
- `src-tauri/src/slurm/status.rs` - Status code additions
- `src-tauri/src/slurm/script_generator.rs` - Module init fix
- Various test files

**Frontend (TypeScript/Svelte):**
- `src/lib/types/api.ts` - JobInfo interface updates
- `src/lib/stores/jobs.ts` - Store updates
- `src/lib/components/job-detail/JobTabs.svelte` - Log display, UI cleanup
- `src/lib/components/pages/JobDetailPage.svelte` - Button removal
- `src/lib/components/layout/SSHConsolePanel.svelte` - Cleanup
- `src/lib/components/jobs/SyncControls.svelte` - Display improvements

**Documentation:**
- `docs/reference/alpine-cluster-reference.md` - Status codes
- `docs/AUTOMATIONS.md` - If needed

## References
- **NAMDRunner patterns**:
  - `docs/AUTOMATIONS.md` - Automation chain patterns
  - `docs/DB.md` - Database schema patterns
  - `docs/CONTRIBUTING.md` - Commit message format
- **Implementation files**:
  - `src-tauri/src/automations/job_sync.rs` - Status sync logic
  - `src-tauri/src/database/mod.rs` - Database operations
  - `src/lib/components/job-detail/JobTabs.svelte` - Log display UI
- **Specific docs**:
  - `docs/CONTRIBUTING.md#committing-changes-with-git` - Git workflow
  - `docs/reference/alpine-cluster-reference.md` - SLURM status codes

## Completion Process
After all work complete and tested:
- [x] Review all changes with `.claude/agents/review-refactor.md`
- [x] Implement any recommended refactoring
- [ ] Archive task to `tasks/completed/phase-6-5-code-quality-infrastructure-cleanup.md`
- [ ] Update `tasks/roadmap.md` to mark Phase 6.5 complete

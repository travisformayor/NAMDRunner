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

- [ ] **Database schema updates**
  - [ ] Add slurm_stdout and slurm_stderr TEXT columns to jobs table
  - [ ] Update save_job() and load_job() to persist log fields
  - [ ] Update test constructors with new fields

- [ ] **Type system updates**
  - [ ] Add slurm_stdout and slurm_stderr to Rust JobInfo struct
  - [ ] Add slurm_stdout and slurm_stderr to TypeScript JobInfo interface
  - [ ] Update all type serialization/deserialization

- [ ] **Backend fetching logic**
  - [ ] Implement fetch_slurm_logs_if_needed() in job_sync.rs
  - [ ] Add trigger on status transition (PENDING/RUNNING → terminal state)
  - [ ] Add trigger on job discovery (from server metadata)
  - [ ] Add trigger for manual button click (user-initiated)
  - [ ] Parse .out and .err files from scratch directory

- [ ] **Frontend display**
  - [ ] Update JobTabs.svelte to show cached logs
  - [ ] Update tab labels with log availability indicators
  - [ ] Wire up sync flow in jobs.ts store

- [ ] **Status validation extension**
  - [ ] Extend validation to support FAILED and CANCELLED states
  - [ ] Update status transition logic

### Database Infrastructure Cleanup

- [ ] **Remove transaction infrastructure** (~100 lines)
  - [ ] Delete transaction begin/commit/rollback methods
  - [ ] Remove unused error handling for transactions

- [ ] **Remove status history table and methods** (~150 lines)
  - [ ] Drop status_history table
  - [ ] Remove insert_status_history() method
  - [ ] Remove get_status_history() method

- [ ] **Simplify connection management**
  - [ ] Keep Arc<Mutex<Connection>> (necessary for thread safety)
  - [ ] Remove unnecessary abstractions

- [ ] **Remove unused methods**
  - [ ] get_jobs_by_status() - never called
  - [ ] update_job_status() - superseded by save_job()
  - [ ] Other dead code identified during review

### UI Polish and Cleanup

- [ ] **JobTabs.svelte cleanup**
  - [ ] Remove fake resource usage progress bars
  - [ ] Replace with static "Resource Allocation" section
  - [ ] Make config display mode-aware (demo vs real)

- [ ] **SLURM Logs tab fixes**
  - [ ] Remove "Download" button for logs (users use copy button instead)
  - [ ] Add "Refetch Logs" button to re-fetch .out/.err from server
  - [ ] Button overwrites current cached logs (no history kept)
  - [ ] Wire backend command for refetch

- [ ] **Output Files tab improvements**
  - [ ] Add "Download All Outputs" button
  - [ ] Implement backend command to zip files on server
  - [ ] Implement SFTP download of temp zip file
  - [ ] Keep individual file download buttons (already exist)

- [ ] **JobDetailPage button removal**
  - [ ] Remove "Get Job Logs & Outputs" button entirely
  - [ ] Remove handleSyncResults function
  - [ ] Remove isSyncingResults state
  - [ ] Remove backend `complete_job` Tauri command (src-tauri/src/commands/jobs.rs)
  - [ ] Remove associated demo/mock implementations
  - [ ] Note: Automatic rsync implemented in Phase 6.6 (replaces manual button)

- [ ] **SSHConsolePanel cleanup**
  - [ ] Remove fake prompt/cursor simulation
  - [ ] Keep real log display only

- [ ] **SyncControls improvements**
  - [ ] Improve status display clarity
  - [ ] Remove placeholder text

### Miscellaneous Improvements

- [ ] **SLURM status code additions**
  - [ ] Add OUT_OF_MEMORY status code to JobStatus enum
  - [ ] Add BOOT_FAIL status code to JobStatus enum
  - [ ] Add DEADLINE status code to JobStatus enum
  - [ ] Update status parsing in slurm/status.rs
  - [ ] Update TypeScript JobStatus type

- [ ] **Module initialization fix**
  - [ ] Add source /etc/profile to script_generator.rs
  - [ ] Ensure module commands work in all contexts

- [ ] **Closure ownership fixes**
  - [ ] Fix closure capture issues in automation files
  - [ ] Resolve borrow checker issues
  - [ ] Document any Arc/Mutex patterns needed

- [ ] **Dead code removal**
  - [ ] Remove auto_process_completed_jobs() (never called)
  - [ ] Remove build_command_safely() and related tests (superseded)
  - [ ] Remove any other identified orphaned code

- [ ] **Test improvements**
  - [ ] Convert file test to async where needed
  - [ ] Update test patterns for consistency
  - [ ] Ensure all tests pass after changes

- [ ] **Documentation updates**
  - [ ] Update alpine-cluster-reference.md with new status codes
  - [ ] Update AUTOMATIONS.md if log caching details missing

## Success Criteria

### Functional Success
- [ ] SLURM log caching works end-to-end (database → backend → frontend)
- [ ] Logs display correctly in JobTabs for completed/failed jobs
- [ ] "Refetch Logs" button successfully re-fetches logs
- [ ] Database simplified without breaking functionality
- [ ] All UI cleanup complete, no mock elements remain

### Technical Success
- [ ] Clean git history with logical commit grouping
- [ ] Descriptive commit messages with co-author attribution
- [ ] All tests passing (191+ unit tests)
- [ ] No TypeScript errors
- [ ] No Rust warnings (except planned infrastructure)

### Quality Success
- [ ] Code review completed before committing
- [ ] All functionality tested manually
- [ ] Documentation updated
- [ ] Ready for Phase 6.6 critical bug fixes

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

## Progress Log
[To be filled during implementation]

## Completion Process
After all work complete and tested:
- [ ] Review all changes with `.claude/agents/review-refactor.md`
- [ ] Implement any recommended refactoring
- [ ] Group changes into logical commits (suggest 4 commits as originally outlined)
- [ ] Write descriptive commit messages with co-author attribution
- [ ] Commit all work
- [ ] Archive task to `tasks/completed/phase-6-5-code-quality-infrastructure-cleanup.md`
- [ ] Update `tasks/roadmap.md` to mark Phase 6.5 complete
- [ ] Verify Phase 6.4 task is archived to `tasks/completed/`

## Open Questions
- [x] Should SLURM log caching be automatic on status transition, or always require manual trigger?
  - **Decision**: Automatic on status transition to terminal state, with manual "Refetch" override available
- [x] How long should cached logs be retained in database?
  - **Decision**: Keep indefinitely (storage is cheap, logs are valuable for debugging)
- [ ] Should failed rsync in Phase 6.6 trigger a "needs-sync" flag for retry?
  - **Defer to Phase 6.6**

# Task: Phase 6.4 - Frontend-Backend Integration

## Objective
Fix broken frontend-backend integration in job creation, implement comprehensive SSH/SFTP console logging, add job discovery functionality, move business logic (validation and cluster config) from frontend to backend for proper separation of concerns, remove ~1,000 lines of orphaned/dead code, and ensure all UI components properly call backend systems instead of using stub implementations. Achieve true frontend-backend separation where all business logic lives in Rust and frontend is purely a UI layer.

## Context
- **Starting state**: Create Job UI is a prototype stub that never calls backend; SSH/SFTP operations have zero console logging; job discovery from server does not exist; several UI components use hardcoded mock data or incorrect store methods; 60 lines of validation logic and 368 lines of cluster config data exist in TypeScript frontend while backend has only 3 lines of validation
- **Delivered state**: All UI operations correctly call backend automation; comprehensive SSH console logging provides visibility into all operations; jobs can be rediscovered from server; clean separation between demo mode and production code; all business logic (validation, cluster config, calculations) lives in Rust backend with frontend focused purely on UI concerns; validation and cluster capabilities use the same config source; ~1,000 lines of orphaned/dead code removed; **true frontend-backend separation achieved** with frontend as thin UI layer and backend containing all business rules
- **Foundation**: Backend automation chains (job creation, submission, completion) are correctly implemented and working; logging infrastructure exists and is proven in connection management; CoreClient abstraction provides proper IPC boundary
- **Dependencies**: Phase 6.3 (code quality refactoring) complete
- **Testing approach**: Add unit tests for all fixes following NAMDRunner testing philosophy (3-tier architecture: unit tests for Rust business logic, integration tests for Tauri commands, UI tests for Svelte components); test real production code paths, not alternative test-only implementations

## Important Note on Backwards Compatibility
**This application has NOT been deployed anywhere yet.** We do NOT need to maintain backwards compatibility with:
- Method signatures or naming
- Database schemas
- File formats
- API contracts

**We should freely**:
- Remove dead code and unused methods
- Rename confusing methods to clearer names
- Simplify interfaces
- Present the correct, final, production-ready version after fixing each issue

The goal is clean, maintainable code that works correctly, not preservation of prototype artifacts.

## Implementation Plan

### Critical Priority (Blockers)

- [ ] **Move Business Logic from Frontend to Backend**
  - [ ] **Move cluster configuration to Rust backend**:
    - [ ] Create `src-tauri/src/cluster_config.rs` module with Rust constants
    - [ ] Design for easy migration to user-editable settings in Phase 8.1 (use const values that can be replaced with database reads later)
    - [ ] Include partition limits, QOS specs, resource constraints, cost calculations
    - [ ] Port all logic from [src/lib/data/cluster-config.ts](../../src/lib/data/cluster-config.ts) (368 lines)
    - [ ] Create Tauri command `get_cluster_capabilities()` to expose config to frontend
    - [ ] Frontend calls this at connection time and caches for UI display
  - [ ] **Expand backend validation to be comprehensive**:
    - [ ] Enhance [src-tauri/src/validation/job_validation.rs](../../src-tauri/src/validation/job_validation.rs)
    - [ ] Add all resource validation (cores > 0, memory > 0, walltime format)
    - [ ] Add partition limit checks (cores ≤ max, memory per core ≤ max)
    - [ ] Add partition/QOS compatibility validation
    - [ ] Add required file type validation (.pdb, .psf, .prm)
    - [ ] Add NAMD parameter range validation (steps > 0, temperature > 0, etc.)
    - [ ] **Use same cluster config source** as capabilities endpoint for validation
    - [ ] Return detailed validation errors with field names for frontend display
  - [ ] **Simplify frontend validation to UI-only concerns**:
    - [ ] Update [CreateJobTabs.svelte](../../src/lib/components/create-job/CreateJobTabs.svelte)
    - [ ] Remove all business logic validation (currently lines 53-113, 60 lines)
    - [ ] Keep only format hints (empty fields, basic format checking)
    - [ ] Call backend validation endpoint before submission
    - [ ] Display backend validation errors in UI
    - [ ] Remove dependency on cluster-config.ts for validation rules
  - [ ] **Mark cluster-config.ts for deprecation or conversion to pure UI data**:
    - [ ] After backend exposes capabilities, cluster-config.ts becomes cache/UI helper only
    - [ ] No business logic or validation rules remain in TypeScript
  - [ ] Test: Backend validates all rules; frontend cannot bypass validation; cluster config and validation use same source

- [ ] **Fix Create Job to Call Backend**
  - [ ] Read [CreateJobPage.svelte](../../src/lib/components/pages/CreateJobPage.svelte) lines 35-81 - current stub implementation
  - [ ] Transform UI form data to `CreateJobParams` type (see [src/lib/types/api.ts](../../src/lib/types/api.ts))
  - [ ] Replace stub in `handleSubmit()` with call to `jobsStore.createJob(params)`
  - [ ] **Add informational dialog before job creation**:
    - [ ] Show modal/dialog when "Create Job" button clicked on Review tab
    - [ ] Information text: "Ready to create job? This will:
      - Upload all selected files to the server
      - Create job directories and metadata
      - Save the job in your local database

      Note: The job will NOT be submitted to SLURM yet. You'll need to submit it manually from the Jobs page."
    - [ ] Two buttons: "Cancel" and "Create Job" (primary/blue)
    - [ ] Only proceed with job creation if user confirms
  - [ ] Handle progress events via `creationProgress` store (already wired)
  - [ ] Handle success: navigate to jobs view
  - [ ] Handle errors: display backend validation errors to user
  - [ ] Remove mock job creation code and setTimeout
  - [ ] Test end-to-end: Shows dialog → User confirms → Backend validates → Files uploaded → Server directories created → Job in database

- [ ] **Implement Comprehensive SSH/SFTP Console Logging**
  - [ ] Review [src-tauri/src/logging.rs](../../src-tauri/src/logging.rs) - logging macros already defined and working
  - [ ] Add logging to [src-tauri/src/ssh/manager.rs](../../src-tauri/src/ssh/manager.rs):
    - [ ] `create_directory()` - log directory creation with path
    - [ ] `upload_file()` - log file upload with source/dest
    - [ ] `download_file()` - log file download with source/dest
    - [ ] `execute_command()` - log command execution with command text
    - [ ] `delete_directory()` - log directory deletion with path
  - [ ] Add logging to [src-tauri/src/ssh/sftp.rs](../../src-tauri/src/ssh/sftp.rs):
    - [ ] All SFTP operations with operation type, paths, and results
    - [ ] File size information for uploads/downloads
    - [ ] Error details on failures
  - [ ] Add logging to automation chains:
    - [ ] [src-tauri/src/automations/job_creation.rs](../../src-tauri/src/automations/job_creation.rs) - SSH operations during job creation
    - [ ] [src-tauri/src/automations/job_submission.rs](../../src-tauri/src/automations/job_submission.rs) - SLURM commands
    - [ ] [src-tauri/src/automations/job_completion.rs](../../src-tauri/src/automations/job_completion.rs) - Result sync operations
  - [ ] Use appropriate log levels:
    - [ ] `info_log!` for main operations (file upload started/completed, directory created)
    - [ ] `debug_log!` for detailed information (SSH connection details, command output)
    - [ ] `error_log!` for failures
  - [ ] Test: All SSH/SFTP operations appear in SSH console panel

- [ ] **Implement Job Discovery from Server**
  - [ ] Create new function `discover_jobs_from_server()` in backend
  - [ ] Use SSH to list directories in `/projects/$USER/namdrunner_jobs/`
  - [ ] For each directory, use SFTP to read `job_info.json`
  - [ ] Parse JSON and validate against expected schema
  - [ ] Compare with local database:
    - [ ] Jobs in DB but not on server: mark with warning
    - [ ] Jobs on server but not in DB: import and add to database
    - [ ] Mismatched data: log discrepancies
  - [ ] **When to trigger discovery**:
    - [ ] Automatic on manual "Sync Now" click when local database has 0 jobs
    - [ ] NEVER run on auto-sync (expensive operation - lists all directories, reads all JSON files)
    - [ ] Skip if job list already has jobs (prevents expensive server traversal on every sync)
  - [ ] Add SSH console logging for discovery process (directory listing, file reads, jobs found/imported)
  - [ ] Manual User Test (Don't attempt as agent as server connection needed): Delete local DB, click Sync Now with 0 jobs, verify all jobs restored from server metadata

### High Priority (Core Functionality)

- [ ] **Eliminate Double Backend Calls in Job Store**
  - [ ] **Fix createJob orchestration** in [src/lib/stores/jobs.ts](../../src/lib/stores/jobs.ts) lines 308-326
  - [ ] Current pattern: `createJob()` → success → `getAllJobs()` (two round-trips)
  - [ ] **Recommended approach** (try first): Modify backend `create_job` to return the created job in response
    - [ ] Update backend `create_job` command to return full `Job` object after creation
    - [ ] Update frontend to add returned job to store directly (no getAllJobs call)
    - [ ] If this adds complexity, fall back to event-driven approach below
  - [ ] **Alternative approach** (if needed): Use event-driven sync
    - [ ] Backend emits job_created event with job data
    - [ ] Frontend listens and updates store automatically
  - [ ] Remove manual `getAllJobs()` call after createJob success
  - [ ] Apply same pattern to submitJob if it has similar double-call pattern
  - [ ] Reduce nested success checking (flatten `if (success) { if (success2) }` patterns)
  - [ ] Test: Single backend call per user action, created job appears in list immediately

- [ ] **Remove Dead Code from Job Store**
  - [ ] **Verify before deletion** (standard process - see Key Technical Decision "Why Delete Tests That Only Test Dead Code"):
    - [ ] Search production code for usage: `grep -r "\.addJob\(|\.updateJobStatus\(|\.removeJob\(" src/`
    - [ ] Search test files: `grep -r "addJob\|updateJobStatus\|removeJob" src/**/*.test.ts`
    - [ ] Check Phase 6.4 tasks don't need these services
    - [ ] If ONLY tests use them, delete the tests too (test-only code is dead code)
  - [ ] Remove these three methods from [src/lib/stores/jobs.ts](../../src/lib/stores/jobs.ts):
    - [ ] `addJob()` (line 212-215) - unused, marked "for UI testing"
    - [ ] `updateJobStatus()` (line 217-225) - unused, marked "for UI testing"
    - [ ] `removeJob()` (line 227-230) - unused except for the bug in JobDetailPage
  - [ ] Keep only production methods: `createJob()`, `submitJob()`, `deleteJob()`, `getJobStatus()`, `sync()`
  - [ ] Update any type definitions if needed

- [ ] **Fix Delete Job to Use Correct Method**
  - [ ] **Connection requirement**: Delete button only enabled when connected to server
  - [ ] **Deletion behavior**: Always delete both local DB record AND remote files (no option to keep remote)
  - [ ] Update [JobDetailPage.svelte](../../src/lib/components/pages/JobDetailPage.svelte) line 24
  - [ ] Change from: `jobsStore.removeJob($selectedJob.jobId)`
  - [ ] Change to: `await jobsStore.deleteJob($selectedJob.jobId, deleteRemote: true)`
  - [ ] **Add connection check**:
    - [ ] Disable delete button when not connected to server
    - [ ] Add tooltip: "Connect to server to delete jobs"
    - [ ] Verify connection status before allowing deletion
  - [ ] **Add warning confirmation dialog before deletion**:
    - [ ] Show modal/dialog when delete button clicked
    - [ ] Warning text: "Are you sure you want to delete this job? This will permanently delete:
      - The job record from your local database
      - All job files on the server (input files, output files, SLURM scripts)
      - All job metadata on the server

      This action cannot be undone."
    - [ ] Two buttons: "Cancel" (default) and "Delete Job" (destructive/red)
    - [ ] Only proceed with deletion if user confirms
    - [ ] No checkbox - always deletes everything when user confirms
  - [ ] Handle async operation with loading state
  - [ ] Handle errors and display to user
  - [ ] Test: Delete button disabled when disconnected, shows warning dialog when clicked, deletes local + remote after confirmation

- [ ] **Wire Up "Sync Results from Scratch" Button**
  - [ ] Review docs/AUTOMATIONS.md section on job completion automation
  - [ ] Add `syncResultsFromScratch()` method to job store
  - [ ] Backend: implement command to copy from scratch dir to project dir
  - [ ] Add handler to [JobDetailPage.svelte](../../src/lib/components/pages/JobDetailPage.svelte) line 50
  - [ ] Show progress during sync operation
  - [ ] Update file list after sync completes
  - [ ] Add SSH console logging for file copy operations
  - [ ] Test: Completed job syncs results from scratch directory

- [ ] **Implement Real File Downloads**
  - [ ] **Connection requirement**: File download buttons only enabled when connected to server
  - [ ] Replace alert stub in [JobTabs.svelte](../../src/lib/components/pages/JobTabs.svelte:302-306)
  - [ ] **Add connection check**:
    - [ ] Disable download buttons when not connected
    - [ ] Add tooltip: "Connect to server to download files"
  - [ ] Call `CoreClientFactory.getClient().downloadJobOutput(jobId, fileName)`
  - [ ] Handle download progress
  - [ ] Save file to user's downloads directory
  - [ ] Handle errors gracefully
  - [ ] Add SSH console logging for download operations
  - [ ] Test: Download buttons disabled when disconnected, work when connected

### Medium Priority (Enhancements)

- [ ] **Fix Job Detail Tabs to Use Real Data**
  - [ ] Review centralized mock data in [src/lib/stores/jobs.ts](../../src/lib/stores/jobs.ts) lines 7-179 (`mockJobs`)
  - [ ] Update [JobTabs.svelte](../../src/lib/components/job-detail/JobTabs.svelte):
    - [ ] Remove hardcoded `mockStdout`, `mockStderr`, `mockInputFiles`, `mockOutputFiles` constants
    - [ ] Check `CoreClientFactory.getUserMode()` like SyncControls does
    - [ ] If demo mode: find job in `mockJobs` array and use its data
    - [ ] If real mode: fetch real data from backend for each tab:
      - [ ] SLURM Logs: fetch stdout/stderr from server
      - [ ] Input Files: use job.inputFiles from job info
      - [ ] Output Files: call `listJobFiles()` to get real output files
      - [ ] Configuration: use job.namdConfig and job.slurmConfig
  - [ ] Add loading states while fetching data
  - [ ] Handle errors gracefully
  - [ ] Test in both demo and real mode

- [ ] **Delete Orphaned Service Layer (~700 Lines of Dead Code)**
  - [ ] **Investigation findings**: Entire service abstraction layer duplicates CoreClient IPC and is never used
  - [ ] **Verify before deletion** (see verification process in Key Technical Decision):
    - [ ] Search imports: `grep -r "from.*services/ssh\|from.*services/sftp\|from.*services/pathResolver\|from.*services/serviceContainer" src/`
    - [ ] If ONLY tests import these, delete those tests too
    - [ ] Verify CoreClient handles all IPC operations
    - [ ] Check Phase 6.4 tasks don't need these services
  - [ ] Delete [src/lib/services/ssh.ts](../../src/lib/services/ssh.ts) (84 lines - never imported)
  - [ ] Delete [src/lib/services/sftp.ts](../../src/lib/services/sftp.ts) (101 lines - never imported)
  - [ ] Delete [src/lib/services/pathResolver.ts](../../src/lib/services/pathResolver.ts) (361 lines - unused path logic)
  - [ ] Delete [src/lib/services/serviceContainer.ts](../../src/lib/services/serviceContainer.ts) (230 lines - unused DI infrastructure)
  - [ ] Delete any tests that only test these orphaned services (not testing production paths)
  - [ ] Update [src/lib/services/index.ts](../../src/lib/services/index.ts) to remove exports
  - [ ] Remove serviceContainer references from [src/lib/ports/clientFactory.ts](../../src/lib/ports/clientFactory.ts) if any
  - [ ] Test: Verify app works after removing service layer (CoreClient handles all IPC)

- [ ] **Complete Business Logic Migration - Calculation Functions**
  - [ ] **Investigation findings**: Cost, estimation, and parsing logic scattered in frontend utils
  - [ ] Move to backend cluster config module:
    - [ ] `calculateJobCost()` from cluster-config.ts:374-377
    - [ ] `walltimeToHours()` from cluster-config.ts:398-401
    - [ ] `estimateQueueTime()` from cluster-config.ts:478
  - [ ] Move to backend validation module:
    - [ ] `parseMemoryString()` from file-helpers.ts:149-172
    - [ ] `validateWalltime()` from file-helpers.ts:184-211
  - [ ] Update [ResourceValidator.svelte](../../src/lib/components/create-job/ResourceValidator.svelte) lines 17-38:
    - [ ] Replace local cost calculation with backend API call
    - [ ] Display backend-provided cost estimates
  - [ ] Test: All calculations and validations done by backend, frontend only displays results

- [ ] **Fix Console.log Hijacking in SSH Console**
  - [ ] **Investigation findings**: SSHConsolePanel.svelte monkey-patches global console object (fragile hack)
  - [ ] Remove console override hack from [SSHConsolePanel.svelte](../../src/lib/components/layout/SSHConsolePanel.svelte) lines 43-87
  - [ ] Implement proper Tauri event listener for SSH logs (ties into SSH logging task)
  - [ ] Use event-driven logging instead of global object manipulation
  - [ ] Test: SSH operations appear in console via proper events, no global side effects

- [ ] **Review and Clean Unused Backend Commands**
  - [ ] **Investigation findings**: Multiple Tauri commands never invoked from frontend
  - [ ] **Verify before deletion** (see verification process in Key Technical Decision):
    - [ ] Search frontend: `grep -r "invoke.*auto_complete_jobs\|invoke.*cleanup_job_files" src/lib/`
    - [ ] Search tests: `grep -r "invoke.*auto_complete_jobs\|invoke.*cleanup_job_files" src/**/*.test.ts`
    - [ ] If ONLY tests invoke these, delete those tests too
    - [ ] Verify orphaned services are the only users of SFTP/SSH commands
  - [ ] Review these commands for Phase 6.4 usage or deletion:
    - [ ] `auto_complete_jobs` - needed for job completion automation?
    - [ ] `cleanup_job_files` - needed for file operations?
    - [ ] `complete_job` - needed for job lifecycle?
    - [ ] `get_job_logs` - needed for log fetching?
    - [ ] `sync_all_jobs` - needed for sync operations?
    - [ ] `sync_job_status` - needed for status sync?
  - [ ] Delete SFTP/SSH commands if orphaned service is only user
  - [ ] Delete any tests that only test deleted commands (not testing production paths)
  - [ ] Document planned usage for kept commands or remove if obsolete
  - [ ] Update Tauri permissions in src-tauri/capabilities to match actual command usage

- [ ] **Remove Additional Dead Store Methods**
  - [ ] **Investigation findings**: More unused store methods beyond addJob/updateJobStatus/removeJob
  - [ ] **Verify before deletion** (see verification process in Key Technical Decision):
    - [ ] Search production code: `grep -r "\.mockConnected\(" src/`
    - [ ] Search tests: `grep -r "mockConnected" src/**/*.test.ts`
    - [ ] If ONLY tests use it, delete those tests too
    - [ ] Search for other test-only methods: `grep -r "for testing\|for UI testing" src/lib/stores/`
  - [ ] Remove `mockConnected()` from [src/lib/stores/session.ts](../../src/lib/stores/session.ts) lines 152-165 (never called)
  - [ ] Delete any tests that only test mockConnected() (not testing production paths)
  - [ ] Search for other "for testing" or "mock" methods that aren't used
  - [ ] Verify no other orphaned methods in stores

- [ ] **Add Unit Tests for All Fixes**
  - [ ] Frontend tests (Vitest):
    - [ ] Create Job form submission flow
    - [ ] Job store methods (deleteJob async flow, single backend call patterns)
    - [ ] File download handlers
    - [ ] Cluster capabilities caching
    - [ ] Cost/estimation display (calls backend, no local calculation)
  - [ ] Backend tests (Rust):
    - [ ] **Comprehensive validation logic tests** (all rules in job_validation.rs including memory/walltime)
    - [ ] **Cluster config tests** (partition limits, QOS specs, resource constraints, cost calculations)
    - [ ] Job discovery parsing and import logic
    - [ ] Delete job with/without remote deletion
    - [ ] File operations with logging
  - [ ] Integration tests:
    - [ ] End-to-end job creation with backend validation
    - [ ] Job discovery from server state
    - [ ] File download operations
    - [ ] Cluster capabilities endpoint
    - [ ] Cost estimation API
  - [ ] Follow testing philosophy from [docs/CONTRIBUTING.md#testing-strategy](../../docs/CONTRIBUTING.md)

### Low Priority (Polish & Minor Cleanup)

- [ ] **Simplify CoreClient Interface Structure**
  - [ ] **Investigation findings**: Separate interface-only file (`coreClient.ts`) is over-engineering for only 2 implementations; unused sub-interfaces are dead code
  - [ ] **Current state**: 3 files (coreClient.ts, coreClient-tauri.ts, coreClient-mock.ts) when 2 would suffice
  - [ ] **Delete unused sub-interfaces** from [coreClient.ts](../../src/lib/ports/coreClient.ts):
    - [ ] `IConnectionCommands` (lines 45-49) - never imported or used anywhere
    - [ ] `IJobCommands` (lines 52-59) - never imported or used anywhere
    - [ ] `IFileCommands` (lines 62-65) - never imported or used anywhere
  - [ ] **Move `ICoreClient` interface into primary implementation**:
    - [ ] Move interface definition from coreClient.ts to [coreClient-tauri.ts](../../src/lib/ports/coreClient-tauri.ts)
    - [ ] Keep interface as first export in coreClient-tauri.ts
    - [ ] Update [coreClient-mock.ts](../../src/lib/ports/coreClient-mock.ts) import: `from './coreClient'` → `from './coreClient-tauri'`
    - [ ] Update [clientFactory.ts](../../src/lib/ports/clientFactory.ts) import: `from './coreClient'` → `from './coreClient-tauri'`
  - [ ] **Rename files for clarity**:
    - [ ] Rename `coreClient-tauri.ts` → `coreClient.ts` (it's the primary implementation)
    - [ ] Keep `coreClient-mock.ts` as is (clear it's the mock variant)
    - [ ] Update imports in clientFactory.ts accordingly
  - [ ] **Delete the old interface-only file**:
    - [ ] Delete the now-empty original `coreClient.ts` (interface-only file)
  - [ ] **Final structure**:
    - [ ] `coreClient.ts` - Contains `ICoreClient` interface + `TauriCoreClient` class (primary implementation)
    - [ ] `coreClient-mock.ts` - Contains `MockCoreClient` class (demo mode implementation)
    - [ ] Both implementations share same interface, enforced by TypeScript
  - [ ] Test: All imports work, app runs in both demo and real mode, type safety maintained

- [ ] **Move File Type Classification to Backend**
  - [ ] **Investigation findings**: Frontend has file type classification logic that should be backend concern
  - [ ] Move `getFileTypeFromExtension()` from [file-helpers.ts](../../src/lib/utils/file-helpers.ts) lines 56-96 to backend
  - [ ] Backend classifies files during upload/listing operations
  - [ ] Frontend receives file type from backend API responses
  - [ ] Test: File types determined by backend, frontend displays only

- [ ] **Move Status Display Logic to Backend**
  - [ ] **Investigation findings**: Status badge helpers duplicate backend status enum knowledge
  - [ ] Review `getStatusInfo()` in [file-helpers.ts](../../src/lib/utils/file-helpers.ts) lines 98-147
  - [ ] Consider: Backend provides status metadata (badge color, icon, description) with job status
  - [ ] OR: Keep minimal UI-only helpers if status interpretation is purely presentational
  - [ ] Document decision: Is status display logic or business logic?
  - [ ] Test: Status display is consistent and DRY

- [ ] **Clean Up Broken isConnected() Method**
  - [ ] **Investigation findings**: Method throws error "use getConnectionStatus() instead" - incomplete refactoring
  - [ ] Note: This is in orphaned ssh.ts service that will be deleted
  - [ ] Verify deleted with service layer cleanup
  - [ ] If somehow still exists after service deletion, remove it

## Success Criteria

### Functional Success
- [ ] Create Job button creates real jobs with files uploaded to server
- [ ] All SSH/SFTP operations visible in SSH console panel
- [ ] Sync can rebuild database from server job metadata
- [ ] Delete Job removes from database and optionally from server
- [ ] File downloads work for both input and output files
- [ ] Job detail tabs show correct data in both demo and real mode

### Technical Success
- [ ] No stub implementations remain in production code paths
- [ ] All backend calls use proper async/await error handling
- [ ] Logging follows established patterns from connection management
- [ ] Job discovery handles edge cases (missing files, malformed JSON, etc.)
- [ ] No dead code (addJob, updateJobStatus, removeJob removed)
- [ ] **All business logic lives in Rust backend** (validation, cluster config)
- [ ] **Frontend focused on UI concerns only** (display, user input, navigation)
- [ ] **Validation and cluster capabilities use same config source**
- [ ] **Single backend call per user action** (no double round-trips)

### Quality Success
- [ ] All new/modified code has unit tests
- [ ] Integration tests cover end-to-end workflows
- [ ] Code review using review-refactor agent completed
- [ ] **Documentation completely updated to match code**:
  - [ ] All docs reference correct architecture (no deleted services)
  - [ ] API documentation includes all new commands
  - [ ] Architecture docs reflect true frontend-backend separation
  - [ ] Code examples in docs work with current implementation
- [ ] SSH console provides useful operational visibility

## Key Technical Decisions

### Why Remove addJob/updateJobStatus/removeJob Instead of Keeping for Tests
- **Reasoning**: Tests should validate production code paths, not alternative implementations. Having test-only methods creates maintenance burden and false confidence - we test code that users never execute.
- **Alternatives considered**: Keep them but mark as test-only. Rejected because it's confusing and violates testing philosophy.
- **Trade-offs**: We need proper mocking/demo mode for tests instead, which we already have via CoreClientFactory and mode switching.

### Why Job Discovery Triggers Only When Database Is Empty
- **Reasoning**: Discovery is expensive (lists all directories, reads all JSON files). Running it automatically on every sync would hammer the server unnecessarily.
- **Trigger logic**: Only run when local database has 0 jobs AND user clicks manual "Sync Now". This handles the "fresh install" or "deleted local DB" scenario without impacting normal sync performance.
- **User experience**: Normal sync updates status of known jobs. Discovery rebuilds the job list from server metadata only when needed (empty database).

### Why Centralize Mock Data in jobs.ts Instead of Per-Component
- **Reasoning**: Single source of truth ensures consistency. Demo mode should show same data across all views.
- **Current state**: mockJobs array already exists and is used by sync; JobTabs just needs to reference it.
- **Benefit**: Changes to demo data automatically reflect everywhere.

### Why Move Validation and Cluster Config to Backend
- **Reasoning**: Business logic belongs in backend where it can be properly tested, enforced, and maintained. Frontend validation can be bypassed by users or bugs. Having validation in two places creates duplication and drift.
- **Current problem**: 60 lines of validation in TypeScript, only 3 lines in Rust. Frontend has 368 lines of cluster config data. Backend can't enforce rules it doesn't know about.
- **Solution**: Single source of truth in Rust. Frontend becomes thin UI layer that displays errors and fetches capabilities. Validation and cluster config use the same source, ensuring consistency.
- **Benefit**: Impossible to bypass validation; single place to update rules; frontend can't submit invalid jobs; cluster config changes don't require frontend recompile.

### Why Delete Tests That Only Test Dead Code
- **Reasoning**: Tests should validate production code paths - the code that users actually execute. If only a test uses a method/function, that method is dead code disguised as "tested code."
- **The problem**: Test-only methods create false confidence - we're testing code that never runs in production. This is maintenance burden without value.
- **Example**: If `addJob()` is only called by `addJob.test.ts`, both are useless. Users never call `addJob()` - they use the real `createJob()` which calls the backend.
- **Principle**: "Being used by a test" is NOT the same as "being used by the app." If production code doesn't use it, delete it AND its tests.
- **Exception**: Shared test utilities (mocks, fixtures, helpers) that support testing production code are fine. But methods that exist ONLY to be tested are dead code.

## Integration with Existing Code

### Leverage Existing Patterns
- **Use logging infrastructure**: `info_log!`, `debug_log!`, `error_log!` from [src-tauri/src/logging.rs](../../src-tauri/src/logging.rs) - proven working in connection.rs
- **Follow automation patterns**: Progress callbacks, event emission, error handling from existing job_creation.rs automation
- **Apply mode switching**: `CoreClientFactory.getUserMode()` pattern from SyncControls.svelte
- **Use CoreClient abstraction**: All backend calls go through `CoreClientFactory.getClient()` for proper IPC boundary

### Where to Hook In
```typescript
// Frontend: CreateJobPage.svelte handleSubmit()
// REMOVE: setTimeout mock, newJob creation
// ADD: await jobsStore.createJob(params) with proper error handling

// Frontend: JobDetailPage.svelte handleDeleteJob()
// CHANGE: jobsStore.removeJob() → await jobsStore.deleteJob(jobId, deleteRemote)

// Frontend: JobTabs.svelte data fetching
// ADD: mode checking and conditional data source (mockJobs vs backend)
```

```rust
// Backend: src-tauri/src/cluster_config.rs (NEW MODULE)
// DEFINE: Cluster configuration as constants or config file
// EXPORT: get_cluster_capabilities() for frontend IPC

// Backend: src-tauri/src/validation/job_validation.rs
// EXPAND: Add comprehensive validation using cluster config
// USE: Same cluster config source as capabilities endpoint

// Backend: src-tauri/src/ssh/manager.rs
// ADD to existing functions: info_log! statements for operations

// Backend: src-tauri/src/commands/jobs.rs
// ADD: discover_jobs_from_server() following pattern of sync_all_jobs()
// MODIFY: create_job to return updated job list (eliminate double call)

// Backend: src-tauri/src/automations/*.rs
// ADD: info_log! for SSH operations within automation chains
```

## References
- **NAMDRunner patterns**:
  - [docs/CONTRIBUTING.md#testing-strategy](../../docs/CONTRIBUTING.md) - Testing philosophy
  - [docs/AUTOMATIONS.md](../../docs/AUTOMATIONS.md) - Complete automation architecture
  - [docs/API.md](../../docs/API.md) - IPC interface specifications
- **Implementation files**:
  - Frontend: CreateJobPage.svelte, JobDetailPage.svelte, JobTabs.svelte, jobs.ts store
  - Backend: commands/jobs.rs, automations/*.rs, ssh/manager.rs, ssh/sftp.rs
- **Specific docs**:
  - Read AUTOMATIONS.md section 1 (Job Creation) for understanding the automation chain
  - Read CONTRIBUTING.md testing section before writing tests
  - Check API.md for correct parameter types and result structures

## Progress Log
2025-09-30 - Task created based on comprehensive investigation of frontend-backend integration gaps
2025-09-30 - Added business logic separation work: move validation and cluster config from TypeScript (428 lines) to Rust backend; ensure validation and capabilities use same config source; eliminate double backend calls
2025-09-30 - Review-refactor agent investigation completed: Found 13 additional issues including ~700 lines of orphaned service layer, calculation functions in frontend utils, console.log hijacking hack, unused Tauri commands, and additional dead store methods. Added 5 new tasks to Medium Priority section.
2025-09-30 - Added Low Priority section with 3 polish tasks (file type classification, status display logic, broken method cleanup)
2025-09-30 - Added verification checkpoints before all code deletion tasks to confirm code is truly unused and not needed for Phase 6.4 implementation
2025-09-30 - Added comprehensive documentation update checklist in Completion Process covering ARCHITECTURE.md, API.md, AUTOMATIONS.md, CONTRIBUTING.md, DB.md, DESIGN.md with requirement that docs must match final code state
2025-09-30 - Updated objective and context to emphasize **true frontend-backend separation** with frontend as thin UI layer and backend containing all business rules
2025-09-30 - Clarified verification principle: "being used by a test" is NOT the same as "being used by the app" - if only tests use code, delete both the code AND the tests; added new Key Technical Decision explaining this principle with examples
2025-09-30 - Added CoreClient interface simplification to Low Priority: consolidate 3 files (coreClient.ts, coreClient-tauri.ts, coreClient-mock.ts) into 2 files by moving ICoreClient interface into primary implementation; delete 3 unused sub-interfaces (IConnectionCommands, IJobCommands, IFileCommands); rename coreClient-tauri.ts → coreClient.ts for clarity
2025-09-30 - Answered all open questions and integrated decisions into plan tasks
2025-09-30 - Added confirmation/warning dialogs to improve user experience and prevent accidental actions:
  - Create Job: Informational dialog explaining files will upload but job won't submit to SLURM yet
  - Delete Job: Warning dialog with detailed explanation of permanent deletion (local DB + all server files)
2025-09-30 - **Clarity improvements to eliminate ambiguity**:
  - Simplified cluster config storage directive (removed redundant "Storage decision" wording)
  - Clarified job discovery trigger logic (expanded explanation of when/why it runs)
  - Specified recommended approach for double backend call fix (try returning job object first, fall back to events if complex)
  - Updated Key Technical Decision section to match implementation (removed "Deep Sync" mention, focused on 0-jobs trigger)
  - Condensed Progress Log to remove redundant details already in plan tasks
  - Templatized verification checkpoints to reference single standard process (reduced repetition)

## Completion Process
After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] **Update ALL documentation to match code changes**:
  - [ ] Update [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md):
    - [ ] Document SSH/SFTP console logging implementation
    - [ ] Document job discovery architecture and workflow
    - [ ] Document cluster config location (Rust backend, not frontend)
    - [ ] Document validation architecture (all in backend)
    - [ ] Document frontend-backend separation (thin UI layer vs business logic layer)
    - [ ] Remove references to deleted service layer
    - [ ] Update module structure diagrams if services are shown
  - [ ] Update [docs/API.md](../../docs/API.md):
    - [ ] Add `get_cluster_capabilities()` IPC command documentation
    - [ ] Add job discovery/deep sync command documentation
    - [ ] Document validation error response format
    - [ ] Document cost estimation API if added
    - [ ] Remove documentation for deleted Tauri commands
    - [ ] Update CreateJobParams with validation expectations
  - [ ] Update [docs/AUTOMATIONS.md](../../docs/AUTOMATIONS.md):
    - [ ] Document job discovery automation if it changes sync behavior
    - [ ] Document "Sync Results from Scratch" workflow
    - [ ] Add SSH logging to automation chain descriptions
  - [ ] Update [docs/CONTRIBUTING.md](../../docs/CONTRIBUTING.md):
    - [ ] Update testing examples if validation moved to backend
    - [ ] Document that business logic belongs in Rust backend
    - [ ] Document frontend is UI-only layer
    - [ ] Update file structure if services deleted
  - [ ] Review [docs/DB.md](../../docs/DB.md):
    - [ ] Verify schema documentation matches any DB changes for job discovery
  - [ ] Review [docs/DESIGN.md](../../docs/DESIGN.md):
    - [ ] Update if UI components changed significantly
  - [ ] **Verify all code examples in docs use correct patterns** (no references to deleted code)
- [ ] Update and archive task to `tasks/completed/phase-6-4-frontend-backend-integration.md`
- [ ] Update `tasks/roadmap.md` - mark 6.4 complete, update 6.5 (comprehensive testing) with new scope

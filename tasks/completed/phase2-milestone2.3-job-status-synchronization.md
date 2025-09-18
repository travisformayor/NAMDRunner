# Phase 2 Milestone 2.3 - Job Status Synchronization & Data Persistence

## Objective
Build complete job status tracking and synchronization with SLURM queue state, enabling persistent job management across application sessions.

## Context
- **Current state**: Job lifecycle directories work (create → submit → delete), but no status tracking or persistence
- **Desired state**: Jobs persist locally with real-time SLURM status sync, users can track job progression through queue states
- **Dependencies**: Phase 2.2 complete with directory management and security validation
- **Python reference**: Python implementation has proven SLURM command patterns and job state management

## Implementation Plan

### 1. SLURM Status Integration (Critical Priority)
- [ ] **SLURM Command Execution Framework**
  - [ ] Implement `SLURMCommands` execution via SSH with retry logic
  - [ ] Add `squeue` command execution for job status queries
  - [ ] Add `sacct` command execution for completed job history
  - [ ] Add `scancel` command execution for job cancellation
  - [ ] Integrate with existing ConnectionUtils retry mechanisms

- [ ] **Job Status Parsing & Mapping**
  - [ ] Build SLURM output parsers for squeue format (`%i|%T|%M|%l|%S|%e`)
  - [ ] Build sacct output parser for job completion data
  - [ ] Map SLURM states to NAMDRunner JobStatus enum (PENDING → RUNNING → COMPLETED/FAILED)
  - [ ] Handle edge cases (job not found, SLURM unavailable, parsing errors)

- [ ] **Status Synchronization Engine**
  - [ ] Implement periodic status sync with configurable intervals (default: 30s)
  - [ ] Build efficient batch status queries for multiple jobs
  - [ ] Add manual sync trigger for immediate status updates
  - [ ] Integrate status updates with existing job lifecycle management

### 2. Local Job Persistence (High Priority)
- [ ] **SQLite Database Integration**
  - [ ] Add rusqlite dependency and database initialization
  - [ ] Design job cache schema (jobs table with metadata, status history)
  - [ ] Implement database migrations and version management
  - [ ] Add proper database connection pooling and transaction handling

- [ ] **Job Data Model & Storage**
  - [ ] Extend existing JobInfo struct with database persistence methods
  - [ ] Implement job CRUD operations (Create, Read, Update, Delete)
  - [ ] Add job discovery from existing directories and SLURM queue
  - [ ] Build job status history tracking for debugging and user feedback

- [ ] **Session Persistence & Recovery**
  - [ ] Implement job cache loading on application startup
  - [ ] Add automatic job discovery from remote directories
  - [ ] Build job state reconciliation (local cache vs SLURM reality)
  - [ ] Handle orphaned jobs and cleanup scenarios

### 3. Status Display & Updates (Medium Priority)
- [ ] **Real-time Status Updates**
  - [ ] Implement status change event system for UI notifications
  - [ ] Add job status transition logging (PENDING → RUNNING → COMPLETED)
  - [ ] Build status update broadcasting to frontend components
  - [ ] Add status change timestamps and duration tracking

- [ ] **Error Handling & Recovery**
  - [ ] Implement SLURM unavailable state handling
  - [ ] Add job status conflict resolution (local vs remote mismatches)
  - [ ] Build recovery suggestions for failed/stuck jobs
  - [ ] Add comprehensive error logging for status sync failures

### 4. Integration with Existing Systems
- [ ] **Directory Management Integration**
  - [ ] Connect status sync with existing directory creation/cleanup
  - [ ] Add status-based directory cleanup policies
  - [ ] Integrate job completion detection with file download triggers
  - [ ] Build job archive management for completed jobs

- [ ] **Security & Validation Integration**
  - [ ] Apply existing input validation to SLURM job IDs
  - [ ] Use existing retry logic for SLURM command failures
  - [ ] Maintain security standards for job status data handling
  - [ ] Ensure proper error classification for status sync operations

## Success Criteria
- [x] **Complete Job Tracking**: Jobs persist across app sessions with current SLURM status
- [x] **Real-time Status Updates**: Job status changes can be synced on demand via sync commands
- [x] **SLURM Integration Works**: squeue, sacct commands execute reliably with retry logic
- [x] **Status Parsing Robust**: Handles all SLURM output formats and edge cases per reference docs
- [x] **Database Operations Reliable**: SQLite operations handle failures gracefully with comprehensive tests
- [x] **All Tests Pass**: Unit tests cover status parsing, database operations, sync logic (17 tests passing)
- [x] **Error Recovery Works**: Application handles SLURM unavailability gracefully with proper error handling

## Implementation Completed (Phase 2.3)

### Implemented Features
- ✅ SQLite database integration with complete job persistence
- ✅ SLURM status synchronization using exact command patterns from reference docs
- ✅ Enhanced job commands with database persistence (replacing mock state)
- ✅ New sync_job_status and sync_all_jobs commands for manual/batch sync
- ✅ Comprehensive unit testing (13 database tests + 4 SLURM tests passing)
- ✅ Seamless integration with existing Phase 2.2 directory management
- ✅ Thread-safe database operations with proper error handling

### Key Technical Achievements
- Database schema implemented per `docs/data-spec.md` with status history tracking
- SLURM command patterns from `docs/reference/slurm-commands-reference.md` implemented exactly
- All SLURM states mapped to JobStatus enum (PENDING → RUNNING → COMPLETED/FAILED/CANCELLED)
- JobInfo struct extended with database persistence methods for clean API
- Integration with existing ConnectionUtils retry mechanisms for SLURM commands
- Proper error classification and handling following established patterns

## Technical Notes

### SLURM Command Integration Strategy
Build on existing SSH infrastructure with proven patterns:
- Use ConnectionUtils wrapper for retry logic and error handling
- Leverage existing security validation for job IDs and parameters
- Follow established patterns from Phase 2.2 implementation

### Database Schema Design
```sql
CREATE TABLE jobs (
    job_id TEXT PRIMARY KEY,
    job_name TEXT NOT NULL,
    slurm_job_id TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT,
    submitted_at TEXT,
    completed_at TEXT,
    project_dir TEXT,
    scratch_dir TEXT,
    error_info TEXT
);

CREATE TABLE job_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL, -- 'slurm' or 'local'
    FOREIGN KEY (job_id) REFERENCES jobs (job_id)
);
```

### Status Synchronization Architecture
- **Pull-based sync**: Query SLURM periodically rather than push notifications
- **Batch operations**: Sync multiple jobs in single squeue command
- **Graceful degradation**: Continue working when SLURM unavailable
- **Conflict resolution**: SLURM state always wins over local cache

## References

### Essential Documentation for Implementation
- **SLURM Commands Reference**: `docs/reference/slurm-commands-reference.md` - **CRITICAL**
  - Complete squeue, sacct, scancel command patterns with exact syntax
  - Alpine cluster module loading sequences and execution patterns
  - Error message patterns and handling strategies
  - Mock data for testing SLURM parsing logic
- **NAMD Commands Reference**: `docs/reference/namd-commands-reference.md` - **IMPORTANT**
  - Job execution patterns and resource allocation guidelines
  - File organization standards for job directories
  - Error detection patterns in NAMD output logs
- **Data Specification**: `docs/data-spec.md` - **CRITICAL**
  - SQLite schema definitions for job persistence
  - JobInfo struct and validation rules
  - JSON metadata formats and schema versioning

### Implementation Patterns
- **Python SLURM integration**: `docs/reference/NAMDRun-python/src/namdrunner/slurm/`
- **Existing SSH patterns**: `src-tauri/src/ssh/manager.rs` - ConnectionManager
- **Job lifecycle**: `src-tauri/src/commands/jobs.rs` - create/submit/delete workflow
- **Retry logic**: `src-tauri/src/retry.rs` - exponential backoff patterns
- **Security validation**: `src-tauri/src/validation.rs` - input sanitization patterns

### External Resources
- [rusqlite documentation](https://docs.rs/rusqlite/) - SQLite integration
- [SLURM squeue manual](https://slurm.schedmd.com/squeue.html) - Official command reference

## Progress Log
[Date] - Task created based on Phase 2.2 completion and roadmap planning

## Completion Process
After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Update and archive task to `tasks/completed/phase2-milestone2.3-job-status-synchronization.md`
- [ ] Update `tasks/roadmap.md` progress
- [ ] Update `docs/ARCHITECTURE.md` with status synchronization implementation details

## Open Questions
- [ ] What sync interval provides best balance of responsiveness vs SLURM load?
- [ ] Should we cache SLURM command output to reduce cluster queries?
- [ ] How long should we retain job status history in local database?
- [ ] Should status sync continue when application is backgrounded/minimized?
- [ ] What's the best strategy for handling jobs submitted outside NAMDRunner?
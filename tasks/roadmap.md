# NAMDRunner Development Roadmap

**🏗️ Architecture Updates**: When completing milestones, always update `docs/ARCHITECTURE.md` to reflect the actual implementation. Architecture doc describes what IS built, this roadmap describes what WILL be built.

## Development Strategy
Build a **single-job MVP first** that handles the core workflow: create job → submit → track → view results.

**Key Design Decision**: Job persistence and discovery are built from Phase 2, not added later. This ensures:
- Test jobs persist between development sessions for proper testing
- No accumulation of "phantom" test data on cluster
- Status sync works from first job submission
- Developers can easily manage and clean up test jobs

**Breaking Changes Policy**: Breaking changes are acceptable and expected during all development phases. Each phase can modify, improve, or completely rewrite previous implementations. No backwards compatibility is required with:
- Previous development phases
- Earlier iterations of schemas or interfaces
- Test data or mock implementations

## Current Status: Phase 5 Complete ✅

**Next Priority**: Phase 6 - Single-Job MVP Completion (testing, polish, and production readiness)

**Current Implementation**: See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for detailed description of what exists now, including module structure, SSH/SFTP integration, and security implementation.

## Phase 1: Foundation ✅ COMPLETED
*Critical path to first working prototype*

### Milestone 1.1: Project Scaffold ✅ COMPLETED
- [x] **Tauri v2 + Svelte Setup** - Initialize project with TypeScript and component structure
- [x] **IPC Boundary Interfaces** - Implement TypeScript/Rust command interfaces
- [x] **JSON Metadata Schema** - Define data structures and validation
- [x] **Rust Module Architecture** - Establish clean separation of concerns

See: [phase1-milestone1.1-foundation.md](tasks/completed/phase1-milestone1.1-foundation.md)

### Milestone 1.2: Mock Infrastructure ✅ COMPLETED
- [x] **Mock IPC Client** - Enable UI development without backend dependency
- [x] **Testing Infrastructure** - WebdriverIO E2E testing with tauri-driver
- [x] **CI Configuration** - Linux and Windows build automation
- [x] **Agent Debug Toolkit** - Development and testing utilities

See: [phase1-milestone1.2-mock-infrastructure.md](tasks/completed/phase1-milestone1.2-mock-infrastructure.md)

### Milestone 1.3: Connection Foundation ✅ COMPLETED
- [x] **SSH/SFTP Interface Design** - Connection state management and error handling
- [x] **Remote Directory Structure** - Define `/projects/$USER/namdrunner_jobs/` patterns
- [x] **Connection Validation** - Testing utilities and connection lifecycle

See: [phase1-milestone1.3-connection-foundation.md](tasks/completed/phase1-milestone1.3-connection-foundation.md)

### Milestone 1.4: Phase 1 Cleanup & Refactoring ✅ COMPLETED
- [x] **Code Review & Refactoring** - Eliminate duplication and ensure architectural consistency
- [x] **Dependency Injection** - Centralize service management and path handling
- [x] **Error Handling Standardization** - Consistent Result<T> patterns throughout

## Phase 2: Core Backend ✅ COMPLETED
*SSH connection and data management*

### Milestone 2.1: SSH/SFTP Implementation ✅ COMPLETED
- [x] **SSH Authentication** - Password-based authentication with ssh2 crate
- [x] **SFTP Operations** - File upload/download and directory management
- [x] **Module Loading** - SLURM environment setup commands
- [x] **Connection Management** - Secure credential handling and error recovery
- [x] **Testing** - 43 unit tests covering business logic without network dependencies

See: [phase2-milestone2.1-ssh-sftp-implementation.md](tasks/completed/phase2-milestone2.1-ssh-sftp-implementation.md)

### Milestone 2.2: SSH/SFTP Critical Fixes & Enhancements ✅ COMPLETED
- [x] **Job Directory Lifecycle** - Project and scratch directory management with validation
- [x] **Retry Logic** - Exponential backoff with configurable limits and error classification
- [x] **SLURM Integration** - Enhanced command parsing and job ID validation
- [x] **Security Implementation** - Defense-in-depth validation and path safety
- [x] **Test Quality** - 116 tests with comprehensive security validation

See: [phase2-milestone2.2-ssh-sftp-critical-fixes.md](tasks/completed/phase2-milestone2.2-ssh-sftp-critical-fixes.md)

### Milestone 2.3: Job Status Synchronization & Data Persistence ✅ COMPLETED
- [x] **SLURM Status Integration** - Complete job status tracking with state transitions
- [x] **Local Job Persistence** - SQLite integration with session continuity
- [x] **Status Management** - Manual sync commands with database consistency

See: [phase2-milestone2.3-job-status-synchronization.md](tasks/completed/phase2-milestone2.3-job-status-synchronization.md)

### Milestone 2.4: Phase 2 Cleanup & Refactoring ✅ COMPLETED
- [x] **Code Review** - Eliminated thin wrappers and duplicate business logic
- [x] **Validation Simplification** - Consolidated patterns and removed over-engineering

## Phase 3: Frontend Development ✅ COMPLETED
*User interface implementation based on React mockup*

### Milestone 3.1: Design System & Layout Components ✅ COMPLETED
- [x] **Application Shell** - Main layout with sidebar, header, and content areas
- [x] **Navigation System** - Breadcrumbs and state management
- [x] **SSH Console Panel** - Collapsible debugging interface

### Milestone 3.2: Jobs Management Interface ✅ COMPLETED
- [x] **Jobs List Page** - Sortable table with status indicators
- [x] **Job Detail View** - Tabbed interface with sync controls
- [x] **Interactive Elements** - Selection and row interactions

### Milestone 3.3: Job Creation Workflow ✅ COMPLETED
- [x] **Multi-Section Form** - SLURM resource allocation and NAMD configuration
- [x] **File Upload** - Drag & drop interface with validation
- [x] **Form Validation** - Error display and user feedback

### Milestone 3.4: Connection UI & Polish ✅ COMPLETED
- [x] **Connection Interface** - Enhanced dropdown matching mockup design
- [x] **Theme Support** - Dark theme and loading states
- [x] **UI Testing** - Complete testing suite

### Milestone 3.5: Phase 3 Cleanup & Refactoring ✅ COMPLETED
- [x] **Code Review** - Component consistency and reusability improvements
- [x] **Design System** - Established patterns and accessible design

See: [phase3-ui-implementation.md](tasks/completed/phase3-ui-implementation.md)

## Phase 4: SLURM Integration ✅ COMPLETED
*Cluster job management*

### Milestone 4.1: Job Submission ✅ COMPLETED
- [x] **SLURM Script Generation** - Template-based NAMD job script creation
- [x] **Real Job Submission** - Direct sbatch integration with job ID parsing
- [x] **Error Handling** - Comprehensive retry logic and user-friendly messages

### Milestone 4.2: Status Tracking & Sync ✅ COMPLETED
- [x] **SLURM Status Integration** - squeue/sacct commands with state transitions
- [x] **Database Persistence** - Jobs persist across app restarts with sync commands
- [x] **Lifecycle Management** - Complete job state tracking from submission to completion

### Milestone 4.3: Phase 4 Cleanup & Refactoring ✅ COMPLETED
- [x] **Pattern Consistency** - Built on existing SSH infrastructure without duplication
- [x] **Error Integration** - Enhanced retry logic with SLURM-specific error mapping

See: [phase4-slurm-job-submission.md](tasks/completed/phase4-slurm-job-submission.md)

## Phase 5: File Operations & Results Management ✅ COMPLETED
*Complete backend file operations for end-to-end workflow*

### Milestone 5.1: Real File Upload Implementation ✅ COMPLETED
- [x] **SFTP File Upload** - Convert from mock to real operations with progress tracking
- [x] **File Validation** - Input validation for PDB, PSF, and parameter files
- [x] **Upload Management** - Project directory storage with integrity checks

### Milestone 5.2: Real File Download & Results Management ✅ COMPLETED
- [x] **SFTP File Download** - Real operations for SLURM and NAMD output files
- [x] **Directory Listing** - Results browsing via SFTP
- [x] **Log Aggregation** - Unified access to SLURM and NAMD logs

### Milestone 5.3: Job Cleanup & Lifecycle Completion ✅ COMPLETED
- [x] **Remote Directory Cleanup** - Safe deletion of project and scratch directories
- [x] **Error Handling** - Network interruption recovery with retry logic

### Milestone 5.4: Code Quality & Architecture Improvements ✅ COMPLETED
- [x] **Code Review** - Eliminated thin wrappers and intermediate business logic
- [x] **Code Reduction** - Achieved ~20% reduction while improving readability

See: [phase5-file-operations-results-management.md](tasks/completed/phase5-file-operations-results-management.md)

## Phase 6: Single-Job MVP Completion
*Testing, polish, and production readiness for core single-job functionality*

### Milestone 6.1: UI Integration & Connection Stability ✅ COMPLETED
- [x] **IPC Boundary Integration** - Fixed command signatures and type alignment between frontend and backend
- [x] **Demo Mode Toggle** - Implemented persistent demo/real mode toggle in connection dropdown
- [x] **SSH Connection Stability** - Enhanced connection debugging, error handling, and user feedback
- [x] **UI-Backend Wiring** - Replaced mock IPC client with real backend integration
- [x] **Connection Management** - Stabilized SSH console logging and session management

See: [phase-6-1-ui-backend-integration.md](tasks/completed/phase-6-1-ui-backend-integration.md)

### Milestone 6.2: Job Automation Implementation & Verification ✅ COMPLETED
- [x] **Job Creation Automation** - Verified proper workflow separation (project directories only)
- [x] **Job Submission Automation** - Verified existing implementation with scratch directory handling
- [x] **Job Completion Automation** - Implemented results preservation from scratch to project directories
- [x] **Status Synchronization** - Verified SLURM integration and database updates work correctly
- [x] **Job Cleanup Security** - Verified comprehensive path validation and safe directory deletion
- [x] **Complete Job Lifecycle** - End-to-end automation chain working with progress tracking

See: [phase-6-2-automation-verification.md](tasks/completed/phase-6-2-automation-verification.md)

### Milestone 6.3: Code Quality & Refactoring ✅ COMPLETED
- [x] Run comprehensive code review using `.claude/agents/review-refactor.md` agent
- [x] **Critical Priority Refactoring** (implement before testing):
  - [x] Eliminate thin wrapper anti-patterns (job_cleanup.rs)
  - [x] Remove hardcoded fallback logic and false compatibility layers
  - [x] Centralize file validation to eliminate duplication
  - [x] Standardize shell command construction for security consistency
  - [x] Add database transaction safety for data consistency
- [x] **Final refactoring pass for consistency and maintainability**
- [x] **Complete security review and hardening recommendations**
- [x] **Implement performance optimization opportunities identified**
- [x] **Comprehensive Security Hardening**:
  - [x] Command injection vulnerability elimination (replaced shell commands with SFTP uploads)
  - [x] Input validation enhancement for configuration management
  - [x] Mode switching simplification (eliminated complex mutex patterns)
  - [x] Security test suite expansion (17 comprehensive tests following testing guidelines)
  - [x] Documentation consolidation and accuracy verification

See: [phase-6-3-code-quality-refactoring.md](tasks/completed/phase-6-3-code-quality-refactoring.md)

### Milestone 6.4: Frontend-Backend Integration ✅ COMPLETED
- [x] **Backend core systems:** cluster.rs, automations/, validation/, logging bridge
- [x] **Frontend service layer removal:** Deleted orphaned/redundant services, tests, fixtures, duplicate logic (handled by backend only)
- [x] **Stores architecture:** Reactive stores consuming backend APIs (clusterConfig, jobs)
- [x] **Job automation chains:** Creation, submission, completion, sync, cleanup with comprehensive logging
- [x] **Type safety:** Snake_case consistency, strict contracts
- [x] **File upload reliability:** Chunked uploads (256KB), per-chunk flush, 300s timeout per chunk
- [x] **Architecture cleanup:** Batch SLURM queries, delete_job cancels SLURM jobs

See: [phase-6-4-frontend-backend-integration.md](tasks/completed/phase-6-4-frontend-backend-integration.md)

### Milestone 6.5: Code Quality & Infrastructure Cleanup ✅ COMPLETED
- [x] **SLURM log caching implementation** - Complete end-to-end feature from database to UI
  - [x] Database schema: slurm_stdout/slurm_stderr columns, save/load methods
  - [x] Backend fetching: fetch_slurm_logs_if_needed() with three trigger points
  - [x] Frontend display: JobTabs cached logs, manual fetch button
  - [x] Status validation: Extend to FAILED/CANCELLED states
- [x] **Database infrastructure simplification** - Remove unused transaction and status history code
- [x] **Mock UI element removal** - Clean up fake progress bars and placeholder UI
- [x] **Miscellaneous improvements** - SLURM status codes, module init fixes, closure ownership

See: [phase-6-5-code-quality-infrastructure-cleanup.md](tasks/completed/phase-6-5-code-quality-infrastructure-cleanup.md)

### Milestone 6.6: Job Lifecycle Reliability & Bug Fixes ✅ COMPLETED
- [x] **Issue 0: Automatic scratch→project rsync** - ARCHITECTURE BUG: Job completion doesn't automatically rsync scratch to project, logs fetch from wrong directory
- [x] **Issue 1: Server metadata sync** - job_info.json not updating on server after status changes
- [x] **Issue 2: Failed job file copying** - Terminal state rsync handles this (fixed by Issue 0)
- [x] **Issue 3a: SLURM memory unit** - Append "GB" to memory parameter (--mem=64GB not --mem=64)
- [x] **Issue 3b: NAMD config file names** - Use actual uploaded file names instead of hardcoded structure.psf/pdb
- [x] **Issue 4: OpenMPI environment export** - Add SLURM_EXPORT_ENV=ALL before mpirun
- [x] **Issue 5: Explicit nodes flag** - Calculate and specify --nodes based on core count for optimal MPI performance

See: [phase-6-6-job-lifecycle-reliability-bug-fixes.md](tasks/completed/phase-6-6-job-lifecycle-reliability-bug-fixes.md)

### Milestone 6.7: Template Type 2 NAMD Configuration Support ✅ COMPLETED
- [x] **CRITICAL: Missing cellBasisVector** - NAMD config never outputs cellBasisVector, causing "PME requires periodic boundaries" error on ALL PME jobs
- [x] **CRITICAL: Missing execution_mode** - Cannot run minimization stage (always generates "run", never "minimize")
- [x] **HIGH: Output frequency bug** - Uses dcd_freq for all outputs instead of separate values (xstFreq, outputEnergies, outputPressure wrong)
- [x] **HIGH: Extrabonds file support** - Add .exb/.enm.extra file type detection and config generation for DNA restraints
- [x] **MEDIUM: Make PME/NPT configurable** - Currently hardcoded to "on", need checkboxes for vacuum simulations and NVT ensemble
- [x] **MEDIUM: Configurable advanced parameters** - langevinDamping, margin, fullElectFrequency currently hardcoded

**Goal**: Enable users to run DNA origami tutorial workflows (explicit solvent equilibration with restraints) on cluster

See: [phase-6-7-template-type-2-namd-config-fixes.md](tasks/completed/phase-6-7-template-type-2-namd-config-fixes.md)


### Milestone 6.8: Pragmatic Testing ✅ COMPLETED
- [x] **Dead Code Removal** - Removed 151 lines from demo/state.rs, slurm/commands.rs, security.rs
- [x] **Mock Data Centralization** - Created src/lib/test/fixtures/mockJobData.ts, eliminated 307 lines of duplication
- [x] **Test Anti-Pattern Fixes** - Found and fixed getter/setter tests in automations/progress.rs
- [x] **Test Coverage Audit** - Reviewed all 188 Rust tests, confirmed comprehensive business logic coverage
- [x] **Quality Verification** - 188 tests passing in 3.15s, zero anti-patterns, follows NAMDRunner testing philosophy
- **Result**: Clean test suite with 458 lines of technical debt eliminated (151 dead code + 307 duplicate mock data)

See: [phase-6-8-pragmatic-testing.md](tasks/completed/phase-6-8-pragmatic-testing.md)

### Milestone 6.9: Production Readiness
- [ ] Git Action x86 Windows executable build
- [ ] Git Action x86 Linux executable build
- [ ] Installation documentation
- [ ] User guide (single-job workflow)
- [ ] Deployment pipeline
- [ ] Final documentation completeness check and cleanup

## Success Metrics

### Phase 1 Complete When: ✅ ACHIEVED
- [x] Tauri app launches with basic UI
- [x] IPC boundary interfaces defined and documented
- [x] JSON metadata schema specified
- [x] Mock SLURM integration working for offline dev
- [x] E2E test takes screenshot 
- [x] CI builds Windows exe
- [x] SSH/SFTP interface patterns defined
- [x] Connection state management architecture established
- [x] Remote directory management foundations implemented

### Phase 2 Complete When: ✅ ACHIEVED
- [x] SSH connection works with password
- [x] Files upload/download via SFTP
- [x] **Job directory lifecycle works correctly** (create → submit → delete)
- [x] **Retry logic handles network interruptions gracefully**
- [x] **Path security prevents directory traversal attacks**
- [x] **File operations are optimized** (avoid redundant uploads)
- [x] SQLite stores and retrieves job data
- [x] App reopening shows previously created jobs (jobs persist in database)
- [x] Job status tracking with database persistence implemented
- [x] SLURM status sync functional with manual sync commands

### Phase 3 Complete When: ✅ ALL ACHIEVED
- [x] UI visually matches React mockup screenshots
- [x] Full navigation between Jobs, Job Detail, and Create Job views works
- [x] All forms validate input with proper error display
- [x] Light/dark themes both functional
- [x] Mock data enables complete UI workflow testing
- [x] UI tests capture screenshots for visual validation
- [x] **BONUS**: Comprehensive refactoring cleanup completed (300+ lines CSS eliminated, utilities centralized)

### Phase 4 Complete When: ✅ ACHIEVED
- [x] Jobs submit to SLURM
- [x] Status updates correctly
- [x] Cache syncs with cluster
- [x] Errors handled gracefully

### Phase 5 Complete When: ✅ ACHIEVED
- [x] Real file upload/download operations working via SFTP
- [x] Can upload input files and download result files
- [x] Directory listing and file browsing backend functional
- [x] Log file aggregation working (SLURM + NAMD logs accessible)
- [x] Job deletion with remote cleanup working
- [x] All file operations integrate with existing retry/error handling
- [x] Code quality significantly improved (~20% reduction, eliminated antipatterns)
- [x] **Backend file operations complete for end-to-end workflow**

### Phase 6 Complete When (Single-Job MVP):
- **Milestone 6.1**: ✅ UI integrated into backend features with automation architecture foundation
- **Milestone 6.2**: ✅ All automation chains verified and working correctly (creation, submission, status sync, completion, cleanup)
- **Milestone 6.3**: ✅ Code quality improvements and refactoring complete (clean, maintainable codebase)
- **Milestone 6.4**: ✅ Frontend-backend integration complete (stores architecture, backend automation chains, removed old code)
- **Milestone 6.5**: ✅ Code quality and infrastructure cleanup (SLURM log caching, database simplification, UI polish)
- **Milestone 6.6**: ✅ Job lifecycle reliability fixes (server metadata sync, failed job handling, SLURM/NAMD config bugs)
- **Milestone 6.7**: ✅ Template Type 2 NAMD config support (cellBasisVector, execution_mode, extrabonds, configurable physics)
- **Milestone 6.8**: ✅ Pragmatic testing complete (188 tests, zero anti-patterns, 458 lines technical debt eliminated)
- **Milestone 6.9**: Production-ready deployment with x86 Windows/Linux builds and documentation
- **Single-job MVP ready for users to run DNA origami tutorial workflows on cluster**

## Phase 7: Template System & Production Hardening

### Milestone 7.1: Template System Refactor ⏳ IN PROGRESS

**Goal**: Replace hardcoded NAMD configuration with flexible template system where templates are stored in database and users can create/edit simulation templates via UI.

**Implementation**:
- [x] **Database Foundation**: Templates table, modified jobs table (template_id + template_values)
- [x] **Data Structures**: Template, VariableDefinition, VariableType (Number, Text, Boolean, FileUpload)
- [x] **Template Renderer**: Regex-based variable substitution for `{{variable}}` patterns
- [x] **JobInfo Refactor**: Removed NAMDConfig struct entirely, removed demo mode entirely
- [x] **Default Templates**: vacuum_optimization_v1.json, explicit_solvent_npt_v1.json (embedded in binary)
- [x] **Template IPC Commands**: list, get, create, update, delete, validate, preview
- [x] **Templates Page UI**: Unified template list with built-in badges, delete functionality
- [x] **Template Editor UI**: Full-page editor with auto-variable detection, variable metadata editor
- [x] **Dynamic Job Form**: 3-tab interface (Resources, Configure, Review) with dynamic form from template
- [ ] **End-to-End Verification**: User testing of complete job lifecycle with templates

**Why**: Hardcoded NAMDConfig prevents supporting different simulation types without code changes. Template-as-data enables runtime modification and user extensibility.

**No Backwards Compatibility**: All existing jobs are test data - will delete old database before running new app. No migration code.

See: [phase-7-1-template-system-refactor.md](tasks/active/phase-7-1-template-system-refactor.md)

### Milestone 7.2: Settings Page with Database Management

**Goal**: Fix AppImage database path bug and add Settings page with database management (backup, restore, reset)

**Current Problem**:
- Production builds use wrong database path (`./namdrunner.db`)
- AppImage completely broken (tries to create DB in read-only mount)
- RPM/DEB work by accident (CWD resolution, not robust)

**Implementation**:
- [ ] **Database Path Migration**: Move initialization to `.setup()` hook, use `app_data_dir()` API
  - [ ] `get_database_path()` - Returns OS-specific path (Linux: `~/.local/share/namdrunner/`, Windows: `%APPDATA%\namdrunner\`)
  - [ ] `reinitialize_database()` - Close and reopen connection (for restore/reset)
  - [ ] Development builds still use `./namdrunner_dev.db` (unchanged)
  - [ ] Ground-up refactor with zero tech debt

- [ ] **Database Management Commands** (`commands/database.rs`):
  - [ ] `get_database_info()` - Returns path and file size
  - [ ] `backup_database()` - SQLite Backup API for safe online backup
  - [ ] `restore_database()` - File dialog, validate, replace DB, reinitialize
  - [ ] `reset_database()` - Delete and recreate with fresh schema

- [ ] **Settings Page UI**:
  - [ ] New Settings page in sidebar navigation
  - [ ] Display database location and size
  - [ ] Backup button (opens save dialog)
  - [ ] Restore button (warning dialog → file dialog → replace)
  - [ ] Reset button (warning dialog → delete all data)
  - [ ] Reuses existing `ConfirmDialog` component

**Why**: AppImage is completely broken without this fix. Settings page provides user control over database management.

**No Migration Code**: App not released yet, user will reset database manually.

See: [phase-7-2-settings-page-database-management.md](tasks/active/phase-7-2-settings-page-database-management.md)

### Milestone 7.3: Request Rate Limiting & Queue Management

**Goal:** Prevent cluster abuse and provide graceful degradation under load

**Current State:**
- Mutex serialization provides implicit rate limiting (one request at a time)
- Single SSH connection physically prevents parallel spam
- No queue depth limits or time-based throttling
- Adequate for MVP testing, but needs hardening for production

**Implementation:**
- [ ] **Rate Limiter Module** (`src-tauri/src/ssh/rate_limiter.rs`)
  - [ ] Configurable requests per second limit (default: 5/sec)
  - [ ] Configurable max queue depth (default: 20 pending requests)
  - [ ] Time-based throttling with token bucket algorithm
  - [ ] Integrates with existing ConnectionManager mutex

- [ ] **Request Deduplication**
  - [ ] Debounce rapid duplicate requests (same command within 1s)
  - [ ] Coalesce multiple identical sync requests into single execution
  - [ ] Track in-flight request signatures to prevent duplicates

- [ ] **Queue Depth Protection**
  - [ ] Reject new requests when queue depth exceeded
  - [ ] Return descriptive error: "Too many pending requests, try again in a moment"
  - [ ] User-facing popup notification via Tauri event system
  - [ ] Log queue depth metrics for monitoring

**Architecture Pattern:**
```rust
// Wrap existing mutex with rate limiter
pub async fn execute_command(&self, command: &str, timeout: Option<u64>) -> Result<CommandResult> {
    self.rate_limiter.wait_for_slot().await?;     // NEW: Rate limit + queue check
    let conn = self.connection.lock().await;       // EXISTING: Mutex serialization
    // ... existing command execution
}
```

**Why:**
- Prevents accidental bugs from DOS'ing cluster, but we dont want that event to silently fail so report it in the SSH Console.

### Milestone 7.4: Job Chaining (Future)

**Note**: Job chaining design is in progress. See [tasks/planning/MultiJob_And_Templates.md](tasks/planning/MultiJob_And_Templates.md) for current design exploration.

**Core Concept**: "Restart after timeout" and "next equilibration stage" are the same mechanism - creating new job that continues from parent job's outputs. Jobs are self-contained islands (each copies necessary files from parent).

**Not Yet Finalized**: Specific implementation details, data model, UI patterns. Will be designed in detail when we reach this milestone.

### Phase 7 Complete When:
- Template system operational (users can create/edit templates via UI)
- Settings page with database management functional (AppImage working)
- Rate limiting prevents cluster abuse
- **Template-based job creation ready for users with production-ready builds**
- Job chaining design finalized (implementation in future phase)

## Phase 8: Multi-Cluster & Settings
*Settings page extensions and multi-cluster support*

### Milestone 8.1: Settings Page Extensions
**Note**: Basic Settings page with database management implemented in Phase 7.2. This milestone extends it with cluster configuration and user preferences.

- [ ] **Settings Database & UI**
  - [ ] Settings database schema for cluster configs and user preferences
  - [ ] Extend Settings page with tabs/sections for configuration
  - [ ] User preferences (default values, UI behavior)
  - [ ] Export/import settings functionality
- [ ] **Cluster Configuration Management**
  - [ ] User-editable cluster configuration (builds on cluster_config.rs Rust constants)
  - [ ] Configurable cluster connection settings (login server, port)
  - [ ] Customizable SLURM partitions list
  - [ ] Configurable QOS options
  - [ ] Module versions configuration (gcc, cuda, namd versions)
  - [ ] Resource limits and defaults per partition
  - [ ] Multiple cluster profile support
  - [ ] Migration from hardcoded Rust constants to settings database

### Milestone 8.2: Dynamic Cluster Detection
- [ ] **Automatic Discovery**
  - [ ] Cluster discovery commands (module avail, module spider)
  - [ ] NAMD version detection (namd2 vs namd3)
  - [ ] Module dependency discovery and validation
  - [ ] Resource limit querying from SLURM partitions
  - [ ] MPI execution pattern detection
  - [ ] Dynamic resource limit detection per cluster

### Milestone 8.3: Automation Builder Foundation
- [ ] **Serializable Automation System**
  - [ ] Implement serialization for automation steps (builds on Phase 6 automation framework)
  - [ ] Automation template database schema and persistence
  - [ ] Template validation and safety checking
  - [ ] Import/export automation templates
- [ ] **Automation Builder UI (Basic)**
  - [ ] Automation explorer page showing existing workflows
  - [ ] Template library with predefined automation workflows
  - [ ] Basic automation editing (parameter modification, not visual yet)
  - [ ] Automation testing and validation tools

### Phase 8 Complete When:
- Settings page functional with cluster configuration
- Dynamic cluster detection working
- Multiple cluster profiles supported
- Automation template system foundation ready
- **Multi-cluster support with settings and automation foundation ready**

## Phase 9: Job Chaining Implementation
*Job continuation and multi-stage workflows*

**Note**: Design in progress at [tasks/planning/MultiJob_And_Templates.md](tasks/planning/MultiJob_And_Templates.md). This section is a placeholder - specific implementation details will be finalized before starting this phase.

### Milestone 9.1: Job Chaining Core (TBD)
- Details to be determined based on finalized design
- Job chain data model (parent-child relationships)
- File propagation system (parent outputs → child inputs)
- Jobs as self-contained islands (copy files, not reference)

### Milestone 9.2: Chaining UI & Workflows (TBD)
- Details to be determined based on finalized design
- Job continuation interface
- Chain visualization and management
- Template integration with job chaining

### Phase 9 Complete When:
- Job chaining implementation complete per finalized design
- **Multi-stage workflows functional**

## Post-MVP Enhancement Roadmap
*Future features beyond core functionality*

### UI/UX Enhancements
1. **Bulk operations** - Multi-select job management
2. **Advanced filtering/search** - Job filtering by status, date, resources
3. **Settings/preferences** - User customization and defaults

### Advanced Features (Future)
- Visual automation builder with drag-and-drop workflow designer
- Parameter sweep automation
- Advanced workflow patterns for scientific computing
- Community template marketplace

## Risk Mitigation

1. **MVP Focus**: Get single-job workflow working before adding complexity
2. **No Backwards Compatibility**: Breaking changes acceptable during all development phases
3. **Security**: Regular security audits, never log credentials, path validation
4. **Windows Compatibility**: Test early and often on Windows
5. **Design Before Code**: Finalize designs (like job chaining) before implementation to avoid rework
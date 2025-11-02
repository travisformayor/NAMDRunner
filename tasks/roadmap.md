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

## Phase 7: Production Hardening & Advanced Features

### Milestone 7.1: Request Rate Limiting & Queue Management

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

### Milestone 7.2: Job Restart Feature
*Job continuation functionality for single-job model*

- [ ] **Restart Data Model**
  - [ ] Add RestartInfo struct to JobInfo (single-job model extension)
  - [ ] Database schema update for restart_info field
  - [ ] Restart job creation and validation logic
- [ ] **Checkpoint File Management**
  - [ ] Checkpoint file detection in completed/failed jobs
  - [ ] File copying from original job to restart job scratch directory
  - [ ] Restart file validation and error handling
- [ ] **Restart Template System**
  - [ ] Restart-specific NAMD config template with checkpoint loading
  - [ ] Template variable injection for restart context
  - [ ] Step calculation logic (completed vs remaining steps)
- [ ] **Restart UI & UX**
  - [ ] "Restart Job" button and workflow
  - [ ] Resource allocation interface for restart (allow different resources)
  - [ ] Restart job lineage display and tracking

### Milestone 7.3: Advanced Restart Features
- [ ] **Automatic Restart Configuration**
  - [ ] Automatic restart configuration generation
  - [ ] Intelligent checkpoint interval recommendations
  - [ ] Resource optimization for restart jobs

## Post-MVP Enhancement Roadmap
*Future features beyond single-job restart functionality*

### UI/UX Enhancements
1. **Multi-stage job groups** - Expandable table rows with aggregate status
2. **Bulk operations** - Multi-select job management
3. **Advanced filtering/search** - Job filtering by status, date, resources
4. **Job templates** - Reusable job configuration templates
5. **Settings/preferences** - User customization and defaults

### Advanced Features
- **Job restart functionality** with checkpoint detection and restart wizard interface
- **Multi-Stage Job Groups** - Expandable table rows will support job groups with aggregate status display and individual stage management
  - [ ] Resource optimization for restart jobs
- [ ] **Checkpoint Management**
  - [ ] Checkpoint file validation and integrity checks
  - [ ] Checkpoint cleanup and retention policies
  - [ ] Checkpoint size estimation and warnings

### Milestone 7.3: Restart Testing & Integration
- [ ] Comprehensive restart functionality testing
- [ ] Integration with existing single-job workflow
- [ ] Error handling for restart failures
- [ ] Performance validation for checkpoint operations

### Milestone 7.4: Restart Documentation & Polish
- [ ] User guide updates for restart functionality
- [ ] Developer documentation for restart architecture
- [ ] Final restart feature cleanup and optimization

### Phase 7 Complete When:
- Job restart functionality working reliably
- Automatic restart configuration generation working
- Restart lineage tracking implemented
- Documentation updated for restart workflows
- **Single-job MVP with restart ready for users**

## Phase 8: Dynamic Configuration & Templates
*Multi-cluster support with configurable settings and templates*

### Milestone 8.1: Settings Page Infrastructure
- [ ] **Settings Database & UI**
  - [ ] Settings database schema for cluster configs, templates, and job types
  - [ ] Settings page UI with forms for configuration
  - [ ] User preferences (default values, UI behavior)
  - [ ] Export/import settings functionality
  - [ ] **Local Database Management**:
    - [ ] "Delete Local Cache" button to clear local job database
    - [ ] Works when connected or disconnected to server
    - [ ] Only deletes local DB, never touches server metadata
    - [ ] After deletion, clicking "Sync Now" rebuilds cache from server metadata (uses Phase 6.4 job discovery)
    - [ ] Warning dialog: "This will delete all local job data. Server metadata will not be affected. Click Sync to restore from server."
    - [ ] Useful for troubleshooting database corruption or starting fresh
- [ ] **Cluster Configuration Management**
  - [ ] User-editable cluster configuration (builds on Phase 6.4 cluster_config.rs Rust constants)
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

### Milestone 8.3: Template Management System
- [ ] **Template Infrastructure**
  - [ ] Template storage and management system
  - [ ] Template editor with syntax highlighting and validation
  - [ ] Variable definition language with `{{var_name}}` syntax
  - [ ] Variable rendering engine with type safety
- [ ] **Template Features**
  - [ ] Template comment syntax for variable metadata
  - [ ] Variable tooltips, descriptions, and validation rules
  - [ ] Default values and suggestions
  - [ ] Template library with common NAMD workflows
  - [ ] Template dropdown selector in Create Job page
  - [ ] Auto-population of form fields from template variables
  - [ ] File upload mapping to template variables
  - [ ] Template versioning and sharing capabilities
  - [ ] Preview of generated NAMD configuration
  - [ ] Export/import templates
- [ ] **Migration & Integration**
  - [ ] Migration from hardcoded Alpine configuration
  - [ ] Cluster-specific template associations
  - [ ] Job type configuration management (not hardcoded)

### Milestone 8.4: Automation Builder Foundation
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
- Template management system operational
- Multiple cluster profiles supported
- Automation template system foundation ready
- **Multi-cluster support with templates and automation foundation ready**

## Phase 9: Multi-Stage Job Workflows
*Architecture evolution for complex simulation workflows*

### Milestone 9.1: Job Workflow Architecture
- [ ] **Job Group Data Model** (Breaking changes acceptable - no backwards compatibility required)
  - [ ] Multi-stage job persistence schema
  - [ ] Job group concept with multiple dependent stages
  - [ ] Stage dependency management and sequencing
  - [ ] Migration strategy from single jobs to job groups
- [ ] **File Propagation System**
  - [ ] Automatic output-to-input file transfer between stages
  - [ ] Stage-specific template rendering with previous stage context
  - [ ] Restart file management across workflow stages
  - [ ] Stage-specific parameter configuration

### Milestone 9.2: Workflow Templates & UI
- [ ] **Multi-Stage Template System**
  - [ ] DNA origami equilibration workflow templates
  - [ ] Stage progression templates (minimization → k=0.5 → k=0.1 → k=0.01 → production)
  - [ ] Structure optimization with restraint files
  - [ ] Multi-stage equilibration workflows
  - [ ] Workflow validation and dependency checking
- [ ] **Stage Management UI**
  - [ ] Progress tracking across multiple stages
  - [ ] Individual stage monitoring and control
  - [ ] Workflow restart and recovery capabilities
  - [ ] Visual workflow designer/editor

### Milestone 9.3: Advanced Workflow Features
- [ ] **Workflow Automation**
  - [ ] Conditional stage execution based on results
  - [ ] Automatic parameter adjustments between stages
  - [ ] Workflow branching and merging
- [ ] **Workflow Management**
  - [ ] Workflow templates library
  - [ ] Workflow sharing and versioning
  - [ ] Workflow performance analytics

### Phase 9 Complete When:
- Multi-stage workflow architecture implemented
- Workflow templates functional for common use cases
- Stage progression tracking and management working
- File propagation between stages automatic
- **Full workflow MVP ready for scientific users**

## Phase 10: Visual Automation Builder & Advanced Workflows
*Complete automation builder with visual workflow designer*

### Milestone 10.1: Visual Automation Builder
- [ ] **Visual Workflow Designer**
  - [ ] Drag-and-drop automation step composer
  - [ ] Visual connection lines showing workflow dependencies
  - [ ] Step parameter editing with inline forms
  - [ ] Real-time workflow validation and error highlighting
  - [ ] Workflow preview and execution planning
- [ ] **Advanced Automation Features**
  - [ ] Conditional logic and branching based on step results
  - [ ] Loop constructs for parameter sweeps and batch operations
  - [ ] Variable passing between automation steps
  - [ ] Custom automation step creation and sharing
  - [ ] Automation debugging and step-by-step execution
- [ ] **Workflow Templates & Community**
  - [ ] Community automation template marketplace
  - [ ] Template rating and review system
  - [ ] Version control for automation workflows
  - [ ] Collaboration features for shared workflow development

### Milestone 10.2: Advanced Workflow Patterns
- [ ] **Multi-Job Workflows**
  - [ ] Parameter sweep automation (multiple jobs with varying parameters)
  - [ ] Ensemble simulation management
  - [ ] Dependency-based job scheduling and execution
  - [ ] Batch result analysis and comparison
- [ ] **Scientific Workflow Templates**
  - [ ] DNA origami equilibration workflows (multi-stage with restraints)
  - [ ] Protein folding simulation pipelines
  - [ ] Drug discovery computational workflows
  - [ ] Materials science simulation templates

### Milestone 10.3: Automation Performance & Monitoring
- [ ] **Workflow Execution Monitoring**
  - [ ] Real-time workflow progress tracking across multiple jobs
  - [ ] Automation performance analytics and optimization suggestions
  - [ ] Workflow execution history and audit trails
  - [ ] Automated error recovery and retry strategies
- [ ] **Resource Management**
  - [ ] Intelligent resource allocation across workflow steps
  - [ ] Queue time optimization and scheduling strategies
  - [ ] Cost estimation and optimization for automation workflows
  - [ ] Cluster utilization monitoring and recommendations

### Phase 10 Complete When:
- Visual automation builder functional with drag-and-drop interface
- Advanced workflow patterns support scientific use cases
- Community template system operational
- Automation performance monitoring implemented
- **Full automation platform ready for scientific computing workflows**

## Phase 11: Monitoring & Management
*Performance monitoring and batch operations*

### Milestone 11.1: Performance Monitoring
- [ ] **Resource Usage Tracking**
  - [ ] Performance metrics and SU usage tracking
  - [ ] Storage usage monitoring and alerts
  - [ ] Job efficiency analysis and recommendations
  - [ ] Resource utilization reports
- [ ] **System Monitoring**
  - [ ] Cluster health monitoring integration
  - [ ] Queue time predictions
  - [ ] Resource availability forecasting

### Milestone 11.2: Batch Operations & Management
- [ ] **Bulk Job Operations**
  - [ ] Batch processing for large job sets
  - [ ] Bulk status updates and filtering
  - [ ] Mass job submission with parameter sweeps
  - [ ] Batch job cancellation and cleanup
- [ ] **Advanced File Management**
  - [ ] Advanced file management and cleanup utilities
  - [ ] Automated archival of completed jobs
  - [ ] Disk space optimization tools

### Milestone 11.3: Enhanced Recovery & Reliability
- [ ] **Error Recovery**
  - [ ] Enhanced error recovery workflows
  - [ ] Automatic job resubmission on transient failures
  - [ ] Smart retry strategies based on failure patterns
- [ ] **Reliability Features**
  - [ ] Health checks and system diagnostics
  - [ ] Automated backup and recovery
  - [ ] Data integrity validation

### Phase 11 Complete When:
- Performance monitoring and alerting functional
- Batch operations working for large-scale usage
- Enhanced error recovery implemented
- System reliability and monitoring operational

## Risk Mitigation

1. **MVP Focus**: Get single-job workflow working before adding complexity
2. **Future-Ready Architecture**: Design data models to support job groups later
3. **Security**: Regular security audits, never log credentials
4. **Windows Compatibility**: Test early and often on Windows
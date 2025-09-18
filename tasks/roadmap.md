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
- Python reference implementation
- Earlier iterations of schemas or interfaces
- Test data or mock implementations

## Current Status: Phase 5 Complete ✅

**Phase 3 Skipped**: UI development is being handled in a separate branch.

**Next Priority**: Phase 6 - Single-Job MVP Completion (testing, polish, and production readiness)

**Current Implementation**: See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for detailed description of what exists now, including module structure, SSH/SFTP integration, and security implementation.

## Phase 1: Foundation ✅ COMPLETED
*Critical path to first working prototype*

### Milestone 1.1: Project Scaffold ✅ COMPLETED
- [x] Initialize Tauri v2 project with Svelte template
- [x] Configure TypeScript with strict settings
- [x] Set up Svelte with component structure
- [x] **Implement IPC boundary interfaces** (see `tasks/phase1-interface-definitions.md`)
- [x] **Implement JSON metadata schema** (see `tasks/phase1-interface-definitions.md`)
- [x] **Implement Rust type definitions** (see `tasks/phase1-interface-definitions.md`)
- [x] Define Rust module architecture

### Milestone 1.2: Mock Infrastructure ✅ COMPLETED
- [x] Implement mock IPC client for UI development
- [x] Create fixture data for testing (job states, SLURM responses)
- [x] Mock SLURM responses for offline development
- [x] Dual-purpose testing infrastructure (UI and E2E)
- [x] WebdriverIO with tauri-driver for E2E testing
- [x] Agent debug toolkit for autonomous development
- [x] Configure CI for Linux and Windows builds

### Milestone 1.3: Connection Foundation ✅ COMPLETED
- [x] SSH/SFTP connection interface definitions
- [x] Connection state management (Disconnected → Connected → Expired)
- [x] Error handling strategy definition
- [x] Remote directory structure setup (`/projects/$USER/namdrunner_jobs/`)
- [x] Connection validation and testing utilities

### Milestone 1.4: Phase 1 Cleanup & Refactoring ✅ COMPLETED
- [x] Run Phase 1 code review using `.claude/agents/review-refactor.md` agent
- [x] Eliminate code duplication discovered during Phase 1 implementation
- [x] Ensure architectural consistency across all components  
- [x] Validate TypeScript/Rust IPC boundary patterns and naming conventions
- [x] Implement dependency injection system with service container
- [x] Centralize path management with PathResolver service
- [x] Standardize error handling with Result<T> patterns
- [x] Remove thin wrappers and redundant fallback code
- [x] Update documentation with clean architecture patterns
- [x] Update mock client factory to use proper dependency injection

## Phase 2: Core Backend
*SSH connection and data management*

### Milestone 2.1: SSH/SFTP Implementation ✅ COMPLETED
- [x] Password authentication with ssh2 crate
- [x] SFTP file upload/download operations
- [x] Module loading commands (`module load slurm/alpine`)
- [x] SSH connection debugging and error recovery
- [x] Real connection establishment and management
- [x] Secure credential handling with automatic memory cleanup
- [x] Comprehensive error mapping with recovery suggestions
- [x] Mock/real mode switching via environment variables
- [x] 43 focused unit tests covering business logic without network dependencies
- [x] Clean architecture with separated concerns and responsibilities

### Milestone 2.2: SSH/SFTP Critical Fixes & Enhancements ✅ COMPLETED
- [x] **Job Directory Management** ✅ Complete lifecycle implementation
  - [x] Directory creation during job creation via SFTP (project: `/projects/$USER/namdrunner_jobs/$JOB_ID/`)
  - [x] Directory creation during job submission (scratch: `/scratch/alpine/$USER/namdrunner_jobs/$JOB_ID/`)
  - [x] Safe directory cleanup during job deletion with validation
  - [x] Existence checking via SFTP stat operations prevents recreation
- [x] **Retry Logic Implementation** ✅ Exponential backoff with pattern-based strategies
  - [x] RetryManager with exponential backoff and jitter (1s → 2s → 4s)
  - [x] Configurable retry limits (default: 3 attempts) with timeout controls
  - [x] ConnectionUtils wrapper provides retry logic for all SSH operations
  - [x] Proper error classification for retryable vs non-retryable errors
- [x] **SLURM Integration Robustness** ✅ Enhanced command parsing and validation
  - [x] Fixed SBATCH output parsing to validate numeric job IDs
  - [x] Proper handling of multiline output and error cases
  - [x] Enhanced validation for "Submitted batch job" format
- [x] **Path Security & Validation** ✅ Defense-in-depth security implementation
  - [x] Input sanitization (`sanitize_job_id()`, `sanitize_username()`) with Unicode rejection
  - [x] Directory traversal protection blocks `../` sequences and absolute paths
  - [x] Shell parameter escaping (`escape_parameter()`) prevents command injection
  - [x] Safe command construction (`build_command_safely()`) with parameter validation
- [x] **Test Suite Quality** ✅ Aligned with NAMDRunner testing philosophy
  - [x] 116 tests passing (resolved 13 failures, removed inappropriate performance tests)
  - [x] Security tests validate comprehensive malicious input patterns
  - [x] SFTP mock filesystem business logic properly simulates directory operations

### Milestone 2.3: Job Status Synchronization & Data Persistence ✅ COMPLETED
- [x] **SLURM Status Integration** ✅ Complete SLURM command execution and parsing
  - [x] Implement SLURM command execution (squeue, sacct) with retry logic via ConnectionUtils
  - [x] Build job status parsing and state mapping for all SLURM states (PENDING → RUNNING → COMPLETED/FAILED/CANCELLED)
  - [x] Add manual status synchronization via sync_job_status and sync_all_jobs commands
  - [x] Integrate job lifecycle status updates with existing directory management seamlessly
- [x] **Local Job Persistence** ✅ Complete SQLite integration and session continuity
  - [x] SQLite integration with rusqlite for local job cache (thread-safe operations)
  - [x] Job cache schema implemented per docs/data-spec.md (jobs table with metadata, status history)
  - [x] Session persistence (jobs saved to database instead of mock state, persist across restarts)
  - [x] JobInfo struct extended with database persistence methods for clean API
- [x] **Status Display & Updates** ✅ Complete status management foundation
  - [x] Manual status updates via sync commands with proper error handling
  - [x] Status transition logging with full history tracking in database
  - [x] Error state handling with existing retry logic and error classification patterns
  - [x] Database operations with comprehensive error handling and recovery

### Milestone 2.4: Phase 2 Cleanup & Refactoring ✅ COMPLETED
- [x] Run Phase 2 code review using `.claude/agents/review-refactor.md` agent
- [x] Eliminate JobService thin wrapper - use DefaultJobRepository directly
- [x] Remove duplicate business logic functions in job commands
- [x] Fix InputFile duplicate fields - consolidate to single file_type with NAMDFileType enum
- [x] Remove unused ModeSwitch trait - keep only execute_with_mode function
- [x] Remove unused ApiResult pattern methods - simplified to essential functions only
- [x] Simplify validation error types - use anyhow::Error instead of complex ValidationError system
- [x] Consolidated validation patterns and removed over-engineered trait implementations

## Phase 3: Frontend Development
*User interface and workflows*

### Milestone 3.1: Core UI Components
- [ ] Connection management dialog
- [ ] Main dashboard with job table (single jobs)
- [ ] Status indicators and badges
- [ ] Navigation and routing

### Milestone 3.2: Job Creation & Management UI
- [ ] **Job Creation Form**
  - [ ] Simple job creation form (job name, input files, basic parameters)
  - [ ] File upload interface with progress tracking
  - [ ] Resource allocation controls (cores, memory, walltime)
  - [ ] Basic NAMD parameter validation (client-side)
- [ ] **Job Management Interface**
  - [ ] Job detail view with status and metadata
  - [ ] Job action buttons (submit, delete, restart later in Phase 7)

### Milestone 3.3: Results & File Management UI
- [ ] **File Operations Interface**
  - [ ] File listing and browsing interface for job results
  - [ ] Download interface for result files
  - [ ] File upload progress and status display
- [ ] **Log Viewing & Debugging UI**
  - [ ] Basic log viewer for SLURM and NAMD output
  - [ ] SSH command logging and console view
  - [ ] Error message display and troubleshooting interface

### Milestone 3.4: Phase 3 Cleanup & Refactoring
- [ ] Run Phase 3 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review UI components for consistency, reusability, and duplication
- [ ] Consolidate similar form validation patterns discovered
- [ ] Ensure accessible design patterns throughout application
- [ ] Validate proper error display and user feedback patterns
- [ ] Document UI component patterns and establish design system

## Phase 4: SLURM Integration ✅ COMPLETED
*Cluster job management*

### Milestone 4.1: Job Submission ✅ COMPLETED
- [x] **SLURM script generation** - Implemented template-based NAMD job script generation
- [x] **Real sbatch job submission** - Real SLURM submission with actual job IDs from cluster
- [x] Job ID parsing - Enhanced sbatch output parsing with comprehensive error handling
- [x] Error handling - Complete integration with retry logic and user-friendly error messages

### Milestone 4.2: Status Tracking & Sync ✅ COMPLETED
- [x] squeue/sacct integration for active jobs - Implemented in `slurm/status.rs`
- [x] Status parsing and updates (PENDING → RUNNING → COMPLETED) - Working with job lifecycle
- [x] Manual sync commands (`sync_job_status`, `sync_all_jobs`) - Implemented and functional
- [x] Update local cache with database persistence - Jobs persist across app restarts
- [x] Handle job state transitions and completion - Full lifecycle management working

### Milestone 4.3: Phase 4 Cleanup & Refactoring ✅ COMPLETED
- [x] Built on existing patterns - No duplicate SLURM command execution, uses established SSH infrastructure
- [x] Consolidated error handling - Enhanced existing retry logic with SLURM-specific error mapping
- [x] Validated async operations - All job submission steps use existing ConnectionManager patterns
- [x] Consistent job lifecycle - Enhanced existing submit_job_real() function without breaking changes
- [x] Simple implementation - No external dependencies, built-in string parsing, leveraged existing code

## Phase 5: File Operations & Results Management ✅ COMPLETED
*Complete backend file operations for end-to-end workflow*

### Milestone 5.1: Real File Upload Implementation ✅ COMPLETED
- [x] **Replace Mock File Upload**
  - [x] Convert `upload_job_files` from mock to real SFTP operations
  - [x] Implement progress tracking for large file uploads
  - [x] Add file validation before upload (PDB, PSF, parameter files)
  - [x] Handle upload failures with retry logic using existing ConnectionUtils
- [x] **Input File Management**
  - [x] Upload files to project directory (`/projects/$USER/namdrunner_jobs/$JOB_ID/input_files/`)
  - [x] Copy files to scratch directory during job submission
  - [x] Validate file integrity after upload (size, basic format checks)

### Milestone 5.2: Real File Download & Results Management ✅ COMPLETED
- [x] **Replace Mock File Download**
  - [x] Convert `download_job_output` from mock to real SFTP operations
  - [x] Implement directory listing via SFTP for results browsing
  - [x] Download SLURM output files (.out, .err) from scratch directory
  - [x] Download NAMD output files (.dcd, .log, checkpoint files)
- [x] **Log File Aggregation**
  - [x] Collect and aggregate SLURM job logs (.out, .err files)
  - [x] Collect NAMD simulation logs (namd_output.log)
  - [x] Provide unified log access for debugging and monitoring

### Milestone 5.3: Job Cleanup & Lifecycle Completion ✅ COMPLETED
- [x] **Remote Directory Cleanup**
  - [x] Implement job deletion with remote directory cleanup
  - [x] Clean up both project and scratch directories safely
  - [x] Preserve important results before cleanup (optional download)
- [x] **File Operation Error Handling**
  - [x] Robust error handling for all SFTP operations
  - [x] Network interruption recovery using existing retry logic
  - [x] Clear error messages for file operation failures

### Milestone 5.4: Code Quality & Architecture Improvements ✅ COMPLETED
- [x] Run Phase 5 code review using `.claude/agents/review-refactor.md` agent
- [x] Eliminate JobRepository trait thin wrapper - use direct database calls
- [x] Remove ValidateId trait wrapper - use validation functions directly
- [x] Remove intermediate business logic functions that just wrap execute_with_mode
- [x] Remove unused mode_switch! macro
- [x] Simplify mock state implementation - remove complex simulation behavior
- [x] Achieved ~20% code reduction without losing functionality
- [x] Improved code readability and maintainability

## Phase 6: Single-Job MVP Completion
*Testing, polish, and production readiness for core single-job functionality*

### Milestone 6.1: Comprehensive Testing
- [ ] Unit test coverage >80%
- [ ] E2E test suite complete
- [ ] Manual testing checklist
- [ ] Performance optimization

### Milestone 6.2: Production Readiness
- [ ] Windows executable build
- [ ] Installation documentation
- [ ] User guide (single-job workflow)
- [ ] Deployment pipeline

### Milestone 6.3: MVP Cleanup & Architecture Review
- [ ] Run comprehensive code review using `.claude/agents/review-refactor.md` agent
- [ ] Final refactoring pass for consistency and maintainability
- [ ] Implement performance optimization opportunities identified
- [ ] Complete security review and hardening recommendations
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

### Phase 3 Complete When:
- Full UI navigation works
- Job creation and management forms complete
- File upload/download interface working
- Log viewing and debugging UI functional
- All forms validate input properly
- UI responsive and accessible
- **Complete frontend ready for backend integration**

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
- All tests passing (>80% coverage)
- Windows exe distributable
- Documentation complete for single-job workflow
- Production-ready deployment
- **Single-job MVP ready for users (without restart or workflows)**

## Phase 7: Job Restart Feature
*Job continuation functionality for single-job model*

### Milestone 7.1: Job Restart Implementation
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

### Milestone 7.2: Restart Testing & Integration
- [ ] Comprehensive restart functionality testing
- [ ] Integration with existing single-job workflow
- [ ] Error handling for restart failures
- [ ] Performance validation for checkpoint operations

### Milestone 7.3: Restart Documentation & Polish
- [ ] User guide updates for restart functionality
- [ ] Developer documentation for restart architecture
- [ ] Final restart feature cleanup and optimization

### Phase 7 Complete When:
- Job restart functionality working reliably
- Restart lineage tracking implemented
- Documentation updated for restart workflows
- **Single-job MVP with restart ready for users**

### Phase 8 Complete When:
- Dynamic cluster configuration working
- Settings page functional
- Template management system operational
- **Multi-cluster support ready**

### Phase 9 Complete When:
- Multi-stage workflow architecture implemented
- Workflow templates functional
- Stage progression tracking working
- **Full workflow MVP ready for users**

## Phase 8: Dynamic Configuration & Settings
*Multi-cluster support and configurable templates*

### Milestone 8.1: Settings Page & Dynamic Configuration
- [ ] **Settings Page Infrastructure**
  - [ ] Settings database schema for cluster configs, templates, and job types
  - [ ] Cluster discovery commands (module avail, module spider)
  - [ ] Template storage and management system
- [ ] **Dynamic Cluster Detection**
  - [ ] NAMD version detection (namd2 vs namd3)
  - [ ] Module dependency discovery and validation
  - [ ] Resource limit querying from SLURM partitions
  - [ ] MPI execution pattern detection

### Milestone 8.2: Template Management System
- [ ] **Configurable Templates**
  - [ ] Template editor with syntax highlighting and validation
  - [ ] Variable definition and rendering engine
  - [ ] Job type configuration management (not hardcoded)
  - [ ] Migration from hardcoded Alpine configuration

### Milestone 8.3: Multi-Cluster Support
- [ ] **Cluster Configuration Management**
  - [ ] Multiple cluster profile support
  - [ ] Cluster-specific template and job type associations
  - [ ] Dynamic resource limit detection per cluster

## Phase 9: Multi-Stage Job Workflows
*Architecture evolution based on single-job lessons learned*

### Milestone 9.1: Job Workflow Architecture
- [ ] **Job Group Data Model** (Breaking change from single jobs version can happen, we do not need to maintain backwards compatibility)
  - [ ] Multi-stage job persistence schema
  - [ ] Stage dependency management and sequencing
  - [ ] Migration strategy from single jobs to job groups
- [ ] **File Propagation System**
  - [ ] Automatic output-to-input file transfer between stages
  - [ ] Stage-specific template rendering with previous stage context
  - [ ] Restart file management across workflow stages

### Milestone 9.2: Workflow Templates
- [ ] **Multi-Stage Template System**
  - [ ] DNA origami equilibration workflow templates
  - [ ] Stage progression templates (minimization → k=0.5 → k=0.1 → k=0.01 → production)
  - [ ] Workflow validation and dependency checking
- [ ] **Stage Management UI**
  - [ ] Progress tracking across multiple stages
  - [ ] Individual stage monitoring and control
  - [ ] Workflow restart and recovery capabilities

## Future Advanced Features (Post-Workflow MVP)
*Additional capabilities that can be added after core workflow functionality*

### Advanced NAMD Features
- [ ] Multiple job types (Structure Optimization, Multi-Stage Equilibration)
- [ ] Automatic restart configuration generation
- [ ] Checkpoint detection and resume capabilities
- [ ] Advanced parameter validation and job type workflows
- [ ] Restraint file handling for structure optimization

### Advanced Job Management
- [ ] Batch processing for large job sets
- [ ] Advanced file management and cleanup utilities
- [ ] Job deletion with remote cleanup
- [ ] Bulk status updates and filtering

### Advanced Features
- [ ] Performance metrics and SU usage tracking
- [ ] Module version configuration and auto-detection
- [ ] Enhanced error recovery workflows
- [ ] Storage usage monitoring and alerts

## Risk Mitigation

1. **MVP Focus**: Get single-job workflow working before adding complexity
2. **Future-Ready Architecture**: Design data models to support job groups later
3. **Security**: Regular security audits, never log credentials
4. **Windows Compatibility**: Test early and often on Windows
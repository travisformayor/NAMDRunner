# NAMDRunner Development Roadmap

**🏗️ Architecture Updates**: When completing milestones, always update `docs/architecture.md` to reflect the actual implementation. Architecture doc describes what IS built, this roadmap describes what WILL be built.

## Development Strategy
Build a **single-job MVP first** that handles the core workflow: create job → submit → track → view results. 

**Key Design Decision**: Job persistence and discovery are built from Phase 2, not added later. This ensures:
- Test jobs persist between development sessions for proper testing
- No accumulation of "phantom" test data on cluster
- Status sync works from first job submission
- Developers can easily manage and clean up test jobs

## Current Status: Phase 2 Milestone 2.2 Complete ✅

**Next Priority**: Milestone 2.3 (Job Status Synchronization) - Build job status tracking and synchronization with SLURM queue state.

**Current Implementation**: See [`docs/architecture.md`](../docs/architecture.md) for detailed description of what exists now, including module structure, SSH/SFTP integration, and security implementation.

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

### Milestone 2.3: Job Status Synchronization & Data Persistence
- [ ] **SLURM Status Integration** (Critical - Core functionality)
  - [ ] Implement SLURM command execution (squeue, sacct, scancel)
  - [ ] Build job status parsing and state mapping (PENDING → RUNNING → COMPLETED)
  - [ ] Add automatic status synchronization with configurable intervals
  - [ ] Integrate job lifecycle status updates with existing directory management
- [ ] **Local Job Persistence** (High Priority - Session continuity)
  - [ ] SQLite integration with rusqlite for local job cache
  - [ ] Job cache schema (jobs table with metadata, status history)
  - [ ] Session persistence (close/reopen shows existing jobs with current status)
  - [ ] Job discovery and recovery from existing directories and metadata
- [ ] **Status Display & Updates** (Medium Priority - User experience)
  - [ ] Real-time status updates via job sync operations
  - [ ] Status transition logging (track PENDING → RUNNING → COMPLETED progression)
  - [ ] Error state handling and recovery suggestions
  - [ ] Background sync with proper retry logic and error classification

### Milestone 2.4: Phase 2 Cleanup & Refactoring  
- [ ] Run Phase 2 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review SSH/SFTP implementation for consistency and patterns
- [ ] Eliminate any duplicated connection handling code discovered
- [ ] Validate database schema and access patterns for optimization
- [ ] Ensure proper error handling across all backend operations
- [ ] Document SSH connection patterns and lessons learned

## Phase 3: Frontend Development
*User interface and workflows*

### Milestone 3.1: Core UI Components
- [ ] Connection management dialog
- [ ] Main dashboard with job table (single jobs)
- [ ] Status indicators and badges
- [ ] Navigation and routing

### Milestone 3.2: Single Job Creation Workflow
- [ ] Simple job creation form
- [ ] File upload interface
- [ ] Basic NAMD parameter validation
- [ ] Resource allocation controls (cores, memory, walltime)

### Milestone 3.3: Phase 3 Cleanup & Refactoring
- [ ] Run Phase 3 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review UI components for consistency, reusability, and duplication
- [ ] Consolidate similar form validation patterns discovered
- [ ] Ensure accessible design patterns throughout application
- [ ] Validate proper error display and user feedback patterns
- [ ] Document UI component patterns and establish design system

## Phase 4: SLURM Integration
*Cluster job management*

### Milestone 4.1: Job Submission
- [ ] SLURM script generation
- [ ] sbatch command execution
- [ ] Job ID parsing
- [ ] Error handling

### Milestone 4.2: Status Tracking & Sync
- [ ] squeue/sacct integration for active jobs
- [ ] Status parsing and updates (PENDING → RUNNING → COMPLETED)
- [ ] Sync button to refresh all job statuses
- [ ] Update both local cache and remote JSON files
- [ ] Handle job state transitions and completion

### Milestone 4.3: Phase 4 Cleanup & Refactoring
- [ ] Run Phase 4 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review SLURM command execution patterns for consistency
- [ ] Consolidate job status parsing and error handling code
- [ ] Validate proper async operation management across commands
- [ ] Ensure consistent job lifecycle management patterns
- [ ] Document SLURM integration patterns and command templates

## Phase 5: NAMD Features (Single-Job MVP)
*Molecular dynamics specifics*

### Milestone 5.1: Basic NAMD Configuration
- [ ] Single NAMD template for basic simulations
- [ ] Essential parameter validation (temp, timestep, steps)
- [ ] Basic job type (General NAMD Simulation)
- [ ] File format validation (PDB, PSF, parameters)

### Milestone 5.2: Results Management
- [ ] Download SLURM output files (.out, .err)
- [ ] Basic log viewing interface
- [ ] Job completion status tracking
- [ ] Simple file browsing for outputs

### Milestone 5.3: Phase 5 Cleanup & Refactoring
- [ ] Run Phase 5 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review NAMD configuration generation patterns for reusability
- [ ] Consolidate parameter validation across different job types
- [ ] Validate file handling and template systems consistency
- [ ] Ensure consistent results processing workflows
- [ ] Document NAMD-specific patterns, configurations, and templates

## Phase 6: Testing & Polish
*Quality assurance and deployment*

### Milestone 6.1: Comprehensive Testing
- [ ] Unit test coverage >80%
- [ ] E2E test suite complete
- [ ] Manual testing checklist
- [ ] Performance optimization

### Milestone 6.2: Production Readiness
- [ ] Windows executable build
- [ ] Installation documentation
- [ ] User guide
- [ ] Deployment pipeline

### Milestone 6.3: Final Cleanup & Architecture Review
- [ ] Run comprehensive code review using `.claude/agents/review-refactor.md` agent
- [ ] Final refactoring pass for consistency and maintainability across all phases
- [ ] Implement performance optimization opportunities identified
- [ ] Complete security review and hardening recommendations
- [ ] Final documentation completeness check and cleanup

## Success Metrics

### Phase 1 Complete When:
- ✅ Tauri app launches with basic UI
- ✅ IPC boundary interfaces defined and documented
- ✅ JSON metadata schema specified
- ✅ Mock SLURM integration working for offline dev
- ✅ E2E test takes screenshot 
- ✅ CI builds Windows exe
- ✅ SSH/SFTP interface patterns defined
- ✅ Connection state management architecture established
- ✅ Remote directory management foundations implemented

### Phase 2 Complete When:
- [x] SSH connection works with password
- [x] Files upload/download via SFTP
- [x] **Job directory lifecycle works correctly** (create → submit → delete)
- [x] **Retry logic handles network interruptions gracefully**
- [x] **Path security prevents directory traversal attacks**
- [x] **File operations are optimized** (avoid redundant uploads)
- [ ] SQLite stores and retrieves job data
- [ ] App reopening shows previously created jobs
- [ ] Job discovery from remote JSON files works
- [ ] Basic SLURM status sync functional

### Phase 3 Complete When:
- Full UI navigation works
- Job creation wizard complete
- All forms validate input
- UI responsive and accessible

### Phase 4 Complete When:
- Jobs submit to SLURM
- Status updates correctly
- Cache syncs with cluster
- Errors handled gracefully

### Phase 5 Complete When:
- Basic NAMD configs generate correctly
- Single job type (General Simulation) supported
- SLURM output files viewable in UI
- Job completion tracking works

### Phase 6 Complete When:
- All tests passing
- Windows exe distributable  
- Documentation complete
- **Single-job MVP ready for users**

## Future Features (Post-MVP)
*Architecture is designed to support these, but not required for initial usefulness*

### Multi-Stage Job Groups
- [ ] Job group concept with multiple dependent stages
- [ ] Stage dependency management and sequencing
- [ ] Automatic file propagation between stages
- [ ] Stage-specific parameter configuration
- [ ] Progress tracking across multiple stages

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
# NAMDRunner Development Roadmap

**Architecture Updates**: When completing milestones, always update `docs/ARCHITECTURE.md` to reflect the actual implementation. Architecture doc describes what IS built, this roadmap describes what WILL be built.

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

## Current Status: Phase 7.1 & 7.2 Complete ✅

**Next Priority**: Phase 8 - Settings Page Configuration Management (User-editable cluster settings and app/module management)

**Current Implementation**: See [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for detailed description of what exists now, including module structure, SSH/SFTP integration, and security implementation.

## Phase 1: Foundation ✅ COMPLETED

Critical path to first working prototype

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

SSH connection and data management

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

Complete backend file operations for end-to-end workflow

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

Testing, polish, and production readiness for core single-job functionality

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

### Phase 6 Complete When (Single-Job MVP)

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

### Milestone 7.1: Template System Refactor ✅ COMPLETED

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
- [x] **End-to-End Verification**: User testing of complete job lifecycle with templates
- [x] **Documentation**: Comprehensive updates across ARCHITECTURE.md, DB.md, API.md, DESIGN.md, CONTRIBUTING.md
- [x] **Testing**: 199 tests passing (173 Rust + 26 frontend), zero warnings

**Why**: Hardcoded NAMDConfig prevents supporting different simulation types without code changes. Template-as-data enables runtime modification and user extensibility.

**Results**: Template system fully operational with zero backwards compatibility concerns.

See: [phase-7-1-template-system-refactor.md](tasks/completed/phase-7-1-template-system-refactor.md)

### Milestone 7.2: Settings Page with Database Management ✅ COMPLETED

**Goal**: Fix AppImage database path bug and add Settings page with database management (backup, restore, reset)

**Problem Solved**:

- AppImage database path fixed (now uses proper user data directories)
- Settings page provides user control over database operations
- Complete theme system unification (Dialog primitive, centralized CSS variables)
- ~800 lines of duplicate code eliminated

**Implementation**:

- [x] **Database Path Migration**: Initialization in `.setup()` hook using `app_data_dir()` API
  - [x] `get_database_path()` - Returns OS-specific path (Linux: `~/.local/share/namdrunner/`, Windows: `%APPDATA%\namdrunner\`)
  - [x] `reinitialize_database()` - Close and reopen connection (for restore/reset)
  - [x] Development builds use `./namdrunner_dev.db` (unchanged)
  - [x] Ground-up refactor with zero tech debt

- [x] **Database Management Commands** (`commands/database.rs`):
  - [x] `get_database_info()` - Returns path and file size
  - [x] `backup_database()` - SQLite Backup API for safe online backup
  - [x] `restore_database()` - File dialog, validate, replace DB, reinitialize
  - [x] `reset_database()` - Delete and recreate with fresh schema

- [x] **Settings Page UI**:
  - [x] Settings page in sidebar navigation
  - [x] Display database location and size
  - [x] Backup button (opens save dialog)
  - [x] Restore button (warning dialog → file dialog → replace)
  - [x] Reset button (warning dialog → delete all data)
  - [x] Uses Dialog primitive and AlertDialog components

- [x] **Theme & Modal System Unification** (Bonus Implementation):
  - [x] Dialog.svelte primitive - Single modal component base
  - [x] AlertDialog.svelte - Replaces native alert()
  - [x] Refactored all modals to use Dialog (ConfirmDialog, PreviewModal, ConnectionDialog)
  - [x] Centralized button system (namd-button classes)
  - [x] Complete CSS variable system for light/dark themes
  - [x] Removed all hardcoded colors

- [x] **Testing**: Manual testing complete (AppImage, RPM, dev builds, theme testing)
- [x] **Documentation**: DB.md and API.md updated with database management patterns

**Why**: AppImage was completely broken without this fix. Settings page provides user control over database management. Theme unification ensures consistent, maintainable UI.

**Results**: AppImage functional, Settings page operational, theme system unified, ~800 lines of duplicate code eliminated.

See: [phase-7-2-db-settings-page-and-theming.md](tasks/completed/phase-7-2-db-settings-page-and-theming.md)

### Phase 7 Complete When

- **Milestone 7.1**: ✅ Template system operational (users can create/edit templates via UI, complete documentation)
- **Milestone 7.2**: ✅ Settings page with database management functional (AppImage working, theme system unified)
- **Milestone 7.3**: ⏸️ Rate limiting (deferred to future work)
- **Milestone 7.4**: ⏸️ Job chaining design (deferred to future work)

**Current Status**: Milestones 7.1 and 7.2 complete. Template-based job creation fully functional with production database management.

## Phase 8: Settings Page - Cluster & App Configuration

User-configurable cluster settings and application module management

**Context**: Currently cluster configuration (partitions, QoS, resource limits) and application modules (NAMD versions, prerequisites) are hardcoded in Rust. If cluster admins rename partitions, change limits, or update module versions, the app breaks. This phase makes all cluster-specific configuration user-editable to future-proof the application.

**Breaking Changes**: New database tables (`cluster_configs`, `pinned_apps`), job metadata schema changes to include `app_module` field. No backwards compatibility needed (app not yet released, user will delete old database).

### Milestone 8.1: User-Editable Cluster Configuration

**Goal**: Move hardcoded cluster settings from Rust constants to database, allowing users to edit partitions, QoS options, and resource limits via Settings page.

**Current Problem**:

- Cluster configuration hardcoded in `cluster.rs` module (970 lines containing partitions, QoS, memory limits, core limits, billing rates)
- Infrastructure exists for profile switching (lazy_static ACTIVE_CLUSTER with RwLock)
- If cluster admin renames partition (e.g., "amilan" → "amilan-ucb") or changes limits, app breaks
- User has no way to fix without code changes and rebuilding

**Current Architecture** (as of Phase 7 complete):

- **Cluster Module** (`src-tauri/src/cluster.rs`): Single source of truth for cluster configuration
  - `alpine_profile()` - Returns hardcoded Alpine cluster configuration
  - `get_cluster_capabilities()` - Returns capabilities for frontend (already has #[tauri::command])
  - `get_partition_limits()`, `calculate_job_cost()`, `suggest_qos()`, `estimate_queue_time()` - Business logic functions
  - Direct registration: Commands call cluster module directly (no wrapper layer after cleanup)
- **NO** `commands/cluster.rs` file exists (thin wrappers deleted during code cleanup)
- **Validation Module** (`src-tauri/src/validation/job_validation.rs`): Resource validation using cluster config
- **Frontend Store** (`src/lib/stores/clusterConfig.ts`): Caches capabilities, invokes cluster commands directly

**Implementation**:

- [ ] **Database Schema** (`cluster_settings` table - key/value store pattern):
  - [ ] `key TEXT PRIMARY KEY` - Setting identifier (e.g., 'billing_rate_cpu', 'partition_amilan_max_cores')
  - [ ] `value TEXT NOT NULL` - Setting value (JSON for complex types)
  - [ ] `updated_at TEXT NOT NULL` - Timestamp
  - [ ] Alternative: Structured tables (`cluster_partitions`, `cluster_qos`) - decide based on query patterns
  - [ ] Initialize table with current hardcoded values on first run via migration
  - [ ] Schema supports multiple cluster profiles (store profile_id in key, e.g., 'alpine.billing_rate_cpu')

- [ ] **Backend Implementation**:
  - [ ] Keep `cluster.rs` as business logic module (calculations, validations)
  - [ ] Add database loading: `load_cluster_settings_from_db()` called during app initialization
  - [ ] Merge DB settings with hardcoded defaults: DB overrides hardcoded values
  - [ ] Add Tauri commands (register directly in cluster.rs with #[tauri::command]):
    - [ ] `get_editable_cluster_config()` - Returns current configuration for Settings UI
    - [ ] `update_partition(partition_config)` - Updates single partition in DB
    - [ ] `update_qos(qos_config)` - Updates single QoS option in DB
    - [ ] `update_billing_rates(cpu_rate, gpu_rate)` - Updates billing configuration
    - [ ] `reset_cluster_config_to_defaults()` - Clears DB settings, reverts to hardcoded defaults
  - [ ] **NO** `commands/cluster.rs` file - register commands directly from cluster module
  - [ ] Update `alpine_profile()` to merge DB settings with hardcoded defaults on load

- [ ] **Settings Page UI** (extend existing SettingsPage.svelte):
  - [ ] Add "Cluster Configuration" section with tabs
  - [ ] **Partitions Tab**: Card-based list of partitions with inline edit capability
    - [ ] Each partition card: name, description, max cores, max memory, max walltime, default QoS
    - [ ] Edit button opens inline form with validation
    - [ ] Save button calls `update_partition()` command
  - [ ] **QoS Options Tab**: Card-based list with inline edit
    - [ ] Each QoS card: name, description, compatible partitions (chips/tags display)
    - [ ] Edit form with multi-select for compatible partitions
  - [ ] **Billing Rates Tab**: Simple form for CPU/GPU rates
  - [ ] **Reset to Defaults** button with ConfirmDialog (uses existing Dialog.svelte primitive)
  - [ ] Use existing design system (namd-button classes, CSS variables)

- [ ] **Migration Strategy**:
  - [ ] Database migration in `database/mod.rs`: Create `cluster_settings` table if not exists
  - [ ] On app startup in `main.rs`: If table empty, populate with `alpine_profile()` values
  - [ ] Load settings from DB → merge with defaults → set active profile in ACTIVE_CLUSTER
  - [ ] Existing validation and calculation functions automatically use updated values (no code changes needed)

**Why**: Cluster admins change configuration over time. User-editable settings prevent app breakage and eliminate need for code changes/rebuilds.

**Architecture Notes**:

- Cluster module remains single source of truth for business logic
- Database provides persistence layer for user customizations
- No thin wrapper layer - Settings UI invokes cluster commands directly
- Hardcoded defaults serve as fallback when DB settings not present
- Profile switching infrastructure already in place via lazy_static RwLock

### Milestone 8.2: App/Module Discovery and Management

**Goal**: Allow users to search for cluster applications (e.g., NAMD), discover prerequisites, and pin specific versions for use in jobs. Replace hardcoded module versions in SLURM scripts with user-selected configurations.

**Current Problem**:

- Module versions hardcoded in job script generation (namd/3.0.1_cpu, gcc/14.2.0, openmpi/5.0.6)
- If cluster admin updates NAMD or changes prerequisite versions, all jobs fail
- User must know correct module names and load order (requires SSH + manual `module spider` commands)
- No support for multiple NAMD versions (e.g., CPU-optimized vs GPU-optimized)

**Implementation**:

**Backend - Module Discovery** (`commands/cluster_apps.rs`):

- [ ] `search_cluster_apps(search_term: string) -> ApiResult<Vec<AppSearchResult>>`
  - [ ] Run `module avail <search_term> 2>&1` via SSH
  - [ ] Parse output to extract available module names/versions
  - [ ] Return list: `[{ name: "namd", version: "3.0.1_cpu", full_name: "namd/3.0.1_cpu" }, ...]`

- [ ] `pin_cluster_app(module_name: string) -> ApiResult<PinnedApp>`
  - [ ] Run `module spider <module_name>` to get immediate prerequisites
  - [ ] **Recursively** run `module spider` on each prerequisite to build full dependency tree
  - [ ] Build directed acyclic graph (DAG) of all dependencies
  - [ ] Perform topological sort to determine correct load order (deepest dependencies first)
  - [ ] Detect circular dependencies (return error if found)
  - [ ] Validate all modules in chain exist on cluster
  - [ ] Store in DB: app_name, version, full_module_name, load_order (array), dependency_tree (JSON)
  - [ ] Return pinned app data to UI

- [ ] `get_pinned_apps() -> ApiResult<Vec<PinnedApp>>`
  - [ ] Return all pinned apps from DB for display in UI and job creation

- [ ] `unpin_cluster_app(id: string) -> ApiResult<()>`
  - [ ] Delete pinned app from DB
  - [ ] Check if any jobs reference this app (warn user but allow deletion)

- [ ] `validate_pinned_app(id: string) -> ApiResult<ValidationResult>`
  - [ ] Test that `module purge && module load <load_order>` succeeds
  - [ ] Useful for user to verify configuration after cluster changes

**Backend - Module Parsing Utilities** (`ssh/module_parser.rs`):

- [ ] Parse `module avail` output (format varies by cluster, needs robust regex)
- [ ] Parse `module spider` output to extract dependencies
- [ ] Handle edge cases: missing modules, version conflicts, circular deps
- [ ] Unit tests for various output formats

**Database Schema** (`pinned_apps` table):

- [ ] `id` TEXT PRIMARY KEY
- [ ] `app_name` TEXT NOT NULL (e.g., "namd")
- [ ] `version` TEXT NOT NULL (e.g., "3.0.1_cpu")
- [ ] `full_module_name` TEXT NOT NULL (e.g., "namd/3.0.1_cpu")
- [ ] `display_name` TEXT (user-friendly label, e.g., "NAMD 3.0.1 (CPU-optimized)")
- [ ] `load_order` TEXT NOT NULL (JSON array: `["gcc/14.2.0", "openmpi/5.0.6", "namd/3.0.1_cpu"]`)
- [ ] `dependency_tree` TEXT (JSON for full DAG, useful for debugging)
- [ ] `pinned_at` TEXT (timestamp)
- [ ] `last_validated_at` TEXT (timestamp, nullable)

**Job Metadata Breaking Change**:

- [ ] Update `JobInfo` struct to include `app_module` field:

  ```rust
  pub struct JobInfo {
      // ... existing fields
      pub app_module: AppModule,
  }

  pub struct AppModule {
      pub app_name: String,
      pub version: String,
      pub full_module_name: String,
      pub display_name: String,
  }
  ```

- [ ] Update `job_info.json` format on cluster to include app_module
- [ ] Update SLURM script generation to use `load_order` from selected pinned app

**Settings Page UI** (new "Applications" section):

- [ ] **App Search**:
  - [ ] Search input field: "Search for cluster applications (e.g., 'namd', 'gcc', 'python')"
  - [ ] Search button triggers `search_cluster_apps()` command
  - [ ] Display results in list with "Pin" button for each result
  - [ ] Show loading state during search (SSH command can take 1-2 seconds)

- [ ] **Pinned Apps List**:
  - [ ] Card-based display of all pinned apps
  - [ ] Each card shows: display_name, full_module_name, load_order preview, pinned date
  - [ ] Actions: Edit display name, Validate (test load order), Unpin (delete)
  - [ ] Expand card to show full dependency tree and load order
  - [ ] "Validate" button runs SSH test and shows success/failure

- [ ] **Pin Dialog** (when user clicks "Pin" on search result):
  - [ ] Show loading: "Discovering prerequisites for <module>..."
  - [ ] On success: Show full load order, prompt for display name
  - [ ] On error: Show which module failed (e.g., "Prerequisite gcc/14.2.0 not found")
  - [ ] Confirm button saves to DB

**Job Creation Integration** (`CreateJobPage.svelte` - Resources tab):

- [ ] Add "Application" dropdown below partition/QoS selection
- [ ] Load pinned apps from DB on page mount
- [ ] Dropdown options: display_name (e.g., "NAMD 3.0.1 (CPU-optimized)")
- [ ] Default to first pinned app or show "No applications pinned" message
- [ ] If no apps pinned, show link to Settings page
- [ ] Store selected app ID in job draft state

**Job Details Integration** (`JobDetailPage.svelte` - Overview tab):

- [ ] Display app_module information in metadata section
- [ ] Show: "Application: NAMD 3.0.1 (CPU-optimized) [namd/3.0.1_cpu]"
- [ ] Show full load order in expandable section

**Implementation Notes**:

- **No auto-refresh**: User manually searches and re-pins when cluster changes (acceptable workflow)
- **Discovery is expensive**: SSH commands take time, only run on explicit user action
- **Multiple pinned apps expected**: Users may pin CPU and GPU versions, or different major versions
- **Validation optional but recommended**: User can test pinned app still works after cluster changes
- **Circular dependency handling**: Detect and reject during pin operation (rare but possible)

**Why**: Cluster admins update software versions regularly. User-managed module configuration eliminates hardcoded dependencies and allows app to adapt to cluster changes without code modifications.

### Phase 8 Complete When

- **Milestone 8.1**: ✅ User can edit cluster configuration (partitions, QoS, limits) via Settings page
- **Milestone 8.2**: ✅ User can search, pin, and manage cluster applications with automatic prerequisite discovery
- **All jobs use user-selected apps with dynamic module loading**
- **Zero hardcoded cluster or module configuration remaining in codebase**

## Future Work: Post-Phase 8 Enhancements

**Note**: The following features are planned for future development after Phase 8 completes. Priorities and implementation details will be determined based on user feedback and actual usage patterns.

### Phase 6.9: Production Readiness (Deferred)

- [ ] Git Action x86 Windows executable build
- [ ] Git Action x86 Linux executable build
- [ ] Installation documentation
- [ ] User guide (template-based job workflow)
- [ ] Deployment pipeline
- [ ] Final documentation completeness check and cleanup

### Request Rate Limiting & Queue Management (Future)

**Goal:** Prevent cluster abuse and provide graceful degradation under load

**Current State:** Mutex serialization provides implicit rate limiting (one request at a time). Single SSH connection physically prevents parallel spam. Adequate for current usage.

**When Needed:** If users report accidental DOS of cluster or app becomes unresponsive under load

**Approach:** Token bucket rate limiter wrapping existing ConnectionManager mutex, request deduplication, queue depth limits

### Job Chaining / Multi-Stage Workflows (Future)

**Note**: Design in progress at [tasks/planning/MultiJob_And_Templates.md](tasks/planning/MultiJob_And_Templates.md)

**Core Concept**: "Restart after timeout" and "next equilibration stage" are the same mechanism - creating new job that continues from parent job's outputs. Jobs are self-contained islands (each copies necessary files from parent).

**When Needed:** Users need to run multi-stage simulations (minimization → equilibration → production) or restart jobs that hit walltime limits

**Approach:** Parent-child job relationships, file propagation system, chain visualization UI

### Multi-Cluster Support (Future)

**Goal:** Support users with accounts on multiple clusters

**Dependencies:** Phase 8.1 cluster configuration must be complete first

**Approach:**

- Multiple cluster profiles in `cluster_configs` table
- Profile switcher in connection UI
- Profile-specific pinned apps
- Migration of connection management to support profile selection

### Automation Builder (Future)

**Goal:** Visual workflow designer for complex job automation patterns

**Dependencies:** Builds on existing Phase 6 automation framework

**Approach:**

- Serializable automation steps (already implemented in Rust)
- Drag-and-drop workflow canvas
- Automation template library
- Parameter sweep automation
- Community template marketplace

### UI/UX Enhancements (Future)

- Bulk operations (multi-select job management)
- Advanced filtering/search (by status, date, resources, templates)
- User preferences (default values, UI behavior)
- Export/import jobs and templates
- Job comparison and diff tools

## Risk Mitigation

1. **MVP Focus**: Get single-job workflow working before adding complexity
2. **No Backwards Compatibility**: Breaking changes acceptable during all development phases
3. **Security**: Regular security audits, never log credentials, path validation
4. **Windows Compatibility**: Test early and often on Windows
5. **Design Before Code**: Finalize designs (like job chaining) before implementation to avoid rework

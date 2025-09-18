# NAMDRunner Development Roadmap

**🏗️ Architecture Updates**: When completing milestones, always update `docs/architecture.md` to reflect the actual implementation. Architecture doc describes what IS built, this roadmap describes what WILL be built.

## Development Strategy
Build a **single-job MVP first** that handles the core workflow: create job → submit → track → view results. 

**Key Design Decision**: Job persistence and discovery are built from Phase 2, not added later. This ensures:
- Test jobs persist between development sessions for proper testing
- No accumulation of "phantom" test data on cluster
- Status sync works from first job submission
- Developers can easily manage and clean up test jobs

## Current Status: Phase 3 Complete - Ready for Phase 4 (SLURM Integration)

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

### Milestone 2.1: SSH/SFTP Implementation
- [ ] Password authentication with ssh2 crate  
- [ ] SFTP file upload/download operations
- [ ] Module loading commands (`module load slurm/alpine`)
- [ ] SSH connection debugging and error recovery
- [ ] Real connection establishment and management

### Milestone 2.2: Data Layer & Job Persistence
- [ ] SQLite integration with rusqlite
- [ ] Job cache schema (jobs table with metadata)
- [ ] JSON metadata file format (job.json on remote)
- [ ] Job discovery from existing remote JSON files
- [ ] Session persistence (close/reopen shows existing jobs)
- [ ] SLURM status sync (update job statuses from cluster)

### Milestone 2.3: Phase 2 Cleanup & Refactoring  
- [ ] Run Phase 2 code review using `.claude/agents/review-refactor.md` agent
- [ ] Review SSH/SFTP implementation for consistency and patterns
- [ ] Eliminate any duplicated connection handling code discovered
- [ ] Validate database schema and access patterns for optimization
- [ ] Ensure proper error handling across all backend operations
- [ ] Document SSH connection patterns and lessons learned

## Phase 3: Frontend Development ✅ COMPLETED
*Complete UI implementation with comprehensive refactoring cleanup*
*User interface implementation based on React mockup*

### Milestone 3.1: Design System & Layout Components ✅ COMPLETED
- [x] Import CSS custom properties and theme configuration
- [x] Build main application shell (sidebar, header, content area)
- [x] Implement navigation state management
- [x] Create breadcrumb navigation system
- [x] Build collapsible SSH console panel

### Milestone 3.2: Jobs Management Interface ✅ COMPLETED
- [x] Jobs list page with sortable table
- [x] Job status badges and indicators
- [x] Job detail view with tabbed interface
- [x] Sync controls and status updates
- [x] Job selection and row interactions

### Milestone 3.3: Job Creation Workflow ✅ COMPLETED
- [x] Multi-section creation form
- [x] SLURM resource allocation interface
- [x] File upload with drag & drop
- [x] NAMD parameter configuration
- [x] Form validation and error display

### Milestone 3.4: Connection UI & Polish ✅ COMPLETED
- [x] Enhanced connection status dropdown (matching mockup)
- [x] Updated connection dialog with proper styling
- [x] Loading states and transitions
- [x] Dark theme support
- [x] Complete UI testing suite (pending integration)

### Milestone 3.5: Phase 3 Cleanup & Refactoring ✅ COMPLETED
- [x] Run Phase 3 code review using `.claude/agents/review-refactor.md` agent
- [x] Review UI components for consistency, reusability, and duplication
- [x] Consolidate similar form validation patterns discovered (FormField component)
- [x] Ensure accessible design patterns throughout application
- [x] Validate proper error display and user feedback patterns
- [x] Document UI component patterns and establish design system

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
- SSH connection works with password
- Files upload/download via SFTP
- SQLite stores and retrieves job data
- App reopening shows previously created jobs
- Job discovery from remote JSON files works
- Basic SLURM status sync functional

### Phase 3 Complete When: ✅ ALL ACHIEVED
- [x] UI visually matches React mockup screenshots
- [x] Full navigation between Jobs, Job Detail, and Create Job views works
- [x] All forms validate input with proper error display
- [x] Light/dark themes both functional
- [x] Mock data enables complete UI workflow testing
- [x] UI tests capture screenshots for visual validation
- [x] **BONUS**: Comprehensive refactoring cleanup completed (300+ lines CSS eliminated, utilities centralized)

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

### Settings Page
*Centralized configuration management for cluster-specific settings*
- [ ] Configurable cluster connection settings (login server, port)
- [ ] Customizable SLURM partitions list
- [ ] Configurable QOS options
- [ ] Module versions configuration (gcc, cuda, namd versions)
- [ ] Resource limits and defaults per partition
- [ ] Alpine-specific settings management
- [ ] User preferences (default values, UI behavior)
- [ ] Export/import settings

### Templates System
*Template-based job creation with variable substitution*
- [ ] Templates management page for creating and editing NAMD templates
- [ ] Variable definition language with syntax like `{{var_name}}`
- [ ] Template comment syntax for variable metadata:
  - [ ] Variable tooltips and descriptions
  - [ ] Validation rules (number ranges, required fields, file types)
  - [ ] Default values and suggestions
- [ ] Template library with common NAMD workflows
- [ ] Template dropdown selector in Create Job page
- [ ] Auto-population of form fields from template variables
- [ ] File upload mapping to template variables (PDB, PSF, parameter files)
- [ ] Template versioning and sharing capabilities
- [ ] Preview of generated NAMD configuration before submission
- [ ] Export/import templates

### Mock Mode
*Offline development and demo capabilities*
- [ ] Mock mode toggle in connection dropdown
- [ ] Pre-populated fake job data for demos
- [ ] Simulated job submission without server connection
- [ ] Mock job state transitions for testing
- [ ] Fake SLURM output generation
- [ ] No actual file uploads in mock mode
- [ ] Useful for UI development and user training

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
- [ ] Enhanced error recovery workflows
- [ ] Storage usage monitoring and alerts

## Risk Mitigation

1. **MVP Focus**: Get single-job workflow working before adding complexity
2. **Future-Ready Architecture**: Design data models to support job groups later
3. **Security**: Regular security audits, never log credentials
4. **Windows Compatibility**: Test early and often on Windows
# Phase 6.4: Frontend-Backend Integration Implementation Prompt

## Project Overview
You are implementing **Phase 6.4: Frontend-Backend Integration** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **critical bug fix and refactoring phase** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), using a proven Python/CustomTkinter implementation as reference.

## Your Mission: Fix Broken Integration & Achieve True Frontend-Backend Separation
Implement **comprehensive frontend-backend integration fixes** with core capabilities:
- Fix Create Job UI to actually call backend (currently stub implementation with setTimeout)
- Implement comprehensive SSH/SFTP console logging for operational visibility
- Add job discovery functionality to rebuild database from server metadata
- Move ALL business logic (validation, cluster config, calculations) from TypeScript to Rust backend
- Remove ~1,000 lines of orphaned/dead code (services, test-only methods, unused commands)
- Ensure single backend call per user action (eliminate double round-trips)
- Achieve true frontend-backend separation: frontend is thin UI layer, backend contains all business rules

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - **WHAT** we're building (business requirements, system design)
- `docs/CONTRIBUTING.md` - **HOW** to build it (development setup, testing, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 6.4 scope and milestones** (your roadmap)

- `docs/AUTOMATIONS.md` - Job lifecycle automation chains and progress tracking patterns

### 3. Reference Implementation Knowledge
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/DB.md` - Database schema and JSON metadata formats
- `docs/API.md` - IPC interfaces and command specifications
- `docs/SSH.md` - Connection management, security patterns, and file operations

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy and workflows (3-tier architecture)
- `docs/CONTRIBUTING.md#anti-patterns-to-avoid` - Critical patterns to avoid (includes Phase 6.4 learnings)
- `.claude/agents/review-refactor.md` - Code review agent for quality checks after implementation
- `docs/DESIGN.md` - UI/UX patterns and component architecture

## 🎯 Phase 6.4 Success Criteria

### Critical Priority: Blockers (Do This First)
- [ ] **Business logic migration**: All validation, cluster config, and calculations moved from TypeScript (428 lines) to Rust backend
- [ ] **Create Job fixed**: Button creates real jobs with files uploaded to server (no more setTimeout stub)
- [ ] **SSH/SFTP logging**: Comprehensive console logging for all operations (connection, file transfers, SLURM commands)
- [ ] **Job discovery**: Can rebuild local database from server metadata when database is empty

### High Priority: Core Functionality
- [ ] **Dead code removed**: addJob(), updateJobStatus(), removeJob() methods deleted from job store
- [ ] **Delete job fixed**: Uses correct deleteJob() method with remote deletion and connection check
- [ ] **Double backend calls eliminated**: Single call per user action (no createJob → getAllJobs pattern)
- [ ] **File downloads implemented**: Real SFTP downloads replace alert() stubs
- [ ] **Sync results working**: "Sync Results from Scratch" button functional

### Medium Priority: Enhancements
- [ ] **Orphaned services deleted**: ~700 lines of service layer removed (never imported by production code)
- [ ] **Job detail tabs use real data**: No more hardcoded mockStdout/mockStderr/mockInputFiles constants
- [ ] **Console.log hijacking removed**: Proper Tauri event listeners instead of global console manipulation
- [ ] **Calculation functions migrated**: Cost, estimation, parsing logic moved to backend
- [ ] **Unused backend commands cleaned**: Delete commands never invoked from frontend

### Low Priority: Polish
- [ ] **CoreClient simplified**: 3 files → 2 files, delete unused sub-interfaces
- [ ] **File type classification**: Move to backend (currently frontend logic)
- [ ] **Status display logic**: Evaluate if business logic or UI-only concern

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ Backend automation chains (job_creation.rs, job_submission.rs, job_completion.rs) - Complete and tested
- ✅ Logging infrastructure (src-tauri/src/logging.rs) - info_log!, debug_log!, error_log! macros proven in connection.rs
- ✅ CoreClient abstraction (IPC boundary) - TauriCoreClient and MockCoreClient working correctly
- ✅ Mode switching (demo vs real) - CoreClientFactory.getUserMode() pattern established
- ✅ SSH/SFTP operations (ssh/manager.rs, ssh/sftp.rs) - Working, just needs logging added
- ✅ Database layer (database/mod.rs) - SQLite operations functional

**What's Missing (Implement This)**:
- ❌ Create Job never calls backend - handleSubmit() uses setTimeout and mock job creation
- ❌ SSH console has zero logging - no visibility into file transfers, SLURM commands, connection events
- ❌ Job discovery doesn't exist - can't rebuild database from server metadata
- ❌ Business logic in frontend - 428 lines of validation and cluster config in TypeScript
- ❌ Double backend calls - createJob() success triggers getAllJobs() (two round-trips)
- ❌ Dead code everywhere - ~1,000 lines of orphaned services, test-only methods, unused commands

### 2. Investigation Commands (Run These First)
```bash
# Check current Create Job stub implementation
cd /media/share/namdrunner
rg "handleSubmit" src/lib/components/pages/CreateJobPage.svelte -A 20

# Look at existing logging patterns
rg "info_log!|debug_log!|error_log!" src-tauri/src/ssh/connection.rs -A 2

# Check cluster config in frontend (should be in backend)
head -n 50 src/lib/data/cluster-config.ts

# Verify backend validation is minimal
rg "validate" src-tauri/src/validation/job_validation.rs -A 5

# Check for orphaned service imports
rg "from.*services/" src/lib/ --type ts
```

**Expected Finding**: Create Job uses setTimeout mock, validation mostly in frontend, services never imported.

### 3. Reference-Driven Development
- **Start with proven patterns** from existing automation chains (job_creation.rs for file upload patterns)
- **Use established logging** from connection.rs (info_log! for operations, error_log! for failures)
- **Follow validation patterns** from Python reference (comprehensive checks before submission)
- **Learn from database patterns** in DB.md (job_info.json schema for discovery)
- **Improve and modernize** with Rust type safety and async patterns

### 4. Implementation Strategy Order
**Step 1: Business Logic Migration (Foundation)**
- Create cluster_config.rs module with Rust constants (partition limits, QOS specs, costs)
- Expand job_validation.rs with comprehensive validation (resources, partitions, files, NAMD params)
- Add get_cluster_capabilities() Tauri command for frontend
- Update CreateJobTabs.svelte to call backend validation
- Test: Backend enforces all rules, frontend displays errors only

**Step 2: Fix Critical Bugs (Core Functionality)**
- Fix Create Job handleSubmit() to call jobsStore.createJob() with real params
- Add confirmation dialog before creation ("files upload but won't submit yet")
- Implement SSH/SFTP logging in manager.rs, sftp.rs, and automation chains
- Fix Delete Job to use deleteJob() with deleteRemote: true
- Add warning dialog before deletion ("permanent deletion of local + remote")
- Test: Real jobs created, SSH console shows all operations, deletions work correctly

**Step 3: Job Discovery & Optimization**
- Implement discover_jobs_from_server() backend function
- Integrate with sync: trigger when database has 0 jobs on manual "Sync Now"
- Eliminate double backend calls (return job from create_job instead of getAllJobs)
- Implement file downloads (replace alert stubs)
- Test: Discovery rebuilds database, single call per action, downloads work

**Step 4: Dead Code Removal & Cleanup**
- Verify dead code with grep searches (see verification process in task plan)
- Delete orphaned services (~700 lines): ssh.ts, sftp.ts, pathResolver.ts, serviceContainer.ts
- Delete dead store methods: addJob(), updateJobStatus(), removeJob(), mockConnected()
- Delete tests that only test dead code (test-only code is dead code)
- Clean unused backend commands
- Test: App works after removal, no import errors

**Step 5: Polish & Documentation**
- Fix Job Tabs to use real data (mockJobs for demo mode, backend for real mode)
- Remove console.log hijacking (use proper Tauri events)
- Simplify CoreClient structure (3 files → 2 files)
- Move calculation functions to backend
- Run review-refactor agent for final quality check
- Update ALL documentation to match code changes
- Test: Everything works, docs accurate

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Verify environment
npm ci
cargo check

# Development workflow
npm run dev              # Svelte dev server
npm run test            # Vitest unit tests (takes 15-20s to start jsdom)
cargo test              # Rust unit tests
npm run tauri dev       # Full Tauri app

# Quality checks
cargo clippy            # Rust linting (deny warnings)
npm run lint            # TypeScript/Svelte linting
npm run lint:fix        # Auto-fix linting issues
```

## 🧭 Implementation Guidance

### Integration Points
```rust
// Existing functions to enhance (don't rebuild)
create_directory() in ssh/manager.rs       // Add: info_log! for directory creation
upload_file() in ssh/manager.rs            // Add: info_log! with file size and paths
execute_command() in ssh/manager.rs        // Add: debug_log! for command text and output
create_job() in automations/job_creation.rs // Add: logging throughout automation chain

// New functions to add (follow established patterns)
get_cluster_capabilities() in cluster_config.rs  // Purpose: Expose config to frontend
discover_jobs_from_server() in commands/jobs.rs  // Purpose: Rebuild DB from server
validate_job_comprehensive() in validation/      // Purpose: All business rule validation
```

### Key Technical Decisions Already Made
- **Cluster config storage** - Use Rust constants (easy to migrate to database in Phase 8.1 settings)
- **Job discovery trigger** - Only when database has 0 jobs on manual "Sync Now" (performance optimization)
- **Double backend call fix** - Return created job from backend, don't call getAllJobs() after
- **Test-only code deletion** - If only tests use it, delete both code AND tests (they test code users never execute)
- **Validation source** - Cluster capabilities and validation use same config source (single source of truth)

### Architecture Patterns to Follow
- **Logging pattern** - Use info_log!, debug_log!, error_log! macros from logging.rs (see connection.rs examples)
- **Automation pattern** - Progress callbacks, event emission, error handling from existing job_creation.rs
- **Mode switching pattern** - CoreClientFactory.getUserMode() for demo vs real (see SyncControls.svelte)
- **Validation pattern** - Backend returns detailed errors with field names, frontend displays only
- **IPC pattern** - All backend calls through CoreClientFactory.getClient() for proper abstraction

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Never log or persist SSH passwords (use SecStr, clear on disconnect)
- Validate all user inputs before shell commands (use shell::build_command_safely)
- Prevent directory traversal (sanitize file paths with validation functions)
- Clear credentials from memory on disconnect
- No credential logging in SSH console (mask passwords in connection logs)

### Quality Requirements
- Comprehensive unit test coverage: Backend validation logic, cluster config, job discovery parsing
- Type safety across IPC boundary: Proper serde attributes, TypeScript type alignment
- Proper error handling: Result<T> patterns, anyhow context, user-friendly messages
- Follow coding standards: No thin wrappers, no stub implementations, business logic in backend only
- Connection-aware UI: Disable destructive actions when disconnected (delete, sync, downloads)
- Confirmation dialogs: Warn before permanent deletions, inform about multi-step operations

### Integration Requirements
- Build on existing automation infrastructure (job_creation.rs, job_submission.rs patterns)
- Use established logging macros (info_log!, debug_log!, error_log!)
- Follow CoreClient abstraction (all backend calls through factory)
- Respect offline-first design (local SQLite cache, manual sync)
- Match established demo mode patterns (CoreClientFactory.getUserMode())

## 🤝 Getting Help

### When You Need Guidance
- **Automation questions** - Check `docs/AUTOMATIONS.md` for job lifecycle patterns
- **Logging questions** - See `src-tauri/src/ssh/connection.rs` for established patterns
- **Validation questions** - Review Python reference for comprehensive validation examples
- **Data format questions** - See `docs/DB.md` for job_info.json schema
- **Architecture decisions** - Review `docs/ARCHITECTURE.md` and ask for input before major changes

### Communication Protocol
- **Present your plan** before starting each priority section (get approval first)
- **Ask specific questions** about patterns, not "how do I..." general queries
- **Update progress** in task plan as you complete each checklist item
- **Run review-refactor agent** before marking phase complete

## 🎯 Your First Steps

1. **Read all required documentation** listed above (especially the task plan!)
2. **Run investigation commands** to verify current state matches task plan description
3. **Review existing patterns** in connection.rs (logging), job_creation.rs (automation), cluster-config.ts (what to port)
4. **Start with Critical Priority** - Business logic migration is foundation for everything else
5. **Get approval for approach** after reading docs but before writing significant code
6. **Update task plan** as you complete each checklist item

## Success Metrics
- Phase 6.4 deliverables completed per `tasks/active/phase-6-4-frontend-backend-integration.md`
- All existing tests continue passing (don't break working features)
- New functionality has comprehensive test coverage (3-tier: frontend/backend/integration)
- Type-safe integration with existing systems (no any types, proper serde)
- Create Job works end-to-end (form → backend validation → file upload → database)
- SSH console provides useful operational visibility (all operations logged)
- True frontend-backend separation achieved (all business logic in Rust)
- ~1,000 lines of dead code removed (verified unused before deletion)
- Documentation completely updated to match code (no references to deleted code)

## Task Management (CRITICAL)
- **Work from the active task plan** at `tasks/active/phase-6-4-frontend-backend-integration.md`
- **Follow priority order** - Critical → High → Medium → Low (dependencies exist)
- **Update progress** - Check off items as you complete them
- **Verify before deletion** - Follow verification process in task plan (grep searches, confirm only tests use it)
- **Get approval** for approaches that deviate from task plan guidance
- **Run review-refactor agent** before completion - `.claude/agents/review-refactor.md`
- **Update ALL docs** - ARCHITECTURE.md, API.md, AUTOMATIONS.md, CONTRIBUTING.md (see completion checklist)

## Important Notes

### No Backwards Compatibility Needed
This application has NOT been deployed yet. Freely:
- Remove dead code and unused methods
- Rename confusing methods to clearer names
- Simplify interfaces
- Present the correct, final, production-ready version

### Verification Process for Code Deletion
Before deleting ANY code:
1. Search production code for usage (grep commands in task plan)
2. Search test files for usage
3. If ONLY tests use it, delete both code AND tests (test-only code is dead code)
4. Verify Phase 6.4 tasks don't need it
5. Document verification results before deletion

### Key Principle: Test Production Code Paths
"Being used by a test" is NOT the same as "being used by the app." If only tests use a method, both the method and its tests should be deleted. Tests should validate production code paths - the code that users actually execute.

Remember: This phase fixes broken integration and achieves true separation. **Frontend becomes thin UI layer, backend contains all business rules.** Leverage established patterns (logging, automation, validation) and integrate seamlessly rather than creating parallel implementations.

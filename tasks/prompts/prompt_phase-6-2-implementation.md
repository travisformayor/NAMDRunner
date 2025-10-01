# Phase 6.2 Automation Chain Verification Implementation Prompt

## Project Overview
You are implementing **Phase 6.2: Automation Chain Verification** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **verification and completion task** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), building on the automation architecture foundation established in Phase 6.1.

## Your Mission: Complete and Verify All Job Lifecycle Automation Chains
Verify and implement complete automation chains for the entire NAMDRunner job lifecycle with core capabilities:
- **Job Creation**: ✅ Verify existing automation creates project directories with integrated file upload
- **Job Submission**: ❌ Implement missing automation to create scratch directories and submit to SLURM
- **Status Synchronization**: ✅ Verify existing status sync commands work correctly
- **Job Completion**: ❌ Implement new automation to preserve results from scratch to project directory
- **Job Cleanup**: ✅ Verify existing cleanup automation has proper security validation

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - **WHAT** we're building (business requirements, system design)
- `docs/CONTRIBUTING.md` - **HOW** to build it (development setup, testing, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 6.2 scope and milestones** (your roadmap)
- `docs/AUTOMATIONS.md` - **Complete automation architecture** with all automation chains documented
- `tasks/active/phase-6-2-automation-verification.md` - Complete implementation requirements and constraints
- `tasks/completed/phase-6-1-ui-backend-integration.md` - Foundation automation architecture context

### 3. Reference Implementation Knowledge
- `docs/reference/slurm-commands-reference.md` - **Working SLURM patterns** and command reference
- `docs/reference/namd-commands-reference.md` - NAMD execution patterns and templates
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/DB.md` - Database schema and JSON metadata formats

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy and workflows
- `tasks/templates/task.md` - Use this template for task planning
- `docs/reference/agent-development-tools.md` - Available tools and testing infrastructure

## 🎯 Phase 6.2 Success Criteria

### Job Creation Automation: Verification (Do This First)
- [ ] Verify job creation only creates project directories (not scratch)
- [ ] Validate integrated file upload works atomically during creation
- [ ] Test progress tracking events emit correctly to UI
- [ ] Confirm database persistence saves job with "CREATED" status

### Job Submission Automation: Implementation
- [ ] Implement `execute_job_submission_with_progress` automation function
- [ ] Create scratch directories during submission process
- [ ] Copy project files to scratch directory
- [ ] Generate and submit SLURM job script with proper parameters

### Status Synchronization: Verification
- [ ] Test `sync_job_status` and `sync_all_jobs` commands work correctly
- [ ] Verify status transitions update database with timestamps
- [ ] Validate error handling for unsubmitted jobs
- [ ] Confirm SLURM query integration functions properly

### Job Completion Automation: Implementation
- [ ] Implement `execute_job_completion_with_progress` automation function
- [ ] Detect completed jobs and preserve critical output files
- [ ] Copy results from scratch to permanent project directory
- [ ] Structure results in `/projects/$USER/namdrunner_jobs/{job_id}/results/`

### Job Cleanup Automation: Verification
- [ ] Test safe directory deletion with security validation
- [ ] Verify both project and scratch directory cleanup
- [ ] Confirm path traversal protection works correctly
- [ ] Validate database cleanup integration

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ **Job Creation Automation** - `src-tauri/src/automations/job_creation.rs` with progress callbacks
- ✅ **Status Sync Commands** - `src-tauri/src/commands/jobs.rs` sync_job_status/sync_all_jobs functions
- ✅ **Job Cleanup Logic** - `src-tauri/src/commands/jobs.rs` delete_job_real with security validation
- ✅ **SSH/SFTP Infrastructure** - Complete connection management and file operations
- ✅ **Database Operations** - SQLite integration with job persistence

**What's Missing (Implement This)**:
- ❌ **Job Submission Automation** - Need `execute_job_submission_with_progress` function
- ❌ **Job Completion Automation** - Need `execute_job_completion_with_progress` function
- ❌ **Automation Module Structure** - Need proper module organization for new automation functions

### 2. Investigation Commands (Run These First)
```bash
# Check current automation implementation
cd /media/share/namdrunner/src-tauri
rg "execute_job_creation_with_progress" src/ -A 10 -B 2

# Look at existing job submission logic
rg "submit_job_real|sbatch" src/ -A 15

# Check status sync implementation
rg "sync_job_status|SlurmStatusSync" src/ -A 10

# Check cleanup implementation
rg "delete_job_real" src/ -A 20
```

**Expected Finding**: Job creation automation exists, submission/completion automations need implementation, status sync and cleanup exist but need verification

### 3. Reference-Driven Development
- **Start with proven patterns** from `src-tauri/src/automations/job_creation.rs`
- **Use established progress callback pattern** with Tauri event emission
- **Follow security validation patterns** from existing job commands
- **Build on existing SSH/SFTP operations** without duplicating infrastructure

### 4. Implementation Strategy Order
**Step 1: Verification of Existing Automations**
- Verify job creation automation workflow separation
- Test status synchronization commands
- Validate job cleanup security protections

**Step 2: Job Submission Automation Implementation**
- Create `src-tauri/src/automations/job_submission.rs`
- Implement scratch directory creation and file copying
- Integrate SLURM job submission with progress tracking

**Step 3: Job Completion Automation Implementation**
- Create `src-tauri/src/automations/job_completion.rs`
- Implement result file preservation logic
- Add automatic completion detection

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Verify environment
npm ci
cargo check

# Development workflow
npm run tauri dev       # Full Tauri app for testing
cargo test              # Rust unit tests
npm run test            # Frontend tests

# Quality checks
cargo clippy            # Rust linting
npm run lint            # TypeScript/Svelte linting
```

## 🧭 Implementation Guidance

### Integration Points
```rust
// Existing functions to enhance (verify these work correctly)
src-tauri/src/automations/job_creation.rs::execute_job_creation_with_progress() // Verify: proper workflow separation
src-tauri/src/commands/jobs.rs::sync_job_status() // Verify: status update workflow
src-tauri/src/commands/jobs.rs::delete_job_real() // Verify: security validation completeness

// New functions to add (follow established patterns)
src-tauri/src/automations/job_submission.rs::execute_job_submission_with_progress() // Purpose: submit job with scratch setup
src-tauri/src/automations/job_completion.rs::execute_job_completion_with_progress() // Purpose: preserve results after completion
```

### Key Technical Decisions Already Made
- **Simple Async Functions** - Use direct async functions with progress callbacks, not complex traits
- **Tauri Event Emission** - Direct `app_handle.emit()` calls for real-time UI progress updates
- **Workflow Separation** - Creation handles project setup, submission handles execution, completion handles results
- **Security First** - Multiple validation layers, path safety, input sanitization throughout

### Architecture Patterns to Follow
- **Progress Callback Pattern** - `move |msg| { let _ = handle_clone.emit("event-name", msg); }`
- **Error Handling Pattern** - Use `anyhow::Result` with comprehensive error messages
- **Database Integration** - Use `with_database(|db| ...)` pattern for SQLite operations
- **SSH Operations** - Use `get_connection_manager()` with existing retry logic

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Never log or persist SSH passwords
- Clear credentials from memory on disconnect
- Use minimal Tauri permissions
- Validate all user inputs with `sanitize_job_id()` and path safety checks
- Prevent directory traversal attacks with multiple validation layers

### Quality Requirements
- Follow simple async function pattern established in job creation automation
- Provide comprehensive progress tracking for all operations
- Maintain atomic operations - either fully succeed or cleanly fail
- Follow coding standards in `docs/CONTRIBUTING.md#developer-standards--project-philosophy`
- Integration with existing error handling and retry logic

### Integration Requirements
- Build on existing automation architecture from Phase 6.1
- Use established SSH/SFTP infrastructure without duplication
- Follow existing database patterns and job status management
- Respect workflow separation boundaries (creation → submission → monitoring → completion → cleanup)

## 🤝 Getting Help

### When You Need Guidance
- **Automation questions** - Check `docs/AUTOMATIONS.md` and existing `job_creation.rs` implementation
- **SLURM integration questions** - See `docs/reference/slurm-commands-reference.md`
- **File operations questions** - Review existing SSH/SFTP patterns in codebase
- **Database questions** - Look at `docs/DB.md` and existing database operations

### Communication Protocol
- **Present your verification plan** before starting major implementation work
- **Ask specific questions** about automation patterns or integration points
- **Update docs** as you implement and learn
- **Share progress updates** with concrete examples of working automation chains

## 🎯 Your First Steps

1. **Read all required documentation** listed above, especially `docs/AUTOMATIONS.md`
2. **Run investigation commands** to understand current automation state
3. **Verify existing automations** work correctly with proper workflow separation
4. **Implement missing automations** following established patterns
5. **Test complete end-to-end workflow** from creation through cleanup
6. **Document any changes** made during verification and implementation

## Success Metrics
- All 5 automation chains (creation, submission, status sync, completion, cleanup) working correctly
- Complete job lifecycle verified end-to-end in both demo and real modes
- Progress tracking provides clear user feedback for all automation operations
- Workflow separation maintains proper boundaries between automation stages
- All existing tests continue passing with new automation functions tested
- Security validation prevents malicious input at all automation steps
- Clean, maintainable code following established automation patterns

## Task Management (CRITICAL)
- **Work from existing task file** - `tasks/active/phase-6-2-automation-verification.md`
- **Update progress** as you verify and implement each automation chain
- **Update AUTOMATIONS.md** if implementation details change during verification
- **Get approval** for implementation approach for missing automation functions

Remember: This builds on the automation architecture foundation established in Phase 6.1. **Verify existing patterns work correctly** and **implement missing automation functions** following the same simple async function approach with progress callbacks.
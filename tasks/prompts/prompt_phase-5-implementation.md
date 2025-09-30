# Phase 5 File Operations Implementation Prompt for Implementation Engineer

## Project Overview
You are implementing **Phase 5: File Operations & Results Management** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **backend infrastructure completion** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), using a proven Python/CustomTkinter implementation as reference.

## Your Mission: Complete File Operations Backend
Implement **real SFTP file operations** to replace mock implementations with core capabilities:
- Real file upload/download operations via SFTP
- Directory listing and results browsing backend
- Log file aggregation from SLURM and NAMD
- Job cleanup with remote directory removal
- Complete backend infrastructure ready for UI integration

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/project-spec.md` - **WHAT** we're building (business requirements)
- `docs/technical-spec.md` - **HOW** to build it (architecture, tech stack, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 5 scope and milestones** (your roadmap)
- `docs/ARCHITECTURE.md` - Implementation progress tracker (update as you build)
- `tasks/active/phase5-file-operations-results-management.md` - Complete implementation requirements and constraints

### 3. Reference Implementation Knowledge
- `docs/reference/slurm-commands-reference.md` - **Working SLURM patterns** and file path structures
- `docs/reference/namd-commands-reference.md` - NAMD file organization and output patterns
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/data-spec.md` - File metadata formats and directory organization standards

### 4. Development Support
- `docs/testing-spec.md` - Testing strategy, debugging workflows, CI setup
- `tasks/templates/task.md` - Use this template for task planning
- `docs/agent-capabilities.md` - Available tools and testing infrastructure

## 🎯 Phase 5 Success Criteria

### Milestone 5.1: Real File Upload Implementation (Do This First)
- [ ] Convert `upload_job_files` from mock to real SFTP operations
- [ ] Implement progress tracking for large file uploads
- [ ] Add file validation before upload (PDB, PSF, parameter files)
- [ ] Handle upload failures with retry logic using existing ConnectionUtils

### Milestone 5.2: Real File Download & Results Management
- [ ] Convert `download_job_output` from mock to real SFTP operations
- [ ] Implement directory listing via SFTP for results browsing
- [ ] Download SLURM output files (.out, .err) from scratch directory
- [ ] Download NAMD output files (.dcd, .log, checkpoint files)

### Milestone 5.3: Job Cleanup & Lifecycle Completion
- [ ] Implement job deletion with remote directory cleanup
- [ ] Clean up both project and scratch directories safely
- [ ] Preserve important results before cleanup (optional download)
- [ ] Robust error handling for all SFTP operations

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ SSH/SFTP infrastructure with retry logic - `src/ssh/sftp.rs`, `src/ssh/manager.rs`
- ✅ Job lifecycle management with database persistence - `src/commands/jobs.rs`
- ✅ SLURM integration and job submission - `src/slurm/` modules
- ✅ Mock file operations providing API contracts - `src/commands/files.rs`
- ✅ Directory management (project + scratch) - Phase 2 implementation
- ✅ ConnectionUtils retry mechanisms - established patterns

**What's Missing (Implement This)**:
- ❌ Real SFTP file upload replacing mock `upload_job_files` function
- ❌ Real SFTP file download replacing mock `download_job_output` function
- ❌ Directory listing via SFTP for results browsing backend
- ❌ Log file aggregation from remote SLURM and NAMD logs
- ❌ Job cleanup with remote directory removal functionality

### 2. Investigation Commands (Run These First)
```bash
# Check existing SFTP infrastructure and patterns
cd /media/share/namdrunner-backend/src-tauri
rg "sftp|SFTP" src/ -A 5 -B 5

# Look at current mock file operations to understand interface
rg "upload_job_files|download_job_output" src/commands/files.rs -A 15

# Check how existing SSH operations work
rg "ConnectionManager|sftp" src/ssh/ -A 10

# Examine retry logic patterns to follow
rg "ConnectionUtils|retry" src/ -A 5
```

**Expected Finding**: Mock file operations exist with proper interfaces, established SFTP infrastructure available, retry patterns implemented.

### 3. Reference-Driven Development
- **Start with proven patterns** from `docs/reference/slurm-commands-reference.md` for file paths
- **Use established data formats** from `docs/data-spec.md` for file organization
- **Learn from Python lessons** in `docs/reference/python-implementation-reference.md` for file workflows
- **Improve and modernize** with existing Rust SSH/SFTP infrastructure

### 4. Implementation Strategy Order
**Step 1: File Upload Foundation**
- Replace mock `upload_job_files` with real SFTP operations
- Integrate with existing ConnectionManager and retry logic
- Add file validation and progress tracking capabilities

**Step 2: File Download & Directory Operations**
- Replace mock `download_job_output` with real SFTP downloads
- Implement directory listing for results browsing backend
- Add log file aggregation from SLURM and NAMD outputs

**Step 3: Job Cleanup & Integration**
- Implement safe remote directory cleanup for job deletion
- Integrate file operations with existing job lifecycle
- Comprehensive error handling and testing validation

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner-backend

# Verify environment
npm ci
cargo check

# Development workflow
npm run dev              # Svelte dev server
npm run test            # Vitest unit tests
cargo test              # Rust unit tests
npm run tauri dev       # Full Tauri app

# Quality checks
cargo clippy            # Rust linting
npm run lint            # TypeScript/Svelte linting
```

## 🧭 Implementation Guidance

### Integration Points
```rust
// Existing functions to enhance (don't rebuild)
upload_job_files() // Replace: mock implementation with real SFTP operations using ConnectionManager
download_job_output() // Replace: mock content generation with real SFTP file retrieval
submit_job_real() // Enhance: integrate file copying from project to scratch directory

// New functions to add (follow established patterns)
list_job_files() // Purpose: SFTP directory listing for results browsing backend
aggregate_job_logs() // Purpose: collect SLURM .out/.err and NAMD logs via SFTP
cleanup_job_directories() // Purpose: safe remote directory cleanup using existing patterns
```

### Key Technical Decisions Already Made
- **SFTP Infrastructure** - Use existing SSH/SFTP patterns from Phase 2, don't create parallel systems
- **Directory Structure** - `/projects/$USER/namdrunner_jobs/$JOB_ID/` and `/scratch/alpine/$USER/namdrunner_jobs/$JOB_ID/` established
- **Retry Logic** - Use established ConnectionUtils patterns for network interruption recovery
- **Error Handling** - Follow existing error classification and user-friendly message patterns

### Architecture Patterns to Follow
- **ConnectionManager integration** - All SFTP operations go through established connection management
- **Retry pattern consistency** - Use ConnectionUtils wrapper for all remote operations
- **Error handling standardization** - Follow existing error mapping and classification patterns
- **Security validation reuse** - Apply established path sanitization and validation functions

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Never log or persist SSH passwords
- Clear credentials from memory on disconnect
- Use minimal Tauri permissions
- Validate all file paths and prevent directory traversal
- Apply existing path sanitization functions for all file operations

### Quality Requirements
- Comprehensive unit test coverage with mocked SFTP operations
- Type safety across the entire IPC boundary
- Proper error handling and user feedback for file operations
- Follow coding standards in `docs/technical-spec.md`
- Integration with existing retry and error handling patterns

### Integration Requirements
- Build on existing SSH/SFTP infrastructure without duplication
- Integrate with established job lifecycle management
- Follow established ConnectionManager and retry logic patterns
- Respect existing directory management and path security constraints

## 🤝 Getting Help

### When You Need Guidance
- **SFTP integration questions** - Check existing `src/ssh/sftp.rs` patterns first
- **File operation questions** - See current mock implementations in `src/commands/files.rs`
- **Architecture decisions** - Review `docs/ARCHITECTURE.md` and ask for input
- **Python implementation questions** - Look at `docs/reference/python-implementation-reference.md`

### Communication Protocol
- **Present your plan** before starting major implementation work
- **Ask specific questions** rather than general "how do I..." queries
- **Update docs** as you learn and implement
- **Share progress updates** with concrete examples of file operations working

## 🎯 Your First Steps

1. **Read all required documentation** listed above
2. **Run investigation commands** to understand current SFTP infrastructure
3. **Analyze mock file operations** in `src/commands/files.rs` to understand interfaces
4. **Examine existing SFTP patterns** in `src/ssh/` to understand integration approach
5. **Plan your implementation strategy** for replacing mocks with real operations
6. **Get approval for your approach** before writing significant code

## Success Metrics
- Phase 5 deliverables completed per `tasks/roadmap.md`
- All existing tests continue passing with new file operations
- Real file upload/download working reliably via SFTP
- Directory listing and results browsing backend functional
- Log file aggregation working (SLURM + NAMD logs accessible)
- Job deletion with remote cleanup working safely
- No code duplication - seamless integration with existing SSH infrastructure
- Clean, maintainable code following project standards

## Task Management (CRITICAL)
- **Work from active task** `tasks/active/phase5-file-operations-results-management.md`
- **Update ARCHITECTURE.md** as you implement each file operation component
- **Test thoroughly** with both mocked and real SFTP operations
- **Get approval** for your implementation approach before major coding

Remember: This builds on established NAMDRunner SSH/SFTP infrastructure. **Leverage proven patterns** and **integrate seamlessly** with existing ConnectionManager, retry logic, and error handling rather than creating parallel file operation systems.

Good luck! 🚀
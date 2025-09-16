# Phase 4 SLURM Integration Implementation Prompt

## Task Overview
Complete Phase 4 by implementing real SLURM job submission with batch script generation. This builds on the existing job lifecycle, status tracking, and SSH infrastructure to enable actual job execution on SLURM clusters.

## Current State Analysis
**What's Already Working (Don't Rebuild)**:
- ✅ Job status tracking with `squeue`/`sacct` integration (`src/slurm/status.rs`)
- ✅ Manual sync commands (`sync_job_status`, `sync_all_jobs`) functional
- ✅ Job lifecycle management with database persistence
- ✅ SSH/SFTP infrastructure with retry logic and error handling
- ✅ Directory management (project + scratch directories)
- ✅ Mock job submission flow in `submit_job_real()` function

**What's Missing (Implement This)**:
- ❌ SLURM batch script generation from NAMD/SLURM config
- ❌ Real `sbatch` execution (currently returns mock job IDs)
- ❌ File transfer from project → scratch before submission
- ❌ Proper error handling for SLURM submission failures

## Implementation Strategy

### 1. SLURM Script Template System
**Goal**: Generate valid SLURM batch scripts from job configuration

**Approach**:
- Create template-based system for NAMD job scripts
- Use variable substitution for job-specific parameters
- Focus on security (prevent script injection)
- Build on existing validation patterns

**Key Files to Reference**:
- `docs/reference/slurm-commands-reference.md` - SLURM sbatch syntax and options
- `docs/reference/namd-commands-reference.md` - NAMD execution commands
- `docs/reference/python-implementation-reference.md` - proven template patterns

### 2. Real Job Submission Integration
**Goal**: Replace mock submission with actual sbatch execution

**Approach**:
- Enhance existing `submit_job_real()` in `src/commands/jobs.rs`
- Build on existing SSH command execution patterns
- Integrate with current retry logic and error handling
- Maintain compatibility with existing job lifecycle

**Integration Points**:
```rust
// Current function to enhance (don't rewrite completely)
submit_job_real(job_id: String) -> SubmitJobResult {
    // Existing: directory creation, job loading from DB
    // ADD: script generation, file transfer, real sbatch execution
    // Keep: database updates, error handling patterns
}
```

### 3. File Transfer Workflow
**Goal**: Move input files to scratch directory before job execution

**Approach**:
- Use existing SFTP operations from SSH infrastructure
- Copy files from project directory to scratch directory
- Validate all required files present before submission
- Follow established directory management patterns

## Specific Implementation Tasks

### Task 1: Script Generation Module
Create `src/slurm/script_generator.rs`:
```rust
pub struct SlurmScriptGenerator;

impl SlurmScriptGenerator {
    pub fn generate_namd_script(job_info: &JobInfo) -> anyhow::Result<String> {
        // Template-based generation with security validation
    }
}
```

**Security Requirements**:
- Sanitize all user inputs before template substitution
- Validate generated script for injection attempts
- Use existing input validation patterns from `src/validation/`

### Task 2: Enhanced Job Submission
Modify `submit_job_real()` in `src/commands/jobs.rs`:
```rust
async fn submit_job_real(job_id: String) -> SubmitJobResult {
    // EXISTING: Load job, create scratch directory ✅

    // NEW: Generate SLURM script
    let script_content = SlurmScriptGenerator::generate_namd_script(&job_info)?;

    // NEW: Upload script to scratch directory
    let script_path = format!("{}/job.sbatch", scratch_dir);
    connection_manager.upload_content(&script_content, &script_path).await?;

    // NEW: Copy input files to scratch
    copy_input_files(&job_info, &connection_manager).await?;

    // NEW: Execute real sbatch command
    let sbatch_cmd = format!("sbatch {}", script_path);
    let result = connection_manager.execute_command(&sbatch_cmd, Some(30)).await?;

    // NEW: Parse real SLURM job ID (build on existing parsing logic)
    let slurm_job_id = parse_sbatch_output(&result.stdout)?;

    // EXISTING: Update database ✅
}
```

### Task 3: File Transfer Integration
Add to `src/commands/jobs.rs`:
```rust
async fn copy_input_files(
    job_info: &JobInfo,
    connection_manager: &ConnectionManager
) -> anyhow::Result<()> {
    // Use existing SFTP operations to copy files
    // Follow established retry and error handling patterns
}
```

### Task 4: Error Handling Enhancement
Extend existing error handling to cover SLURM-specific failures:
- Map SLURM error messages to user-friendly explanations
- Integrate with existing retry logic for transient failures
- Validate resource requirements against partition limits

## Success Validation

### Functional Tests
1. **End-to-End Workflow**: Create job → Submit → Track to completion
2. **Error Scenarios**: Handle submission failures gracefully
3. **File Management**: Input files properly transferred before execution

### Technical Integration
1. **Existing Tests Pass**: No regression in current functionality
2. **SSH Integration**: Build on established connection patterns
3. **Database Consistency**: Job state properly tracked through submission

### Security Validation
1. **Script Injection Prevention**: Generated scripts safe from malicious input
2. **Path Security**: Follow existing directory traversal protection
3. **Input Validation**: Use established sanitization patterns

## Key Architectural Principles

### Build on Existing Patterns
- **Don't rebuild SSH infrastructure** - use existing ConnectionManager
- **Don't recreate job lifecycle** - enhance existing submission flow
- **Don't duplicate validation** - extend current security patterns

### Follow Established Conventions
- **Error handling**: Use existing anyhow::Result patterns
- **Retry logic**: Build on ConnectionManager retry infrastructure
- **Database operations**: Use existing JobRepository patterns
- **Testing approach**: Follow NAMDRunner testing philosophy

### Maintain Security Standards
- **Input sanitization**: Use existing validation functions
- **Command execution**: Follow established SSH command patterns
- **File operations**: Use existing SFTP security measures

## Expected Deliverables

1. **Script Generation System** - Secure template-based SLURM script creation
2. **Real Job Submission** - Actual sbatch execution with proper error handling
3. **File Transfer Integration** - Seamless input file management
4. **Enhanced Error Mapping** - User-friendly SLURM error explanations
5. **Comprehensive Testing** - Unit tests for new components, integration validation

## Post-Implementation
After completing the implementation:
1. Run code review with `.claude/agents/review-refactor.md`
2. Implement any simplification recommendations
3. Update `docs/architecture.md` with SLURM integration details
4. Mark Phase 4 complete in `tasks/roadmap.md`
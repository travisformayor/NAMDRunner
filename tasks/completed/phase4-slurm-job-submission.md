# Task: Complete Phase 4 SLURM Integration - Real Job Submission

## Objective
Enable users to submit NAMD molecular dynamics jobs to real SLURM clusters with generated batch scripts and track their execution through completion.

## Context
- **Starting state**: Mock job submission working with fake SLURM job IDs, complete status tracking system, job lifecycle management, and database persistence
- **Delivered state**: Real SLURM batch script generation and submission, actual job execution on cluster, full end-to-end workflow from job creation to completion
- **Foundation**: Phase 2 SSH/SFTP infrastructure, job status synchronization (`sync_job_status`, `sync_all_jobs`), database persistence, retry logic, and secure command execution
- **Dependencies**: SSH connection working, job directory management operational, status tracking functional
- **Testing approach**: Aligned with NAMDRunner philosophy - business logic unit tests with mocked SLURM commands, integration tests with real cluster when available, security tests for script injection prevention

## Implementation Plan

### Critical Priority (Blockers)

- [ ] **SLURM Batch Script Generation**
  - [ ] Design NAMD script template system with variable substitution
  - [ ] Implement script generation from JobInfo (NAMD config, SLURM config, file paths)
  - [ ] Add security validation for script content (prevent injection attacks)
  - [ ] Create script upload to project directory via existing SFTP infrastructure

- [ ] **Real SLURM Job Submission**
  - [ ] Replace mock submission in `submit_job_real()` with actual sbatch execution
  - [ ] Integrate with existing SSH command execution and retry patterns
  - [ ] Parse real SLURM job IDs from sbatch output (build on existing parsing logic)
  - [ ] Handle SLURM submission errors with proper error classification

### High Priority (Core Functionality)

- [ ] **File Transfer Integration**
  - [ ] Copy input files from project directory to scratch directory before submission
  - [ ] Integrate with existing directory management and SFTP operations
  - [ ] Validate all required files present before job submission

- [ ] **Enhanced Error Handling**
  - [ ] Map SLURM-specific errors (quota exceeded, invalid partition, etc.) to user-friendly messages
  - [ ] Integrate with existing retry logic for transient failures
  - [ ] Add submission validation (check resources against partition limits)

### Medium Priority (Enhancements)

- [ ] **Script Template Optimization**
  - [ ] Support multiple NAMD versions (namd2 vs namd3) based on cluster detection
  - [ ] Add configurable module loading (build on existing module load patterns)
  - [ ] Template validation and syntax checking

- [ ] **Job Monitoring Enhancements**
  - [ ] Improve status sync frequency for recently submitted jobs
  - [ ] Add submission timestamp tracking and queue time calculation

## Success Criteria

### Functional Success
- [ ] User can create job, submit to real SLURM cluster, and track execution to completion
- [ ] Generated SLURM scripts are valid and execute NAMD jobs correctly
- [ ] Job status updates reflect real cluster state (PENDING → RUNNING → COMPLETED/FAILED)

### Technical Success
- [ ] SLURM script generation follows cluster security requirements (no injection vulnerabilities)
- [ ] Integration with existing SSH/SFTP infrastructure seamless (no code duplication)
- [ ] Error handling provides actionable feedback for submission failures

### Quality Success
- [ ] All existing tests continue passing with new submission logic
- [ ] New SLURM script generation has comprehensive unit tests with mocked cluster responses
- [ ] Security tests validate script injection prevention

## Key Technical Decisions

### Why Template-Based Script Generation
- **Reasoning**: Provides flexibility for different NAMD job types while maintaining security through controlled variable substitution
- **Alternatives considered**: Hardcoded script strings (not flexible), complex script builders (over-engineered)
- **Trade-offs**: Simple templates vs. advanced features - optimizing for maintainability and security

### Why Integrate with Existing Job Lifecycle
- **Reasoning**: Leverages battle-tested directory management, status sync, and database persistence
- **Integration**: Enhance existing `submit_job_real()` function rather than creating parallel submission path
- **Benefits**: Maintains consistency with established patterns and error handling

### Why Build on Existing SSH Infrastructure
- **Reasoning**: SSH command execution, retry logic, and SFTP operations already working and tested
- **Alternatives considered**: New SLURM-specific connection handling (duplicates functionality)
- **Trade-offs**: Reuse vs. custom optimization - prioritizing reliability and consistency

## Integration with Existing Code

### Leverage Existing Patterns
- **Use SSH ConnectionManager**: Build script upload and sbatch execution on existing command patterns
- **Follow JobRepository patterns**: Extend existing job persistence for submission metadata
- **Apply existing validation**: Use established input sanitization and path security

### Where to Hook In
```rust
// Existing functions to enhance (don't rebuild)
submit_job_real() // Add: actual SLURM script generation and sbatch execution
create_job_real() // Add: validate SLURM config against templates

// New functions to add (follow established patterns)
generate_slurm_script() // Create batch script from JobInfo using template system
upload_and_submit_job() // Upload script and execute sbatch with existing retry logic
validate_slurm_config() // Check resource requirements against cluster limits
```

## References
- **NAMDRunner patterns**: `docs/architecture.md` for SSH integration, `docs/testing-spec.md` for test approach
- **Implementation files**:
  - `src/commands/jobs.rs` - submit_job_real() function to enhance
  - `src/slurm/` - existing command builders and status parsing
  - `src/ssh/manager.rs` - command execution patterns to follow
- **Specific docs**:
  - `docs/reference/slurm-commands-reference.md` - SLURM command patterns and sbatch usage
  - `docs/reference/namd-commands-reference.md` - NAMD execution patterns and script structure
  - `docs/reference/python-implementation-reference.md` - proven job submission workflow
- **Python reference**: NAMDRunner Python implementation shows working NAMD script templates and submission patterns

## Progress Log

**2025-01-15** - **Phase 4 Implementation Complete ✅**

### ✅ Completed Work
1. **SLURM Script Generation Module** - Created `src/slurm/script_generator.rs`
   - Template-based SLURM batch script generation from JobInfo
   - NAMD configuration file generation with proper parameters
   - Comprehensive validation for job parameters and resource limits
   - Security validation prevents dangerous configurations

2. **Real Job Submission** - Enhanced `submit_job_real()` in `src/commands/jobs.rs`
   - Real sbatch execution replacing mock job IDs
   - Integrated with existing SSH command execution and retry patterns
   - Proper SLURM job ID parsing from sbatch output
   - Complete error handling for submission failures

3. **File Transfer Workflow** - Added file management functions
   - `copy_input_files()` - Copy files from project to scratch directory
   - `upload_content()` - Upload generated scripts using SSH commands
   - Uses existing SSH infrastructure, no additional dependencies

4. **Enhanced Error Handling** - Comprehensive error management
   - SLURM-specific error detection and user-friendly messages
   - Integration with existing retry logic for transient failures
   - Validation errors provide actionable feedback

### 🎯 Technical Implementation Details
- **Built on existing patterns** - No code duplication, leverages SSH/SFTP infrastructure
- **Simple dependencies** - Used built-in string parsing, no external regex/base64 crates
- **Security focused** - Input validation, safe script generation, path security
- **Comprehensive testing** - Unit tests for script generation and validation logic

## Completion Process
After implementation and testing:
- [x] Run code review using `.claude/agents/review-refactor.md` - **COMPLETED 2025-01-15**
- [x] Implement recommended refactoring improvements - **COMPLETED 2025-01-15**
- [x] Update `tasks/roadmap.md` to mark Phase 4 complete - **COMPLETED 2025-01-15**
- [x] Update `docs/architecture.md` with SLURM integration implementation details - **COMPLETED 2025-01-15**
- [x] Archive task to `tasks/completed/phase4-slurm-job-submission.md` - **COMPLETED NOW**

## Final Status: ✅ PHASE 4 COMPLETE
**Completion Date**: 2025-01-15
**Next Phase**: Phase 5 - File Operations & Results Management

## Open Questions
- [ ] What NAMD script template variables are required for the target cluster environment?
- [ ] Should we support multiple template formats or start with a single general-purpose template?
- [ ] What cluster-specific module loading patterns need to be configurable vs hardcoded?
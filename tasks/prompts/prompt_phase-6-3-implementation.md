# Phase 6.3 Implementation Prompt for Code Quality & Refactoring Engineer

## Project Overview
You are implementing **Phase 6.3: Code Quality & Refactoring** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **critical refactoring phase** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), addressing security vulnerabilities and code quality issues identified by comprehensive review.

## Your Mission: Production-Quality Code Refinement
Implement **critical security fixes and code quality improvements** with core capabilities:
- Fix shell command injection vulnerabilities through proper parameter escaping
- Add database transaction safety for atomic operations
- Eliminate thin wrapper anti-patterns and code duplication
- Centralize validation logic and remove hardcoded values

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - System design and current implementation state
- `docs/CONTRIBUTING.md#developer-standards--project-philosophy` - Anti-patterns to avoid and coding standards
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 6.3 scope** (Code Quality & Refactoring requirements)
- `tasks/active/phase-6-3-code-quality-refactoring.md` - Complete refactoring requirements with specific line numbers
- Review-refactor agent output - Detailed analysis of issues with locations and priorities

### 3. Security & Validation References
- `docs/SSH.md` - Security patterns for shell command construction
- `docs/CONTRIBUTING.md#security-requirements` - Security requirements and patterns
- `docs/AUTOMATIONS.md` - Automation architecture showing where fixes are needed

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy for validation
- `.claude/agents/review-refactor.md` - Review agent for verification
- `src-tauri/src/validation.rs` - Existing validation module to extend

## 🎯 Phase 6.3 Success Criteria

### Critical Priority: Security & Data Integrity (Do This First)
- [ ] All shell commands use proper parameter escaping - no raw `format!()` concatenation
- [ ] Database operations wrapped in transactions for atomicity
- [ ] Security tests pass for malicious input scenarios
- [ ] Transaction rollback tests verify data consistency

### High Priority: Code Quality
- [ ] Thin wrapper `job_cleanup.rs` deleted entirely
- [ ] File validation centralized in `validation::files` module
- [ ] SLURM module paths made configurable
- [ ] No duplicated validation logic remains

### Medium Priority: Consistency
- [ ] Error handling standardized across automations
- [ ] Progress reporting uses structured data
- [ ] Minor issues fixed (test assertions, path wrappers)

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Break)**:
- ✅ All automation chains functional (creation, submission, completion, cleanup)
- ✅ SSH/SFTP operations with retry logic
- ✅ Database persistence and job tracking
- ✅ Path validation in `validation::paths` module

**Critical Issues to Fix (From Review)**:
- ❌ Shell commands built with raw `format!()` - Lines identified in task document
- ❌ Missing database transactions for atomic operations
- ❌ Thin wrapper in `job_cleanup.rs` violating project philosophy
- ❌ Duplicated file validation logic

### 2. Investigation Commands (Run These First)
```bash
# Find all shell command construction patterns
cd /media/share/namdrunner
rg 'format!\(' src-tauri/src/automations/ -A 2 -B 2

# Check current validation module structure
rg 'pub fn' src-tauri/src/validation.rs

# Find database operations needing transactions
rg 'insert|update|delete' src-tauri/src/database/ -A 5

# Check for other thin wrappers
rg 'pub async fn.*\n.*\{$\n.*await' src-tauri/src/ -U
```

**Expected Finding**: Multiple unsafe shell commands, no transaction support, one thin wrapper

### 3. Reference-Driven Development
- **Start with SSH security patterns** from `docs/SSH.md`
- **Use validation module patterns** from existing `validation::paths`
- **Follow Result<T> patterns** for consistent error handling
- **Apply project philosophy** from `docs/CONTRIBUTING.md`

### 4. Implementation Strategy Order
**Step 1: Critical Security Fixes**
- Create `validation::shell` module with safe command builders
- Add `escape_parameter()` and `build_command_safely()` functions
- Update all automation modules to use safe builders
- Add security tests for command injection prevention

**Step 2: Database Transaction Safety**
- Implement transaction support in `JobDatabase`
- Add transaction guard with automatic rollback
- Wrap all atomic operations in transactions
- Add rollback tests for failure scenarios

**Step 3: Code Quality Improvements**
- Delete `job_cleanup.rs` and update references
- Centralize file validation in `validation::files`
- Make module paths configurable
- Standardize error handling

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Run existing tests to ensure nothing broken
cargo test
npm run test

# Development workflow
npm run tauri dev       # Test changes in full app

# After implementation
cargo test              # Verify all tests pass
cargo clippy           # Check for any new issues
```

## 🧭 Implementation Guidance

### Integration Points
```rust
// New modules to create
validation::shell::escape_parameter() // Proper shell escaping
validation::shell::build_command_safely() // Safe command construction
validation::shell::safe_cp() // Helper for cp commands
validation::shell::safe_find() // Helper for find commands
validation::files::validate_upload() // Centralized file validation

// Existing functions to enhance
JobDatabase::begin_transaction() // Add transaction support
JobDatabase::commit() // Commit transaction
JobDatabase::rollback() // Rollback on error

// Files to modify (with specific lines from review)
src-tauri/src/automations/job_completion.rs:78 // Fix find command
src-tauri/src/automations/job_completion.rs:92 // Fix cp command
src-tauri/src/automations/job_submission.rs:73 // Fix cp command
src-tauri/src/automations/job_submission.rs:96 // Fix cd && sbatch

// Files to delete
src-tauri/src/automations/job_cleanup.rs // Remove entirely
```

### Key Technical Decisions Already Made
- **Shell safety over convenience** - Always escape parameters, no exceptions
- **Atomic operations required** - Database consistency is critical
- **No thin wrappers** - Project philosophy is explicit about this
- **Centralization over duplication** - Single source of truth for validation

### Architecture Patterns to Follow
- **Validation module pattern** - Extend existing `validation` module structure
- **Result<T> error handling** - Consistent with rest of codebase
- **Transaction guard pattern** - Auto-rollback on drop for safety
- **Helper function pattern** - Common operations get dedicated safe helpers

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Every shell parameter must be escaped
- No string concatenation for shell commands
- Validate all paths before operations
- Test with malicious inputs (../../../etc/passwd, etc.)
- Database operations must be atomic

### Quality Requirements
- Zero thin wrappers remaining
- No code duplication for validation
- Comprehensive security tests
- Transaction rollback tests
- Follow anti-pattern guidelines from `docs/CONTRIBUTING.md`

### Integration Requirements
- Build on existing validation module structure
- Maintain compatibility with all automation chains
- Preserve existing functionality while fixing issues
- Keep error messages user-friendly

## 🤝 Getting Help

### When You Need Guidance
- **Shell escaping questions** - Check `docs/SSH.md` security patterns
- **Transaction patterns** - Look at SQLite transaction documentation
- **Validation patterns** - Review existing `validation::paths` module
- **Anti-patterns** - See `docs/CONTRIBUTING.md#developer-standards--project-philosophy`

### Communication Protocol
- **Show specific fixes** with before/after code examples
- **Run security tests** to verify fixes work
- **Update documentation** as patterns are established
- **Get review** using review-refactor agent after completion

## 🎯 Your First Steps

1. **Read the review output** in task document for specific issues
2. **Create shell safety module** in validation with proper escaping
3. **Fix critical security issues** in automation modules
4. **Add transaction support** to database module
5. **Delete thin wrapper** and centralize validation
6. **Run comprehensive tests** including new security tests

## Success Metrics
- All shell commands use safe construction - no raw format!()
- Database operations are atomic with proper rollback
- Thin wrapper eliminated (~50 lines removed)
- File validation centralized (~100 lines consolidated)
- Security tests pass for malicious inputs
- Transaction tests verify rollback behavior
- Code review shows no remaining anti-patterns
- ~10-15% overall code reduction from cleanup

## Task Management (CRITICAL)
- **Active task exists** at `tasks/active/phase-6-3-code-quality-refactoring.md`
- **Track progress** by updating the task document
- **Run review-refactor agent** after implementation for verification
- **Update ARCHITECTURE.md** with refactoring details when complete
- **Archive task** to completed folder after finishing

Remember: This is about **security and quality**, not features. Fix the identified issues systematically, test thoroughly, and ensure no regression in functionality. The automation chains must continue working perfectly after refactoring.
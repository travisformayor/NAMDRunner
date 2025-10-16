# Phase 6.7 Comprehensive Testing Implementation Prompt

## Project Overview
You are implementing **Phase 6.7: Comprehensive Testing** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **testing and validation phase** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), focusing on production readiness through comprehensive test coverage and autonomous UI validation.

## Your Mission: Achieve Production-Ready Testing Coverage
Implement **comprehensive testing infrastructure** with core capabilities:
- >80% unit test coverage focused on business logic (not external libraries)
- Autonomous UI testing framework using Playwright for self-validation in demo mode
- Enhanced test infrastructure for continuous integration and reliability
- Performance and security validation testing

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - **WHAT** we're building (business requirements, system design)
- `docs/CONTRIBUTING.md` - **HOW** to build it (development setup, testing, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 6 scope and milestones** (your roadmap)
- `docs/ARCHITECTURE.md` - Implementation progress tracker (update as you build)
- `tasks/active/phase-6-4-comprehensive-testing.md` - Complete implementation requirements and constraints

### 3. Critical Testing Requirements (MUST READ)
- `docs/CONTRIBUTING.md#testing-strategy` - **REQUIRED READING** - NAMDRunner testing philosophy that defines what to test vs what NOT to test
- `docs/reference/agent-development-tools.md` - **REQUIRED READING** - Autonomous UI testing capabilities, Playwright usage, mock infrastructure, and headless browser configuration

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy and workflows
- `tasks/templates/task.md` - Use this template for task planning
- `docs/reference/agent-development-tools.md` - Available tools and testing infrastructure

## 🎯 Phase 6.7 Success Criteria

### Testing Infrastructure: Unit Test Coverage (Do This First)
- [ ] Achieve >80% unit test coverage for `src-tauri/src/validation.rs` (security, shell safety, file validation)
- [ ] Database transaction testing in `src-tauri/src/database/mod.rs` (rollback scenarios, atomic operations)
- [ ] Automation workflow testing in `src-tauri/src/automations/*.rs` (business logic only, no server calls)
- [ ] SLURM command generation testing in `src-tauri/src/slurm/commands.rs` (template generation, parameter escaping)
- [ ] Frontend business logic testing (`src/lib/stores/jobs.ts`, `src/lib/utils/`, form validation)

### Interactive UI Testing: Autonomous Validation Framework
- [ ] Implement Playwright-based autonomous UI testing for demo mode navigation and workflows
- [ ] Create headless testing framework with automated clicking, form filling, and visual validation
- [ ] Develop complete job lifecycle testing in demo mode (create → submit → track → view results)
- [ ] Implement visual regression testing with screenshot capture and validation

### Test Infrastructure: Automation & CI Integration
- [ ] Enhance CI test pipeline with automated execution, coverage reporting, and performance testing
- [ ] Create deterministic test fixtures and scenarios with proper data management
- [ ] Implement performance and reliability testing (startup time, memory usage, error recovery)

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ Basic test infrastructure with Vitest and Rust testing - Existing foundation in `tests/` directory
- ✅ Mock data and fixtures - Available in `src/lib/test/fixtures/` for UI development
- ✅ Demo mode functionality - Working demo mode toggle for safe testing without server connections
- ✅ Clean, refactored codebase from Phase 6.3 - Security and quality improvements completed

**What's Missing (Implement This)**:
- ❌ Comprehensive unit test coverage (currently low coverage across critical modules)
- ❌ Autonomous UI testing framework for self-validation
- ❌ Performance and security validation testing
- ❌ Test coverage reporting and enforcement

### 2. Investigation Commands (Run These First)
```bash
# Check current test coverage
cd /media/share/namdrunner
cargo test --no-run  # Check Rust test compilation
npm run test         # Run existing Vitest tests

# Examine existing test infrastructure
rg "test|spec" tests/ src/ -A 5 -B 5
rg "#\[cfg\(test\)\]|#\[test\]" src-tauri/src/ -A 10

# Check UI testing capabilities
ls tests/ui/
rg "playwright|browser" tests/ -A 5

# Examine validation module (priority testing target)
rg "validation::" src-tauri/src/ -A 5
```

**Expected Finding**: Limited test coverage with basic infrastructure but missing comprehensive business logic testing

### 3. Reference-Driven Development
- **Follow testing philosophy** from `docs/CONTRIBUTING.md#testing-strategy` - Test our logic, not external libraries
- **Use autonomous UI testing patterns** from `docs/reference/agent-development-tools.md`
- **Build on existing mock infrastructure** in `src/lib/test/fixtures/`
- **Leverage demo mode** for safe UI testing without server connections

### 4. Implementation Strategy Order
**Step 1: Unit Test Coverage Foundation**
- Implement security testing for validation module (malicious inputs, injection prevention)
- Add database transaction safety testing (rollback scenarios, atomic operations)
- Create automation workflow business logic testing (command generation, metadata handling)

**Step 2: Autonomous UI Testing Framework**
- Set up Playwright for headless UI testing in demo mode
- Implement autonomous navigation and interaction testing
- Create visual regression testing with screenshot validation

**Step 3: Test Infrastructure & Automation**
- Enhance CI pipeline with coverage reporting
- Implement performance and reliability testing
- Create test data management and cleanup systems

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Verify current test environment
npm ci
cargo check

# Development workflow
npm run dev              # Svelte dev server
npm run test            # Vitest unit tests
cargo test              # Rust unit tests
npm run test:ui         # UI testing toolkit

# New testing commands to implement
npm run test:coverage   # Coverage reporting (implement this)
npm run test:ui:auto    # Autonomous UI testing (implement this)

# Quality checks
cargo clippy            # Rust linting
npm run lint            # TypeScript/Svelte linting
```

## 🧭 Implementation Guidance

### Integration Points
```rust
// Existing validation module to test comprehensively
validation::shell::escape_parameter() // Add: Security injection testing
validation::files::validate_upload_file() // Add: File validation testing
validation::input::sanitize_job_id() // Add: Malicious input testing

// Database module transaction testing
JobDatabase::begin_transaction() // Add: Rollback scenario testing
JobDatabase::save_job_in_transaction() // Add: Atomic operation testing

// Automation workflow testing (business logic only)
execute_job_submission_with_progress() // Add: Command generation testing
execute_job_completion_with_progress() // Add: File handling logic testing
```

```typescript
// Frontend business logic testing
// Job store validation (src/lib/stores/jobs.ts)
jobStore.addJob() // Add: State management testing
jobStore.updateJobStatus() // Add: Data transformation testing

// UI utility function testing (src/lib/utils/)
parseMemoryString() // Add: Input parsing testing
validateResourceRequest() // Add: Validation logic testing
```

### Key Technical Decisions Already Made
- **Testing Philosophy** - Focus on business logic, not external dependencies (SSH, SLURM servers)
- **Demo Mode Testing** - Use demo mode for all UI testing to avoid server connections
- **Coverage Requirement** - >80% coverage for critical modules (validation, database, automation)

### Architecture Patterns to Follow
- **NAMDRunner Testing Strategy** - Test our logic, not external libraries like ssh2 or SLURM
- **Autonomous UI Testing** - Playwright-based testing with headless browser configuration
- **Mock-First Development** - Use deterministic mocks, avoid random behavior or server simulation

## ⚠️ Critical Constraints & Requirements

### Testing Philosophy (NON-NEGOTIABLE)
- **MUST READ**: `docs/CONTRIBUTING.md#testing-strategy` before writing any tests
- **Focus on business logic**: Test NAMDRunner's code, not external libraries (ssh2, SLURM, etc.)
- **No server integration testing**: All tests must work without actual SSH/SLURM connections
- **Deterministic behavior**: No random errors, delays, or "stress testing" in unit tests
- **Fast execution**: Unit tests should complete in <30 seconds total

### UI Testing Requirements (CRITICAL)
- **MUST READ**: `docs/reference/agent-development-tools.md` for autonomous testing capabilities
- **Demo mode only**: All UI testing must use demo mode to avoid server connections
- **Headless configuration**: Must work in SSH environments with proper browser setup
- **Visual validation**: Screenshot capture and comparison for regression testing

### Security Testing (NON-NEGOTIABLE)
- Test malicious input handling in validation module
- Validate command injection prevention in shell builders
- Test path traversal prevention in file operations
- Ensure transaction rollback works correctly for data safety

### Coverage Requirements
- >80% test coverage for critical modules (validation, database, automation)
- Focus on security-critical code paths and business logic
- Quality over quantity - meaningful tests, not just coverage numbers

## 🤝 Getting Help

### When You Need Guidance
- **Testing strategy questions** - Check `docs/CONTRIBUTING.md#testing-strategy` FIRST
- **UI testing questions** - See `docs/reference/agent-development-tools.md` for Playwright patterns
- **Coverage questions** - Focus on validation, database, and automation modules
- **Architecture decisions** - Review `docs/ARCHITECTURE.md` and ask for input

### Communication Protocol
- **Present your testing plan** before implementing comprehensive test suites
- **Ask specific questions** about testing patterns rather than general approaches
- **Update coverage reports** as you implement tests
- **Share test failures** with concrete examples and error messages

## 🎯 Your First Steps

1. **Read all required documentation** - especially testing strategy and agent development tools
2. **Run investigation commands** to understand current test coverage and infrastructure
3. **Analyze existing test patterns** in `tests/` and `src-tauri/src/` directories
4. **Plan your testing approach** focusing on business logic validation
5. **Implement security testing first** for validation module (highest priority)
6. **Get approval for your testing strategy** before writing comprehensive test suites

## Success Metrics
- >80% unit test coverage achieved for critical modules
- Autonomous UI testing framework functional in demo mode
- All existing tests continue passing with new test additions
- Test suite runs efficiently (<30 seconds for unit tests)
- Performance and security validation testing implemented
- Clean, maintainable test code following project standards

## Task Management (CRITICAL)
- **Work from active task** in `tasks/active/phase-6-4-comprehensive-testing.md`
- **Update ARCHITECTURE.md** as you implement testing infrastructure
- **Focus on business logic testing** as defined in NAMDRunner testing philosophy
- **Use demo mode exclusively** for UI testing to avoid server dependencies

Remember: This builds on the clean, secure codebase from Phase 6.3. **Follow NAMDRunner testing philosophy** and **focus on business logic validation** rather than external system integration testing.
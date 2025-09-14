# Phase 1 Implementation Prompt for Claude Code Agent

## Project Overview
You are implementing **Phase 1** of NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **new application** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), using a proven Python/CustomTkinter implementation as reference.

## Your Mission: Phase 1 Implementation
Implement a **single-job MVP** with core functionality:
- SSH connection management to SLURM clusters
- Basic job creation, submission, and status checking
- Local job persistence with SQLite
- Simple Svelte UI for the essential workflow

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/project-spec.md` - **WHAT** we're building (business requirements)
- `docs/technical-spec.md` - **HOW** to build it (architecture, tech stack, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase Details
- `tasks/roadmap.md` - **Phase 1 scope and milestones** (your roadmap)
- `docs/architecture.md` - Implementation progress tracker (update as you build)
- `tasks/phase1-interface-definitions.md` - Complete IPC interface contracts

### 3. Reference Implementation Knowledge
- `docs/reference/slurm-commands-reference.md` - **Working SLURM patterns** and command reference
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/data-spec.md` - Database schema and JSON metadata formats

### 4. Development Support
- `docs/testing-spec.md` - Testing strategy, debugging workflows, CI setup
- `tasks/templates/task.md` - Use this template for task planning

## 🎯 Phase 1 Success Criteria

### Milestone 1.1: Foundation (Do This First)
- [ ] Create Tauri project with Svelte template
- [ ] Set up TypeScript/Rust IPC boundary with type safety
- [ ] Implement connection management (connect/disconnect/status)
- [ ] Create basic UI with connection state display

### Milestone 1.2: Job Management
- [ ] Implement job creation with NAMD/SLURM configuration
- [ ] Add job submission to cluster via SSH
- [ ] Build job status checking with SLURM commands
- [ ] Create job list UI with status display

### Milestone 1.3: Persistence & Testing
- [ ] Add SQLite database for local job cache
- [ ] Implement file upload/download via SFTP
- [ ] Add comprehensive unit tests with mocks
- [ ] Create basic E2E tests

## 🔧 Implementation Approach

### 1. Reference-Driven Development
- **Start with proven patterns** from `docs/reference/slurm-commands-reference.md`
- **Use established data formats** from `docs/data-spec.md`
- **Learn from Python lessons** in `docs/reference/python-implementation-reference.md`
- **Improve and modernize** with Tauri/Rust advantages

### 2. Task Management (CRITICAL)
- **Create task files** using `tasks/templates/task.md` before coding
- **Work on one task at a time** - move to `tasks/active/` when starting
- **Update architecture.md** as you implement each component
- **Get approval** for your implementation plan before coding

### 3. Technology Constraints
- **Password-only SSH** (no key support per cluster requirements)
- **Never persist credentials** (memory only, clear on disconnect)
- **Type-safe IPC boundary** (strict contracts between TS/Rust)
- **Offline-first design** (local SQLite cache, manual sync)

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Install dependencies (if not done)
npm ci
npx playwright install --with-deps
cargo install tauri-cli@^2.0

# Create Tauri project
npm create tauri-app@latest . -- --template svelte
npm ci

# Verify setup
npm run tauri dev  # Should launch empty Tauri app

# Daily development
npm run dev              # Svelte dev server
npm run test            # Vitest unit tests
cargo test              # Rust unit tests
npm run tauri dev       # Full Tauri app
```

## 🧭 Implementation Guidance

### Start Here
1. **Read the specs** - Understand what we're building before coding
2. **Check Phase 1 scope** in `tasks/roadmap.md`
3. **Review interface definitions** in `tasks/phase1-interface-definitions.md`
4. **Plan your first task** using the task template

### Key Technical Decisions Already Made
- **SQLite schema** - Use structure from `docs/data-spec.md`
- **SLURM commands** - Use proven patterns from `docs/reference/slurm-commands-reference.md`
- **Job metadata** - JSON format defined in `tasks/phase1-interface-definitions.md`
- **Directory structure** - `/projects/$USER/namdrunner_jobs/` pattern

### Architecture Patterns
- **Port/adapter pattern** for external dependencies (SSH, SLURM)
- **Mock implementations** for testing (see `docs/testing-spec.md`)
- **Type-safe IPC** with comprehensive error handling
- **Reactive Svelte stores** for session and job state

## ⚠️ Critical Success Factors

### Security (Non-Negotiable)
- Never log or persist SSH passwords
- Clear credentials from memory on disconnect
- Use minimal Tauri permissions
- Validate all user inputs

### Quality Requirements
- Comprehensive unit test coverage with mocks
- Type safety across the entire IPC boundary
- Proper error handling and user feedback
- Follow coding standards in `docs/technical-spec.md`

### Reference Implementation Respect
- Use proven SLURM integration patterns
- Respect working directory structures
- Follow established JSON schemas
- Learn from Python version mistakes

## 🤝 Getting Help

### When You Need Guidance
- **SLURM integration questions** - Check `docs/reference/slurm-commands-reference.md` first
- **Data format questions** - See `docs/data-spec.md`
- **Architecture decisions** - Review `docs/architecture.md` and ask for input
- **Python implementation questions** - Look at `docs/reference/python-implementation-reference.md`

### Communication Protocol
- **Present your plan** before starting major implementation work
- **Ask specific questions** rather than general "how do I..." queries
- **Update docs** as you learn and implement
- **Share progress updates** with concrete examples

## 🎯 Your First Steps

1. **Read all required documentation** listed above
2. **Create your first task** for Milestone 1.1 foundation work
3. **Set up the development environment** using the setup commands
4. **Plan your IPC interface implementation** based on the phase1 definitions
5. **Get approval for your approach** before writing significant code

## Success Metrics
- Phase 1 milestones completed per `tasks/roadmap.md`
- All tests passing with good coverage
- Type-safe IPC boundary with comprehensive error handling
- Working connection to real SLURM cluster
- Basic job lifecycle working end-to-end
- Clean, maintainable code following project standards

Remember: This is **not a migration** but a **new application with reference implementation**. Use the Python patterns as a starting point, then improve and modernize with Tauri's capabilities.

Good luck! 🚀
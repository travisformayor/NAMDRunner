# CLAUDE.md - AI Assistant Guide for NAMDRunner

## Project Context
NAMDRunner: Desktop app for NAMD molecular dynamics on SLURM clusters. **Tauri v2 (Rust) + Svelte (TypeScript)**. Reference Python implementation (`docs/reference/`) provides proven patterns to adapt.

## Core Principles
- **Security**: No credential persistence, password-only SSH, minimal attack surface
- **Type Safety**: Strict TypeScript ↔ Rust contracts
- **Reliability**: Scientists need stability over features
- **Reference-Driven**: Adapt proven Python patterns

## Essential Documentation (Read These First!)

### Essential Docs
- `docs/README.md` - Documentation index (keep up-to-date)
- `docs/project-spec.md` - Business requirements and goals
- `docs/technical-spec.md` - Stack, architecture, development setup  
- `docs/developer-guidelines.md` - Code quality standards and architectural patterns
- `docs/agent-capabilities.md` - Available tools and testing infrastructure
- `tasks/roadmap.md` - Current phase and development plan
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/reference/slurm-commands-reference.md` - Complete SLURM command reference
- `docs/cluster-guide.md` - HPC cluster configuration and setup requirements

## Development Workflow

### Before Starting ANY Work
1. **Read the specs** - `docs/project-spec.md` and `docs/technical-spec.md`
2. **Check current phase** - `tasks/roadmap.md` shows where we are
3. **Look for active tasks** - `tasks/active/` (should be empty to start new work)
4. **Understand the feature** - Check `docs/reference/python-implementation-reference.md` for Python implementation

### Task Management (Critical!)
- **One task at a time** in `tasks/active/` - No exceptions
- **Use task template** from `tasks/templates/task.md`
- **Get approval before coding** - Present plan, wait for confirmation
- **Track progress** - Update active task document with implementation steps and progress

#### Task Completion Process
After implementation and testing, before archiving:
1. **Code Review & Refactor** - Use `.claude/agents/review-refactor.md` agent to analyze completed work
2. **Apply Improvements** - Implement recommended refactoring based on what was learned
3. **Update Documentation** - Update `tasks/roadmap.md` and `docs/architecture.md` with final implementation details
4. **Update and Archive Task** - Move to `tasks/completed/`

### When You're Stuck
- Review `tasks/roadmap.md` for current development phase
- Check `docs/reference/` docs for Python implementation patterns
- Look at actual Python code in `docs/reference/NAMDRun-python/src/namdrunner/`
- Document findings in appropriate reference file
- Update `docs/reference/python-implementation-reference.md` with new insights
- **Read `docs/agent-capabilities.md`** - Available tools and testing infrastructure
- **Investigate with temporary scripts** - Use testing tools (`docs/testing-spec.md`) to understand current behavior
- **Ask specific questions** about the requested feature or Python implementation details

## Quick Commands

See `docs/technical-spec.md` for complete setup and development commands.

## Critical Success Factors

### Data Compatibility (Where It Makes Sense)
- **SQLite schema** - Consider Python patterns, improve if possible while maintaining user data compatibility
- **JSON metadata format** - Use schema_version field approach, adapt structure as needed
- **Directory structure** - `/projects/$USER/namdrunner_jobs/` pattern worked well
- **SLURM commands** - Use proven working patterns from Python as starting point

### Security Requirements (Non-Negotiable)  
- **Never log credentials** - Mask passwords, clear memory on disconnect
- **No credential persistence** - In-memory only, re-auth on session expire
- **Minimal Tauri permissions** - Only enable required commands/APIs
- **Password-only SSH** - No key support (cluster requirement)

### Architecture Constraints
- **Type-safe IPC boundary** - Strict contracts between TS and Rust
- **Offline-first design** - Local SQLite cache, manual sync
- **Single-writer rule** - App writes JSON, jobs write to scratch only
- **Portable Windows exe** - Primary deployment target

## Common Pitfalls to Avoid

### From Python Experience
- **Don't hardcode module versions** - Make configurable (gcc/14.2.0, etc.)
- **Don't skip error handling** - SSH operations WILL fail
- **Don't block UI thread** - All SSH/SFTP operations must be async

### Reference Implementation Specific
- **Don't ignore working SLURM patterns** - Python version has proven integration approaches
- **Don't reinvent solved problems** - Check reference docs before implementing from scratch
- **Don't skip reference docs** - Python version has 9 completed tasks worth of learnings
- **Don't work without task plan** - Complex application needs structured approach

## What Success Looks Like
- **Complete functionality** - All planned features working reliably in Tauri
- **Windows deployment** - Portable exe that "just works"
- **Maintainability** - New developers can understand and extend
- **Reliability** - Works for months without maintenance

## When to Ask for Help
- **SLURM integration questions** - Python patterns are proven starting points
- **Data format questions** - User compatibility considerations
- **Architecture decisions** - Get input before big design choices  
- **Task planning** - Break down complex work before starting
- **Stuck on Python code** - Ask for specific clarification rather than guessing
- **Streamlining opportunities** - When you see ways to improve on reference patterns
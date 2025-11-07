# CLAUDE.md - AI Assistant Guide for NAMDRunner

## Project Context
NAMDRunner: Desktop app for NAMD molecular dynamics on SLURM clusters built with Tauri v2 (Rust) + Svelte (TypeScript).

## Core Principles
- **Security**: No credential persistence, password-only SSH, minimal attack surface
- **Type Safety**: Strict TypeScript ↔ Rust contracts
- **Reliability**: Scientists need stability over features
- **Pattern-Driven**: Use proven architectural patterns and cluster integration approaches

## Context-Specific Reading Guide

**Quick navigation: What should I read for my current task?**

### 🧪 Working on Unit Tests?
- [`docs/CONTRIBUTING.md#testing-strategy`](docs/CONTRIBUTING.md#testing-strategy) - Testing philosophy and 3-tier architecture
- [`docs/reference/agent-development-tools.md`](docs/reference/agent-development-tools.md) - Testing tools and debugging infrastructure

### 🔧 Refactoring Code?
- **Use the review-refactor agent first**: `.claude/agents/review-refactor.md`
- [`docs/CONTRIBUTING.md#developer-standards--project-philosophy`](docs/CONTRIBUTING.md#developer-standards--project-philosophy) - Development principles and anti-patterns
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System design and architectural constraints

### 🎨 Working on UI?
- [`docs/DESIGN.md`](docs/DESIGN.md) - UI/UX specifications, component architecture, and Svelte implementation patterns

### 💼 Working on Job Management & Automation?
- [`docs/AUTOMATIONS.md`](docs/AUTOMATIONS.md) - **Complete automation architecture** with all job lifecycle automation chains
- [`docs/reference/slurm-commands-reference.md`](docs/reference/slurm-commands-reference.md) - SLURM command patterns and job management
- [`docs/API.md`](docs/API.md) - Job management IPC interfaces and command specifications

### ⚡ Working on SLURM Integration?
- [`docs/reference/slurm-commands-reference.md`](docs/reference/slurm-commands-reference.md) - Complete job script template combining SLURM + Alpine + MPI + NAMD
- [`docs/reference/alpine-cluster-reference.md`](docs/reference/alpine-cluster-reference.md) - Alpine-specific requirements (MPI, memory units, node calculation)
- [`docs/reference/namd-commands-reference.md`](docs/reference/namd-commands-reference.md) - NAMD config templates and file naming requirements

### 🧬 Working on NAMD?
- [`docs/reference/namd-commands-reference.md`](docs/reference/namd-commands-reference.md) - NAMD config file templates (.namd) and file naming (use actual uploaded names!)
- [`docs/reference/slurm-commands-reference.md`](docs/reference/slurm-commands-reference.md) - For complete job script with NAMD execution

### 🔐 Working on SSH/SFTP?
- [`docs/SSH.md`](docs/SSH.md) - Connection management, security patterns, and file operations

### 🖥️ Making Cluster Allocation Decisions?
- [`docs/reference/alpine-cluster-reference.md`](docs/reference/alpine-cluster-reference.md) - Resource limits, partitions, and QoS options
- [`docs/reference/slurm-commands-reference.md`](docs/reference/slurm-commands-reference.md) - Resource allocation commands

### 🔌 Working on IPC/Commands (Non-Job Related)?
- [`docs/API.md`](docs/API.md) - IPC interfaces and command specifications
- [`docs/CONTRIBUTING.md#security-requirements`](docs/CONTRIBUTING.md#security-requirements) - Security patterns for command handling
- For job-related commands, see **Job Management & Automation** section above

### 🗄️ Working with Database/Data?
- [`docs/DB.md`](docs/DB.md) - SQLite schemas and JSON metadata formats

## Essential Documentation (Always Available)

### Start Here
- [`docs/README.md`](docs/README.md) - Documentation index and quick start
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System design and project overview
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) - Development setup and standards
- [`tasks/roadmap.md`](tasks/roadmap.md) - Current development phase

## Development Workflow

### Before Starting ANY Work
1. **Read the specs** - Use the Context-Specific Reading Guide above for your task type
2. **Check current phase** - `tasks/roadmap.md` shows where we are
3. **Look for active tasks** - `tasks/active/` (should be empty to start new work)

### Task Management (Critical!)
- **One task at a time** in `tasks/active/` - No exceptions
- **Use task template** from `tasks/templates/task.md`
- **Get approval before coding** - Present plan, wait for confirmation
- **Track progress** - Update active task document with implementation steps and progress

#### Task Completion Process
After implementation and testing, before archiving:
1. **Code Review & Refactor** - Use `.claude/agents/review-refactor.md` agent to analyze completed work
2. **Apply Improvements** - Implement recommended refactoring based on what was learned
3. **Update Documentation** - Update `tasks/roadmap.md` and `docs/ARCHITECTURE.md` with final implementation details (see Documentation Rules below)
4. **Update and Archive Task** - Move to `tasks/completed/`

#### Documentation Update Rules

When updating task plans (`tasks/active/*.md`) or project documentation (`docs/*.md`):

**Integrate Changes Contextually:**
- Insert new information where it belongs topically, not at the end
- Update existing sections rather than creating duplicate sections
- Preserve document structure and logical flow

**Maintain Proportional Detail:**
- Match the level of detail used for comparable topics
- Avoid over-emphasizing content just because you're currently adding it
- If related content needs more detail to match new additions, research and expand it first

**Keep Documentation Lean:**
- Identify and remove redundant information when adding new content
- Delete outdated or incorrect information discovered during updates
- Consolidate verbose sections - prefer focused, essential information

**Document Current State:**
- Describe what the system does NOW, not its planning history
- Avoid narratives like "originally we planned X, then implemented Y, now it's Z"
- Remove implementation evolution details - readers need current facts, not historical context

### When You're Stuck
- **Use the Context-Specific Reading Guide above** - Find docs relevant to your current task
- Review `tasks/roadmap.md` for current development phase
- **Ask specific questions** about the requested feature or cluster integration details

### Dangerous Practices to Avoid

**Never Use `git checkout/restore` to Fix Broken Code:**
- ❌ **DO NOT** use `git checkout HEAD -- file.ts` to "fix" syntax errors
- ❌ This reverts ALL work in that file, not just the broken parts
- ❌ Causes massive loss of progress when file contains completed features
- ✅ **Instead:** Manually fix the specific syntax error or logic issue
- ✅ Use `git diff` to understand what changed, then fix the actual problem
- ✅ If truly unsalvageable, discuss with user before reverting

**Why This Matters:**
- A single syntax error doesn't invalidate all work in the file
- Automated refactoring (sed, find/replace) can break syntax while logic is sound
- Restoring from git discards hours of implementation
- Always fix forward - never revert completed features to fix syntax

## Quick Commands

See `docs/CONTRIBUTING.md` for complete setup and development commands.

## Critical Success Factors

### Cluster Integration Patterns
- **SQLite schema** - Simple and focused schema design (see `docs/DB.md`)
- **JSON metadata format** - Serialized from Rust `JobInfo` struct (see `docs/DB.md`)
- **Directory structure** - `/projects/$USER/namdrunner_jobs/` pattern for persistent storage
- **SLURM commands** - Use proven working patterns (see `docs/reference/slurm-commands-reference.md`)

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

### Code Quality Standards (Non-Negotiable)

**Zero Tolerance for Warnings:**
- **Treat warnings as errors** - Compiler warnings, linter warnings, build warnings all indicate problems
- Warnings hint at dead code, incorrect patterns, or incomplete implementations
- "It builds/runs fine" is never justification for ignoring warnings
- Investigate and fix ALL warnings, every time

**Holistic Refactoring Over Patches:**
- **Never patch on top of patches** - If code needs changing, step back and ask: "If I knew the requirement from the start, how would I implement this correctly?"
- Delete and refactor rather than adding conditional logic to existing code
- Don't be biased toward keeping existing code just because it exists
- Prefer solutions that result in less code, simpler code, easier-to-reason-about code

**No "Quick Fixes":**
- Always choose the solution that removes the most tech debt, even if it takes longer
- "This is easier" is never a reason to choose inferior approach
- Centralize patterns, extract utilities, remove duplication - do it right the first time

**Complete Work Honestly:**
- Never claim work is "complete" or "100% done" if stubs, broken tests, or known issues remain
- Be transparent about all remaining work - list it explicitly
- Don't hide decisions to skip or defer things - communicate openly

**Code Quality Applies Everywhere:**
- Internal code, naming, patterns matter just as much as user-facing code
- "Users won't see this" is never justification for bad code
- Consistency matters in variable names, function names, logging patterns, error handling
- Fix inconsistencies even in code that works

**No Priority Levels for Issues:**
- All identified issues must be resolved - there are no "low priority" bugs
- Organize work by optimal implementation order (dependencies, logical flow)
- Don't use "priority" as excuse to defer work
- If something is truly optional, discuss explicitly - don't just deprioritize

**Balance in Abstraction:**
- Avoid both extremes: don't copy-paste code, but don't over-abstract for DRY's sake
- Extract when sharing logic reduces errors and improves maintainability
- Don't create abstractions that make code harder to understand
- Prefer simple, obvious code over clever abstractions

### Cluster Integration Pitfalls
- **Don't hardcode module versions** - Make configurable (gcc/14.2.0, etc.)
- **Don't skip error handling** - SSH operations WILL fail
- **Don't block UI thread** - All SSH/SFTP operations must be async
- **Don't ignore SLURM and NAMD documentation** - Check docs for offical approaches
- **Don't reinvent solved problems** - Review docs and related code before implementing from scratch
- **Don't skip documentation** - Learn from documented patterns
- **Don't work without task plan** - Complex application needs structured approach

### Code Completeness
- **Don't leave stubs in production** - Replace all setTimeout/mock data with real backend calls
- **Don't implement business logic in frontend** - Validation, calculations, cluster config belong in Rust
- **Don't keep orphaned code** - Delete service layers and utilities that production code never imports
- **Don't treat test usage as real usage** - If only tests use code, delete both the code and tests

## What Success Looks Like
- **Complete functionality** - All planned features working reliably in Tauri
- **Windows deployment** - Working portable exe
- **Maintainability** - New developers can understand and extend
- **Reliability** - Works for months without maintenance

## When to Ask for Help
- **SLURM integration questions** - Reference docs provide proven patterns
- **Data format questions** - Architecture decisions need clarity
- **Architecture decisions** - Get input before big design choices
- **Task planning** - Break down complex work before starting
- **Unclear cluster behavior** - Ask for specific clarification rather than guessing
- **Streamlining opportunities** - When you see ways to improve existing patterns
# [Phase Milestone] Implementation Prompt

## Project Overview
You are implementing **[Phase X.Y: Milestone Name]** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **[new feature/enhancement/refactoring]** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend).

## Your Mission: [Implementation Goal]
Implement **[specific functionality]** with core capabilities:
- [Key capability 1]
- [Key capability 2]
- [Key capability 3]
- [Key capability 4]

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - **WHAT** we're building (business requirements, system design)
- `docs/CONTRIBUTING.md` - **HOW** to build it (development setup, testing, coding standards)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **[Current phase] scope and milestones** (your roadmap)
- `docs/ARCHITECTURE.md` - Implementation progress tracker (update as you build)
- `tasks/[specific-task].md` - Complete implementation requirements and constraints

### 3. Cluster Integration Reference
- `docs/reference/slurm-commands-reference.md` - **Working SLURM patterns** and command reference
- `docs/reference/namd-commands-reference.md` - NAMD execution patterns and templates
- `docs/reference/alpine-cluster-reference.md` - Alpine cluster configuration and resource limits
- `docs/DB.md` - Database schema and JSON metadata formats

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy and workflows
- `tasks/templates/task.md` - Use this template for task planning
- `docs/reference/agent-development-tools.md` - Available tools and testing infrastructure

## 🎯 [Phase/Milestone] Success Criteria

### [Category 1]: [Milestone Name] (Do This First)
- [ ] [Specific deliverable 1]
- [ ] [Specific deliverable 2]
- [ ] [Specific deliverable 3]
- [ ] [Specific deliverable 4]

### [Category 2]: [Milestone Name]
- [ ] [Specific deliverable 1]
- [ ] [Specific deliverable 2]
- [ ] [Specific deliverable 3]
- [ ] [Specific deliverable 4]

### [Category 3]: [Milestone Name]
- [ ] [Specific deliverable 1]
- [ ] [Specific deliverable 2]
- [ ] [Specific deliverable 3]
- [ ] [Specific deliverable 4]

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ [Existing capability 1] - [Brief description and location]
- ✅ [Existing capability 2] - [Brief description and location]
- ✅ [Existing capability 3] - [Brief description and location]

**What's Missing (Implement This)**:
- ❌ [Missing capability 1] - [What needs to be built]
- ❌ [Missing capability 2] - [What needs to be built]
- ❌ [Missing capability 3] - [What needs to be built]

### 2. Investigation Commands (Run These First)
```bash
# Check existing [relevant functionality]
cd /media/share/namdrunner-backend/src-tauri
rg "[search-pattern-1]|[search-pattern-2]" src/ -A 5 -B 5

# Look at existing [related component]
rg "[search-pattern-3]" src/ -A 10

# Check how [existing feature] currently works
rg "[function-pattern]" src/[specific-file].rs -A 15
```

**Expected Finding**: [What you should discover about current implementation]

### 3. Pattern-Driven Development
- **Start with proven patterns** from `docs/reference/[specific-reference].md`
- **Use established data formats** from `docs/DB.md`
- **Learn from documented patterns** in cluster integration reference docs
- **Leverage Tauri/Rust advantages** for improved security and performance

### 4. Implementation Strategy Order
**Step 1: [Foundation Work]**
- [Specific task 1]
- [Specific task 2]
- [Integration requirement]

**Step 2: [Core Functionality]**
- [Specific task 1]
- [Specific task 2]
- [Testing requirement]

**Step 3: [Integration & Polish]**
- [Specific task 1]
- [Specific task 2]
- [Quality requirement]

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner-backend

# Verify environment (if needed)
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
[existing_function]() // Add: [new capability description]
[existing_function]() // Enhance: [enhancement description]

// New functions to add (follow established patterns)
[new_function]() // Purpose: [specific purpose and integration point]
[new_function]() // Purpose: [specific purpose and integration point]
```

### Key Technical Decisions Already Made
- **[Decision 1]** - [Rationale and pattern to follow]
- **[Decision 2]** - [Rationale and existing implementation to build on]
- **[Decision 3]** - [Rationale and constraints to respect]

### Architecture Patterns to Follow
- **[Pattern 1]** - [Description and where it's used]
- **[Pattern 2]** - [Description and integration approach]
- **[Pattern 3]** - [Description and quality requirements]

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Never log or persist SSH passwords
- Clear credentials from memory on disconnect
- Use minimal Tauri permissions
- Validate all user inputs
- [Phase-specific security requirement]

### Quality Requirements
- Comprehensive unit test coverage with mocks
- Type safety across the entire IPC boundary
- Proper error handling and user feedback
- Follow coding standards in `docs/CONTRIBUTING.md#developer-standards--project-philosophy`
- [Phase-specific quality requirement]

### Integration Requirements
- Build on existing [specific system] infrastructure
- Improve upon [existing feature] as needed
- Follow established [specific pattern] patterns where beneficial
- Respect [specific constraint] constraints

## 🤝 Getting Help

### When You Need Guidance
- **[Domain] questions** - Check `docs/reference/[specific-reference].md` first
- **Data format questions** - See `docs/DB.md`
- **Architecture decisions** - Review `docs/ARCHITECTURE.md` and ask for input
- **Cluster integration patterns** - Review reference docs for proven approaches

### Communication Protocol
- **Present your plan** before starting major implementation work
- **Ask specific questions** rather than general "how do I..." queries
- **Update docs** as you learn and implement
- **Share progress updates** with concrete examples

## 🎯 Your First Steps

1. **Read all required documentation** listed above
2. **Run investigation commands** to understand current state
3. **Create your implementation task** using `tasks/templates/task.md`
4. **Analyze existing patterns** that you'll build upon
5. **Plan your integration approach** based on established systems
6. **Get approval for your approach** before writing significant code

## Success Metrics
- [Phase/Milestone] deliverables completed per `tasks/roadmap.md`
- All existing tests continue passing
- New functionality has comprehensive test coverage
- Type-safe integration with existing systems
- [Specific functional requirement]
- [Specific quality requirement]
- Clean, maintainable code following project standards

## Task Management (CRITICAL)
- **Create task files** using `tasks/templates/task.md` before coding
- **Work on one task at a time** - move to `tasks/active/` when starting
- **Update ARCHITECTURE.md** as you implement each component
- **Get approval** for your implementation plan before coding

Remember: This builds on existing NAMDRunner infrastructure. **Leverage established patterns** and **integrate seamlessly** with proven systems rather than creating parallel implementations.

# Phase 6.1: UI Integration with Backend - Implementation Prompt for Frontend/Backend Integration Engineer

## Project Overview
You are implementing **Phase 6.1: UI Integration with Backend** for NAMDRunner, a desktop application for managing NAMD molecular dynamics simulations on SLURM HPC clusters. This is a **critical integration milestone** built with Tauri v2 (Rust backend) + Svelte (TypeScript frontend), connecting the completed Phase 3 UI with the completed Phases 1-5 backend infrastructure.

## Your Mission: Frontend-Backend Integration with Demo Mode
Implement **seamless UI-backend integration** with core capabilities:
- Replace mock IPC client with real Tauri IPC bridge
- Add demo/real mode toggle in connection dropdown for demonstrations
- Wire all UI actions to backend commands (connection, jobs, files)
- Maintain existing demo functionality while enabling real cluster operations

## 📋 Before You Start - Required Reading

### 1. Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/ARCHITECTURE.md` - **System design and IPC boundary patterns** (critical for integration)
- `docs/API.md` - **Complete IPC command specifications** (your integration contract)
- `CLAUDE.md` - Development guidelines and workflow

### 2. Current Phase/Implementation Details
- `tasks/roadmap.md` - **Phase 6.1 scope and dependencies** (understand what's been built)
- `tasks/active/phase-6-1-ui-backend-integration.md` - **Complete integration requirements** (your primary task)
- `docs/ARCHITECTURE.md` - Current implementation status (what exists in frontend/backend)

### 3. Integration-Specific Knowledge
- `docs/API.md` - **IPC command specifications** and type definitions
- `docs/DESIGN.md` - UI component patterns and interaction flows
- `src/lib/ports/coreClient.ts` - **IPC client interface definition**
- `src-tauri/src/commands/` - **Backend command implementations**

### 4. Development Support
- `docs/CONTRIBUTING.md#testing-strategy` - Testing strategy for both UI and backend
- `tasks/templates/task.md` - Task planning template
- `docs/reference/agent-development-tools.md` - Available development tools

## 🎯 Phase 6.1 Success Criteria

### Critical Priority: IPC Boundary Integration (Do This First)
- [ ] Fix command signature mismatches between frontend and backend
- [ ] Align ConnectParams structure (frontend object vs backend structured params)
- [ ] Resolve TypeScript/Rust type definition inconsistencies
- [ ] Validate all command parameter and result type alignment

### Critical Priority: Demo Mode Toggle Integration
- [ ] Add demo/real mode toggle UI in connection dropdown
- [ ] Implement mode persistence across application sessions
- [ ] Update CoreClientFactory to respect user mode selection
- [ ] Provide clear visual indication of current mode

### High Priority: Core Workflow Integration
- [ ] Wire connection management to real SSH backend
- [ ] Integrate job lifecycle operations with SLURM backend
- [ ] Connect file operations to SFTP backend
- [ ] Replace hardcoded mock data when in real mode

### Medium Priority: Testing & Polish
- [ ] Update E2E tests for both demo and real modes
- [ ] Implement proper error handling and loading states
- [ ] Validate seamless mode switching behavior
- [ ] Clean up remaining mock data references

## 🔧 Implementation Approach

### 1. Current State Analysis (Critical!)
**What's Already Working (Don't Rebuild)**:
- ✅ **CoreClientFactory pattern** - Clean client switching architecture in `src/lib/ports/clientFactory.ts`
- ✅ **Backend commands registered** - All IPC commands implemented and registered in `src-tauri/src/lib.rs`
- ✅ **UI stores abstracted** - Svelte stores in `src/lib/stores/` already use CoreClientFactory
- ✅ **Type system foundation** - TypeScript types in `src/lib/types/api.ts` align with Rust types

**What's Missing (Implement This)**:
- ❌ **IPC boundary alignment** - Command signatures don't match between frontend/backend
- ❌ **User mode preference** - CoreClientFactory only uses environment detection
- ❌ **Demo mode toggle UI** - No user interface for switching modes
- ❌ **Real backend integration** - UI stores still operate with mock data only

### 2. Investigation Commands (Run These First)
```bash
# Check current IPC command signatures
cd /media/share/namdrunner
rg "connect_to_cluster|create_job|submit_job" src-tauri/src/commands/ -A 5 -B 5

# Examine frontend IPC client implementations
rg "connect\(|createJob\(|submitJob\(" src/lib/ports/ -A 10

# Check type alignment between TS and Rust
rg "ConnectParams|CreateJobParams" src/lib/types/ src-tauri/src/types/ -A 15
```

**Expected Finding**: Parameter structure mismatches, particularly in connection commands where frontend expects `{ host, username, password }` but backend may expect different structure.

### 3. Reference-Driven Development
- **Start with established IPC patterns** from `docs/API.md` and existing command structure
- **Use existing UI store patterns** from `src/lib/stores/session.ts` and `jobs.ts`
- **Follow existing type definitions** in `src/lib/types/api.ts` and `src-tauri/src/types/`
- **Build on CoreClientFactory** existing client switching mechanism

### 4. Implementation Strategy Order
**Step 1: IPC Boundary Alignment**
- Fix command signature mismatches in `src/lib/ports/coreClient-tauri.ts`
- Align parameter structures between frontend and backend
- Test basic connection flow end-to-end

**Step 2: Demo Mode Toggle Implementation**
- Add toggle UI component in connection dropdown
- Extend CoreClientFactory with user preference support
- Implement mode persistence and visual indicators

**Step 3: Core Integration & Testing**
- Wire all UI operations to real backend commands
- Update error handling for backend-specific errors
- Validate both demo and real modes work correctly

## 📁 Project Setup Commands

```bash
cd /media/share/namdrunner

# Verify environment
npm ci
cargo check

# Development workflow
npm run dev              # Svelte dev server
npm run test            # Vitest unit tests
cargo test              # Rust unit tests
npm run tauri dev       # Full Tauri app with backend

# Quality checks
cargo clippy            # Rust linting
npm run lint            # TypeScript/Svelte linting
```

## 🧭 Implementation Guidance

### Integration Points
```typescript
// Frontend IPC clients (fix signatures)
TauriCoreClient.connect(params) // Align with backend connect_to_cluster
TauriCoreClient.createJob(params) // Align with backend create_job
TauriCoreClient.submitJob(jobId) // Align with backend submit_job

// Enhanced factory (add user preference)
CoreClientFactory.getClient(forceMock?) // Add mode preference support
CoreClientFactory.setMode('demo'|'real') // New: User mode selection
```

```rust
// Backend commands (ensure alignment)
connect_to_cluster(params: ConnectParams) // Match frontend expectations
create_job(params: CreateJobParams) // Verify parameter mapping
submit_job(job_id: String) // Ensure consistent parameter types
```

### Key Technical Decisions Already Made
- **CoreClientFactory Pattern** - Client switching mechanism already established and working
- **Store Architecture** - UI stores properly abstracted to consume any IPC client implementation
- **Type System Alignment** - Rust and TypeScript types already aligned via serde attributes

### Architecture Patterns to Follow
- **Dependency Injection** - Use existing service container patterns for demo mode dependencies
- **Error Handling** - Follow established Result<T> patterns and error categorization
- **State Management** - Build on existing Svelte store patterns and reactive updates

## ⚠️ Critical Constraints & Requirements

### Security (Non-Negotiable)
- Never log or persist SSH passwords in either mode
- Clear credentials from memory on disconnect
- Demo mode must not accidentally connect to real clusters
- Validate all user inputs in both demo and real modes
- Maintain existing security patterns from backend implementation

### Quality Requirements
- Both demo and real modes must work reliably
- Mode switching must be seamless and immediate
- All existing tests must continue passing
- Type safety maintained across IPC boundary
- Follow established architectural patterns from existing codebase

### Integration Requirements
- Build on existing CoreClientFactory infrastructure
- Preserve all current demo functionality and test data
- Integrate with existing SSH/SFTP backend without modification
- Respect established error handling and retry patterns
- Maintain compatibility with existing UI components

## 🤝 Getting Help

### When You Need Guidance
- **IPC boundary questions** - Check `docs/API.md` and compare `src/lib/ports/` with `src-tauri/src/commands/`
- **Type alignment issues** - Review serde attributes in `src-tauri/src/types/core.rs`
- **UI integration questions** - See existing store patterns in `src/lib/stores/session.ts`
- **Demo mode questions** - Look at existing mock client in `src/lib/ports/coreClient-mock.ts`

### Communication Protocol
- **Test IPC commands individually** before attempting full integration
- **Validate type alignment** with simple connection test first
- **Present integration approach** before implementing demo mode toggle
- **Share progress updates** with concrete examples of what's working

## 🎯 Your First Steps

1. **Read all required documentation** listed above, focusing on API.md and existing IPC client code
2. **Run investigation commands** to understand current command signature mismatches
3. **Test basic IPC connection** by fixing ConnectParams alignment first
4. **Plan demo mode toggle approach** based on existing CoreClientFactory patterns
5. **Create integration test plan** for both demo and real modes
6. **Get approval for your integration approach** before implementing major changes

## Success Metrics
- Demo mode preserves rich mock experience for demonstrations
- Real mode enables full cluster integration workflows
- Seamless switching between modes with clear user indication
- All existing UI components work in both modes
- Type-safe integration across IPC boundary
- Both modes support development and production workflows
- Clean, maintainable code following project standards

## Task Management (CRITICAL)
- **Work from active task** in `tasks/active/phase-6-1-ui-backend-integration.md`
- **Update progress** as you fix IPC boundary issues and implement demo mode
- **Document decisions** about mode switching implementation
- **Get approval** before major architectural changes to CoreClientFactory

Remember: This integrates two complete systems (Phase 3 UI + Phases 1-5 backend). **Leverage established patterns** and **test incrementally** rather than attempting full integration at once. The demo mode toggle preserves valuable demonstration capabilities while enabling real functionality.
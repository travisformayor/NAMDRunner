# Phase 1 Milestone 1.3: Connection Architecture Implementation

## Mission: Establish Connection Architecture Foundation

You are implementing **Phase 1 Milestone 1.3** of NAMDRunner, building the connection architecture and interface definitions that will support secure SSH/SFTP connectivity to SLURM clusters. This is **architecture work** - you're defining interfaces, patterns, and mock implementations, not building actual SSH connections yet.

## 🎯 Core Objectives

### Primary Goals
1. **Connection Interface Design** - Define clean, testable interfaces for SSH/SFTP operations
2. **State Management Architecture** - Robust connection state transitions and persistence
3. **Error Handling Strategy** - Comprehensive, user-friendly error management system
4. **Remote Directory Patterns** - Consistent directory structure and path management
5. **Validation Framework** - Connection testing and validation utilities

### Success Definition
By completion, the project should have:
- Complete TypeScript interfaces for all connection operations
- Observable connection state management working with mocks
- Comprehensive error handling with actionable user guidance
- Well-defined remote directory structure patterns
- Enhanced mock system supporting all new connection patterns

## 📋 Required Reading (Critical - Read First!)

### Essential Specifications (READ FIRST)
- `README.md` - Project overview and quick start
- `docs/project-spec.md` - **WHAT** we're building (business requirements)
- `docs/technical-spec.md` - **HOW** to build it (architecture, tech stack, coding standards)
- `docs/agent-capabilities.md` - **Complete reference for available tools and testing infrastructure**
- `CLAUDE.md` - Development guidelines and workflow

### Current Phase Context
- `tasks/active/phase1-milestone1.3-connection-foundation.md` - **YOUR DETAILED TASK PLAN**
- `tasks/roadmap.md` - **Phase 1 scope and milestones** (your roadmap)
- `docs/architecture.md` - Implementation progress tracker (update as you build)
- `tasks/phase1-interface-definitions.md` - Complete IPC interface contracts

### Current Implementation Status  
- `tasks/completed/phase1-milestone1.2-mock-infrastructure.md` - Recently completed testing infrastructure
- `tasks/completed/phase1-milestone1.1-foundation.md` - Foundation work completed

### Reference Implementation Knowledge
- `docs/reference/slurm-commands-reference.md` - **Working SLURM patterns** and command reference
- `docs/reference/python-implementation-reference.md` - Comprehensive lessons from Python implementation
- `docs/data-spec.md` - Database schema and JSON metadata formats
- `src/lib/ports/coreClient-mock.ts` - Existing mock client patterns to extend

### Testing Infrastructure (Ready to Use!)
**The previous milestone built comprehensive testing tools for you:**
- `npm run test:ui` - Fast UI testing with Playwright + mock backend
- `npm run test:e2e` - Full desktop app testing with WebdriverIO + Tauri
- `npm run dev` - Vite dev server for UI development  
- `npm run test` - Vitest unit tests
- `cargo test` - Rust unit tests
- Complete CI/CD pipeline for Linux and Windows builds
- Agent debug toolkit with visual debugging capabilities
- `docs/testing-spec.md` - Testing strategy, debugging workflows, CI setup

## 🗺️ Implementation Approach

### Phase A: Interface Design (Priority 1)
**Goal**: Define clean, comprehensive TypeScript interfaces

#### A1. Connection Core Interfaces
```typescript
// Focus: src/lib/types/connection.ts
- SSHConnection interface with all operations
- SFTPConnection interface for file operations  
- ConnectionStateManager for state transitions
- SessionManager for persistence and recovery
```

**Key Design Principles**:
- **Dependency injection ready** - All interfaces should be mockable
- **Observable patterns** - State changes must be observable for UI
- **Error-first design** - Every operation returns Result<T, Error> patterns
- **Future-proof** - Design for extensibility without breaking changes

#### A2. Error Handling Architecture
```typescript
// Focus: src/lib/types/errors.ts  
- ConnectionError with categories and recovery hints
- ErrorRecoveryStrategy for different error types
- User-friendly error messages with actionable guidance
```

**Critical Requirements**:
- Categorized errors (Network, Auth, Timeout, Permission)
- Actionable user guidance for every error type
- Retry strategies with exponential backoff
- No sensitive information exposed in error messages

### Phase B: State Management (Priority 2)
**Goal**: Robust connection state with clear transitions

#### B1. Connection State Machine
```typescript
// Focus: src/lib/services/connectionState.ts
- ConnectionState enum with all possible states
- State transition validation and logging
- Observable state changes for UI integration
- Persistence across application restarts
```

#### B2. Session Management
```typescript
// Focus: src/lib/services/sessionManager.ts  
- Session validation and expiration handling
- Secure session persistence (no credentials stored)
- Automatic session refresh scheduling
- Clean session cleanup on disconnect
```

### Phase C: Directory & Validation (Priority 3) 
**Goal**: Consistent remote directory patterns and validation

#### C1. Remote Directory Management
```typescript
// Focus: src/lib/services/directoryManager.ts
- Standardized path generation functions
- Directory setup and validation utilities
- Job workspace organization patterns
- Cleanup and maintenance operations
```

#### C2. Connection Validation Framework
```typescript
// Focus: src/lib/services/connectionValidator.ts
- Multi-stage connection validation
- SSH and SFTP capability testing
- SLURM access verification
- Comprehensive validation reporting
```

### Phase D: Enhanced Mocks (Priority 4)
**Goal**: Update existing mock system to support new patterns

#### D1. Mock State Management
```typescript
// Enhance: src/lib/ports/coreClient-mock.ts
- Realistic connection state transitions
- Error scenario simulation
- Deterministic behavior for tests
- Integration with existing test data manager
```

## 🔧 Technical Implementation Guidelines

### TypeScript Interface Patterns
```typescript
// Use Result patterns for error handling
type Result<T, E = Error> = { success: true; data: T } | { success: false; error: E };

// Observable patterns for state management  
interface Observable<T> {
  subscribe(observer: (value: T) => void): Unsubscribe;
  getValue(): T;
}

// Dependency injection patterns
interface ConnectionFactory {
  createSSHConnection(config: ConnectionConfig): SSHConnection;
  createSFTPConnection(config: ConnectionConfig): SFTPConnection;
}
```

### State Management Patterns
```typescript
// Clear state machine with validation
class ConnectionStateMachine {
  private validTransitions: Map<ConnectionState, ConnectionState[]>;
  
  canTransition(from: ConnectionState, to: ConnectionState): boolean;
  transition(to: ConnectionState, reason?: string): void;
  onStateChange(callback: (state: ConnectionState) => void): void;
}
```

### Error Handling Patterns  
```typescript
// User-friendly error messages
interface UserError {
  title: string;           // "Connection Failed"
  message: string;         // "Cannot reach cluster host"
  suggestions: string[];   // ["Check network", "Verify host"]
  retryable: boolean;      // Can user retry this operation?
}
```

## ⚠️ Critical Implementation Notes

### Architecture Focus (Not Implementation)
- **This milestone is about design, not SSH implementation**
- Focus on interfaces, patterns, and architecture decisions
- Mock implementations should be realistic but don't need full SSH logic
- Prepare for Phase 2 SSH implementation without building it

### Technology Constraints (From Phase 1)
- **Password-only SSH** (no key support per cluster requirements)
- **Never persist credentials** (memory only, clear on disconnect)
- **Type-safe IPC boundary** (strict contracts between TS/Rust)
- **Offline-first design** (local SQLite cache, manual sync)

### Connection Security Considerations
- **Never log or persist SSH passwords** - Session tokens only, not credentials  
- **Clear credentials from memory on disconnect** - Implement secure cleanup patterns
- **Error message safety** - No credential leakage in error messages
- **Timeout handling** - All operations must have reasonable timeouts
- **Use minimal Tauri permissions** - Only what's needed for SSH/SFTP
- **Validate all user inputs** - Prevent injection attacks

### Integration with Existing System
- **Extend existing mocks** - Build on `coreClient-mock.ts` patterns
- **Use test data manager** - Integrate with `testDataManager.ts` scenarios
- **Follow IPC patterns** - Match existing TypeScript ↔ Rust boundaries
- **Maintain test compatibility** - Don't break existing tests
- **Port/adapter pattern** for external dependencies (SSH, SLURM)
- **Reactive Svelte stores** for session and job state

### Key Technical Decisions Already Made
- **SQLite schema** - Use structure from `docs/data-spec.md`
- **SLURM commands** - Use proven patterns from `docs/reference/slurm-commands-reference.md`
- **Job metadata** - JSON format defined in `tasks/phase1-interface-definitions.md`
- **Directory structure** - `/projects/$USER/namdrunner_jobs/` pattern

## 🚨 Common Pitfalls to Avoid

### Architecture Antipatterns
- **Don't**: Over-engineer interfaces - keep them focused and simple
- **Don't**: Implement actual SSH operations - this is architecture only
- **Don't**: Create tight coupling between components - use dependency injection
- **Don't**: Ignore error scenarios - design error-first from the beginning

### State Management Issues  
- **Don't**: Create complex state machines - focus on essential states only
- **Don't**: Make state transitions unpredictable - all transitions must be deterministic
- **Don't**: Forget cleanup - all resources must have proper cleanup patterns
- **Don't**: Ignore concurrency - state changes must be thread-safe

### Mock Implementation Problems
- **Don't**: Make mocks too simple - they should catch design issues
- **Don't**: Make mocks too complex - focus on interface validation
- **Don't**: Ignore existing patterns - extend current mock system
- **Don't**: Create inconsistent behavior - mocks should be deterministic

## 📚 Implementation Resources

### Key Files to Study
- **Existing Mock System**: `src/lib/ports/coreClient-mock.ts` - Current patterns to extend
- **Type Definitions**: `src/lib/types/api.ts` - Existing type patterns to follow  
- **Test Infrastructure**: `src/lib/test/fixtures/testDataManager.ts` - Test scenario management
- **IPC Boundaries**: `src-tauri/src/types/` - Rust type definitions to align with

### Testing Tools Available
- **UI Testing**: `npm run test:ui` - Fast iteration with mock backend
- **E2E Testing**: `npm run test:e2e` - Full desktop app testing
- **Agent Debug Toolkit**: Visual debugging with screenshots
- **Unit Testing**: `npm run test` - Vitest for TypeScript components

### Documentation to Update
- `docs/architecture.md` - Add connection architecture decisions
- Connection interface documentation with examples
- Error handling strategy guide for developers
- State management pattern documentation

## 🎯 Milestone Completion Checklist

### Implementation Complete
- [ ] All connection interfaces defined in TypeScript
- [ ] Connection state management architecture implemented with mocks
- [ ] Comprehensive error handling system with user-friendly messages
- [ ] Remote directory structure patterns established
- [ ] Connection validation framework built and tested
- [ ] Enhanced mock implementations support all new patterns
- [ ] Unit tests written for all components with >80% coverage

### Documentation Updated
- [ ] Architecture decisions documented with clear rationale
- [ ] Interface patterns documented with usage examples
- [ ] Error handling strategy documented for developers
- [ ] State management patterns explained
- [ ] Integration guide for future SSH implementation

### Quality Validation
- [ ] All acceptance criteria from task plan validated
- [ ] TypeScript compilation succeeds without warnings
- [ ] All unit tests pass consistently
- [ ] Mock implementations validate interface designs
- [ ] Error scenarios comprehensive and user-tested
- [ ] State transitions work correctly in all test scenarios

## 🚀 Getting Started

### Step 1: Understand Current State
```bash
# ESSENTIAL: Read the capability reference first
cat docs/agent-capabilities.md

# Verify the working infrastructure  
cd /media/share/namdrunner
source ~/.cargo/env
npm run dev              # Should start successfully
npm run test:ui          # Should run UI tests
npm run test:e2e         # Should run E2E tests
```

### Step 2: Study Existing Patterns
```bash
# Understand current architecture
cat src/lib/ports/coreClient-mock.ts      # Current mock patterns
cat src/lib/types/api.ts                  # Existing type definitions
cat src/lib/test/fixtures/testDataManager.ts  # Test scenario patterns
```

### Step 3: Create Your Implementation Plan
1. Read the detailed task plan thoroughly  
2. Design your interface hierarchy on paper first
3. Plan state machine transitions and validation
4. Map out error scenarios and recovery strategies
5. Get approval for your architectural approach before coding

### Step 4: Implement Incrementally
- **Start with core interfaces** - Build foundation types first
- **Add state management** - Implement observable state patterns
- **Design error handling** - Focus on user experience
- **Enhance mocks** - Update existing mock system
- **Test thoroughly** - Use both UI and E2E testing tools

## 💡 Success Tips

- **Design first, implement second** - Sketch interfaces before coding
- **Use the testing tools** - Leverage the comprehensive testing infrastructure built in 1.2
- **Study existing patterns** - Follow established mock and type patterns
- **Focus on user experience** - Especially for error messages and state feedback
- **Think about Phase 2** - Design interfaces that will support real SSH implementation
- **Test error scenarios** - Use mock error injection to validate error handling

Remember: This milestone creates the **architectural foundation** for all connection operations. Quality architecture here enables smooth SSH implementation in Phase 2!

## 🔄 Development Workflow

### Daily Development
```bash
# Testing and validation  
npm run test            # Unit tests
npm run test:ui         # UI integration testing
npm run lint            # Code quality checks
npm run check           # TypeScript validation
```

### When You Need Help
- **SLURM integration questions** - Check `docs/reference/slurm-commands-reference.md` first
- **Data format questions** - See `docs/data-spec.md`
- **Architecture decisions** - Review `docs/architecture.md` and ask for input
- **Python implementation questions** - Look at `docs/reference/python-implementation-reference.md`
- **Read `docs/agent-capabilities.md`** first for complete tooling reference
- **Use investigation tools** - `npm run test:ui` for debugging UI integration  
- **Ask specific questions** rather than general "how do I..." queries

### Task Management (CRITICAL)
- **Create task files** using `tasks/templates/task.md` before coding
- **Work on one task at a time** - move to `tasks/active/` when starting
- **Update architecture.md** as you implement each component
- **Get approval** for your implementation plan before coding
- **Present your plan** before starting major implementation work
- **Update docs** as you learn and implement
- **Share progress updates** with concrete examples

### Quality Requirements
- Comprehensive unit test coverage with mocks (>80%)
- Type safety across the entire IPC boundary
- Proper error handling and user feedback
- Follow coding standards in `docs/technical-spec.md`

### Reference Implementation Respect
- Use proven SLURM integration patterns
- Respect working directory structures
- Follow established JSON schemas
- Learn from Python version mistakes

Remember: This is **not a migration** but a **new application with reference implementation**. Use the Python patterns as a starting point, then improve and modernize with Tauri's capabilities.

The comprehensive testing infrastructure is ready to support your development. Focus on clean architecture and the SSH implementation in Phase 2 will build smoothly on your foundation!

Good luck! 🍀
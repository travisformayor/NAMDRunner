# Phase 2 Milestone 2.1 Implementation Prompt for Claude Code Agent

## Mission: Add Real SSH/SFTP Connectivity Alongside Mocks

You are implementing **Phase 2 Milestone 2.1** of NAMDRunner, adding real SSH/SFTP operations alongside the existing mock foundation. Your mission is to create production SSH connectivity while preserving the development workflow and testing infrastructure.

## 🎯 Core Objectives

### Primary Goals
1. **Add Real SSH Connectivity** - Create new Tauri SSH service with ssh2 crate alongside existing mocks
2. **SFTP File Operations** - Implement actual file upload/download and directory management in Rust
3. **Command Execution** - Enable real command execution on remote SLURM clusters via IPC
4. **Preserve Development Workflow** - Keep existing mock client for fast UI development and testing
5. **Robust Error Handling** - Map SSH errors to established error handling patterns

### Success Definition
By completion, the application should:
- Support both mock (development) and real (production) SSH operations via client factory
- Establish real SSH connections to SLURM clusters with password authentication
- Execute actual commands and return real stdout/stderr/exit codes
- Transfer files via SFTP with basic progress tracking and error handling
- Preserve existing development workflow with fast mock-based testing
- Provide meaningful error diagnostics for connection failures
- Maintain security (no credential persistence, memory cleanup)

## 📋 Required Reading (Critical - Read First!)

### Essential Context
- `tasks/active/phase2-milestone2.1-ssh-sftp-implementation.md` - **YOUR DETAILED TASK PLAN**
- `tasks/roadmap.md` - Phase 2 objectives and context
- `docs/architecture.md` - Phase 1 foundation to build upon
- `docs/technical-spec.md` - Development standards and Rust patterns

### Foundation Architecture (Phase 1)
**Connection Management** (leverage these existing patterns):
- `src/lib/services/connectionState.ts` - State machine for connection lifecycle
- `src/lib/services/sessionManager.ts` - Session handling and validation
- `src/lib/services/connectionValidator.ts` - Connection validation framework
- `src/lib/types/connection.ts` - Connection interfaces and types
- `src/lib/types/errors.ts` - Error categorization and handling

**Current Mock Implementation** (to preserve alongside real):
- `src/lib/ports/coreClient-mock.ts` - Mock SSH operations (keep for development)
- `src/lib/ports/clientFactory.ts` - Chooses mock vs real implementation
- Mock methods: `connect()`, `executeCommand()`, SFTP operations
- Keep the same interface contracts for both mock and real implementations

### Reference Implementation Patterns
- `docs/reference/python-implementation-reference.md` - SSH/SFTP lessons from Python version  
- `docs/reference/slurm-integration-examples.md` - Working SLURM command patterns
- `docs/api-spec.md` - IPC contracts that must be maintained

## 🏗️ Implementation Strategy

### Phase 2.1 Architecture Pattern
**Add Real Alongside Mock**: The core strategy is to add real SSH implementations while preserving existing mock infrastructure and development workflow.

```
Development: Frontend → ICoreClient (mock) → Simulated responses
Production:  Frontend → ICoreClient (tauri) → Rust SSH Service → ssh2 crate → Cluster
Factory:     clientFactory.ts chooses mock vs real based on environment
```

### Core Implementation Areas

#### 1. New Tauri SSH Implementation  
**Goal**: Create `coreClient-tauri.ts` with real SSH via Rust service
- Create new Rust SSH service in `src-tauri/` using ssh2 crate
- Implement Tauri IPC commands for SSH operations
- Add password authentication (no SSH keys per cluster requirements)
- Preserve existing mock client for development use

#### 2. Command Execution System  
**Goal**: Implement real SSH command execution in Rust service
- Execute actual commands on remote systems via SSH
- Capture real stdout, stderr, and exit codes
- Support command timeouts and cancellation
- Implement module loading (`module load slurm/alpine`)

#### 3. SFTP File Operations  
**Goal**: Implement real SFTP operations in Rust service
- File uploads/downloads with basic progress tracking
- Directory creation, listing, and deletion
- Proper error handling for permissions and disk space
- Focus on reliability over advanced features

#### 4. Error Integration
**Goal**: Map SSH errors to existing error handling system
- Preserve existing error categories and user-friendly messages  
- Add SSH-specific context while maintaining abstraction
- Ensure retry logic works with real network conditions
- Maintain diagnostic information for debugging

## 🔧 Technical Implementation Guidelines

### Rust Integration (Backend)
Since Tauri v2 uses Rust backend, you'll likely need to:
1. **Add SSH Dependencies**: Add ssh2 crate to `src-tauri/Cargo.toml`
2. **Implement SSH Service**: Create Rust SSH service that handles actual connections
3. **IPC Integration**: Connect Rust SSH service to existing Tauri commands
4. **Error Mapping**: Map Rust SSH errors to frontend error types

### Security Requirements (Critical)
- **Password Security**: Store passwords only in memory, clear on disconnect
- **No Persistence**: Never store credentials to disk or logs
- **Session Cleanup**: Properly dispose of SSH sessions and clear memory
- **Error Messages**: Don't leak credential information in error messages

### Testing Strategy
- **Frontend Testing**: Continue using existing mock client for fast UI development and testing
- **Rust Unit Tests**: Test SSH service business logic with simple mocked ssh2 responses (no real connections)
- **Manual Validation**: Developers occasionally test real SSH functionality against actual clusters
- **No Test Infrastructure**: Avoid complexity of test servers or integration test environments
- **Focus on Logic**: Test our business logic, not ssh2 crate functionality

## 🎯 Acceptance Criteria

### Must Have (Required for Milestone 2.1)
- [ ] Client factory chooses between mock and real implementations correctly
- [ ] Mock client continues to work for development workflow
- [ ] Real SSH connection establishes with password authentication via Rust service
- [ ] Commands execute on remote system and return actual results
- [ ] File upload/download works via SFTP
- [ ] Directory operations create/list/delete real directories
- [ ] Error handling provides useful diagnostics for SSH failures
- [ ] Memory cleanup prevents credential leaks
- [ ] All existing interfaces and contracts preserved

### Should Have (Important for Quality)
- [ ] Connection recovery after network interruption
- [ ] Progress tracking for large file transfers  
- [ ] Timeout handling for all SSH operations
- [ ] Module loading commands work with cluster environment
- [ ] Comprehensive error diagnostics for troubleshooting

### Could Have (Nice to Have)
- [ ] Connection keepalive to prevent session timeout
- [ ] Concurrent file transfer support
- [ ] SSH connection pooling optimization
- [ ] Advanced retry strategies for different error types

## 🚨 Critical Success Factors

### 1. Preserve Development Workflow  
- **Keep mock client**: `coreClient-mock.ts` remains for fast development and testing
- **Don't break interfaces**: Frontend code should work unchanged
- **Preserve patterns**: Use existing ConnectionStateMachine, error types, etc.
- **Add, don't replace**: Both mock and real implementations coexist

### 2. Security First
- **No credential storage**: Memory only, clear on disconnect
- **Error safety**: Don't expose passwords or sensitive info in logs/errors
- **Session management**: Proper cleanup prevents security vulnerabilities

### 3. Robust Error Handling
- **Network errors**: Handle timeouts, connection drops, DNS failures
- **Authentication errors**: Clear messages for wrong passwords, account issues
- **Permission errors**: Helpful guidance for file access problems
- **Cluster errors**: Module loading failures, command not found, etc.

## 🔍 Testing Approach

### Frontend Testing (Unchanged)
- **Mock client**: Continue using `coreClient-mock.ts` for all UI testing
- **Unit tests**: TypeScript tests remain fast and reliable
- **Error scenarios**: Test error handling logic with mock responses
- **State management**: Test ConnectionStateMachine with simulated scenarios

### Rust SSH Service Testing
- **Business logic**: Test command parsing, error mapping, path handling  
- **Mocked responses**: Return canned ssh2 responses, no network calls
- **Error mapping**: Test SSH error to application error conversion
- **Memory safety**: Verify credential cleanup in unit tests

### Manual Quality Assurance
- **Real Cluster Testing**: Developers manually test against actual SLURM clusters occasionally  
- **Error Recovery**: Manually verify reconnection after network failures
- **Security Verification**: Ensure no credential persistence or logging
- **No Automated Integration**: Keep it simple - avoid complex test infrastructure

## 🎓 Key Lessons from Python Implementation

### What Worked Well
- **Paramiko Integration**: SSH operations were reliable once properly configured
- **Connection Pooling**: Reusing SSH connections improved performance
- **Error Categorization**: Clear error types helped users troubleshoot
- **Offline Development**: Mock fallback enabled development without cluster access

### What to Avoid
- **Complex Connection Logic**: Keep SSH management simple and focused
- **Blocking Operations**: All SSH operations should be async to avoid UI freezing
- **Poor Error Messages**: SSH errors can be cryptic, provide better context
- **Memory Leaks**: SSH connections must be properly cleaned up

## 🚀 Getting Started

### Step 1: Review Current State
1. Read the detailed task plan: `tasks/active/phase2-milestone2.1-ssh-sftp-implementation.md`
2. Understand existing mock implementation: `src/lib/ports/coreClient-mock.ts`
3. Review connection interfaces: `src/lib/types/connection.ts`
4. Understand error handling: `src/lib/types/errors.ts`

### Step 2: Plan Implementation
1. Identify where Rust SSH code will live (likely `src-tauri/src/ssh/`)
2. Plan how ssh2 operations will integrate with Tauri IPC
3. Design error mapping from Rust SSH errors to TypeScript error types
4. Plan testing strategy for real SSH operations

### Step 3: Implement Incrementally  
1. Start with basic SSH connection establishment
2. Add command execution with simple commands
3. Implement SFTP file operations
4. Add error handling and recovery
5. Integrate with connection state management
6. Test thoroughly and refine

## 💡 Success Tips

1. **Preserve Interfaces**: Frontend should work unchanged after implementation
2. **Test Early**: Set up SSH testing environment as soon as possible
3. **Security Focus**: Always consider credential security in every decision
4. **Error Quality**: Spend time on clear, actionable error messages
5. **Documentation**: Update architecture docs as you implement patterns

Good luck! You're building on an excellent Phase 1 foundation. The interfaces are well-designed and the testing infrastructure is solid. Focus on implementing robust SSH operations that integrate seamlessly with the existing architecture.
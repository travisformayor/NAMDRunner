# Phase 2 Milestone 2.1 - SSH/SFTP Implementation

## Objective
Add real SSH/SFTP connectivity using Rust ssh2 crate alongside existing mocks, enabling production connections while preserving development workflow.

## Context
- **Current state**: Phase 1 complete with comprehensive mock implementations, connection state management, and validation framework
- **Desired state**: Real SSH/SFTP operations added alongside existing mocks for multi-environment support
- **Dependencies**: Phase 1 foundation with connection interfaces, state management, and error handling
- **Python reference**: Python implementation used paramiko for SSH - lessons learned about connection management and error handling
- **Testing approach**: Pragmatic "good enough" testing - no test servers, focus on business logic over protocol testing

## Implementation Plan
1. [ ] **Rust SSH Dependencies & Setup**
   - Add ssh2 crate to Cargo.toml
   - Configure SSH connection builder with timeout settings
   - Set up SFTP subsystem initialization

2. [ ] **Tauri SSH Service Implementation**
   - Create new `coreClient-tauri.ts` that calls Rust SSH service via IPC
   - Implement Rust SSH service with ssh2 crate integration
   - Add password authentication with secure credential handling (memory-only)
   - Preserve existing mock client for development workflow

3. [ ] **SSH Command Execution**
   - Implement real SSH channel operations in Rust service
   - Capture stdout/stderr and exit code handling
   - Add command timeout and cancellation support
   - Implement module loading commands (`module load slurm/alpine`)

4. [ ] **SFTP File Operations**
   - Implement real SFTP operations in Rust service
   - Add directory creation, file listing, and deletion
   - Handle SFTP-specific errors and permissions
   - Basic progress tracking for file transfers

5. [ ] **Connection Management Integration**
   - Connect Rust SSH service to existing ConnectionStateMachine via IPC
   - Implement connection keepalive and session expiration detection
   - Add connection recovery and reconnection logic
   - Integrate with existing session management

6. [ ] **Error Handling & Testing**
   - Map SSH/SFTP errors to existing error categories in Rust service
   - Implement retry logic for transient network errors  
   - Add debugging information for connection failures
   - Create simple Rust unit tests with mocked ssh2 responses (business logic only)
   - Preserve existing TypeScript mock client for UI testing

## Success Criteria
- [ ] Real SSH connection established with password authentication
- [ ] Command execution returns actual stdout/stderr from remote system
- [ ] File upload/download operations work via SFTP
- [ ] Directory operations (create, list, delete) functional
- [ ] Module loading commands execute successfully
- [ ] Connection state transitions work with real connections
- [ ] Error handling provides useful diagnostics
- [ ] Frontend mock client preserved for development workflow
- [ ] Client factory correctly chooses mock vs real implementation
- [ ] Rust unit tests validate SSH service business logic (no real connections)
- [ ] Connection recovery works after network interruption
- [ ] Memory cleanup occurs on disconnect (no credential leaks)

## Technical Notes

### SSH2 Crate Integration
- Use ssh2::Session for connection management
- Implement proper timeout handling for all operations
- Ensure thread safety for async operations in Tauri context

### Security Requirements
- Password-only authentication (no SSH key support per cluster requirements)
- In-memory credential storage only - clear on disconnect
- No credential logging or persistence
- Proper session cleanup to prevent memory leaks

### Connection Architecture
- Leverage existing ConnectionStateMachine for state management
- Use existing ValidationFramework to verify real connections
- Integrate with PathResolver for consistent remote paths
- Maintain compatibility with existing service container pattern

### Error Mapping Strategy
- Map SSH errors to existing CONNECTION_ERRORS categories
- Preserve user-friendly error messages established in Phase 1
- Add SSH-specific error context while maintaining abstraction
- Ensure error recovery strategies work with real network conditions

## References
- Python implementation: Paramiko-based SSH manager with similar patterns
- Related docs: `docs/api-spec.md` for connection interfaces
- External docs: [ssh2 crate documentation](https://docs.rs/ssh2/)
- Connection patterns: `src/lib/services/connectionState.ts`
- Error handling: `src/lib/types/errors.ts`

## Progress Log
[Start Date] - Task created, ready to begin SSH/SFTP implementation

## Completion Process
After implementation and testing:
- [ ] Run code review using `.claude/agents/review-refactor.md`
- [ ] Implement recommended refactoring improvements
- [ ] Update and archive task to `tasks/completed/phase2-milestone2.1-ssh-sftp-implementation.md`
- [ ] Update `tasks/roadmap.md` progress
- [ ] Update `docs/architecture.md` with SSH/SFTP implementation details

## Testing Approach

### Frontend Development & Testing
- **Continue using mocks**: `coreClient-mock.ts` preserved for fast UI development
- **Client factory selection**: Environment-based switching between mock and real implementations
- **TypeScript unit tests**: Continue testing UI logic, state management, error handling with mocks

### Rust SSH Service Testing  
- **Business logic focus**: Test command parsing, error mapping, file path handling
- **Simple mocked responses**: Return canned ssh2 responses, no real network operations
- **Unit test coverage**: Test our logic, not ssh2 crate functionality
- **Manual validation**: Developers occasionally test against real clusters

## Open Questions
- [ ] What timeout values work best for different cluster environments?  
- [ ] How should we handle SSH host key verification (likely accept new keys)?
- [ ] Should SFTP operations support resume for interrupted transfers?
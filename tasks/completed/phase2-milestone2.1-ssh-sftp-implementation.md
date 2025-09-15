# Phase 2 Milestone 2.1 - SSH/SFTP Implementation

## Objective
Add real SSH/SFTP connectivity using Rust ssh2 crate alongside existing mocks, enabling production connections while preserving development workflow.

## Context
- **Current state**: Phase 1 complete with comprehensive mock implementations, connection state management, and validation framework
- **Desired state**: Real SSH/SFTP operations added alongside existing mocks for multi-environment support
- **Dependencies**: Phase 1 foundation with connection interfaces, state management, and error handling
- **Python reference**: Python implementation used paramiko for SSH - lessons learned about connection management and error handling
- **Testing approach**: Pragmatic "good enough" testing - no test servers, focus on business logic over protocol testing

## Implementation Plan ✅ COMPLETED
1. [x] **Rust SSH Dependencies & Setup**
   - ✅ Added ssh2, secstr, anyhow crates to Cargo.toml
   - ✅ Configured SSH connection builder with timeout settings
   - ✅ Set up SFTP subsystem initialization

2. [x] **SSH Service Implementation**
   - ✅ Implemented comprehensive SSH service in `src/ssh/` module
   - ✅ Added ssh2 crate integration with connection pooling
   - ✅ Implemented password authentication with SecurePassword wrapper
   - ✅ Preserved existing mock client for development workflow
   - ✅ Added environment-based mock/real mode switching

3. [x] **SSH Command Execution**
   - ✅ Implemented SSH channel operations with CommandExecutor
   - ✅ Added stdout/stderr capture and exit code handling
   - ✅ Implemented command timeout and cancellation support
   - ✅ Added module loading commands and SLURM integration

4. [x] **SFTP File Operations**
   - ✅ Implemented complete SFTP operations in SFTPOperations service
   - ✅ Added directory creation, file listing, and deletion
   - ✅ Implemented SFTP-specific error handling and permissions
   - ✅ Added basic progress tracking for file transfers

5. [x] **Connection Management Integration**
   - ✅ Connected SSH service to ConnectionManager with proper lifecycle
   - ✅ Implemented connection keepalive and session management
   - ✅ Added connection recovery and reconnection logic
   - ✅ Integrated with existing validation framework

6. [x] **Error Handling & Testing**
   - ✅ Mapped SSH/SFTP errors to structured error categories
   - ✅ Implemented retry logic for transient network errors
   - ✅ Added comprehensive debugging information for connection failures
   - ✅ Created 43 focused unit tests with mock infrastructure (business logic only)
   - ✅ Preserved existing TypeScript mock client for UI testing
   - ✅ Cleaned up anti-pattern tests and focused on valuable business logic validation

## Success Criteria
- [x] Real SSH connection established with password authentication
- [x] Command execution returns actual stdout/stderr from remote system
- [x] File upload/download operations work via SFTP
- [x] Directory operations (create, list, delete) functional
- [x] Module loading commands execute successfully
- [x] Connection state transitions work with real connections
- [x] Error handling provides useful diagnostics
- [x] Frontend mock client preserved for development workflow
- [x] Client factory correctly chooses mock vs real implementation
- [x] Rust unit tests validate SSH service business logic (no real connections)
- [x] Connection recovery works after network interruption
- [x] Memory cleanup occurs on disconnect (no credential leaks)

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
- [x] Run code review using `.claude/agents/review-refactor.md`
- [x] Implement recommended refactoring improvements
- [x] Update and archive task to `tasks/completed/phase2-milestone2.1-ssh-sftp-implementation.md`
- [x] Update `tasks/roadmap.md` progress
- [x] Update `docs/architecture.md` with SSH/SFTP implementation details

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
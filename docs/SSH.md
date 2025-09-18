# SSH & SFTP Integration Guide

**SSH/SFTP connection management and file operations** - This document covers all aspects of SSH connectivity, SFTP file management, security patterns, and implementation best practices for NAMDRunner.

> **For IPC interfaces and command contracts**, see [`API.md`](API.md)
> **For system architecture principles**, see [`ARCHITECTURE.md`](ARCHITECTURE.md)
> **For development workflow and coding standards**, see [`CONTRIBUTING.md`](CONTRIBUTING.md)

## Table of Contents
- [Connection Management](#connection-management)
  - [Password-Only Authentication](#password-only-authentication)
  - [Session Lifecycle](#session-lifecycle)
  - [Connection State Machine](#connection-state-machine)
- [SFTP Operations](#sftp-operations)
  - [File Upload Patterns](#file-upload-patterns)
  - [File Download Operations](#file-download-operations)
  - [Directory Management](#directory-management)
- [Security Patterns](#security-patterns)
  - [Secure Password Handling](#secure-password-handling)
  - [Memory Management](#memory-management)
  - [Connection Cleanup](#connection-cleanup)
  - [Input Validation](#input-validation)
- [Error Handling](#error-handling)
  - [SSH Error Classification](#ssh-error-classification)
  - [Retry Strategies](#retry-strategies)
  - [Recovery Patterns](#recovery-patterns)
- [Performance & Optimization](#performance--optimization)
  - [Connection Pooling](#connection-pooling)
  - [Async Patterns](#async-patterns)
  - [Background Operations](#background-operations)
- [Implementation Patterns](#implementation-patterns)
  - [Rust Implementation](#rust-implementation)
  - [TypeScript Integration](#typescript-integration)
  - [Mock/Real Mode Switching](#mockreal-mode-switching)
- [Testing & Development](#testing--development)
  - [SSH-Specific Testing Notes](#ssh-specific-testing-notes)
- [Python Implementation Lessons](#python-implementation-lessons)
  - [Key SSH/SFTP Insights Applied to Rust Implementation](#key-sshsftp-insights-applied-to-rust-implementation)
- [Troubleshooting](#troubleshooting)
  - [SSH-Specific Issues](#ssh-specific-issues)

## Connection Management

### Password-Only Authentication

NAMDRunner uses password-only SSH authentication to comply with cluster requirements that disable SSH key access.

#### Connection Requirements
- **Password authentication only** - SSH keys are not supported by target clusters
- **Interactive prompts** - Handle keyboard-interactive authentication when required
- **Session persistence** - Maintain connection for multiple operations
- **Automatic cleanup** - Clear credentials from memory on disconnect

#### Authentication Flow
See `src-tauri/src/ssh/manager.rs:25-48` for the actual connection implementation using SecurePassword and proper async patterns.

### Session Lifecycle

#### Connection States
```typescript
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
```

#### State Transitions
- **Disconnected** → **Connecting**: User initiates connection
- **Connecting** → **Connected**: Authentication succeeds
- **Connecting** → **Disconnected**: Authentication fails
- **Connected** → **Expired**: Session timeout or network failure
- **Connected** → **Disconnected**: User-initiated disconnect
- **Expired** → **Connecting**: Automatic reconnection attempt

#### Session Management
See `src-tauri/src/ssh/manager.rs:51-57` for connection cleanup and `src-tauri/src/ssh/connection.rs:66-109` for the actual connection establishment process.

### Connection State Machine

#### Observable State Management
Frontend connection state management follows the state machine defined in `docs/API.md`. The actual connection commands are implemented in `src-tauri/src/commands/connection.rs`.

## SFTP Operations

### File Upload Patterns

#### Single File Upload
File upload implementation with progress tracking is in `src-tauri/src/ssh/sftp.rs:61-115`. The actual upload process uses 32KB buffers and proper error handling.

#### Batch File Upload
Batch upload operations are handled in `src-tauri/src/commands/files.rs` with individual file uploads using the SFTP operations from `src-tauri/src/ssh/sftp.rs`.

### File Download Operations

#### Download with Retry
File download with retry logic is implemented in `src-tauri/src/ssh/manager.rs:108-123` using the retry patterns from `src-tauri/src/retry.rs`.

### Directory Management

#### Automated Workspace Setup
Directory creation is handled by `src-tauri/src/ssh/sftp.rs:218-248` with recursive directory support. Job workspace setup follows the directory patterns defined in `docs/DB.md`.

## Security Patterns

### Secure Password Handling

#### SecurePassword Implementation
Secure password handling is implemented in `src-tauri/src/security.rs` using the `secstr` crate for automatic memory clearing and secure access patterns.

### Memory Management

> **For complete security requirements and architectural principles**, see [`docs/CONTRIBUTING.md#security-requirements`](CONTRIBUTING.md#security-requirements).

#### Credential Security Summary
- **Passwords exist only in memory during active sessions**
- **Use SecStr for password handling with automatic cleanup**
- **Clear memory on disconnect**
- **Never log or persist credentials**
- **Validate SSH connection before SLURM operations**

### Connection Cleanup

#### Proper Resource Management
Connection cleanup is handled in the Drop implementation for SSHConnection in `src-tauri/src/ssh/connection.rs:158-167`.

### Input Validation

#### Path Safety Validation
Path validation to prevent traversal attacks and ensure safe file operations is implemented in `src-tauri/src/validation.rs`.

## Error Handling

> **For complete error handling contracts and API specifications**, see [`docs/API.md#error-handling-strategy`](API.md#error-handling-strategy).

### SSH Error Classification

#### Error Categories
SSH error classification and mapping is implemented in `src-tauri/src/ssh/errors.rs` with categories for Network, Authentication, Permission, FileSystem, Protocol, Timeout, and Internal errors.

#### Error Mapping
SSH error mapping from ssh2 library errors to application-specific error types is implemented in `src-tauri/src/ssh/errors.rs` with proper categorization and retry logic.

### Retry Strategies

#### Exponential Backoff Implementation
Exponential backoff retry logic with jitter and maximum delay is implemented in `src-tauri/src/retry.rs` and used throughout the SSH operations.

### Recovery Patterns

#### Connection Recovery
Connection state checking and recovery logic is in `src-tauri/src/ssh/manager.rs:59-69`. Note that passwords are not persisted, requiring manual re-authentication for expired sessions.

## Performance & Optimization

### Connection Pooling

#### Reusing Connections
Connection management and lifecycle is handled by the singleton ConnectionManager in `src-tauri/src/ssh/manager.rs`. Multiple operations reuse the same connection instance.

### Async Patterns

#### Non-Blocking SSH Operations
Command execution using `tokio::spawn_blocking` to handle synchronous SSH operations is implemented in `src-tauri/src/ssh/commands.rs`.

### Background Operations

#### Progress Reporting
File transfer progress tracking is implemented in `src-tauri/src/ssh/sftp.rs:61-115` with callback support for UI progress updates.

## Implementation Patterns

> **For architectural principles and coding standards**, see [`docs/CONTRIBUTING.md#developer-standards--project-philosophy`](CONTRIBUTING.md#developer-standards--project-philosophy).

### Rust Implementation

#### Module Organization
SSH modules are organized in `src-tauri/src/ssh/` with connection, manager, commands, sftp, errors, and test_utils modules as shown in `src-tauri/src/ssh/mod.rs`.

### TypeScript Integration

#### Frontend SSH Service
Frontend SSH service integration follows the IPC patterns defined in [`docs/API.md`](API.md) with proper error mapping between Rust and TypeScript.

### Mock/Real Mode Switching

#### Environment-Based Service Selection
Mock mode selection based on environment variables and build configuration is implemented in `src-tauri/src/mode_switching.rs`.

## Testing & Development

> **For complete testing strategies, mock infrastructure setup, and development workflows**, see [`docs/CONTRIBUTING.md#testing-strategy`](CONTRIBUTING.md#testing-strategy).

### SSH-Specific Testing Notes
- Mock SSH infrastructure is in `src-tauri/src/ssh/test_utils.rs`
- Environment-based mock switching via `USE_MOCK_SSH=true`
- Error classification testing focuses on business logic, not ssh2 library

## Python Implementation Lessons

> **For comprehensive lessons learned from 18 months of Python implementation**, see [`docs/reference/python-implementation-reference.md`](reference/python-implementation-reference.md).

### Key SSH/SFTP Insights Applied to Rust Implementation

1. **Manager Separation Pattern**: The Python `SSHManager` separation is maintained in Rust modules (`src-tauri/src/ssh/`)
2. **Command Execution**: Critical pattern of `source /etc/profile && module load slurm/alpine` is implemented in `src-tauri/src/ssh/commands.rs`
3. **Error Handling**: Retry patterns from Python experience inform `src-tauri/src/retry.rs`
4. **Progress Reporting**: Chunked 32KB uploads with progress callbacks are in `src-tauri/src/ssh/sftp.rs`
5. **Mock Mode**: Environment-based mock switching is implemented in `src-tauri/src/mode_switching.rs`

## Troubleshooting

### SSH-Specific Issues
- **Connection timeouts**: Check network and firewall settings
- **Authentication failures**: Verify credentials and account status
- **File transfer failures**: Check disk space and permissions
- **Debug logging**: Use `RUST_LOG=ssh2=debug` for detailed SSH logs

---

*For complete SSH command patterns and cluster-specific details, see [`reference/alpine-cluster-reference.md`](reference/alpine-cluster-reference.md)*

*For IPC interfaces and API contracts, see [`API.md`](API.md)*

*For database schemas and data management, see [`DB.md`](DB.md)*
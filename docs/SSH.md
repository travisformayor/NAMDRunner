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
    - [Frontend IPC Layer](#frontend-ipc-layer)
    - [Connection Error Detection](#connection-error-detection)
    - [Offline Mode Support](#offline-mode-support)
- [Testing & Development](#testing--development)
  - [SSH-Specific Testing Notes](#ssh-specific-testing-notes)
    - [Mock Infrastructure](#mock-infrastructure)
    - [Testing Best Practices](#testing-best-practices)
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

#### Chunked File Upload

**Implementation**: `src-tauri/src/ssh/sftp.rs`

File uploads use chunked transfer with per-chunk flush to prevent timeout accumulation:

```rust
// Upload in 256KB chunks with per-chunk timeout
const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks
let file_transfer_timeout = Duration::from_secs(300); // 5 minutes per chunk

for chunk in file_data.chunks(CHUNK_SIZE) {
    // Each chunk gets fresh 300s timeout window
    remote_file.write_all(chunk)?;
    remote_file.flush()?; // fsync() after each chunk

    // Emit progress event
    emit_progress_event(bytes_uploaded, total_bytes);
}
```

**Benefits:**
- Large files (10MB+) no longer timeout
- Each 256KB chunk has independent 300s timeout
- Progress tracking per chunk
- Prevents timeout accumulation

**Progress Tracking:**
```rust
pub struct FileUploadProgress {
    pub file_name: String,
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}
```

#### Single File Upload
File upload implementation with progress tracking is in `src-tauri/src/ssh/sftp.rs:61-115`. The upload process uses 256KB chunks with per-chunk flush and comprehensive error handling.

#### Batch File Upload
Batch upload operations are handled in `src-tauri/src/commands/files.rs` with individual file uploads using the chunked SFTP operations from `src-tauri/src/ssh/sftp.rs`.

### File Download Operations

#### Download with Retry
File download with retry logic is implemented in `src-tauri/src/ssh/manager.rs:108-123` using the retry patterns from `src-tauri/src/retry.rs`.

### Directory Management

#### Automated Workspace Setup
Directory creation is handled by `src-tauri/src/ssh/sftp.rs:218-248` with recursive directory support. Job workspace setup follows the directory patterns defined in `docs/DB.md`.

## SSH Logging Infrastructure

### Logging Bridge Architecture

**Implementation**: `src-tauri/src/logging.rs`

Bridges Rust backend logs to frontend logs panel for real-time operation visibility.

#### Logging Macros
```rust
// In Rust backend
log_info!(category: "Connection", message: "Starting connection", details: "Host: {}", host);
log_debug!(category: "Job Sync", message: "Job sync completed");
log_error!(category: "SSH", message: "Connection failed", details: "{}", error);
// For user-facing events that should show toast notifications, add `show_toast: true`
log_info!(category: "Connection", message: "Connected", details: "{}@{}", username, host, show_toast: true);

// Emits 'app-log' event → Frontend logs panel
```

#### What Gets Logged
- **Connection operations**: SSH connect/disconnect with status
- **Job operations**: Creation, submission, sync with progress
- **Database operations**: Job queries and updates
- **Command execution**: SLURM commands with results
- **Errors**: Failed operations with error messages

#### Frontend Logs Panel

**Implementation**: `src/lib/components/layout/LogsPanel.svelte`

The Logs panel displays all backend logs in real-time. Frontend has no logging system - all logging happens in the backend.

**Backend Log Subscription:**
```typescript
// Frontend subscribes to 'app-log' events from Rust backend
listen('app-log', (event) => {
    const logData = event.payload;
    // logData contains: level, category, message, details, show_toast, timestamp
    // Display formatted as: [LEVEL] [category] {details or message}
    // If show_toast is true, also display a toast notification
});
```

**Panel Features:**
- Real-time backend log streaming
- Scrollable history with timestamps
- Color-coded log levels (ERROR, WARN, INFO, DEBUG)
- Copy all logs and clear functionality
- Collapsible panel to save screen space

For automation progress tracking, see [docs/AUTOMATIONS.md](AUTOMATIONS.md#progress-tracking-system)

## Security Patterns

### Core Security Principles

#### Essential Requirements
- **Secure credential handling**: Always use SecStr for passwords with automatic memory cleanup
- **No credential persistence**: Passwords exist only in memory during active sessions
- **Safe logging**: Never log credentials, passwords, or sensitive configuration
- **Input sanitization**: Validate and sanitize all user input before use
- **Path safety**: Prevent directory traversal and injection attacks

#### Secure Password Handling

**SecurePassword Implementation**
Secure password handling is implemented in `src-tauri/src/security.rs` using the `secstr` crate for automatic memory clearing and secure access patterns.

**Password Lifecycle**
- Passwords exist only in memory during active sessions
- Use SecStr for password handling with automatic cleanup
- Clear memory on disconnect
- Never log or persist credentials
- Validate SSH connection before SLURM operations

### Input Validation

#### Path Security & Input Validation
**Essential Principles**:
- Never use user input directly in path construction or shell commands
- Always sanitize and validate input before use
- Prevent directory traversal attacks (`../`, null bytes, etc.)
- Use allow-lists for valid characters (alphanumeric, `_`, `-`)
- Validate path length limits and component restrictions

**Path Safety Validation**
Path validation to prevent traversal attacks and ensure safe file operations is implemented in `src-tauri/src/validation.rs`.

#### Command Injection Prevention
**Essential Principles**:
- Always escape shell parameters when executing remote commands
- Sanitize filenames and command arguments
- Use parameter validation before shell execution
- Never use user input directly in command construction

### Memory Management

#### Connection Lifecycle Management
**Essential Principles**:
- Always clean up connections properly
- Clear credentials from memory on disconnect
- Validate SSH connection before SLURM operations
- Handle connection expiration gracefully

### Connection Cleanup

#### Proper Resource Management
Connection cleanup is handled in the Drop implementation for SSHConnection in `src-tauri/src/ssh/connection.rs:158-167`.

## Error Handling

> **For complete error handling contracts and API specifications**, see [`docs/API.md#error-handling-strategy`](API.md#error-handling-strategy).

### SSH Error Classification

#### Error Categories
SSH error classification and mapping is implemented in `src-tauri/src/ssh/errors.rs` with categories for Network, Authentication, Permission, FileSystem, Protocol, Timeout, and Internal errors.

#### Error Mapping
SSH error mapping from ssh2 library errors to application-specific error types is implemented in `src-tauri/src/ssh/errors.rs` with proper categorization and retry logic.

### Retry Strategies

#### Exponential Backoff Implementation
**Essential Principles**:
- Implement exponential backoff for retryable operations
- Distinguish between retryable and non-retryable errors
- Use appropriate timeout limits and maximum attempts
- Add jitter to prevent thundering herd effects

**Implementation Details**
Exponential backoff retry logic with jitter and maximum delay is implemented in `src-tauri/src/retry.rs` and used throughout the SSH operations.

### Error Mapping for User Experience

#### Error Classification Principles
**Essential Principles**:
- Convert technical errors to actionable user messages
- Categorize errors by type (Network, Authentication, Permission, etc.)
- Provide recovery suggestions for each error category
- Maintain error context throughout the system

**Error Categories in SSH Operations**:
- **Network Errors**: Connection timeouts, DNS failures, unreachable hosts
- **Authentication Errors**: Invalid credentials, expired passwords, account lockouts
- **Permission Errors**: Insufficient privileges, file access denied
- **FileSystem Errors**: Disk full, permission denied, invalid paths
- **Protocol Errors**: SSH protocol issues, incompatible versions
- **Timeout Errors**: Operation timeouts, slow network responses

#### User-Friendly Error Messages
SSH error mapping provides clear, actionable feedback:
```rust
// Example: Convert ssh2 errors to user-friendly messages
match ssh_error {
    SshError::Network(_) => "Connection failed. Check your network and try again.",
    SshError::Authentication(_) => "Login failed. Please verify your username and password.",
    SshError::Permission(_) => "Access denied. Contact your system administrator.",
    // ... more mappings
}
```

### Recovery Patterns

#### Connection Recovery
Connection state checking and recovery logic is in `src-tauri/src/ssh/manager.rs:59-69`. Note that passwords are not persisted, requiring manual re-authentication for expired sessions.

#### Automatic Recovery Strategies
- **Retryable Operations**: Network errors, temporary failures, timeouts
- **Non-Retryable Operations**: Authentication failures, permission denials
- **Progressive Backoff**: Increase delay between retry attempts
- **Maximum Attempts**: Prevent infinite retry loops

## Performance & Optimization

### Connection Pooling

#### Reusing Connections
Connection management and lifecycle is handled by the singleton ConnectionManager in `src-tauri/src/ssh/manager.rs`. Multiple operations reuse the same connection instance.

**Connection reuse benefits:**
- SSH connection establishment is expensive (~500ms per connection)
- Reuse connections for multiple operations when possible
- Limit concurrent connections (3 max recommended)
- Batch related operations to minimize connection overhead

**Performance impact:**
- Individual connections: 10 operations × 500ms = 5 seconds
- Reused connection: 10 operations × ~50ms = 500ms + 500ms initial = 1 second

### Performance Bottlenecks

**Common performance issues:**
- **SFTP uploads**: Large files (>50MB) take minutes (limited by SSH protocol, not bandwidth)
- **Network latency**: ~200ms overhead per SSH command minimum
- **SLURM queries**: `sacct` can take 5-10 seconds when scheduler is busy
- **Excessive queries**: Individual status checks scale poorly (N jobs = N SSH commands)

**Optimization strategies:**
- **Batch SLURM queries**: Single SSH command for multiple jobs (comma-separated IDs)
- **Status caching**: 30-60 second TTL prevents redundant queries (see [slurm-commands-reference.md](reference/slurm-commands-reference.md#status-caching-strategy))
- **Background operations**: Never block UI thread for network operations
- **Progress indicators**: Show user feedback for long operations (uploads, sync)
- **Connection reuse**: Establish once, use for multiple operations

### Async Patterns

#### Async Operations with Blocking Libraries
**Essential Principles**:
- Use `spawn_blocking` for CPU-intensive blocking operations
- Avoid blocking the async runtime with synchronous operations
- Handle ssh2 and other blocking libraries properly
- Maintain async interface boundaries for UI responsiveness

**Implementation Details**
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

#### Frontend IPC Layer
Frontend communicates with Rust SSH backend via Tauri IPC commands, following patterns defined in [`docs/API.md`](API.md) with proper error mapping between Rust and TypeScript.

#### Connection Error Detection
The frontend automatically detects connection failures and updates session state accordingly:

**Implementation**: `src/lib/stores/jobs.ts:8-24` and `src/lib/stores/session.ts:136-143`

```typescript
// Helper: Detect if error indicates connection failure
function isConnectionError(errorMessage: string): boolean {
  const msg = errorMessage.toLowerCase();
  return msg.includes('timeout') ||
         msg.includes('not connected') ||
         msg.includes('connection') ||
         msg.includes('broken pipe') ||
         msg.includes('network') ||
         msg.includes('ssh');
}

// Helper: Handle connection failure by updating session state
function handleConnectionFailure(error: string) {
  if (isConnectionError(error)) {
    sessionActions.markExpired(error);
  }
}
```

**Connection State Transitions:**
- All job operations (sync, create, submit, delete) check for connection errors
- When detected, session state transitions to 'Expired'
- User is prompted to reconnect before attempting further operations

#### Offline Mode Support
Jobs store maintains cached data for offline viewing:

**Implementation**: `src/lib/stores/jobs.ts`

```typescript
// Load jobs from database (for offline/startup)
loadFromDatabase: async () => {
  const result = await invoke<ApiResult<JobInfo[]>>('get_all_jobs');
  if (result.success && result.data) {
    update(state => ({
      ...state,
      jobs: result.data || []
    }));
  }
}
```

**Offline Features:**
- Jobs cached in local SQLite database persist after disconnect
- Users can view job details, configurations, and metadata offline
- Sync operations gracefully fail with error messages when disconnected
- Connection state clearly indicates when operations require active connection

## Testing & Development

> **For complete testing strategies, mock infrastructure setup, and development workflows**, see [`docs/CONTRIBUTING.md#testing-strategy`](CONTRIBUTING.md#testing-strategy).

### SSH-Specific Testing Notes

#### Mock Infrastructure
**Implementation**: `src-tauri/src/ssh/test_utils.rs`

Test utilities provide mock helpers to test business logic without requiring actual SSH connections:
- `MockFileSystem`: Simulates file system state for SFTP operations
- `MockFile`: Represents files with size, permissions, content
- Mock helpers for testing SSH command execution and error handling

#### Testing Best Practices
- Mock SSH infrastructure focuses on business logic testing
- Error classification tests verify error mapping and categorization
- Use mock file system to test SFTP operations without network
- Integration tests use real SSH connections (require test server)

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
# SSH & SFTP Integration Guide

**SSH/SFTP connection management, file operations, and security patterns.**

> See project README for project overview.
>
> **Related Docs:**
>
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
> - [API.md](API.md) - IPC interfaces

## Connection Management

### Password-Only Authentication

NAMDRunner uses password-only SSH authentication to comply with cluster requirements that disable SSH key access.

#### Connection Requirements

- **Password authentication only** - SSH keys are not supported by target clusters
- **Interactive prompts** - Handle keyboard-interactive authentication when required
- **Session persistence** - Maintain connection for multiple operations
- **Automatic cleanup** - Clear credentials from memory on disconnect

#### Authentication Flow

See `src-tauri/src/ssh/manager.rs` for connection implementation using SecurePassword.

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

#### Batch File Upload

Batch upload operations are handled in `src-tauri/src/commands/files.rs` with individual file uploads using the chunked SFTP operations.

### Directory Management

#### Automated Workspace Setup

Directory creation handled by `src-tauri/src/ssh/sftp.rs` with recursive directory support. Job workspace setup follows the directory patterns defined in `docs/DB.md`.

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

#### Frontend Logs Panel

**Implementation**: `src/lib/components/layout/LogsPanel.svelte`

The Logs panel displays all backend logs in real-time. Frontend has no logging system - all logging happens in the backend.

**Backend Log Subscription:**

```typescript
// Frontend subscribes to 'app-log' events from Rust backend
listen('app-log', (event) => {
    const logData = event.payload;
    // logData contains: level, category, message, details, show_toast, timestamp
});
```

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

**Path Safety**
Path utilities for safe path construction and validation are implemented in `src-tauri/src/ssh/paths.rs`. Input sanitization is handled by `src-tauri/src/security/input.rs`.

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

## Error Handling

### SSH Error Classification

#### Error Categories

SSH error classification and mapping is implemented in `src-tauri/src/ssh/errors.rs` with categories for Network, Authentication, Permission, FileSystem, Protocol, Timeout, and Internal errors.

### Retry Strategies

#### Exponential Backoff Implementation

**Essential Principles**:

- Implement exponential backoff for retryable operations
- Distinguish between retryable and non-retryable errors
- Use appropriate timeout limits and maximum attempts
- Add jitter to prevent thundering herd effects

### Recovery Patterns

#### Connection Recovery

Connection state checking and recovery logic is in `src-tauri/src/ssh/manager.rs`. Note that passwords are not persisted, requiring manual re-authentication for expired sessions.

## Performance & Optimization

### Connection Pooling

#### Reusing Connections

Connection management and lifecycle is handled by the singleton ConnectionManager in `src-tauri/src/ssh/manager.rs`. Multiple operations reuse the same connection instance.

**Connection reuse benefits:**

- SSH connection establishment is expensive (~500ms per connection)
- Reuse connections for multiple operations when possible
- Limit concurrent connections (3 max recommended)
- Batch related operations to minimize connection overhead

### Performance Bottlenecks

**Optimization strategies:**

- **Batch SLURM queries**: Single SSH command for multiple jobs (comma-separated IDs)
- **Status caching**: 30-60 second TTL prevents redundant queries
- **Background operations**: Never block UI thread for network operations
- **Progress indicators**: Show user feedback for long operations (uploads, sync)

### Async Patterns

#### Async Operations with Blocking Libraries

**Essential Principles**:

- Use `spawn_blocking` for CPU-intensive blocking operations
- Avoid blocking the async runtime with synchronous operations
- Handle ssh2 and other blocking libraries properly
- Maintain async interface boundaries for UI responsiveness

## Implementation Patterns

### Rust Implementation

#### Module Organization

SSH modules are organized in `src-tauri/src/ssh/` with connection, manager, commands, sftp, errors, and test_utils modules as shown in `src-tauri/src/ssh/mod.rs`.

### TypeScript Integration

#### Frontend IPC Layer

Frontend communicates with Rust SSH backend via Tauri IPC commands, following patterns defined in [`docs/API.md`](API.md) with proper error mapping between Rust and TypeScript.

#### Connection Error Detection

The frontend automatically detects connection failures and updates session state accordingly:

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
```

**Connection State Transitions:**

- All job operations (sync, create, submit, delete) check for connection errors
- When detected, session state transitions to 'Expired'
- User is prompted to reconnect before attempting further operations

#### Offline Mode Support

Jobs store maintains cached data for offline viewing:

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

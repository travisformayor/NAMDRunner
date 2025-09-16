# NAMDRunner System Architecture

NAMDRunner is a Tauri v2 + Svelte TypeScript desktop application for managing NAMD molecular dynamics simulations on SLURM clusters. The architecture provides a secure, type-safe interface between a Rust backend and TypeScript frontend, with comprehensive SSH/SFTP integration for cluster operations.

## Core Architecture

**Clean Separation of Concerns**:
- **Frontend**: Svelte components with TypeScript, reactive stores, and comprehensive IPC client
- **Backend**: Rust command handlers with SSH/SFTP services and security validation
- **IPC Layer**: Strongly-typed communication layer between frontend and backend
- **Testing**: Mock implementations enable offline development and fast test execution

## Module Structure

### Frontend (Svelte/TypeScript)

```
src/
├── lib/
│   ├── ports/                   # IPC communication layer
│   │   ├── coreClient.ts        # IPC interface definition
│   │   ├── coreClient-tauri.ts  # Production Tauri implementation
│   │   ├── coreClient-mock.ts   # Mock for offline development
│   │   └── clientFactory.ts     # Smart client selection (dev/prod)
│   ├── types/                   # TypeScript type definitions
│   │   ├── api.ts               # Core API types and interfaces
│   │   ├── connection.ts        # Connection state and session types
│   │   └── errors.ts            # Error handling types
│   ├── services/                # Business logic services
│   │   ├── connectionState.ts   # Observable connection state management
│   │   ├── sessionManager.ts    # Session lifecycle and validation
│   │   ├── directoryManager.ts  # Remote directory operations
│   │   ├── connectionValidator.ts # Multi-stage connection validation
│   │   ├── pathResolver.ts      # Centralized path generation
│   │   └── serviceContainer.ts  # Dependency injection container
│   ├── stores/                  # Reactive state management
│   │   └── session.ts           # Session state with reactive updates
│   ├── test/                    # Testing infrastructure
│   │   ├── fixtures/
│   │   │   └── testDataManager.ts # Test data and scenarios
│   │   └── services/            # Unit tests for business logic
│   │       ├── connectionState.test.ts
│   │       ├── sessionManager.test.ts
│   │       ├── connectionValidator.test.ts
│   │       └── directoryManager.test.ts
│   └── components/              # Svelte UI components
│       ├── ConnectionStatus.svelte # Connection status indicator
│       └── ConnectionDialog.svelte # Authentication dialog
├── routes/
│   └── +page.svelte             # Main application interface
└── app.html
```

### Backend (Rust)

```
src-tauri/
├── src/
│   ├── main.rs                  # Application entry point
│   ├── lib.rs                   # Tauri app configuration and setup
│   ├── commands/                # Tauri IPC command handlers
│   │   ├── mod.rs              # Command module exports
│   │   ├── connection.rs       # Connection management commands
│   │   ├── jobs.rs             # Job lifecycle commands (create/submit/delete)
│   │   └── files.rs            # File management commands
│   ├── ssh/                    # SSH/SFTP service implementation
│   │   ├── mod.rs              # SSH module exports
│   │   ├── connection.rs       # Low-level SSH connection handling
│   │   ├── manager.rs          # Connection lifecycle and directory management
│   │   ├── commands.rs         # SSH command execution and parsing
│   │   ├── sftp.rs             # File transfer operations
│   │   ├── errors.rs           # SSH error mapping and classification
│   │   └── test_utils.rs       # Mock infrastructure for testing
│   ├── slurm/                  # SLURM integration
│   │   ├── mod.rs              # SLURM module exports
│   │   ├── commands.rs         # SLURM command builder and patterns
│   │   └── status.rs           # Job status synchronization
│   ├── database/               # Data persistence layer
│   │   ├── mod.rs              # Database module exports
│   │   └── job_repository.rs   # Repository pattern for job data access
│   ├── types/                  # Rust type definitions
│   │   ├── mod.rs              # Type module exports
│   │   ├── core.rs             # Core domain types (JobInfo, SessionInfo)
│   │   └── commands.rs         # Command parameter and result types
│   ├── retry.rs                # Exponential backoff retry implementation
│   ├── validation.rs           # Input sanitization and path safety
│   ├── validation_traits.rs    # Unified validation trait patterns
│   ├── security.rs             # Secure password handling with SecStr
│   ├── mode_switching.rs       # Mock/real mode switching patterns
│   └── mock_state.rs           # Mock state management for development
├── Cargo.toml                  # Dependencies (ssh2, secstr, rusqlite)
└── tauri.conf.json            # Tauri configuration
```

## IPC Command Interface

### Available Commands

The application provides a complete set of commands for job management and cluster operations:

```typescript
// Connection Management
interface ConnectionCommands {
  connect(host: string, username: string, password: string): Promise<ConnectResult>;
  disconnect(): Promise<DisconnectResult>;
  getConnectionStatus(): Promise<ConnectionStatusResult>;
}

// Job Lifecycle Management
interface JobCommands {
  createJob(params: CreateJobParams): Promise<CreateJobResult>;
  submitJob(jobId: JobId): Promise<SubmitJobResult>;
  getJobStatus(jobId: JobId): Promise<JobStatusResult>;
  getAllJobs(): Promise<GetAllJobsResult>;
  syncJobs(): Promise<SyncJobsResult>;
  deleteJob(jobId: JobId, deleteRemote: boolean): Promise<DeleteJobResult>;
}

// File Operations
interface FileCommands {
  uploadJobFiles(jobId: JobId, files: FileUpload[]): Promise<UploadResult>;
  downloadJobOutput(jobId: JobId, fileName: string): Promise<DownloadResult>;
  listJobFiles(jobId: JobId): Promise<ListFilesResult>;
}
```

## Data Models

### TypeScript Type System
Core types defined in `src/lib/types/api.ts`:

```typescript
// Core state types
type ConnectionState = 'Disconnected' | 'Connecting' | 'Connected' | 'Expired';
type JobStatus = 'CREATED' | 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

// Job management types
interface CreateJobParams {
  jobName: string;
  namdConfig: NAMDConfig;
  slurmConfig: SlurmConfig;
  inputFiles: InputFile[];
}

interface JobInfo {
  jobId: JobId;
  jobName: string;
  status: JobStatus;
  slurmJobId?: SlurmJobId;
  createdAt: Timestamp;
  updatedAt?: Timestamp;
  // ... additional fields
}
```

### Rust Type System
Core types defined in `src-tauri/src/types/`:

```rust
// Connection management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    #[serde(rename = "connectedAt")]
    pub connected_at: String, // ISO 8601 format
}

// Job management with proper serde attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "jobName")]
    pub job_name: String,
    pub status: JobStatus,
    #[serde(rename = "slurmJobId")]
    pub slurm_job_id: Option<String>,
    // ... additional fields with proper camelCase mapping
}
```

## System Architecture Patterns

### Data Flow
1. User action in Svelte component
2. Call through coreClient interface  
3. Tauri command invoked
4. Rust module processes request
5. Result returned through IPC boundary
6. Svelte store updated
7. UI re-renders

### Error Handling
Comprehensive error management system:
- **Result Types**: All operations return `Result<T>` with structured error information
- **Error Classification**: Network, Authentication, Timeout, Permission, Configuration errors
- **Retry Logic**: Exponential backoff with jitter for transient failures
- **User-Friendly Messages**: Error categorization provides actionable user guidance
- **Recovery Strategies**: Automatic retry vs manual intervention based on error type

### Security Architecture
Security-first design throughout the system:
- **No Credential Persistence**: Passwords exist only in memory during active sessions
- **Input Sanitization**: Comprehensive validation prevents injection attacks and path traversal
- **Secure Memory**: SecStr-based password handling with automatic cleanup
- **Type-Safe IPC**: Strongly-typed communication prevents data corruption
- **Minimal Permissions**: Tauri configured with minimal required permissions

### Testing Strategy
Fast, reliable testing without external dependencies:
- **Unit Tests**: Business logic focus using Vitest (TypeScript) and cargo test (Rust)
- **Mock Infrastructure**: Complete offline development environment
- **Security Testing**: Comprehensive validation against malicious inputs
- **No Network Dependencies**: All tests use mocks to avoid infrastructure complexity

### Development Environment
Optimized for rapid development and deployment:
- **Dual Mode Operation**: Mock mode for development, real mode for production
- **Hot Reload**: Fast development iteration with `npm run tauri dev`
- **Type Safety**: Full TypeScript ↔ Rust type checking
- **Portable Builds**: Single executable for Windows deployment

## SSH/SFTP Integration

### Connection Management
**Secure SSH Operations**: Password-based authentication with comprehensive lifecycle management:
- **Connection Establishment**: ssh2-based connections with proper async handling
- **Session Lifecycle**: Automatic cleanup and secure memory management
- **State Transitions**: Observable state machine (`Disconnected` → `Connecting` → `Connected` → `Expired`)
- **Session Validation**: Multi-stage validation (connectivity, SSH access, SFTP operations, SLURM integration)
- **Retry Logic**: Exponential backoff with jitter for network interruption recovery

### Directory Management
**Automated Workspace Setup**: Complete job directory lifecycle management:
- **Project Directories**: `/projects/$USER/namdrunner_jobs/$JOB_ID/` with `inputs/`, `outputs/`, `scripts/` subdirectories
- **Scratch Directories**: `/scratch/alpine/$USER/namdrunner_jobs/$JOB_ID/` with execution subdirectories
- **SFTP Operations**: Native SFTP for reliable directory creation and cleanup
- **Safety Validation**: Path sanitization prevents directory traversal and unauthorized access
- **Cleanup Management**: Safe deletion with validation to prevent accidental data loss

### SLURM Integration
**Production-Ready Cluster Operations**: Complete integration with SLURM workload manager:
- **Command Execution**: Remote SSH command execution with timeout and retry support
- **Job Submission**: SBATCH script generation and execution with job ID parsing
- **Status Monitoring**: Integration points for squeue/sacct status synchronization
- **Error Handling**: Comprehensive SLURM error classification and recovery suggestions

## Clean Architecture Implementation

For comprehensive architectural patterns and code quality standards, see [`docs/developer-guidelines.md`](developer-guidelines.md).

### Dependency Injection
**Service Container Pattern**: Clean separation of concerns through dependency injection:
- `ServiceContainer` manages service instantiation and dependencies
- Constructor injection for all service dependencies
- Singleton pattern for stateful services, factory pattern for stateless utilities
- Mock service containers for testing environments
- Clear service boundaries with explicit interfaces

### Centralized Path Management
**PathResolver Service**: All remote path operations centralized with security validation:
- Template-based path generation with variable substitution
- Path validation and sanitization preventing directory traversal
- Consistent directory structure across all operations
- Job ID sanitization with Unicode character rejection
- Path security checks for user isolation

**Directory Structure**:
```
/projects/$USER/namdrunner_jobs/
├── {jobId}/
│   ├── inputs/     # Input files
│   ├── outputs/    # Output files
│   ├── scripts/    # SLURM scripts
│   └── job.json    # Job metadata
```

### Security Implementation
**Defense-in-Depth Validation**: Multiple layers of security protection:
- **Input Sanitization**: `sanitize_job_id()` and `sanitize_username()` with strict character validation
- **Path Safety**: `validate_path_safety()` prevents traversal attacks and unauthorized access
- **Command Safety**: `build_command_safely()` with proper parameter escaping
- **Memory Safety**: SecStr-based password handling with automatic cleanup
- **Shell Protection**: `escape_parameter()` prevents command injection attacks

### Repository Pattern Implementation
**Separation of Database Concerns**: Clean data access layer with repository pattern:
- **JobRepository trait**: Standardized interface for job data operations
- **DefaultJobRepository**: Concrete implementation using existing database infrastructure
- **JobService**: Business logic layer providing domain-specific operations
- **Mock repositories**: In-memory implementations for testing without database dependencies
- **Domain separation**: Core types focus on business logic, repositories handle persistence

### Validation Pattern Implementation
**Unified Validation System**: Trait-based validation with consistent error handling:
- **Validate trait**: Generic validation interface for command parameters
- **ValidateId trait**: Specialized validation for ID parameters with security checks
- **ValidationError**: Structured error types with field-specific context
- **Validator utilities**: Reusable validation functions for common patterns
- **Type safety**: Validated types ensure clean input throughout the system

### SLURM Command Builder
**Consistent Command Construction**: Centralized SLURM command generation:
- **SlurmCommand builder**: Fluent interface for building complex commands
- **Standard prefixes**: All commands include proper module loading sequences
- **Helper functions**: Pre-built patterns for common operations (status, submit, cancel)
- **Reference compliance**: Commands match documented SLURM patterns exactly
- **Maintainability**: Single place to update command patterns and module versions

### Async Pattern Optimization
**Efficient Memory Usage**: Optimized async patterns without unnecessary allocations:
- **Direct async functions**: Eliminated Box::pin allocations where possible
- **Helper methods**: Private async functions for cleaner retry integration
- **Performance**: Reduced heap allocations for frequently-called operations
- **Maintainability**: Cleaner code structure without boxing overhead

### Mock Infrastructure
**Offline Development Environment**: Complete mock system for fast development:
- **UI Development**: Fast iteration without network dependencies
- **Unit Testing**: Comprehensive coverage with predictable responses
- **Error Scenarios**: Configurable error injection for robustness testing
- **Development Speed**: No external dependencies for core functionality

## Current Implementation Status

The NAMDRunner application currently provides **complete job lifecycle management** with secure SSH/SFTP cluster connectivity and a clean, refactored architecture:

### Job Management Capabilities
- **Job Creation**: Complete directory structure setup with security validation
- **Job Submission**: SLURM script generation and SBATCH execution with job ID parsing
- **Job Status Sync**: Real-time status monitoring with batch operations
- **Job Deletion**: Safe cleanup with path validation and directory removal
- **File Operations**: SFTP-based file upload/download with retry logic
- **Directory Management**: Automated project and scratch directory lifecycle

### Refactored Architecture
- **Repository Pattern**: Clean separation between domain logic and data persistence
- **Unified Validation**: Trait-based validation system with consistent error handling
- **SLURM Command Builder**: Centralized, maintainable command construction
- **Optimized Async**: Efficient async patterns without unnecessary Box::pin allocations
- **Mode Switching**: Clean mock/real operation patterns for development and testing

### SSH/SFTP Integration
- **Real SSH Operations**: Production-ready ssh2-based connectivity with optimized async patterns
- **Secure Authentication**: Password-only authentication with memory-safe credential handling
- **Network Resilience**: Exponential backoff retry logic integrated directly into ConnectionManager
- **Error Recovery**: Comprehensive error classification with recovery strategies
- **Dual Mode Operation**: Mock mode for development, real mode for production

### Security Implementation
- **Input Validation**: Comprehensive sanitization preventing injection attacks
- **Path Safety**: Directory traversal protection and Unicode character rejection
- **Memory Safety**: SecStr-based password handling with automatic cleanup
- **Command Safety**: Shell parameter escaping and safe command construction
- **No Persistence**: Credentials exist only in memory during active sessions

### Testing Infrastructure
- **Comprehensive Test Coverage**: Complete unit test coverage using NAMDRunner testing philosophy
- **Mock Infrastructure**: Fast offline development environment with repository mocks
- **Business Logic Focus**: Tests validate our code, not external libraries
- **Security Testing**: Comprehensive validation against malicious inputs
- **No External Dependencies**: All tests use mocks for reliability

## Next Steps

The current implementation provides a solid foundation for continued development. See [`tasks/roadmap.md`](../tasks/roadmap.md) for planned features and development timeline, including the next priority: **job status synchronization and data persistence**.
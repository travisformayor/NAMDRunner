# System Architecture (Living Document)

*This document describes the actual implementation as it exists. Update it whenever the real code structure changes.*

**📋 Planning & Roadmap**: See `tasks/roadmap.md` for what will be built and implementation timeline. Always check roadmap when planning architecture changes to understand how they fit with planned features.

**📚 Complete Specifications**: See `docs/api-spec.md` for IPC interfaces and SLURM patterns, `docs/data-spec.md` for schemas and validation rules, and `docs/testing-spec.md` for testing strategies and error handling.

## Current Implementation Overview

NAMDRunner is a Tauri v2 + Svelte TypeScript desktop application for managing NAMD molecular dynamics simulations on SLURM clusters. The architecture follows a clean separation between:

- **Frontend**: Svelte components with TypeScript, reactive stores, and comprehensive IPC client
- **Backend**: Rust command handlers with type-safe IPC boundary and mock implementations  
- **IPC Layer**: Strongly-typed communication layer between frontend and backend
- **Testing**: Mock implementations enable full offline development and testing

## Current Module Structure (As Implemented)

### Frontend (Svelte/TypeScript)

```
src/
├── lib/
│   ├── ports/
│   │   ├── coreClient.ts        # IPC interface definition ✅
│   │   ├── coreClient-tauri.ts  # Production Tauri implementation ✅ 
│   │   ├── coreClient-mock.ts   # Mock for testing ✅
│   │   └── clientFactory.ts     # Smart client selection ✅
│   ├── types/
│   │   ├── api.ts               # Complete TypeScript types ✅
│   │   ├── connection.ts        # Connection interfaces & types ✅
│   │   └── errors.ts            # Error handling system ✅
│   ├── services/
│   │   ├── connectionState.ts   # Observable state management ✅
│   │   ├── sessionManager.ts    # Session lifecycle management ✅
│   │   ├── directoryManager.ts  # Remote directory operations ✅
│   │   ├── connectionValidator.ts # Connection validation framework ✅
│   │   ├── pathResolver.ts      # Centralized path generation & validation ✅
│   │   └── serviceContainer.ts  # Dependency injection container ✅
│   ├── stores/
│   │   └── session.ts           # Reactive session state ✅
│   ├── test/
│   │   ├── fixtures/
│   │   │   └── testDataManager.ts  # Enhanced test scenarios ✅
│   │   └── services/            # Comprehensive unit tests ✅
│   │       ├── connectionState.test.ts
│   │       ├── sessionManager.test.ts
│   │       ├── connectionValidator.test.ts
│   │       └── directoryManager.test.ts
│   └── components/
│       ├── ConnectionStatus.svelte  # Connection UI ✅
│       └── ConnectionDialog.svelte  # Login dialog ✅
├── routes/
│   └── +page.svelte             # Main application UI ✅
└── app.html
```

### Backend (Rust)

```
src-tauri/
├── src/
│   ├── main.rs                  # Entry point ✅
│   ├── lib.rs                   # App configuration ✅
│   ├── commands/                # Tauri command handlers
│   │   ├── mod.rs              # Module exports ✅
│   │   ├── connection.rs      # SSH mock implementation ✅
│   │   ├── jobs.rs            # Job management mock ✅
│   │   └── files.rs           # File operations mock ✅
│   └── types/                  # Type definitions
│       ├── mod.rs             # Module exports ✅
│       ├── core.rs           # Core domain types ✅
│       └── commands.rs       # Command types ✅
├── Cargo.toml                 # Dependencies configured ✅
└── tauri.conf.json           # Tauri configuration ✅
```

## IPC Command Interface

### Implemented Commands ✅

All Phase 1 commands are implemented with mock backends:

```typescript
// Connection Management (Fully Implemented)
interface ConnectionCommands {
  connect(host: string, username: string, password: string): Promise<ConnectResult>;
  disconnect(): Promise<DisconnectResult>;
  getConnectionStatus(): Promise<ConnectionStatusResult>;
}

// Job Management (Fully Implemented)
interface JobCommands {
  createJob(params: CreateJobParams): Promise<CreateJobResult>;
  submitJob(jobId: JobId): Promise<SubmitJobResult>;
  getJobStatus(jobId: JobId): Promise<JobStatusResult>;
  getAllJobs(): Promise<GetAllJobsResult>;
  syncJobs(): Promise<SyncJobsResult>;
  deleteJob(jobId: JobId, deleteRemote: boolean): Promise<DeleteJobResult>;
}

// File Operations (Fully Implemented)
interface FileCommands {
  uploadJobFiles(jobId: JobId, files: FileUpload[]): Promise<UploadResult>;
  downloadJobOutput(jobId: JobId, fileName: string): Promise<DownloadResult>;
  listJobFiles(jobId: JobId): Promise<ListFilesResult>;
}
```

## Data Models

### TypeScript Interfaces (Implemented)
All types defined in `src/lib/types/api.ts`:

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

### Rust Structs (Implemented)
All types defined in `src-tauri/src/types/`:

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

## Component Relationships

*Add diagrams and descriptions as components are built*

### Data Flow
1. User action in Svelte component
2. Call through coreClient interface  
3. Tauri command invoked
4. Rust module processes request
5. Result returned through IPC boundary
6. Svelte store updated
7. UI re-renders

### Error Handling
Current error handling patterns:
- IPC commands return `Result` types with success/error information
- Frontend displays user-friendly error messages via reactive stores
- Mock implementations simulate realistic error scenarios
- Connection errors are handled gracefully with retry capabilities

### Testing Infrastructure
Currently implemented:
- **Unit Tests**: Vitest for TypeScript, built-in test runner for Rust
- **Mock System**: Comprehensive mock implementations for offline development
- **E2E Framework**: Playwright configured for Tauri WebDriver testing
- **Test Utilities**: Session store testing with mock client integration

### Security Considerations
Current security implementations:
- **No Credential Persistence**: Passwords stored only in memory during session
- **Type-Safe IPC**: All communications between frontend/backend are strongly typed
- **Mock Security**: Development mocks simulate auth without real credentials
- **Tauri Permissions**: Minimal permission set configured

### Build & Deployment
Current build configuration:
- **Frontend**: Vite build system with SvelteKit adapter-static
- **Backend**: Tauri v2 build system with Rust cargo
- **Development**: Hot reload supported via `npm run tauri dev`
- **Dependencies**: All required crates and npm packages configured

## Connection Architecture (Phase 1 Milestone 1.3) ✅

### Connection State Management
**Observable State Machine Pattern**: Connection states are managed through a type-safe state machine with validated transitions:
- `Disconnected` → `Connecting` → `Connected` (normal flow)
- `Connected` → `Expired` (session timeout)
- `Expired` → `Connecting` (reconnection)
- State transitions are observable for reactive UI updates
- Invalid transitions are blocked with detailed error messages

**Key Components**:
- `ConnectionStateMachine`: Core state management with history tracking
- Retry logic with exponential backoff
- Time-based session expiration detection
- Diagnostic utilities for debugging connection issues

### Session Management
**Secure Session Lifecycle**: Session handling follows security-first principles:
- **No credential persistence** - passwords stored only in memory during active session
- Configurable session validity periods (default: 4 hours)
- Automatic session refresh scheduling with callback support
- Session age tracking and expiration warnings
- Secure memory cleanup on disconnect

**Session Validation**:
- Real-time validation of session freshness
- Detection of expired sessions with appropriate error handling
- Session diagnostics for monitoring and debugging

### Error Handling System
**Categorized Error Management**: Comprehensive error classification with user-friendly messaging:
- **Error Categories**: Network, Authentication, Timeout, Permission, Configuration, Validation, FileOperation
- **Recovery Strategies**: Automatic retry with backoff, manual intervention required, session refresh
- **User Guidance**: Each error includes actionable suggestions for resolution
- **Error Context**: Rich debugging information without exposing sensitive data

**Error Recovery Patterns**:
- Network errors: Automatic retry with exponential backoff
- Authentication errors: Require user intervention
- Session expiration: Automatic refresh where possible
- Timeout errors: Configurable retry limits

## Clean Architecture Patterns (Phase 1 Refactoring) ✅

For comprehensive architectural patterns and code quality standards, see [`docs/developer-guidelines.md`](developer-guidelines.md).

### Dependency Injection System
**Service Container Pattern**: All services use dependency injection for clean separation of concerns:
- `ServiceContainer` manages service instantiation and dependencies
- Constructor injection for all service dependencies
- Singleton pattern for stateful services, factory pattern for stateless utilities
- Mock service containers for testing environments
- Clear service boundaries with explicit interfaces

**Benefits**:
- Testable components through dependency mocking
- Clear service dependencies and relationships
- Easy service composition and configuration
- Consistent service lifecycle management

### Centralized Path Management
**PathResolver Service**: All remote path operations centralized in a single, validated service:
- Template-based path generation with variable substitution
- Path validation and sanitization for security
- Consistent directory structure across all operations
- Job ID sanitization and validation
- Path security checks for user isolation

**Path Templates**:
- User directories: `/projects/$USER/namdrunner_jobs/`
- Job structure: `{jobId}/{logs,inputs,outputs,scratch}/`
- Configuration files: `job.json`, `job.slurm`
- Log files: `job.out`, `job.err`, `slurm.log`

### Error Handling Standardization
**Result<T> Pattern**: Consistent error handling across all service layers:
- `Result<T>` return type for all operations that can fail
- Error chaining utilities for complex operations
- Error normalization for consistent client handling
- Retry logic with configurable backoff strategies
- Error context preservation without credential exposure

**Error Utilities**:
- `toConnectionError()`: Convert errors to structured format
- `wrapWithResult()`: Wrap functions with Result pattern
- `chainResults()`: Chain multiple Result operations
- `retryWithResult()`: Retry operations with exponential backoff

### Directory Management
**Consistent Remote Organization**: Standardized directory structure on SLURM clusters:
```
/projects/$USER/namdrunner_jobs/
├── {jobId}/
│   ├── inputs/     # Input files
│   ├── outputs/    # Output files  
│   ├── logs/       # Log files
│   ├── job.json    # Job metadata
│   └── job.slurm   # SLURM script
```

**Directory Operations**:
- Automated workspace setup and validation
- Job directory creation and cleanup
- Disk space monitoring and reporting
- Permission validation and troubleshooting
- Path utilities with sanitization and validation

### Connection Validation Framework
**Multi-Stage Validation**: Comprehensive connection testing before job operations:
1. **Basic Connectivity**: Network reachability and latency testing
2. **SSH Access**: Command execution, shell detection, home directory permissions
3. **SFTP Operations**: File listing, upload/download, directory creation
4. **SLURM Integration**: Module system, SLURM commands, partition access

**Validation Reporting**:
- Pass/fail status for each validation stage
- Recommendations for failed validations
- System information gathering (modules, partitions, user limits)

### Mock Implementation (Preserved for Development)
**Fast Development Environment**: Mock client provides reliable development workflow:
- **UI Development**: Fast iteration without network dependencies
- **Unit Testing**: Comprehensive test coverage with predictable responses
- **Error Scenarios**: Configurable error injection for robustness testing
- **Offline Development**: Complete functionality without cluster access

**Development Benefits**:
- No network delays or connection complexity during UI development
- Consistent test data and predictable scenarios
- Fast test suite execution (all frontend tests use mocks)
- Agent debugging toolkit works reliably with mock backend

## Current Status Summary

### Phase 1: Foundation - COMPLETED ✅

All Phase 1 milestones successfully completed with comprehensive implementation:

### Completed in Phase 1:
- ✅ Full TypeScript/Rust type system with proper serialization
- ✅ Complete IPC boundary with all Phase 1 commands
- ✅ **Connection Architecture Foundation** (Milestone 1.3)
  - ✅ Observable state machine with validated transitions
  - ✅ Secure session management with expiration handling
  - ✅ Comprehensive error handling with recovery strategies
  - ✅ Remote directory management patterns
  - ✅ Multi-stage connection validation framework
  - ✅ Enhanced mock implementation with realistic scenarios
  - ✅ >80% test coverage with comprehensive unit tests
- ✅ **Clean Architecture Refactoring** (Milestone 1.4)
  - ✅ Dependency injection system with service container
  - ✅ Centralized path management with PathResolver
  - ✅ Standardized Result<T> error handling patterns
  - ✅ Eliminated thin wrappers and redundant fallback code
  - ✅ Single responsibility principle across all services
  - ✅ Type-safe service boundaries with explicit interfaces
- ✅ Mock implementations for offline development
- ✅ Connection management UI with reactive state
- ✅ Smart client factory for dev/prod switching
- ✅ Enhanced test infrastructure (unit + E2E + scenarios)
- ✅ All command handlers registered and working

### Architecture Notes:
The current implementation provides a complete foundation with comprehensive mock implementations and robust connection architecture. **Phase 1 is complete and ready for Phase 2 SSH/SFTP implementation.**

Key foundations established:
- **Phase 2 SSH/SFTP Implementation**: Clean interfaces and patterns established
- **Error Recovery**: Comprehensive error handling with automatic retry strategies
- **Session Management**: Secure, observable session lifecycle management
- **Testing**: Pragmatic testing approach balancing coverage with simplicity
- **Validation**: Multi-stage validation framework for connection reliability

### Phase 2 SSH/SFTP Strategy:
**Dual Implementation Approach**: Add real SSH operations alongside existing mocks rather than replacing them.

#### Architecture Pattern:
```
Development: Frontend → coreClient-mock.ts → Simulated responses (fast)
Production:  Frontend → coreClient-tauri.ts → Rust SSH Service → ssh2 crate → Cluster
Selection:   clientFactory.ts chooses implementation based on environment
```

#### Testing Strategy:
- **Frontend**: Continue using mock client for fast UI development and testing
- **Rust SSH Service**: Simple unit tests with mocked ssh2 responses (business logic only)
- **Integration**: Manual validation against real clusters, no complex test infrastructure
- **Focus**: Test our logic, not ssh2 crate functionality

#### Benefits:
- Preserves fast development workflow
- Avoids complex SSH test server setup
- Good enough test coverage without over-engineering
- Clear separation between mock and real implementations

**Next Implementation**: See `tasks/active/phase2-milestone2.1-ssh-sftp-implementation.md` for detailed SSH/SFTP implementation plan.
# NAMDRunner System Architecture

**How the system currently works and is structured** - This document explains the NAMDRunner system design, component architecture, business requirements, and technology choices.

> **For development practices and coding standards**, see [`CONTRIBUTING.md`](CONTRIBUTING.md)
> **For SSH/SFTP connection patterns and security**, see [`SSH.md`](SSH.md)
> **For UI/UX design patterns**, see [`DESIGN.md`](DESIGN.md)
> **For database schemas and data management**, see [`DB.md`](DB.md)
> **For adding commands or API changes**, see [`API.md`](API.md)
> **For SLURM/NAMD command patterns**, see [`reference/`](reference/) directory

## Table of Contents
- [Project Overview](#project-overview)
  - [Purpose](#purpose)
  - [Core Design Principles](#core-design-principles)
  - [Success Criteria](#success-criteria)
    - [User Experience Goals](#user-experience-goals)
    - [Technical Goals](#technical-goals)
- [Architecture Principles & Constraints](#architecture-principles--constraints)
  - [Security Requirements](#security-requirements)
    - [Security Anti-Patterns to Avoid](#security-anti-patterns-to-avoid)
  - [Architecture Constraints](#architecture-constraints)
  - [Fundamental System Requirements](#fundamental-system-requirements)
  - [Code Quality Standards](#code-quality-standards)
- [Technology Stack](#technology-stack)
  - [Shell/Runtime](#shellruntime)
  - [Frontend](#frontend)
  - [Backend (App Core)](#backend-app-core)
  - [SLURM Integration](#slurm-integration)
  - [Why This Stack](#why-this-stack)
- [System Design](#system-design)
  - [End-to-End Workflow](#end-to-end-workflow)
    - [Primary Workflow (New Jobs)](#primary-workflow-new-jobs)
    - [Job Restart Workflow (Post-MVP)](#job-restart-workflow-post-mvp)
  - [Data Placement Strategy](#data-placement-strategy)
    - [Storage Architecture](#storage-architecture)
    - [Directory Structure](#directory-structure)
  - [Architectural Overview](#architectural-overview)
  - [Module Structure](#module-structure)
    - [Frontend (Svelte/TypeScript)](#frontend-sveltetypescript)
    - [Backend (Rust)](#backend-rust)
- [Data Models & Interfaces](#data-models--interfaces)
  - [Data Models](#data-models)
    - [TypeScript Type System](#typescript-type-system)
    - [Rust Type System](#rust-type-system)
  - [IPC Command Interface](#ipc-command-interface)
    - [Available Commands](#available-commands)
- [Implementation Architecture](#implementation-architecture)
  - [Rust Development Patterns](#rust-development-patterns)
  - [Security Implementation](#security-implementation)
  - [SSH/SFTP Integration](#sshsftp-integration)
  - [Clean Architecture Patterns](#clean-architecture-patterns)
    - [Dependency Injection](#dependency-injection)
    - [Repository Pattern Implementation](#repository-pattern-implementation)
    - [Validation Pattern Implementation](#validation-pattern-implementation)
    - [SLURM Command Builder](#slurm-command-builder)
    - [Async Pattern Optimization](#async-pattern-optimization)
  - [System Architecture Patterns](#system-architecture-patterns)
    - [Data Flow](#data-flow)
    - [Error Handling](#error-handling)
- [Development Environment](#development-environment)
- [Build & Deployment](#build--deployment)
  - [Target Platforms & Build Matrix](#target-platforms--build-matrix)
    - [Primary Targets](#primary-targets)
    - [CI/CD Strategy](#cicd-strategy)
  - [Packaging & Delivery](#packaging--delivery)
    - [Distribution Artifacts](#distribution-artifacts)
    - [Build Commands](#build-commands)
- [Status & Planning](#status--planning)
  - [Open Items (Affect Implementation Details)](#open-items-affect-implementation-details)
    - [Pending Decisions](#pending-decisions)
  - [Next Steps](#next-steps)

## Project Overview

NAMDRunner is a **local desktop application** that runs on a researcher's PC to prepare, submit, and track **NAMD** simulations on a remote **SLURM** cluster. The application provides a secure, type-safe interface between a Rust backend and TypeScript frontend, with comprehensive SSH/SFTP integration for cluster operations.

### Purpose
A desktop application for scientists to manage NAMD molecular dynamics simulations without command-line complexity. The application is designed for **fellow researchers** (not paying customers) and prioritizes stability and low-maintenance operation over enterprise features.

### Core Design Principles
- **Local-only operation** - No hosted services or local HTTP servers (avoids CORS entirely)
- **SSH password-only authentication** - No SSH keys (cluster requirement)
- **No credential persistence** - Credentials exist only in memory during active sessions
- **Minimal cluster footprint** - No cluster-resident applications or databases
- **Type-safe boundaries** - Strict TypeScript ↔ Rust contracts

### Success Criteria

#### User Experience Goals
* Scientists can submit NAMD jobs without command line complexity
* Jobs don't mysteriously fail due to our tool
* Tool works reliably for months without maintenance
* Scientists can reopen the tool and see past submitted jobs
* Scientists can restart failed/incomplete jobs with different resources
* New developers can understand and modify code

#### Technical Goals
* Portable Windows executable that "just works"
* No credential persistence (security requirement)
* Offline mode with local cache
* Type-safe boundaries between frontend and backend
* Clean architecture without unnecessary abstractions

## Architecture Principles & Constraints

### Security Requirements
- **No Credential Persistence**: Passwords exist only in memory during active sessions
- **Input Sanitization**: Comprehensive validation prevents injection attacks and path traversal
- **Secure Memory**: SecStr-based password handling with automatic cleanup
- **Type-Safe IPC**: Strongly-typed communication prevents data corruption
- **Minimal Permissions**: Tauri configured with minimal required permissions

#### Security Anti-Patterns to Avoid
* Storing credentials in any form
* Logging secrets or passwords
* Printing raw command lines with secrets
* Over-broad Tauri permissions

### Architecture Constraints
* Adding hosted services or local HTTP servers (re-introduces CORS)
* Switching to SSH keys (cluster disallows)
* Installing servers or databases on the cluster
* Desktop E2E testing on macOS (no WKWebView WebDriver support)

### Fundamental System Requirements
- **Local-only desktop app** - No hosted backend or local HTTP servers
- **SSH auth = username/password only** - No SSH keys support
- **Never persist credentials** - Re-authentication required when sessions expire
- **No cluster-resident application** - Only files and SLURM CLI interactions
- **Local cache DB** - SQLite only
- **Strict typing and code quality** - Required for all code

## Technology Stack

### Shell/Runtime
* **Tauri v2** (desktop shell using system WebView). The frontend is built to **static assets** (HTML/CSS/JS) and **embedded** into the binary; no Node runtime ships, and there's no local server. This avoids CORS, keeps the footprint small, and minimizes maintenance.

### Frontend
* **Svelte + TypeScript**
  * Compile-time reactivity → small output and fewer "mystery re-renders."
  * Single-file components and explicit `$:` derivations → easier for juniors to read and reason about.
  * Solid unit/component testing story (Vitest + Svelte Testing Library).

### Backend (App Core)
* **Rust** with **Tauri commands** as the IPC boundary from UI.
* **SSH/SFTP** via Rust `ssh2` (libssh2). Password and keyboard-interactive auth match cluster policies.
* **SQLite** via `rusqlite` (or the Tauri SQL plugin). See [`DB.md`](DB.md) for complete database schemas and data management patterns.
* **Templating** (NAMD `.conf` + Slurm scripts) via `tera` or `handlebars` - **Phase 5+**: Templates stored as configurable data in database.
  > **For NAMD configuration patterns**, see [`reference/namd-commands-reference.md`](reference/namd-commands-reference.md)
* **Settings Management** - Dynamic cluster configuration and template management via Settings page.

### SLURM Integration
NAMDRunner integrates with SLURM workload managers for job submission and monitoring on HPC clusters.

> **For SLURM command patterns and implementation details**, see [`reference/slurm-commands-reference.md`](reference/slurm-commands-reference.md)

### Why This Stack
* **Security/stability**: Rust core, minimal attack surface, no secrets on disk.
* **Maintainability**: typed boundaries (TS ↔ Rust), clear module seams, small binary.
* **UI velocity**: Svelte's component model is simple, predictable, and testable.

## System Design

### End-to-End Workflow

#### Primary Workflow (New Jobs)
1. **Connect via SSH** - Password authentication, session lives in memory only
2. **Wizard** - Build NAMD `.conf` from templates; user attaches input files
3. **Stage & upload** - Upload via SFTP to `/projects/$USER/namdrunner_jobs/...`; write JSON metadata files
4. **Submit** - Copy staged inputs to scratch directory; submit job to SLURM; capture JobID
5. **Track** - Monitor job status and update local cache + remote JSON
6. **Results** - Browse remote folders, download outputs as needed

#### Job Restart Workflow (Post-MVP)
7. **Restart Option** - For completed/failed jobs with checkpoint files, provide "Restart Job" option
8. **Resource Selection** - Allow researcher to choose different resource allocation for restart
9. **Checkpoint Detection** - Automatically detect and validate NAMD checkpoint files (`.restart.coor`, `.restart.vel`, `.restart.xsc`)
10. **Restart Submission** - Create new job with restart template, copy checkpoint files, submit with remaining steps
11. **Lineage Tracking** - Maintain connection between original job and restart jobs for progress tracking

### Data Placement Strategy

#### Storage Architecture
* **Local**: SQLite cache with job metadata, timestamps, status history. See [`DB.md`](DB.md) for complete schema definitions.
* **Remote** (cluster filesystem): JSON metadata files under project/job folders; job scratch directories contain runtime outputs
* **Single-writer rule**: The application writes JSON metadata; jobs write only inside their scratch/results directories

#### Directory Structure
```
/projects/$USER/namdrunner_jobs/     # Persistent storage
└── {job_id}/
    ├── job_info.json               # Job metadata
    ├── input_files/                # User input files
    ├── config.namd                 # Generated NAMD config
    └── job.sbatch                  # Generated SLURM script

/scratch/alpine/$USER/namdrunner_jobs/  # Execution workspace
└── {job_id}/                       # Working directory during execution
    ├── [copied input files]
    ├── output.dcd                  # Trajectory files
    └── restart.*                   # Restart files
```

### Architectural Overview

**Clean Separation of Concerns**:
- **Frontend**: Svelte components with TypeScript, reactive stores, and comprehensive IPC client
  > **For UI/UX design patterns**, see [`DESIGN.md`](DESIGN.md)
- **Backend**: Rust command handlers with SSH/SFTP services and security validation
- **IPC Layer**: Strongly-typed communication layer between frontend and backend
- **Development**: Mock implementations enable offline development

### Module Structure

#### Frontend (Svelte/TypeScript)

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
│   │   ├── errors.ts            # Error handling types
│   │   └── errorUtils.ts        # Error utility functions
│   ├── services/                # Business logic services
│   │   ├── connectionState.ts   # Observable connection state management
│   │   ├── sessionManager.ts    # Session lifecycle and validation
│   │   ├── directoryManager.ts  # Remote directory operations
│   │   ├── connectionValidator.ts # Multi-stage connection validation
│   │   ├── pathResolver.ts      # Centralized path generation
│   │   ├── serviceContainer.ts  # Dependency injection container
│   │   ├── sftp.ts              # SFTP service operations
│   │   ├── ssh.ts               # SSH service operations
│   │   └── index.ts             # Service exports
│   ├── stores/                  # Reactive state management
│   │   ├── session.ts           # Session state with reactive updates
│   │   └── session.test.ts      # Store unit tests
│   ├── test/                    # Testing infrastructure
│   │   ├── fixtures/            # Test data and scenarios
│   │   │   ├── testDataManager.ts # Test data management
│   │   │   ├── sessionFixtures.ts # Session test data
│   │   │   ├── jobFixtures.ts     # Job test data
│   │   │   ├── fileFixtures.ts    # File test data
│   │   │   └── slurmFixtures.ts   # SLURM test data
│   │   ├── services/            # Unit tests for business logic
│   │   │   ├── connectionState.test.ts
│   │   │   ├── sessionManager.test.ts
│   │   │   ├── connectionValidator.test.ts
│   │   │   └── directoryManager.test.ts
│   │   ├── utils/               # Test utilities
│   │   │   └── connectionMocks.ts # Mock connection utilities
│   │   └── setup.ts             # Test setup configuration
│   └── components/              # Svelte UI components
│       ├── ConnectionStatus.svelte # Connection status indicator
│       └── ConnectionDialog.svelte # Authentication dialog
├── routes/
│   ├── +layout.ts              # Layout configuration
│   └── +page.svelte            # Main application interface
└── app.html
```

#### Backend (Rust)

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
│   │   └── test_utils.rs       # Development utilities
│   ├── slurm/                  # SLURM integration
│   │   ├── mod.rs              # SLURM module exports
│   │   ├── commands.rs         # SLURM command builder and patterns
│   │   ├── script_generator.rs # SLURM script generation
│   │   └── status.rs           # Job status synchronization
│   ├── database/               # Data persistence layer
│   │   ├── mod.rs              # Database module exports
│   │   ├── helpers.rs          # Database helper functions
│   │   └── mod 2               # Additional database modules
│   ├── types/                  # Rust type definitions
│   │   ├── mod.rs              # Type module exports
│   │   ├── core.rs             # Core domain types (JobInfo, SessionInfo)
│   │   └── commands.rs         # Command parameter and result types
│   ├── retry.rs                # Exponential backoff retry implementation
│   ├── validation.rs           # Input sanitization and path safety
│   ├── security.rs             # Secure password handling with SecStr
│   ├── security_tests.rs       # Security validation tests
│   ├── mode_switching.rs       # Mock/real mode switching patterns
│   ├── mock_state.rs           # Mock state management for development
│   └── integration_tests.rs    # Integration test suite
├── Cargo.toml                  # Dependencies (ssh2, secstr, rusqlite)
└── tauri.conf.json            # Tauri configuration
```

## Data Models & Interfaces

### Data Models

#### TypeScript Type System
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

#### Rust Type System
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
    #[serde(rename = "createdAt")]
    pub created_at: String, // ISO 8601 format
    // Phase 6+ addition for job restart functionality
    #[serde(rename = "restartInfo")]
    pub restart_info: Option<RestartInfo>,
}

// Phase 6+ addition: Job restart data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartInfo {
    #[serde(rename = "parentJobId")]
    pub parent_job_id: String,
    #[serde(rename = "checkpointFiles")]
    pub checkpoint_files: Vec<String>,
    #[serde(rename = "completedSteps")]
    pub completed_steps: u64,
    #[serde(rename = "remainingSteps")]
    pub remaining_steps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "PENDING")]
    Pending,
    // ... other statuses
}
```

### IPC Command Interface

The application provides a complete set of commands for job management and cluster operations.

> **For detailed IPC interfaces and command specifications**, see [`API.md`](API.md)

#### Available Commands

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

## Implementation Architecture

### Rust Development Patterns
*Essential patterns for SSH/SFTP backend implementation*

**Error Chaining with Anyhow**: Use anyhow for complex error handling in Rust services.

> **For detailed error handling patterns**, see [`CONTRIBUTING.md#2-resultt-error-handling`](CONTRIBUTING.md#2-resultt-error-handling)

**Memory Management for Credentials**: Always wrap passwords and sensitive data in secure types. See [`API.md`](API.md) for complete SecurePassword implementation.

**Module Organization**: Organize Rust modules by responsibility layers.

> **For module organization patterns**, see [`CONTRIBUTING.md#service-development-patterns`](CONTRIBUTING.md#service-development-patterns)

**Async/Blocking Integration**: Handle ssh2's blocking nature properly.

> **For async integration patterns**, see [`CONTRIBUTING.md#async-operations-with-blocking-libraries`](CONTRIBUTING.md#async-operations-with-blocking-libraries)

**Type Safety for IPC**: Keep Rust and TypeScript types in sync with proper serde attributes.

For complete Rust development standards and anti-patterns, see [`CONTRIBUTING.md#developer-standards--project-philosophy`](CONTRIBUTING.md#developer-standards--project-philosophy).

### Security Implementation

The system implements comprehensive security measures including input sanitization, path safety validation, command injection prevention, and secure memory management.

> **For security principles and requirements**, see [`CONTRIBUTING.md#security-requirements`](CONTRIBUTING.md#security-requirements)
> **For SSH/SFTP security implementation details**, see [`SSH.md#security-patterns`](SSH.md#security-patterns)

### SSH/SFTP Integration

NAMDRunner provides secure, password-based SSH connectivity with comprehensive SFTP file management for cluster operations.

> **For complete SSH/SFTP implementation details**, see [`SSH.md`](SSH.md)

### Clean Architecture Patterns

For comprehensive architectural patterns and code quality standards, see [`CONTRIBUTING.md#developer-standards--project-philosophy`](CONTRIBUTING.md#developer-standards--project-philosophy).

#### Dependency Injection
**Service Container Pattern**: Clean separation of concerns through dependency injection:
- `ServiceContainer` manages service instantiation and dependencies
- Constructor injection for all service dependencies
- Singleton pattern for stateful services, factory pattern for stateless utilities
- Mock service containers for development environments
- Clear service boundaries with explicit interfaces

#### Repository Pattern Implementation
**Separation of Database Concerns**: Clean data access layer with repository pattern:
- **JobRepository trait**: Standardized interface for job data operations
- **DefaultJobRepository**: Concrete implementation using existing database infrastructure
- **JobService**: Business logic layer providing domain-specific operations
- **Mock repositories**: In-memory implementations for development without database dependencies
- **Domain separation**: Core types focus on business logic, repositories handle persistence

#### Validation Pattern Implementation
**Unified Validation System**: Trait-based validation with consistent error handling:
- **Validate trait**: Generic validation interface for command parameters
- **ValidateId trait**: Specialized validation for ID parameters with security checks
- **ValidationError**: Structured error types with field-specific context
- **Validator utilities**: Reusable validation functions for common patterns
- **Type safety**: Validated types ensure clean input throughout the system

#### SLURM Command Builder
**Consistent Command Construction**: Centralized command generation with fluent builder interface for maintainable SLURM integration.

#### Async Pattern Optimization
**Efficient Memory Usage**: Optimized async patterns without unnecessary allocations:
- **Direct async functions**: Eliminated Box::pin allocations where possible
- **Helper methods**: Private async functions for cleaner retry integration
- **Performance**: Reduced heap allocations for frequently-called operations
- **Maintainability**: Cleaner code structure without boxing overhead

### System Architecture Patterns

#### Data Flow
1. User action in Svelte component
2. Call through coreClient interface
3. Tauri command invoked
4. Rust module processes request
5. Result returned through IPC boundary
6. Svelte store updated
7. UI re-renders

#### Error Handling
Comprehensive error management system:
- **Result Types**: All operations return `Result<T>` with structured error information
- **Error Classification**: Network, Authentication, Timeout, Permission, Configuration errors
- **Retry Logic**: Exponential backoff with jitter for transient failures
- **User-Friendly Messages**: Error categorization provides actionable user guidance
- **Recovery Strategies**: Automatic retry vs manual intervention based on error type

## Development Environment
Optimized for rapid development and deployment:
- **Dual Mode Operation**: Mock mode for development, real mode for production
- **Hot Reload**: Fast development iteration with `npm run tauri dev`
- **Type Safety**: Full TypeScript ↔ Rust type checking
- **Portable Builds**: Single executable for Windows deployment

## Build & Deployment

### Target Platforms & Build Matrix

#### Primary Targets
* **Users:** Windows (primary distribution target) - Portable `.exe` via GitHub Actions
* **Development:** Linux (primary dev environment) - Day-to-day development work
* **macOS:** Local builds for manual smoke testing (developer machine validation only)

#### CI/CD Strategy
* **Windows CI:** GitHub Actions Windows runner for release builds
* **Linux CI:** Optional for comprehensive validation

### Packaging & Delivery

#### Distribution Artifacts
* **Windows:** Portable `.exe` built via GitHub Actions (Windows runner)
* **Linux:** Developer builds for internal validation (optional packaging)
* **macOS:** Local builds only for manual validation (not distributed to end users)

#### Build Commands
```bash
# Development build
npm run tauri dev

# Production build
npm run tauri build

# Platform-specific builds (CI)
npm run tauri build -- --target x86_64-pc-windows-msvc  # Windows
npm run tauri build -- --target x86_64-unknown-linux-gnu # Linux
```


## Status & Planning

### Open Items (Affect Implementation Details)

#### Pending Decisions
* **SLURM Version Compatibility**: Confirm JSON output support; plan formatted fallbacks
* **Scratch Purge Cadence**: Determine when to copy back results from scratch directories
* **Module Versions**: Make gcc/NAMD module versions configurable (not hardcoded)
* **Resource Limits**: Document cluster-specific partition limits and QoS specifications

### Next Steps

See [`tasks/roadmap.md`](../tasks/roadmap.md) for planned features and development timeline.